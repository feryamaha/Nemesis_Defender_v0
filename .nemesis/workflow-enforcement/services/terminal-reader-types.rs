use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encoding: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadResult {
    pub content: String,
    pub method: String,
    pub fallbacks: Vec<String>,
    pub duration: u64,
    pub success: bool,
    pub os: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub lines: Vec<String>,
    pub method: String,
    pub fallbacks: Vec<String>,
    pub success: bool,
    pub os: String,
    pub duration: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathValidation {
    pub is_project_root: bool,
    pub is_within_project: bool,
    pub is_git_ignored: bool,
    pub normalized_path: String,
    pub security_level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub operation: String,
    pub file_path: String,
    pub method: String,
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    pub fallbacks: Vec<String>,
    pub os: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalCommand {
    pub command: String,
    pub description: String,
    pub alternatives: Vec<String>,
    pub available: bool,
}
