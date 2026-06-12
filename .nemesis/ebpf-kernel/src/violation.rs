use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::sync::Mutex;

lazy_static::lazy_static! {
    static ref VIOLATIONS: Mutex<Vec<Violation>> = Mutex::new(Vec::new());
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ViolationType {
    PermissionDenied,
    RuleViolation,
    SyntaxError,
    SecurityViolation,
    GateViolation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Violation {
    #[serde(rename = "type")]
    pub violation_type: ViolationType,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rule: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    pub timestamp: String,
    #[serde(rename = "llmModel", skip_serializing_if = "Option::is_none")]
    pub llm_model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub layer: Option<String>,
}

pub struct ViolationLogger;

impl ViolationLogger {
    pub fn log_violation(violation: &Violation) {
        {
            let mut violations = VIOLATIONS.lock().unwrap();
            violations.push(violation.clone());
        }

        // Ledger ÚNICO: o arquivo legado `logs/violations.log` (relativo ao cwd) foi REMOVIDO.
        // Toda violação vai apenas para o ledger unificado `.nemesis/logs/nemesis-violations.log`.

        // Ledger unificado (.nemesis/logs/nemesis-violations.log): normaliza para o
        // vocabulário das 6 mensagens. Bloqueio de comando do kernel → COMANDO NAO PERMITIDO.
        let unified_msg = match violation.command {
            Some(ref cmd) => format!("NEMESIS SEC - COMANDO NAO PERMITIDO · {}", cmd),
            None => violation.message.clone(),
        };
        append_unified_ledger("ebpf-kernel", &unified_msg);

        eprintln!("[VIOLATION] {:?}: {}", violation.violation_type, violation.message);
        if let Some(ref rule) = violation.rule {
            eprintln!("  Rule: {}", rule);
        }
        if let Some(ref command) = violation.command {
            eprintln!("  Command: {}", command);
        }
        if let Some(ref layer) = violation.layer {
            eprintln!("  Layer: {}", layer);
        }
        eprintln!("  Timestamp: {}", violation.timestamp);
        eprintln!();
    }

}

/// Anexa um evento de bloqueio ao ledger unificado `.nemesis/logs/nemesis-violations.log`.
/// Cópia local (o crate ebpf-kernel é desacoplado do nemesis-defender). Mesmo schema:
/// {ts, date, time, layer, message}. Path resolvido subindo do binário até `.nemesis/`.
fn append_unified_ledger(layer: &str, message: &str) {
    use std::path::PathBuf;
    let ledger: PathBuf = std::env::current_exe()
        .ok()
        .and_then(|exe| {
            exe.ancestors()
                .find(|a| a.file_name().map(|n| n == ".nemesis").unwrap_or(false))
                .map(|nem| nem.join("logs").join("nemesis-violations.log"))
        })
        .unwrap_or_else(|| PathBuf::from(".nemesis/logs/nemesis-violations.log"));

    let now = chrono::Local::now();
    let entry = serde_json::json!({
        "ts": now.to_rfc3339(),
        "date": now.format("%Y-%m-%d").to_string(),
        "time": now.format("%H:%M:%S").to_string(),
        "layer": layer,
        "message": message,
    });
    if let Some(dir) = ledger.parent() {
        let _ = fs::create_dir_all(dir);
    }
    if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(&ledger) {
        let _ = writeln!(f, "{}", entry);
    }
}
