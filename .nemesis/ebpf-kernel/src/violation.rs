use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::sync::Mutex;

lazy_static::lazy_static! {
    static ref VIOLATIONS: Mutex<Vec<Violation>> = Mutex::new(Vec::new());
}

const LOG_DIR: &str = "logs";
const LOG_FILE: &str = "logs/violations.log";

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
    fn ensure_log_directory() {
        if !Path::new(LOG_DIR).exists() {
            if let Err(e) = fs::create_dir_all(LOG_DIR) {
                eprintln!("Failed to create log directory: {}", e);
            }
        }
    }

    pub fn log_violation(violation: &Violation) {
        {
            let mut violations = VIOLATIONS.lock().unwrap();
            violations.push(violation.clone());
        }

        Self::write_to_file(violation);

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

    fn write_to_file(violation: &Violation) {
        Self::ensure_log_directory();
        let log_entry = Self::format_log_entry(violation);

        let mut file = match OpenOptions::new().append(true).create(true).open(LOG_FILE) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("Failed to write violation to log file: {}", e);
                return;
            }
        };

        if let Err(e) = writeln!(file, "{}", log_entry) {
            eprintln!("Failed to write violation to log file: {}", e);
        }
    }

    fn format_log_entry(violation: &Violation) -> String {
        let entry = serde_json::json!({
            "timestamp": violation.timestamp,
            "type": violation.violation_type,
            "message": violation.message,
            "rule": violation.rule.as_ref().unwrap_or(&"".to_string()),
            "command": violation.command.as_ref().unwrap_or(&"".to_string()),
            "llmModel": violation.llm_model.as_ref().unwrap_or(&"unknown".to_string()),
            "layer": violation.layer.as_ref().unwrap_or(&"unknown".to_string()),
        });
        entry.to_string()
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
