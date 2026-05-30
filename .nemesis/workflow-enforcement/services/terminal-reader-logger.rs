use crate::services::terminal_reader_types::LogEntry;
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Mutex;

lazy_static::lazy_static! {
    static ref LOGS: Mutex<Vec<LogEntry>> = Mutex::new(Vec::new());
}

pub struct TerminalReaderLogger;

const MAX_LOGS: usize = 100;

impl TerminalReaderLogger {
    pub fn log(
        level: &str,
        operation: &str,
        file_path: &str,
        method: &str,
        success: bool,
        fallbacks: Vec<String>,
        duration: Option<u64>,
        error: Option<String>,
    ) {
        let os = Self::detect_os();
        let fallbacks_clone = fallbacks.clone();

        let entry = LogEntry {
            timestamp: Utc::now().to_rfc3339(),
            level: level.to_string(),
            operation: operation.to_string(),
            file_path: file_path.to_string(),
            method: method.to_string(),
            success,
            duration,
            error: error.clone(),
            fallbacks,
            os,
        };

        {
            let mut logs = LOGS.lock().unwrap();
            logs.push(entry);

            // Mantém apenas os logs mais recentes
            if logs.len() > MAX_LOGS {
                let excess = logs.len() - MAX_LOGS;
                logs.drain(0..excess);
            }
        }

        // Exibe logs no console
        let fallbacks_str = fallbacks_clone.join(" -> ");
        match level {
            "error" => {
                eprintln!("[ERROR] {} em {}: {}", operation, file_path, error.unwrap_or_default());
            }
            "warn" => {
                eprintln!("[WARN] {} em {}: {}", operation, file_path, fallbacks_str);
            }
            "debug" => {
                println!("[DEBUG] {} em {} via {}", operation, file_path, fallbacks_str);
            }
            _ => {
                println!("[INFO] {} em {} via {}", operation, file_path, method);
            }
        }
    }

    pub fn get_logs(level: Option<&str>) -> Vec<LogEntry> {
        let logs = LOGS.lock().unwrap();
        
        match level {
            Some(l) => logs.iter().filter(|log| log.level == l).cloned().collect(),
            None => logs.clone(),
        }
    }

    pub fn clear_logs() {
        let mut logs = LOGS.lock().unwrap();
        logs.clear();
    }

    pub fn export_logs() -> Vec<LogEntry> {
        let logs = LOGS.lock().unwrap();
        logs.clone()
    }

    pub fn get_recent_logs(count: usize) -> Vec<LogEntry> {
        let logs = LOGS.lock().unwrap();
        let start = logs.len().saturating_sub(count);
        logs[start..].to_vec()
    }

    fn detect_os() -> String {
        let platform = std::env::consts::OS;
        if platform == "macos" {
            "mac".to_string()
        } else if platform == "windows" {
            "windows".to_string()
        } else {
            "linux".to_string()
        }
    }
}
