#[cfg(target_os = "linux")]
use anyhow::Result;
#[cfg(target_os = "linux")]
use chrono::Utc;
#[cfg(target_os = "linux")]
use nemesis_ebpf_kernel::config::EbpfConfigBundle;
#[cfg(target_os = "linux")]
use nemesis_ebpf_kernel::landlock::{apply_sandbox, LandlockStatus};
#[cfg(target_os = "linux")]
use nemesis_ebpf_kernel::loader::{
    collect_actions_required, detect_enforcement_level, detect_health, doctor, load_linux_backend,
    run_self_test,
};
#[cfg(target_os = "linux")]
use nemesis_ebpf_kernel::logger::log_block_event;
#[cfg(target_os = "linux")]
use nemesis_ebpf_kernel::seccomp::apply_seccomp_filter;
#[cfg(target_os = "linux")]
use nemesis_ebpf_kernel::transport::{default_status_socket_path, run_status_server};
#[cfg(target_os = "linux")]
use nemesis_ebpf_kernel::{EbpfBlockEvent, EbpfEventKind};
#[cfg(target_os = "linux")]
use std::env;
#[cfg(target_os = "linux")]
use std::path::PathBuf;
#[cfg(target_os = "linux")]
use std::sync::atomic::{AtomicBool, Ordering};

#[cfg(target_os = "linux")]
static SIGHUP_RECEIVED: AtomicBool = AtomicBool::new(false);

#[cfg(target_os = "linux")]
extern "C" fn sighup_handler() {
    SIGHUP_RECEIVED.store(true, Ordering::Release);
}

#[cfg(not(target_os = "linux"))]
fn main() {
    eprintln!("[nemesis-ebpf-kernel] eBPF kernel enforcement is available only on Linux.");
    eprintln!("[nemesis-ebpf-kernel] Non-Linux host detected — running stub placeholder (no-op).");
}

#[tokio::main]
#[cfg(target_os = "linux")]
async fn main() -> Result<()> {
    let root = detect_ebpf_root()?;
    let config = EbpfConfigBundle::load_from(&root)?;

    let mut print_status = false;
    let mut doctor_mode = false;
    let mut log_sample = false;
    let mut self_test = false;
    let mut sandbox_mode = false;
    let mut start_mode = false;
    let mut ensure_daemon = false;

    for arg in env::args().skip(1) {
        match arg.as_str() {
            "--print-status" => print_status = true,
            "--doctor" => doctor_mode = true,
            "--log-sample" => log_sample = true,
            "--self-test" => self_test = true,
            "--sandbox" => sandbox_mode = true,
            "--start" => start_mode = true,
            "--ensure-daemon" => ensure_daemon = true,
            _ => {}
        }
    }

    // --ensure-daemon: verifica se ja esta rodando, inicia em background se necessario
    if ensure_daemon {
        return ensure_daemon_running();
    }

    let response = detect_health(&root, &config);

    if log_sample {
        let sample = EbpfBlockEvent {
            pid: std::process::id(),
            tgid: std::process::id(),
            kind: EbpfEventKind::CommandBlocked,
            subject: "sample-ebpf-command".to_string(),
            decision: "blocked".to_string(),
            timestamp: Utc::now().to_rfc3339(),
        };
        log_block_event(&sample);
    }

    if start_mode {
        run_bpf_lsm_daemon(&root, &config)?;
        return Ok(());
    }

    if sandbox_mode {
        run_sandbox_mode(&config);
        return Ok(());
    }

    if doctor_mode {
        let report = doctor(&root, &config);
        let enforcement_level = detect_enforcement_level();
        let actions = collect_actions_required();
        let expanded = serde_json::json!({
            "platform": report.platform,
            "kernel_version": report.kernel_release,
            "landlock_active": nemesis_ebpf_kernel::landlock::landlock_in_lsm_list(),
            "bpf_lsm_active": report.bpf_lsm_active,
            "can_load_bpf": report.has_cap_bpf || report.is_root,
            "enforcement_level": enforcement_level,
            "blocking_constraints": report.blocking_constraints,
            "action_required": actions,
            "full_report": report,
        });
        println!("{}", serde_json::to_string_pretty(&expanded)?);
        return Ok(());
    }

    if self_test {
        let results = run_self_test(&root, &config)?;
        println!("{}", serde_json::to_string_pretty(&results)?);
        return Ok(());
    }

    if print_status {
        println!("{}", serde_json::to_string_pretty(&response)?);
        return Ok(());
    }

    if let Some(socket_path) = default_status_socket_path() {
        run_status_server(socket_path, response).await?;
    } else {
        println!("{}", serde_json::to_string_pretty(&response)?);
    }

    Ok(())
}

#[cfg(target_os = "linux")]
fn run_sandbox_mode(config: &EbpfConfigBundle) {
    let allowed_exec: Vec<&str> = config
        .landlock
        .allowed_exec
        .iter()
        .map(|s| s.as_str())
        .collect();

    match apply_sandbox(&allowed_exec) {
        LandlockStatus::Active { abi } => {
            eprintln!("[nemesis] Landlock ABI v{abi} active — process tree sandboxed");
            match apply_seccomp_filter() {
                Ok(()) => eprintln!("[nemesis] seccomp filter applied"),
                Err(e) => eprintln!("[nemesis] seccomp warning: {e}"),
            }
        }
        LandlockStatus::Unsupported => {
            eprintln!("[nemesis] Landlock not supported on this kernel — pretool only");
        }
        LandlockStatus::Error(e) => {
            eprintln!("[nemesis] Landlock error: {e} — pretool only");
        }
    }
}

#[cfg(target_os = "linux")]
fn run_bpf_lsm_daemon(root: &std::path::Path, config: &EbpfConfigBundle) -> Result<()> {
    eprintln!("[nemesis] loading BPF LSM program into kernel...");

    // Registrar handler SIGHUP para recarregar commands.toml em runtime
    // Uso: kill -HUP <pid_do_daemon>
    unsafe {
        libc::signal(libc::SIGHUP, sighup_handler as *const () as libc::sighandler_t);
    }
    eprintln!("[nemesis] SIGHUP handler registered — \"kill -HUP <pid>\" reloads commands.toml");

    let program = load_linux_backend(root, config, |event| {
        nemesis_ebpf_kernel::logger::log_block_event(&event);
        eprintln!(
            "[nemesis] BLOCKED pid={} cmd={}",
            event.pid, event.subject
        );
    })?;
    eprintln!("[nemesis] BPF LSM attached — enforcement active. Ctrl-C to stop.");

    loop {
        program.poll_events()?;

        // Bridge: SIGHUP_RECEIVED → program.needs_reload
        if SIGHUP_RECEIVED.load(Ordering::Acquire) {
            program.needs_reload.store(true, Ordering::Release);
            SIGHUP_RECEIVED.store(false, Ordering::Release);
        }
    }
}

#[cfg(target_os = "linux")]
fn detect_ebpf_root() -> Result<PathBuf> {
    let cwd = env::current_dir()?;
    let direct = cwd.join("ebpf-kernel");
    if direct.join("denylist-ebpf").exists() {
        return Ok(direct);
    }

    let nested = cwd.join(".nemesis").join("ebpf-kernel");
    if nested.join("denylist-ebpf").exists() {
        return Ok(nested);
    }

    if cwd.file_name().and_then(|s| s.to_str()) == Some("ebpf-kernel") {
        return Ok(cwd);
    }

    anyhow::bail!("could not locate .nemesis/ebpf-kernel from {}", cwd.display())
}

#[cfg(target_os = "linux")]
fn ensure_daemon_running() -> Result<()> {
    // Verificar se o daemon ja esta rodando
    let my_pid = std::process::id();
    let output = std::process::Command::new("pgrep")
        .args(["-f", "nemesis-ebpf-daemon.*--start"])
        .output()
        .ok();

    if let Some(out) = output {
        let pids: Vec<u32> = String::from_utf8_lossy(&out.stdout)
            .lines()
            .filter_map(|line| line.trim().parse::<u32>().ok())
            .filter(|pid| *pid != my_pid)
            .collect();

        if !pids.is_empty() {
            // Daemon ja esta rodando — saida silenciosa
            return Ok(());
        }
    }

    // Localizar o binario do daemon
    let exe = env::current_exe()?;
    let daemon_path = if exe.file_name().and_then(|s| s.to_str()) == Some("nemesis-ebpf-daemon") {
        exe.clone()
    } else {
        // Tenta encontrar pelo path do projeto
        let cwd = env::current_dir()?;
        let candidate = cwd.join(".nemesis/target/release/nemesis-ebpf-daemon");
        if candidate.exists() {
            candidate
        } else {
            let debug = cwd.join(".nemesis/target/debug/nemesis-ebpf-daemon");
            if debug.exists() {
                debug
            } else {
                anyhow::bail!("nemesis-ebpf-daemon binary not found. Run cargo build --release first.");
            }
        }
    };

    let root = match detect_ebpf_root() {
        Ok(r) => r,
        Err(_) => {
            anyhow::bail!("could not locate .nemesis/ebpf-kernel directory");
        }
    };

    eprintln!("[nemesis] Starting eBPF daemon in background...");
    let child = std::process::Command::new(&daemon_path)
        .arg("--start")
        .current_dir(&root.parent().unwrap_or(&root))
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn();

    match child {
        Ok(_) => {
            eprintln!("[nemesis] eBPF daemon started in background");
            Ok(())
        }
        Err(e) => {
            anyhow::bail!("failed to spawn eBPF daemon: {e}. Try running install-service.sh with sudo.");
        }
    }
}
