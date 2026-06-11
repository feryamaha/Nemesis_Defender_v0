//! Unified block ledger — `.nemesis/logs/nemesis-violations.log` (JSONL).
//!
//! TODO bloqueio, de QUALQUER camada (pretool, posttool, nemesis-defender, ebpf-kernel),
//! grava UMA linha padronizada aqui. Schema (a spec do mantenedor):
//!   { "ts", "date", "time", "layer", "message" }
//! - `layer`   ∈ pretool | posttool | nemesis-defender | ebpf-kernel
//! - `message` = a mensagem PADRÃO de bloqueio (vocabulário das 6 mensagens
//!               `NEMESIS SEC ...` / `NEMESIS QUALITY ...`), já carregando o alvo (`· <alvo>`).
//!
//! O caminho é resolvido de forma ABSOLUTA (sobe do executável até `.nemesis/`), então
//! independe do CWD do processo — acaba com a dispersão `logs/` (raiz) vs `.nemesis/logs/`.
//! eBPF é Linux-only e tem cópia própria deste helper (crate desacoplado).

use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

/// Resolve `.nemesis/logs/nemesis-violations.log` subindo do path do binário até `.nemesis/`.
/// Fallback: relativo ao CWD (`.nemesis/logs/...`).
pub fn ledger_path() -> PathBuf {
    if let Ok(exe) = std::env::current_exe() {
        for anc in exe.ancestors() {
            if anc.file_name().map(|n| n == ".nemesis").unwrap_or(false) {
                return anc.join("logs").join("nemesis-violations.log");
            }
        }
    }
    PathBuf::from(".nemesis/logs/nemesis-violations.log")
}

/// Anexa um evento de bloqueio padronizado ao ledger unificado.
/// Best-effort: nunca paniqueia nem propaga erro (logar não pode quebrar o enforcement).
pub fn append(layer: &str, message: &str) {
    let now = chrono::Local::now();
    let entry = serde_json::json!({
        "ts": now.to_rfc3339(),
        "date": now.format("%Y-%m-%d").to_string(),
        "time": now.format("%H:%M:%S").to_string(),
        "layer": layer,
        "message": message,
    });
    let path = ledger_path();
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(&path) {
        let _ = writeln!(f, "{}", entry);
    }
}
