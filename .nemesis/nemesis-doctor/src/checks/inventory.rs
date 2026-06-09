use crate::checks::nemesis_dir;
use crate::report::{CheckResult, CheckStatus};

const EXPECTED_BINARIES: &[&str] = &[
    "nemesis-pretool-check",
    "nemesis-pretool-check-unix",
    "nemesis-pretool-check-windows",
    "nemesis-pretool-hook",
    "nemesis-posttool-check-unix",
    "pre-edit-hook",
    "debug-hook-env",
    "nemesis-lsp",
    "nemesis-defender",
    "nemesis-ebpf-daemon",
    "nemesis-cgroup-watcher",
];

pub fn run() -> CheckResult {
    let mut res = CheckResult::new("G3 - Inventario target/release");
    let release = nemesis_dir().join("target").join("release");

    if !release.is_dir() {
        res.push(format!("Diretorio nao encontrado: {}", release.display()));
        res.push("Acao: 'cd .nemesis && cargo build --release --workspace'.");
        return res.status(CheckStatus::Fail);
    }

    let mut missing = Vec::new();
    for bin in EXPECTED_BINARIES {
        let exists =
            release.join(bin).exists() || release.join(format!("{}.exe", bin)).exists();
        if exists {
            res.push(format!("OK    {}", bin));
        } else {
            res.push(format!("FALTA {}", bin));
            missing.push(*bin);
        }
    }

    if missing.is_empty() {
        res.push("Todos os binarios esperados presentes.");
        res.status(CheckStatus::Ok)
    } else {
        res.push(format!(
            "Faltando {} binario(s). Acao: 'cargo build --release --workspace'.",
            missing.len()
        ));
        res.status(CheckStatus::Fail)
    }
}
