use crate::cgroup;
use crate::config::EbpfConfigBundle;
use crate::egress::EgressAllowlist;
use crate::logger::kernel_event_to_block_event;
use crate::{
    EbpfBlockEvent, EbpfDoctorReport, EbpfHealthResponse, EbpfRuntimeStatus, KernelCommandKey,
    KernelEvent,
};
use anyhow::{anyhow, Context, Result};
use bytemuck::bytes_of;
use chrono::{TimeZone, Utc};
use libbpf_rs::{MapCore, MapFlags, ObjectBuilder, RingBufferBuilder};
use std::fs;
use std::os::unix::process::ExitStatusExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};
use std::time::Duration;

pub struct LoadedEbpfProgram {
    object: Mutex<libbpf_rs::Object>,
    _links: Vec<libbpf_rs::Link>,
    ringbuf: libbpf_rs::RingBuffer<'static>,
    pub needs_reload: Arc<AtomicBool>,
    root: PathBuf,
    in_cgroup: AtomicBool,
}

impl LoadedEbpfProgram {
    pub fn poll_events(&self) -> Result<()> {
        self.ringbuf.poll(Duration::from_millis(100))?;

        // Verificar se SIGHUP marcou reload
        if self.needs_reload.load(Ordering::Acquire) {
            let config = EbpfConfigBundle::load_from(&self.root)?;
            let mut object = self.object.lock().unwrap();
            populate_command_map(&mut object, &config)?;
            populate_egress_maps(&mut object, &config)?;
            self.needs_reload.store(false, Ordering::Release);
            eprintln!("[nemesis] commands.toml reloaded via SIGHUP");
        }

        // AUTO-CGROUP: retentar mover daemon para cgroup do agente
        // Necessario ate que systemd ExecStartPost coloque o daemon no cgroup
        // ou se o cgroup foi removido e recriado
        if !self.in_cgroup.load(Ordering::Acquire) {
            let pid = std::process::id();
            if crate::cgroup::assign_pid_to_agent_cgroup(pid).is_ok() {
                self.in_cgroup.store(true, Ordering::Release);
                eprintln!("[nemesis] Daemon PID {} assigned to agent cgroup (auto-retry)", pid);
            }
        }

        Ok(())
    }
}

pub fn compile_bpf(root: &Path) -> Result<PathBuf> {
    let status = Command::new("make")
        .arg("-C")
        .arg(root)
        .arg("all")
        .status()
        .context("failed to invoke make for ebpf object")?;

    if !status.success() {
        return Err(anyhow!("make failed for ebpf object"));
    }

    Ok(root.join("ebpf").join("nemesis-block.bpf.o"))
}

/// Eleva RLIMIT_MEMLOCK para RLIM_INFINITY — necessário para carga de programas BPF.
/// Requer CAP_SYS_RESOURCE ou root. Sem isso, libbpf falha silenciosamente.
fn bump_memlock_rlimit() -> Result<()> {
    let rlim = libc::rlimit {
        rlim_cur: libc::RLIM_INFINITY,
        rlim_max: libc::RLIM_INFINITY,
    };
    let ret = unsafe { libc::setrlimit(libc::RLIMIT_MEMLOCK, &rlim) };
    if ret != 0 {
        return Err(anyhow!(
            "setrlimit RLIMIT_MEMLOCK failed (errno {}): need CAP_SYS_RESOURCE or root",
            ret
        ));
    }
    Ok(())
}

pub fn load_linux_backend(
    root: &Path,
    config: &EbpfConfigBundle,
    on_event: impl FnMut(EbpfBlockEvent) + 'static,
) -> Result<LoadedEbpfProgram> {
    ensure_can_attempt_load(root, config)?;
    bump_memlock_rlimit().context("bump RLIMIT_MEMLOCK before BPF load")?;
    let object_path = compile_bpf(root)?;

    let open_object = ObjectBuilder::default()
        .open_file(&object_path)
        .with_context(|| format!("failed to open {}", object_path.display()))?;
    let mut object = open_object.load().context("failed to load ebpf object")?;

    populate_command_map(&mut object, config)?;
    populate_egress_maps(&mut object, config)?;

    let mut links = Vec::new();
    for program in object.progs_mut() {
        let prog_name = program.name().to_owned();
        let link = program
            .attach_lsm()
            .with_context(|| format!("failed to attach_lsm program {:?}", prog_name))?;
        links.push(link);
    }

    // Registrar o cgroup_id do agente no mapa BPF
    let agent_cgroup_id = cgroup::create_agent_cgroup()
        .context("failed to create agent cgroup")?;
    let cgroup_map = object
        .maps_mut()
        .find(|map| map.name().to_str() == Some("agent_cgroup_map"))
        .context("failed to locate agent_cgroup_map")?;
    let key: u32 = 0;
    cgroup_map
        .update(bytes_of(&key), bytes_of(&agent_cgroup_id), MapFlags::ANY)
        .context("failed to set agent cgroup_id in BPF map")?;

    eprintln!("[nemesis] Agent cgroup_id {} registered in BPF", agent_cgroup_id);

    // Mover o próprio daemon para o cgroup do agente.
    // Todos os subprocessos spawnados herdarão o cgroup automaticamente.
    // Se falhar agora (ex: cgroup nao pronto), tenta de novo no poll_events()
    let own_pid = std::process::id();
    let daemon_in_cgroup = cgroup::assign_pid_to_agent_cgroup(own_pid).is_ok();
    if daemon_in_cgroup {
        eprintln!("[nemesis] Daemon PID {} assigned to agent cgroup — all child processes will be enforced", own_pid);
    } else {
        eprintln!("[nemesis] WARNING: could not assign daemon PID {} to agent cgroup now. Will retry in poll loop.", own_pid);
    }

    let events = object
        .maps()
        .find(|map| map.name().to_str() == Some("events"))
        .context("failed to locate events ringbuf map")?;

    let mut callback = on_event;
    let mut builder = RingBufferBuilder::new();
    builder
        .add(&events, move |data| {
            let Some(event) = KernelEvent::from_bytes(data) else {
                return 0;
            };
            callback(kernel_event_to_block_event(&event));
            0
        })
        .context("failed to add ringbuf callback")?;

    let ringbuf = builder.build().context("failed to build ringbuf")?;

    let needs_reload = Arc::new(AtomicBool::new(false));
    let daemon_was_assigned = cgroup::assign_pid_to_agent_cgroup(own_pid).is_ok();

    Ok(LoadedEbpfProgram {
        object: Mutex::new(object),
        _links: links,
        ringbuf,
        needs_reload,
        root: root.to_path_buf(),
        in_cgroup: AtomicBool::new(daemon_was_assigned),
    })
}

pub fn detect_health(root: &Path, config: &EbpfConfigBundle) -> EbpfHealthResponse {
    if !config.runtime.enabled {
        return EbpfHealthResponse {
            status: EbpfRuntimeStatus::Disabled,
            platform: std::env::consts::OS.to_string(),
            reason: "ebpf layer disabled in config.toml".to_string(),
        };
    }

    #[cfg(not(target_os = "linux"))]
    {
        return EbpfHealthResponse {
            status: EbpfRuntimeStatus::Unsupported,
            platform: std::env::consts::OS.to_string(),
            reason: "kernel eBPF enforcement is linux-first; fallback remains in user-space"
                .to_string(),
        };
    }

    #[cfg(target_os = "linux")]
    {
        let report = doctor(root, config);
        if report.can_attempt_load {
            EbpfHealthResponse {
                status: EbpfRuntimeStatus::Enabled,
                platform: report.platform,
                reason: format!(
                    "linux backend ready with {}",
                    report.recommended_backend
                ),
            }
        } else {
            EbpfHealthResponse {
                status: EbpfRuntimeStatus::Error,
                platform: report.platform,
                reason: report.blocking_constraints.join("; "),
            }
        }
    }
}

pub fn doctor(root: &Path, config: &EbpfConfigBundle) -> EbpfDoctorReport {
    let active_lsms = read_active_lsms();
    let has_btf = Path::new("/sys/kernel/btf/vmlinux").exists();
    let has_clang = command_exists("clang");
    let has_bpftool = command_exists("bpftool");
    let kernel_supports_bpf_lsm = file_contains("/boot/config-", "CONFIG_BPF_LSM=y");
    let bpf_lsm_active = active_lsms.iter().any(|lsm| lsm == "bpf");
    let unprivileged_bpf_disabled = read_unprivileged_bpf_disabled();
    let is_root = nix_like_euid_is_root();
    let cap_eff = read_cap_eff();
    let has_cap_bpf = cap_eff_bit(cap_eff, 39);
    let has_cap_perfmon = cap_eff_bit(cap_eff, 38);
    let has_cap_sys_admin = cap_eff_bit(cap_eff, 21);
    let kernel_release = read_kernel_release();
    let object_exists = root.join("ebpf").join("nemesis-block.bpf.c").exists();

    let mut blocking_constraints = Vec::new();
    if !config.runtime.enabled {
        blocking_constraints.push("config.toml has enabled = false".to_string());
    }
    if !object_exists {
        blocking_constraints.push("ebpf program source not found".to_string());
    }
    if !has_clang {
        blocking_constraints.push("clang not found".to_string());
    }
    if !has_bpftool {
        blocking_constraints.push("bpftool not found".to_string());
    }
    if !has_btf {
        blocking_constraints.push("/sys/kernel/btf/vmlinux not found".to_string());
    }
    if !(is_root || has_cap_bpf || has_cap_sys_admin || has_cap_perfmon) {
        blocking_constraints.push(
            "process lacks root/CAP_BPF/CAP_PERFMON/CAP_SYS_ADMIN required to load eBPF"
                .to_string(),
        );
    }

    let recommended_backend = if bpf_lsm_active {
        "bpf_lsm".to_string()
    } else {
        "kprobe_execve_override".to_string()
    };

    EbpfDoctorReport {
        platform: std::env::consts::OS.to_string(),
        kernel_release,
        config_enabled: config.runtime.enabled,
        active_lsms,
        kernel_supports_bpf_lsm,
        bpf_lsm_active,
        unprivileged_bpf_disabled,
        has_btf,
        has_clang,
        has_bpftool,
        is_root,
        has_cap_bpf,
        has_cap_perfmon,
        has_cap_sys_admin,
        recommended_backend,
        can_attempt_load: blocking_constraints.is_empty(),
        blocking_constraints,
    }
}

pub fn run_self_test(root: &Path, config: &EbpfConfigBundle) -> Result<Vec<crate::EbpfSelfTestResult>> {
    let captured_events = Arc::new(Mutex::new(Vec::<EbpfBlockEvent>::new()));
    let captured_for_callback = Arc::clone(&captured_events);
    let program = load_linux_backend(root, config, move |event| {
        if let Ok(mut events) = captured_for_callback.lock() {
            events.push(event);
        }
    })?;

    let mut results = Vec::new();
    for command in [
        "python3 -V",
        "sed --version",
        "awk --version",
        "curl --version",
        "dd --version",
    ] {
        let output = Command::new("bash")
            .arg("-lc")
            .arg(command)
            .output()
            .with_context(|| format!("failed to execute self-test command `{command}`"))?;
        let status_code = output
            .status
            .code()
            .or_else(|| output.status.signal().map(|s| 128 + s));

        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let blocked = status_code != Some(0)
            && (stderr.contains("Operation not permitted")
                || stderr.contains("Permissão negada")
                || stderr.contains("EPERM"));

        results.push(crate::EbpfSelfTestResult {
            command: command.to_string(),
            attempted: true,
            blocked,
            status: status_code,
            stderr,
        });
        program.poll_events()?;
    }

    let _ = captured_events;
    Ok(results)
}

fn ensure_can_attempt_load(root: &Path, config: &EbpfConfigBundle) -> Result<()> {
    let report = doctor(root, config);
    if !report.can_attempt_load {
        return Err(anyhow!(
            "cannot load ebpf backend: {}",
            report.blocking_constraints.join("; ")
        ));
    }
    Ok(())
}

fn populate_command_map(object: &mut libbpf_rs::Object, config: &EbpfConfigBundle) -> Result<()> {
    let map = object
        .maps_mut()
        .find(|map| map.name().to_str() == Some("blocked_commands"))
        .context("failed to locate blocked_commands map")?;

    for command in &config.commands.blocked_commands {
        let key = KernelCommandKey::from_command(command);
        let value = [1u8];
        map.update(bytes_of(&key), &value, MapFlags::ANY)
            .with_context(|| format!("failed to insert command `{command}` into map"))?;
    }

    Ok(())
}

/// Popula os maps de egress (flag enforce + LPM tries IPv4/IPv6) a partir de `config.egress`.
/// Chave do LPM trie: [prefixlen: u32 native][addr: bytes em ordem de rede]. Valor: u16 porta.
fn populate_egress_maps(object: &mut libbpf_rs::Object, config: &EbpfConfigBundle) -> Result<()> {
    // 1) flag enforce (ARRAY índice 0)
    {
        let enforce_map = object
            .maps_mut()
            .find(|m| m.name().to_str() == Some("egress_enforce"))
            .context("failed to locate egress_enforce map")?;
        let key: u32 = 0;
        let val: u8 = if config.egress.enforce { 1 } else { 0 };
        enforce_map
            .update(bytes_of(&key), &[val], MapFlags::ANY)
            .context("failed to set egress_enforce flag")?;
    }

    // Parsing da allowlist (erro de config é fatal — não abrir silenciosamente)
    let allow = EgressAllowlist::parse(&config.egress.allowlist)
        .map_err(|e| anyhow!("egress allowlist inválida: {e}"))?;

    // 2) LPM trie IPv4
    {
        let v4_map = object
            .maps_mut()
            .find(|m| m.name().to_str() == Some("egress_allow_v4"))
            .context("failed to locate egress_allow_v4 map")?;
        for r in allow.rules4() {
            let mut key = Vec::with_capacity(8);
            key.extend_from_slice(&(r.prefix_len as u32).to_ne_bytes());
            key.extend_from_slice(&r.network.to_be_bytes()); // ordem de rede
            let val = r.port.to_ne_bytes();
            v4_map
                .update(&key, &val, MapFlags::ANY)
                .context("failed to insert egress v4 rule")?;
        }
    }

    // 3) LPM trie IPv6
    {
        let v6_map = object
            .maps_mut()
            .find(|m| m.name().to_str() == Some("egress_allow_v6"))
            .context("failed to locate egress_allow_v6 map")?;
        for r in allow.rules6() {
            let mut key = Vec::with_capacity(20);
            key.extend_from_slice(&(r.prefix_len as u32).to_ne_bytes());
            key.extend_from_slice(&r.network.to_be_bytes()); // u128 → 16 bytes big-endian
            let val = r.port.to_ne_bytes();
            v6_map
                .update(&key, &val, MapFlags::ANY)
                .context("failed to insert egress v6 rule")?;
        }
    }

    Ok(())
}

/// Retorna true se o BPF LSM está ativo no boot atual.
pub fn bpf_lsm_active() -> bool {
    fs::read_to_string("/sys/kernel/security/lsm")
        .map(|s| s.split(',').any(|l| l.trim() == "bpf"))
        .unwrap_or(false)
}

/// Retorna true se o processo tem CAP_BPF efetiva (bit 39).
pub fn has_cap_bpf() -> bool {
    cap_eff_bit(read_cap_eff(), 39)
}

/// Detecta o nível de enforcement disponível neste host/boot.
pub fn detect_enforcement_level() -> &'static str {
    if bpf_lsm_active() && (nix_like_euid_is_root() || has_cap_bpf()) {
        "bpf_lsm"
    } else if crate::landlock::landlock_in_lsm_list() {
        "landlock"
    } else {
        "pretool_only"
    }
}

/// Coleta ações necessárias para atingir enforcement completo.
pub fn collect_actions_required() -> Vec<String> {
    let mut actions = vec![];
    if !bpf_lsm_active() {
        actions.push(
            "Add 'bpf' to lsm= in /etc/default/grub and reboot to enable BPF LSM".to_string(),
        );
    }
    if !has_cap_bpf() && !nix_like_euid_is_root() {
        actions.push(
            "Run as root or grant CAP_BPF+CAP_PERFMON to load BPF programs".to_string(),
        );
    }
    actions
}

/// Tenta conectar a camada BPF LSM (requer bpf ativo no boot + CAP_BPF ou root).
pub fn attach_bpf_lsm(object: &mut libbpf_rs::Object) -> Result<Vec<libbpf_rs::Link>> {
    if !bpf_lsm_active() {
        return Err(anyhow!(
            "bpf not in /sys/kernel/security/lsm — add 'bpf' to lsm= in /etc/default/grub and reboot"
        ));
    }

    let mut links = Vec::new();
    for program in object.progs_mut() {
        let prog_name = program.name().to_owned();
        let link = program
            .attach_lsm()
            .with_context(|| format!("failed to attach_lsm program {:?}", prog_name))?;
        links.push(link);
    }
    Ok(links)
}

fn read_active_lsms() -> Vec<String> {
    fs::read_to_string("/sys/kernel/security/lsm")
        .ok()
        .map(|raw| raw.trim().split(',').map(|s| s.trim().to_string()).collect())
        .unwrap_or_default()
}

fn read_unprivileged_bpf_disabled() -> Option<i32> {
    fs::read_to_string("/proc/sys/kernel/unprivileged_bpf_disabled")
        .ok()
        .and_then(|raw| raw.trim().parse::<i32>().ok())
}

fn read_kernel_release() -> String {
    fs::read_to_string("/proc/sys/kernel/osrelease")
        .map(|raw| raw.trim().to_string())
        .unwrap_or_else(|_| "unknown".to_string())
}

fn read_cap_eff() -> u64 {
    fs::read_to_string("/proc/self/status")
        .ok()
        .and_then(|raw| {
            raw.lines()
                .find(|line| line.starts_with("CapEff:"))
                .and_then(|line| line.split_whitespace().nth(1))
                .and_then(|hex| u64::from_str_radix(hex, 16).ok())
        })
        .unwrap_or(0)
}

fn cap_eff_bit(cap_eff: u64, bit: u32) -> bool {
    if bit >= 64 {
        return false;
    }
    cap_eff & (1u64 << bit) != 0
}

fn nix_like_euid_is_root() -> bool {
    Command::new("id")
        .arg("-u")
        .output()
        .ok()
        .and_then(|out| String::from_utf8(out.stdout).ok())
        .map(|value| value.trim() == "0")
        .unwrap_or(false)
}

fn command_exists(name: &str) -> bool {
    Command::new("bash")
        .arg("-lc")
        .arg(format!("command -v {name} >/dev/null 2>&1"))
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

fn file_contains(prefix: &str, pattern: &str) -> bool {
    let path = format!("{prefix}{}", read_kernel_release());
    fs::read_to_string(path)
        .map(|content| content.contains(pattern))
        .unwrap_or(false)
}

#[allow(dead_code)]
fn ns_to_rfc3339(ns: u64) -> String {
    let secs = (ns / 1_000_000_000) as i64;
    let nanos = (ns % 1_000_000_000) as u32;
    Utc.timestamp_opt(secs, nanos)
        .single()
        .unwrap_or_else(Utc::now)
        .to_rfc3339()
}
