use crate::checks::{command_exists, nemesis_dir};
use crate::report::{CheckResult, CheckStatus};

pub fn run() -> CheckResult {
    let mut res = CheckResult::new(
        "G2 - Testes unitarios (cargo test --release --workspace)",
        "G2 - Unit tests (cargo test --release --workspace)",
    );

    if !command_exists("cargo") {
        res.push(
            "cargo nao encontrado no PATH - pulando testes.",
            "cargo not found in PATH - skipping tests.",
        );
        return res.status(CheckStatus::Skip);
    }

    // --release e obrigatorio: o crate nemesis-ebpf-kernel depende de libbpf-sys, cujo build
    // nativo (make vendored) FALHA no perfil debug neste toolchain mas compila no release.
    // Sem --release, o G2 daria falso-negativo e nao rodaria os testes do egress.
    let output = std::process::Command::new("cargo")
        .args(["test", "--release", "--workspace"])
        .current_dir(nemesis_dir())
        .output();

    let output = match output {
        Ok(o) => o,
        Err(e) => {
            res.push(
                format!("Falha ao executar cargo test: {}", e),
                format!("Failed to run cargo test: {}", e),
            );
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

    res.push(
        format!("Passed: {} | Failed: {}", passed, failed),
        format!("Passed: {} | Failed: {}", passed, failed),
    );
    res.push(
        "Como ler: 'test result: ok' por suite = verde. Qualquer 'failed' > 0 = regressao.",
        "How to read: 'test result: ok' per suite = green. Any 'failed' > 0 = regression.",
    );

    if !output.status.success() || failed > 0 {
        res.push(
            "Acao: 'cd .nemesis && cargo test --release --workspace' e investigue os FAILED.",
            "Action: 'cd .nemesis && cargo test --release --workspace' and investigate the FAILED.",
        );
        res.status(CheckStatus::Fail)
    } else if passed == 0 {
        res.push(
            "Nenhum teste unitario encontrado no workspace.",
            "No unit tests found in the workspace.",
        );
        res.status(CheckStatus::Warn)
    } else {
        res.status(CheckStatus::Ok)
    }
}
