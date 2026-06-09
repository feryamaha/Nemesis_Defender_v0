use crate::report::{CheckResult, CheckStatus};

pub fn run() -> CheckResult {
    let mut res = CheckResult::new("G5 - eBPF Kernel LSM (Linux)");

    #[cfg(not(target_os = "linux"))]
    {
        res.push("Sistema nao-Linux: eBPF nao se aplica (sem impacto).");
        return res.status(CheckStatus::Na);
    }

    #[cfg(target_os = "linux")]
    {
        use crate::checks::nemesis_dir;
        let mut problems = 0;

        let lsm = std::fs::read_to_string("/sys/kernel/security/lsm").unwrap_or_default();
        if lsm.contains("bpf") {
            res.push("OK    BPF LSM ativo (/sys/kernel/security/lsm contem 'bpf').");
        } else {
            res.push("FALHA BPF LSM ausente. Acao: habilitar lsm=...,bpf no kernel cmdline.");
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
                res.push("OK    Capabilities cap_bpf presentes no nemesis-ebpf-daemon.");
            } else {
                res.push("ATENCAO sem cap_bpf. Acao: sudo setcap cap_bpf,cap_perfmon,cap_sys_resource+eip <daemon>.");
                problems += 1;
            }
        } else {
            res.push("ATENCAO nemesis-ebpf-daemon nao compilado.");
            problems += 1;
        }

        if std::path::Path::new("/sys/fs/cgroup/nemesis-agent").exists() {
            res.push("OK    cgroup nemesis-agent existe.");
        } else {
            res.push("ATENCAO cgroup nemesis-agent ausente. Acao: sudo mkdir -p /sys/fs/cgroup/nemesis-agent.");
        }

        if problems == 0 {
            return res.status(CheckStatus::Ok);
        }
        return res.status(CheckStatus::Warn);
    }
}
