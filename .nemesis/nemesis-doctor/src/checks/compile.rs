use crate::checks::{command_exists, nemesis_dir};
use crate::report::{CheckResult, CheckStatus};

pub fn run() -> CheckResult {
    let mut res = CheckResult::new("G1 - Compilacao (cargo check --workspace)");

    if !command_exists("cargo") {
        res.push("cargo nao encontrado no PATH - pulando verificacao de compilacao.");
        return res.status(CheckStatus::Skip);
    }

    let output = std::process::Command::new("cargo")
        .args(["check", "--workspace", "--message-format=json"])
        .current_dir(nemesis_dir())
        .output();

    let output = match output {
        Ok(o) => o,
        Err(e) => {
            res.push(format!("Falha ao executar cargo check: {}", e));
            return res.status(CheckStatus::Fail);
        }
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut errors = 0usize;
    let mut warnings = 0usize;

    for line in stdout.lines() {
        let v: serde_json::Value = match serde_json::from_str(line) {
            Ok(v) => v,
            Err(_) => continue,
        };
        if v["reason"] == "compiler-message" {
            match v["message"]["level"].as_str() {
                Some("error") => errors += 1,
                Some("warning") => warnings += 1,
                _ => {}
            }
        }
    }

    res.push(format!("Erros: {} | Warnings: {}", errors, warnings));
    res.push("Como ler: 0 erros = compila. 'error[Exxx]' bloqueia o build; 'warning' nao bloqueia mas deve ser revisado.");

    if errors > 0 {
        res.push("Acao: 'cd .nemesis && cargo check --workspace' e corrija os error[Exxx].");
        res.status(CheckStatus::Fail)
    } else if warnings > 0 {
        res.status(CheckStatus::Warn)
    } else {
        res.push("cargo check --workspace: PASS (Finished).");
        res.status(CheckStatus::Ok)
    }
}
