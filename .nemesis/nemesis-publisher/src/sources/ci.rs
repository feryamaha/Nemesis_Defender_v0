//! Leitura de CI runs via git log. Portado de local-source.ts:417-459.

use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CiRun {
    pub id: i64,
    pub status: String,
    pub branch: String,
    pub sha: String,
    pub event: String,
    pub started_at: String,
    pub duration_sec: i64,
    pub url: String,
}

const REPO_URL: &str = "https://github.com/feryamaha/Nemesis_Defender_v2.0";

/// Executa git log --oneline -10 e retorna Vec<CiRun>.
pub fn read_ci_runs(nemesis_path: &std::path::Path) -> Vec<CiRun> {
    let output = Command::new("git")
        .args(["log", "--oneline", "-10", "--format=%H|%h|%s|%ci|%d"])
        .current_dir(nemesis_path)
        .output();

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let lines: Vec<&str> = stdout
                .trim()
                .split('\n')
                .filter(|l| !l.is_empty())
                .collect();
            let mut runs = Vec::new();

            for (i, line) in lines.iter().enumerate() {
                let parts: Vec<&str> = line.split('|').collect();
                if parts.len() < 4 {
                    continue;
                }
                let short_hash = parts.get(1).map(|s| s.trim()).unwrap_or("unknown");
                let date = parts.get(3).map(|s| s.trim()).unwrap_or("");
                let refs = parts.get(4).map(|s| s.trim()).unwrap_or("");
                let is_main = refs.contains("main") || refs.contains("HEAD");

                runs.push(CiRun {
                    id: 90000 - i as i64,
                    status: "success".to_string(),
                    branch: if is_main { "main".to_string() } else { "unknown".to_string() },
                    sha: short_hash.to_string(),
                    event: "push".to_string(),
                    started_at: date.to_string(),
                    duration_sec: 300 + (i as i64) * 10,
                    url: format!("{}/actions", REPO_URL),
                });
            }
            runs
        }
        Err(_) => vec![CiRun {
            id: 0,
            status: "failure".to_string(),
            branch: "main".to_string(),
            sha: "unknown".to_string(),
            event: "push".to_string(),
            started_at: chrono::Local::now().to_rfc3339(),
            duration_sec: 0,
            url: format!("{}/actions", REPO_URL),
        }],
    }
}
