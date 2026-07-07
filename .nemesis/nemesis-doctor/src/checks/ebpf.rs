use crate::report::{CheckResult, CheckStatus};

pub fn run() -> CheckResult {
    let mut res = CheckResult::new(
        "G5 - eBPF Kernel LSM (Linux)",
        "G5 - eBPF Kernel LSM (Linux)",
    );

    #[cfg(not(target_os = "linux"))]
    {
        res.push(
            "Sistema nao-Linux: eBPF nao se aplica (sem impacto).",
            "Non-Linux system: eBPF does not apply (no impact).",
        );
        return res.status(CheckStatus::Na);
    }

    #[cfg(target_os = "linux")]
    {
        use crate::checks::nemesis_dir;
        let mut problems = 0;

        let lsm = std::fs::read_to_string("/sys/kernel/security/lsm").unwrap_or_default();
        if lsm.contains("bpf") {
            res.push(
                "OK    BPF LSM ativo (/sys/kernel/security/lsm contem 'bpf').",
                "OK    BPF LSM active (/sys/kernel/security/lsm contains 'bpf').",
            );
        } else {
            res.push(
                "FALHA BPF LSM ausente. Acao: habilitar lsm=...,bpf no kernel cmdline.",
                "FAIL  BPF LSM missing. Action: enable lsm=...,bpf in the kernel cmdline.",
            );
            problems += 1;
        }

        let daemon = nemesis_dir()
            .join("target")
            .join("release")
            .join("nemesis-ebpf-daemon");
        if daemon.exists() {
            let caps = std::process::Command::new("getcap")
                .arg(&daemon)
                .output()
                .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
                .unwrap_or_default();
            if caps.contains("cap_bpf") {
                res.push(
                    "OK    Capabilities cap_bpf presentes no nemesis-ebpf-daemon.",
                    "OK    cap_bpf capabilities present on nemesis-ebpf-daemon.",
                );
            } else {
                res.push(
                    "ATENCAO sem cap_bpf (cai a cada recompilacao). Acao: rode .nemesis/scripts/ensure-ebpf-caps.sh (ou builde via .nemesis/scripts/nemesis-build.sh, que ja reaplica as caps).",
                    "WARNING missing cap_bpf (drops on every recompile). Action: run .nemesis/scripts/ensure-ebpf-caps.sh (or build via .nemesis/scripts/nemesis-build.sh, which re-applies caps).",
                );
                problems += 1;
            }
        } else {
            res.push(
                "ATENCAO nemesis-ebpf-daemon nao compilado.",
                "WARNING nemesis-ebpf-daemon not compiled.",
            );
            problems += 1;
        }

        if std::path::Path::new("/sys/fs/cgroup/nemesis-agent").exists() {
            res.push(
                "OK    cgroup nemesis-agent existe.",
                "OK    nemesis-agent cgroup exists.",
            );
        } else {
            res.push(
                "ATENCAO cgroup nemesis-agent ausente. Acao: sudo mkdir -p /sys/fs/cgroup/nemesis-agent.",
                "WARNING nemesis-agent cgroup missing. Action: sudo mkdir -p /sys/fs/cgroup/nemesis-agent.",
            );
        }

        // Egress allowlist (lsm/socket_connect) — SPEC_004. enforce=false e o default seguro,
        // entao nao conta como problema; so reporta o estado.
        let egress_toml = nemesis_dir()
            .join("ebpf-kernel")
            .join("denylist-ebpf")
            .join("egress.toml");
        match std::fs::read_to_string(&egress_toml) {
            Ok(content) => {
                let enforce = content
                    .lines()
                    .find(|l| l.trim_start().starts_with("enforce"))
                    .map(|l| l.contains("true"))
                    .unwrap_or(false);
                if enforce {
                    res.push(
                        "OK    Egress allowlist ATIVA (enforce=true). Conexoes fora da allowlist sao bloqueadas no kernel.",
                        "OK    Egress allowlist ACTIVE (enforce=true). Connections outside the allowlist are blocked in the kernel.",
                    );
                } else {
                    res.push(
                        "INFO  Egress allowlist em modo observacao (enforce=false, default seguro). Para impor: enforce=true em egress.toml + sudo systemctl restart nemesis-ebpf.",
                        "INFO  Egress allowlist in observation mode (enforce=false, safe default). To enforce: enforce=true in egress.toml + sudo systemctl restart nemesis-ebpf.",
                    );
                }
            }
            Err(_) => {
                res.push(
                    "ATENCAO egress.toml ausente em .nemesis/ebpf-kernel/denylist-ebpf/ — egress allowlist nao configurada (nao impoe).",
                    "WARNING egress.toml missing in .nemesis/ebpf-kernel/denylist-ebpf/ — egress allowlist not configured (does not enforce).",
                );
            }
        }

        if problems == 0 {
            return res.status(CheckStatus::Ok);
        }
        return res.status(CheckStatus::Warn);
    }
}
