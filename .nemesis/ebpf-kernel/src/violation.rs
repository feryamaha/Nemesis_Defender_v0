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
