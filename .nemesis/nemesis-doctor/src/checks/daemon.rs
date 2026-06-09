use crate::checks::nemesis_dir;
use crate::report::{CheckResult, CheckStatus};

pub fn run() -> CheckResult {
    let mut res = CheckResult::new("G6 - Daemon nemesis-defender");
    let pid_file = nemesis_dir().join("runtime").join("defender.pid");

    let pid = std::fs::read_to_string(&pid_file)
        .ok()
        .and_then(|s| s.trim().parse::<u32>().ok());

    let pid = match pid {
        Some(p) => p,
        None => {
            res.push("PID file ausente - daemon nao esta rodando.");
            res.push("Acao: .nemesis/target/release/nemesis-defender --ensure-daemon");
            return res.status(CheckStatus::Fail);
        }
    };

    #[cfg(target_os = "linux")]
    {
        let comm = std::fs::read_to_string(format!("/proc/{}/comm", pid)).unwrap_or_default();
        if !comm.trim().starts_with("nemesis-defende") {
            res.push(format!(
                "PID {} no PID file, mas processo nao esta vivo (stale).",
                pid
            ));
            res.push("Acao: .nemesis/target/release/nemesis-defender --ensure-daemon");
            return res.status(CheckStatus::Fail);
        }
        res.push(format!("OK    daemon vivo (PID {}).", pid));

        let mut inotify = 0;
        if let Ok(entries) = std::fs::read_dir(format!("/proc/{}/fd", pid)) {
            for e in entries.flatten() {
                if let Ok(target) = std::fs::read_link(e.path()) {
                    if target.to_string_lossy().contains("inotify") {
                        inotify += 1;
                    }
                }
            }
        }
        if inotify > 0 {
            res.push(format!(
                "OK    {} descritor(es) inotify aberto(s) - watcher ativo.",
                inotify
            ));
            return res.status(CheckStatus::Ok);
        }
        res.push("ATENCAO nenhum fd inotify - watcher pode estar inativo (fs.inotify.max_user_watches?).");
        return res.status(CheckStatus::Warn);
    }

    #[cfg(not(target_os = "linux"))]
    {
        res.push(format!(
            "PID file indica PID {} (verificacao de inotify so no Linux).",
            pid
        ));
        return res.status(CheckStatus::Ok);
    }
}
