//! PID file management for nemesis-defender daemon
//!
//! PID file location: .nemesis/runtime/defender.pid
//! (same directory as permission-gate.state.json)

use std::path::PathBuf;

fn pid_path() -> PathBuf {
    // Resolve `.nemesis/runtime/defender.pid` subindo do path do binário até o ancestral
    // chamado `.nemesis` — robusto para AMBOS os layouts (não assume profundidade fixa):
    //   dev:    .nemesis/target/release/nemesis-defender  → ancestral .nemesis → .nemesis/runtime/
    //   distro: .nemesis/bin/nemesis-defender             → ancestral .nemesis → .nemesis/runtime/
    // (Mesma estratégia de violations_log::ledger_path; evita o overshoot que criava
    //  `<raiz do projeto>/runtime/` solto no layout distribuído.)
    if let Ok(exe) = std::env::current_exe() {
        for anc in exe.ancestors() {
            if anc.file_name().map(|n| n == ".nemesis").unwrap_or(false) {
                let runtime = anc.join("runtime").join("defender.pid");
                if let Some(parent) = runtime.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }
                return runtime;
            }
        }
    }

    // Fallback (resolução pelo binário falhou): SEMPRE ancora em `.nemesis/` relativo ao CWD —
    // nunca solto na raiz do projeto.
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let _ = std::fs::create_dir_all(cwd.join(".nemesis").join("runtime"));
    cwd.join(".nemesis").join("runtime").join("defender.pid")
}

/// Path to the exclusive spawn-lock file (prevents duplicate daemon spawning)
pub fn lock_path() -> std::path::PathBuf {
    pid_path().with_extension("lock")
}

pub fn write_pid_file() {
    let pid = std::process::id();
    let path = pid_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let _ = std::fs::write(&path, pid.to_string());
}

pub fn remove_pid_file() {
    let _ = std::fs::remove_file(pid_path());
}

pub fn read_pid() -> Option<u32> {
    std::fs::read_to_string(pid_path())
        .ok()
        .and_then(|s| s.trim().parse::<u32>().ok())
}

pub fn is_daemon_running() -> bool {
    let Some(pid) = read_pid() else { return false };

    // Check if process with this PID is actually alive AND is nemesis-defender
    #[cfg(unix)]
    {
        let proc_comm = std::path::PathBuf::from(format!("/proc/{}/comm", pid));
        let proc_exe = std::path::PathBuf::from(format!("/proc/{}/exe", pid));

        let alive = proc_comm.exists()
            && proc_exe.exists()
            && std::fs::read_to_string(&proc_comm)
                .map(|s| s.trim().starts_with("nemesis-defende"))
                .unwrap_or(false);

        if !alive {
            // Stale PID file — clean up so next caller spawns a fresh daemon
            let _ = std::fs::remove_file(pid_path());
        }

        alive
    }

    #[cfg(not(unix))]
    {
        // Windows: check via tasklist
        let out = std::process::Command::new("tasklist")
            .args(["/FI", &format!("PID eq {}", pid)])
            .output();
        let alive = out
            .map(|o| String::from_utf8_lossy(&o.stdout).contains(&pid.to_string()))
            .unwrap_or(false);
        if !alive {
            let _ = std::fs::remove_file(pid_path());
        }
        alive
    }
}
