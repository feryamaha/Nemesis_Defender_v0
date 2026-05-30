//! python_import_injection visitor — Vetor 7b: suspicious Python imports
//!
//! Detects malicious patterns in Python imports:
//! - import urllib.request + exec (downloads + executes code)
//! - from requests import Session + unknown server calls
//! - __init__.py importing from suspicious modules
//! - os.system / subprocess.Popen in import-time code

use crate::DefenderViolation;
use tree_sitter::Node;

const SUSPICIOUS_IMPORT_PATTERNS: &[&str] = &[
    "urllib.request",
    "urllib.urlopen",
    "requests.Session",
    "httpx",
    "socket",
    "paramiko",
];

const EXECUTION_INDICATORS: &[&str] = &[
    "os.system",
    "subprocess.Popen",
    "subprocess.call",
    "subprocess.run",
    "exec(",
    "eval(",
    "__import__(",
];

fn is_suspicious_url(text: &str) -> bool {
    // Check for suspicious host patterns in URLs
    let text_lower = text.to_lowercase();

    // Known malicious domain patterns from TrapDoor
    text_lower.contains("127.0.0.1") && text_lower.contains(":9999")
        || text_lower.contains("localhost:") && text_lower.contains("9999")
        || text_lower.contains(".pastebin.") && text_lower.contains("raw")
        || text_lower.contains(".github.io/")
}

pub fn visit_python_node(node: &Node, source: &str) -> Vec<DefenderViolation> {
    let mut violations = Vec::new();

    let node_text = node.utf8_text(source.as_bytes()).unwrap_or("");

    // Check for import statements with suspicious modules
    if node.kind() == "import_statement" {
        for pattern in SUSPICIOUS_IMPORT_PATTERNS {
            if node_text.contains(pattern) {
                // Look for execution indicators in the same scope
                let parent = node.parent();
                if let Some(p) = parent {
                    let parent_text = p.utf8_text(source.as_bytes()).unwrap_or("");

                    for exec_indicator in EXECUTION_INDICATORS {
                        if parent_text.contains(exec_indicator) {
                            violations.push(DefenderViolation {
                                visitor: "python_import_injection".to_string(),
                                line: (node.start_position().row + 1) as u32,
                                col: (node.start_position().column + 1) as u32,
                                evidence: format!("import {} + {}", pattern, exec_indicator),
                                decoded: None,
                                message: format!(
                                    "Python imports {} and executes code. \
                                     __init__.py import-time code execution (supply chain attack vector).",
                                    pattern
                                ),
                                suggestion: Some("Remove network operations from import-time code. Load and execute code only when needed, not at import.".to_string()),
                            });
                            return violations;
                        }
                    }
                }
            }
        }
    }

    // Check for from...import with suspicious modules
    if node.kind() == "import_from_statement" {
        if node_text.contains("import") {
            for pattern in SUSPICIOUS_IMPORT_PATTERNS {
                if node_text.contains(pattern) {
                    violations.push(DefenderViolation {
                        visitor: "python_import_injection".to_string(),
                        line: (node.start_position().row + 1) as u32,
                        col: (node.start_position().column + 1) as u32,
                        evidence: node_text.to_string(),
                        decoded: None,
                        message: format!(
                            "Suspicious import of {}. \
                             Potential network-based supply chain attack in module initialization.",
                            pattern
                        ),
                        suggestion: Some("Verify that imports do not execute remote code. Use static imports only.".to_string()),
                    });
                    break;
                }
            }
        }
    }

    // Check for function calls that create network connections
    if node.kind() == "call" {
        if node_text.contains("socket.socket") || node_text.contains("urllib.request.urlopen") {
            if is_suspicious_url(node_text) {
                violations.push(DefenderViolation {
                    visitor: "python_import_injection".to_string(),
                    line: (node.start_position().row + 1) as u32,
                    col: (node.start_position().column + 1) as u32,
                    evidence: node_text.to_string(),
                    decoded: None,
                    message: "Python module makes suspicious network connection at import time.".to_string(),
                    suggestion: Some("Network operations must not occur at import time. Use lazy loading or explicit function calls.".to_string()),
                });
            }
        }
    }

    violations
}
