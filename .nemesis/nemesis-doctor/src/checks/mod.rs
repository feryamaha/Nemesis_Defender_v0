//! Orquestracao e helpers compartilhados dos checks.

pub mod compile;
pub mod daemon;
pub mod ebpf;
pub mod inventory;
pub mod pentest;
pub mod scaffold;
pub mod unit_tests;

use crate::report::CheckResult;
use std::path::PathBuf;

/// Raiz do projeto (dir que contem `.nemesis/`).
/// Derivada do binario: .nemesis/target/release/nemesis-doctor
pub fn project_root() -> PathBuf {
    if let Ok(exe) = std::env::current_exe() {
        if let Some(p) = exe
            .parent()
            .and_then(|r| r.parent())
            .and_then(|t| t.parent())
            .and_then(|n| n.parent())
        {
            return p.to_path_buf();
        }
    }
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

pub fn nemesis_dir() -> PathBuf {
    project_root().join(".nemesis")
}

/// Verifica se um comando existe no PATH via `<cmd> --version`.
pub fn command_exists(cmd: &str) -> bool {
    std::process::Command::new(cmd)
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Executa todos os checks na ordem definida.
pub fn run_all(quick: bool) -> Vec<CheckResult> {
    use crate::report::CheckStatus;
    let mut results = Vec::new();

    if !quick {
        results.push(compile::run());
        results.push(unit_tests::run());
    } else {
        results.push(
            CheckResult::new("G1 - Compilacao")
                .status(CheckStatus::Skip)
                .line("Pulado (--quick)."),
        );
        results.push(
            CheckResult::new("G2 - Testes unitarios")
                .status(CheckStatus::Skip)
                .line("Pulado (--quick)."),
        );
    }

    results.push(inventory::run());
    results.push(scaffold::run());
    results.push(ebpf::run());
    results.push(daemon::run());

    if !quick {
        results.push(pentest::run());
    } else {
        results.push(
            CheckResult::new("G7 - Pentest Red-Team")
                .status(CheckStatus::Skip)
                .line("Pulado (--quick)."),
        );
    }

    results
}
