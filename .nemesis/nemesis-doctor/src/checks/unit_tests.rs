use crate::checks::{command_exists, nemesis_dir};
use crate::report::{CheckResult, CheckStatus};

pub fn run() -> CheckResult {
    let mut res = CheckResult::new("G2 - Testes unitarios (cargo test --workspace)");

    if !command_exists("cargo") {
        res.push("cargo nao encontrado no PATH - pulando testes.");
        return res.status(CheckStatus::Skip);
    }

    let output = std::process::Command::new("cargo")
        .args(["test", "--workspace"])
        .current_dir(nemesis_dir())
        .output();

    let output = match output {
        Ok(o) => o,
        Err(e) => {
            res.push(format!("Falha ao executar cargo test: {}", e));
            return res.status(CheckStatus::Fail);
        }
    };

    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let mut passed = 0usize;
    let mut failed = 0usize;
    for line in combined.lines() {
        let l = line.trim();
        if l.starts_with("test result:") {
            // Format: "test result: ok. 99 passed; 0 failed; ..."
            // Extract numbers before "passed" and "failed"
            if let Some(n_str) = l.split("passed").next().and_then(|s| s.trim().rsplit(' ').next()) {
                if let Ok(n) = n_str.parse::<usize>() {
                    passed += n;
                }
            }
            if let Some(n_str) = l.split("failed").next().and_then(|s| s.trim().rsplit(' ').next()) {
                if let Ok(n) = n_str.parse::<usize>() {
                    failed += n;
                }
            }
        }
    }

    res.push(format!("Passed: {} | Failed: {}", passed, failed));
    res.push("Como ler: 'test result: ok' por suite = verde. Qualquer 'failed' > 0 = regressao.");

    if !output.status.success() || failed > 0 {
        res.push("Acao: 'cd .nemesis && cargo test --workspace' e investigue os FAILED.");
        res.status(CheckStatus::Fail)
    } else if passed == 0 {
        res.push("Nenhum teste unitario encontrado no workspace.");
        res.status(CheckStatus::Warn)
    } else {
        res.status(CheckStatus::Ok)
    }
}
