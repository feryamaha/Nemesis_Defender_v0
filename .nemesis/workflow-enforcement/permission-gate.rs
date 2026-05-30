use crate::types::PermissionRequest;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

lazy_static::lazy_static! {
    static ref PENDING_REQUESTS: Mutex<Vec<PermissionRequest>> = Mutex::new(Vec::new());
    static ref GRANTED_PERMISSIONS: Mutex<HashSet<String>> = Mutex::new(HashSet::new());
    static ref DENIED_PERMISSIONS: Mutex<HashSet<String>> = Mutex::new(HashSet::new());
    static ref WORKFLOW_PERMISSION: Mutex<Option<WorkflowPermissionRecord>> = Mutex::new(None);
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WorkflowPermissionState {
    #[serde(rename = "idle")]
    Idle,
    #[serde(rename = "awaiting")]
    Awaiting,
    #[serde(rename = "granted")]
    Granted,
    #[serde(rename = "denied")]
    Denied,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowPermissionRecord {
    pub workflow_name: String,
    pub plan_summary: String,
    pub requested_at: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolved_at: Option<i64>,
    pub state: WorkflowPermissionState,
}

#[derive(Debug, Clone)]
pub struct PermissionCheckResult {
    pub allowed: bool,
    pub reason: String,
}

#[derive(Debug, Clone)]
pub struct InterceptResult {
    pub allowed: bool,
    pub reason: String,
    pub risk_level: RiskLevel,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    BlockedByWorkflow,
}

#[derive(Debug, Clone)]
pub struct CommandSafetyResult {
    pub is_safe: bool,
    pub risk_level: RiskLevel,
    pub reasons: Vec<String>,
}

const FILE_MODIFICATION_PATTERNS: &[&str] = &[
    r"\bwrite\b",
    r"\bedit\b",
    r"Set-Content",
    r"New-Item",
    r"Out-File",
    r"fs\.writeFile",
    r"fs\.appendFile",
    r">\s*[^/dev/null]",
];

const EXPLICIT_AUTHORIZATION_TOKENS: &[&str] = &[
    "sim", "yes", "pode", "prossiga", "confirmo", "autorizo", "execute", "ok", "proceed",
];

fn runtime_dir() -> PathBuf {
    std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(".nemesis")
        .join("runtime")
}

fn state_file() -> PathBuf {
    runtime_dir().join("permission-gate.state.json")
}

fn persist_state(record: Option<&WorkflowPermissionRecord>) {
    let runtime_dir = runtime_dir();
    let state_file = state_file();

    if let Err(e) = fs::create_dir_all(&runtime_dir) {
        eprintln!("[PERMISSION GATE] WARNING: Failed to create runtime directory: {}", e);
        return;
    }

    let payload = match record {
        Some(r) => serde_json::to_string_pretty(r),
        None => serde_json::to_string_pretty(&serde_json::json!({
            "workflowName": "none",
            "planSummary": "none",
            "requestedAt": 0,
            "state": "idle"
        })),
    };

    match payload {
        Ok(json) => {
            if let Err(e) = fs::write(&state_file, json) {
                eprintln!("[PERMISSION GATE] WARNING: Failed to persist state to disk: {}", e);
                eprintln!("[PERMISSION GATE] In-memory state remains authoritative.");
            }
        }
        Err(e) => {
            eprintln!("[PERMISSION GATE] WARNING: Failed to serialize state: {}", e);
        }
    }
}

fn load_persisted_state() -> Option<WorkflowPermissionRecord> {
    let state_file = state_file();

    if !state_file.exists() {
        let initial_state = WorkflowPermissionRecord {
            workflow_name: "none".to_string(),
            plan_summary: "none".to_string(),
            requested_at: 0,
            resolved_at: None,
            state: WorkflowPermissionState::Idle,
        };
        persist_state(Some(&initial_state));
        return None;
    }

    match fs::read_to_string(&state_file) {
        Ok(raw) => {
            match serde_json::from_str::<WorkflowPermissionRecord>(&raw) {
                Ok(parsed) => {
                    if parsed.state == WorkflowPermissionState::Idle || parsed.workflow_name == "none" {
                        None
                    } else {
                        Some(parsed)
                    }
                }
                Err(_) => None,
            }
        }
        Err(_) => None,
    }
}

pub struct PermissionGate;

impl PermissionGate {
    // Initialize static state from disk
    pub fn initialize() {
        if let Some(state) = load_persisted_state() {
            let mut workflow_perm = WORKFLOW_PERMISSION.lock().unwrap();
            *workflow_perm = Some(state);
        }
    }

    pub fn open_permission_gate(workflow_name: &str, plan_summary: &str) {
        let record = WorkflowPermissionRecord {
            workflow_name: workflow_name.to_string(),
            plan_summary: plan_summary.to_string(),
            requested_at: chrono::Utc::now().timestamp_millis(),
            resolved_at: None,
            state: WorkflowPermissionState::Awaiting,
        };

        {
            let mut workflow_perm = WORKFLOW_PERMISSION.lock().unwrap();
            *workflow_perm = Some(record.clone());
        }

        persist_state(Some(&record));

        println!("[PERMISSION GATE] STATE -> awaiting");
        println!("[PERMISSION GATE] Workflow: {}", workflow_name);
        println!("[PERMISSION GATE] Plan: {}", plan_summary);
        println!("[PERMISSION GATE] All file modifications BLOCKED until user sends explicit YES.");
        println!("[PERMISSION GATE] State persisted to: {:?}", state_file());
    }

    pub fn resolve_permission_gate(user_message: &str) -> &'static str {
        let workflow_perm = WORKFLOW_PERMISSION.lock().unwrap();
        
        let current_state = match workflow_perm.as_ref() {
            Some(wp) if wp.state == WorkflowPermissionState::Awaiting => wp.clone(),
            _ => return "denied",
        };
        drop(workflow_perm);

        let normalized = user_message.trim().to_lowercase();
        let is_explicit_yes = EXPLICIT_AUTHORIZATION_TOKENS.iter().any(|token| {
            normalized == *token || normalized.starts_with(&format!("{} ", token))
        });

        if is_explicit_yes {
            let mut workflow_perm = WORKFLOW_PERMISSION.lock().unwrap();
            if let Some(ref mut wp) = workflow_perm.as_mut() {
                wp.state = WorkflowPermissionState::Granted;
                wp.resolved_at = Some(chrono::Utc::now().timestamp_millis());
                let updated = wp.clone();
                persist_state(Some(&updated));
                
                println!("[PERMISSION GATE] STATE -> granted");
                println!("[PERMISSION GATE] Authorization GRANTED for: {}", updated.workflow_name);
                println!("[PERMISSION GATE] State persisted to: {:?}", state_file());
            }
            return "granted";
        }

        eprintln!("[PERMISSION GATE] STATE -> still awaiting");
        eprintln!("[PERMISSION GATE] Message \"{}\" is NOT an explicit authorization.", user_message);
        eprintln!("[PERMISSION GATE] File modifications remain BLOCKED.");
        eprintln!("[PERMISSION GATE] Valid tokens: {}", EXPLICIT_AUTHORIZATION_TOKENS.join(" | "));
        "ambiguous"
    }

    pub fn can_modify_file(_command_context: &str) -> PermissionCheckResult {
        let disk_state = load_persisted_state();
        
        {
            let mut workflow_perm = WORKFLOW_PERMISSION.lock().unwrap();
            if let Some(current) = workflow_perm.as_mut() {
                if let Some(ref disk) = disk_state {
                    if disk.state != current.state {
                        println!("[PERMISSION GATE] State re-synced from disk: {:?}", disk.state);
                        *current = disk.clone();
                    }
                }
            }
        }

        let workflow_perm = WORKFLOW_PERMISSION.lock().unwrap();
        
        let result = match workflow_perm.as_ref() {
            None => PermissionCheckResult {
                allowed: false,
                reason: "No active workflow permission context. File modifications require workflow-main Step 7 authorization.".to_string(),
            },
            Some(wp) => match wp.state {
                WorkflowPermissionState::Awaiting => PermissionCheckResult {
                    allowed: false,
                    reason: format!("Permission gate is AWAITING explicit user authorization. Workflow: {}. Do not proceed until user sends explicit YES.", wp.workflow_name),
                },
                WorkflowPermissionState::Denied => PermissionCheckResult {
                    allowed: false,
                    reason: format!("Permission was DENIED for workflow: {}.", wp.workflow_name),
                },
                WorkflowPermissionState::Granted => PermissionCheckResult {
                    allowed: true,
                    reason: format!("Permission GRANTED for workflow: {}.", wp.workflow_name),
                },
                WorkflowPermissionState::Idle => PermissionCheckResult {
                    allowed: false,
                    reason: "Unknown permission state. Defaulting to blocked.".to_string(),
                },
            },
        };
        
        result
    }

    pub fn is_file_modification_command(command: &str) -> bool {
        FILE_MODIFICATION_PATTERNS.iter().any(|pattern| {
            Regex::new(pattern).map(|re| re.is_match(command)).unwrap_or(false)
        })
    }

    pub fn intercept_command(command: &str) -> InterceptResult {
        if Self::is_file_modification_command(command) {
            let workflow_check = Self::can_modify_file(command);
            if !workflow_check.allowed {
                return InterceptResult {
                    allowed: false,
                    reason: workflow_check.reason,
                    risk_level: RiskLevel::BlockedByWorkflow,
                };
            }
        }

        let safety_check = Self::check_command_safety(command);
        if !safety_check.is_safe {
            return InterceptResult {
                allowed: false,
                reason: safety_check.reasons.join(", "),
                risk_level: safety_check.risk_level,
            };
        }

        InterceptResult {
            allowed: true,
            reason: "Command passed workflow permission gate and safety validation.".to_string(),
            risk_level: RiskLevel::Low,
        }
    }

    pub fn close_permission_gate() {
        {
            let workflow_perm = WORKFLOW_PERMISSION.lock().unwrap();
            if let Some(ref wp) = workflow_perm.as_ref() {
                println!("[PERMISSION GATE] STATE -> idle");
                println!("[PERMISSION GATE] Gate closed for workflow: {}", wp.workflow_name);
            }
        }
        
        {
            let mut workflow_perm = WORKFLOW_PERMISSION.lock().unwrap();
            *workflow_perm = None;
        }
        
        persist_state(None);
        println!("[PERMISSION GATE] State cleared from disk: {:?}", state_file());
    }

    pub fn get_workflow_permission_state() -> Option<WorkflowPermissionRecord> {
        let workflow_perm = WORKFLOW_PERMISSION.lock().unwrap();
        workflow_perm.clone()
    }

    // Original command safety gate
    pub async fn request_permission(request: &PermissionRequest) -> bool {
        let request_key = Self::generate_request_key(request);

        {
            let granted = GRANTED_PERMISSIONS.lock().unwrap();
            if granted.contains(&request_key) {
                return true;
            }
        }

        {
            let denied = DENIED_PERMISSIONS.lock().unwrap();
            if denied.contains(&request_key) {
                return false;
            }
        }

        {
            let mut pending = PENDING_REQUESTS.lock().unwrap();
            pending.push(request.clone());
        }

        let is_safe = Self::is_safe_command(&request.command);

        if is_safe && !request.requires_confirmation {
            let mut granted = GRANTED_PERMISSIONS.lock().unwrap();
            granted.insert(request_key);
            return true;
        }

        if request.requires_confirmation {
            eprintln!("Permission required for command: {}", request.command);
            eprintln!("Reason: {}", request.reason);
            eprintln!("Workflow: {}", request.workflow);
            
            let mut denied = DENIED_PERMISSIONS.lock().unwrap();
            denied.insert(request_key);
            return false;
        }

        false
    }

    fn generate_request_key(request: &PermissionRequest) -> String {
        format!("{}:{}", request.workflow, request.command)
    }

    fn is_safe_command(command: &str) -> bool {
        let dangerous_patterns: Vec<Regex> = vec![
            Regex::new(r"rm\s+-rf").unwrap(),
            Regex::new(r"sudo\s+").unwrap(),
            Regex::new(r"format\s+").unwrap(),
            Regex::new(r"dd\s+if=").unwrap(),
            Regex::new(r">\s*/dev/null").unwrap(),
            Regex::new(r"kill\s+-9").unwrap(),
            Regex::new(r"shutdown").unwrap(),
            Regex::new(r"reboot").unwrap(),
            Regex::new(r"passwd").unwrap(),
            Regex::new(r"chmod\s+777").unwrap(),
            Regex::new(r"chown\s+root").unwrap(),
        ];

        let safe_patterns: Vec<Regex> = vec![
            Regex::new(r"^ls\s").unwrap(),
            Regex::new(r"^cat\s").unwrap(),
            Regex::new(r"^echo\s").unwrap(),
            Regex::new(r"^mkdir\s").unwrap(),
            Regex::new(r"^touch\s").unwrap(),
            Regex::new(r"^cp\s").unwrap(),
            Regex::new(r"^mv\s").unwrap(),
            Regex::new(r"^grep\s").unwrap(),
            Regex::new(r"^find\s").unwrap(),
            Regex::new(r"^npm\s+(install|list|info)").unwrap(),
            Regex::new(r"^yarn\s+(install|list|info)").unwrap(),
        ];

        for pattern in &dangerous_patterns {
            if pattern.is_match(command) {
                return false;
            }
        }

        for pattern in &safe_patterns {
            if pattern.is_match(command) {
                return true;
            }
        }

        false
    }

    pub fn create_permission_request(
        command: &str,
        reason: &str,
        workflow: &str,
        requires_confirmation: bool,
    ) -> PermissionRequest {
        PermissionRequest {
            command: command.to_string(),
            reason: reason.to_string(),
            workflow: workflow.to_string(),
            requires_confirmation,
        }
    }

    pub fn get_pending_requests() -> Vec<PermissionRequest> {
        let pending = PENDING_REQUESTS.lock().unwrap();
        pending.clone()
    }

    pub fn clear_pending_requests() {
        let mut pending = PENDING_REQUESTS.lock().unwrap();
        pending.clear();
    }

    pub fn grant_permission(request: &PermissionRequest) {
        let request_key = Self::generate_request_key(request);
        let mut granted = GRANTED_PERMISSIONS.lock().unwrap();
        granted.insert(request_key);
    }

    pub fn deny_permission(request: &PermissionRequest) {
        let request_key = Self::generate_request_key(request);
        let mut denied = DENIED_PERMISSIONS.lock().unwrap();
        denied.insert(request_key);
    }

    pub fn reset() {
        {
            let mut pending = PENDING_REQUESTS.lock().unwrap();
            pending.clear();
        }
        {
            let mut granted = GRANTED_PERMISSIONS.lock().unwrap();
            granted.clear();
        }
        {
            let mut denied = DENIED_PERMISSIONS.lock().unwrap();
            denied.clear();
        }
        {
            let mut workflow_perm = WORKFLOW_PERMISSION.lock().unwrap();
            *workflow_perm = None;
        }
        persist_state(None);
    }

    pub fn check_command_safety(command: &str) -> CommandSafetyResult {
        let mut reasons: Vec<String> = Vec::new();
        let mut risk_level = RiskLevel::Low;

        if Regex::new(r"rm\s+").unwrap().is_match(command) {
            reasons.push("File deletion operation".to_string());
            risk_level = RiskLevel::Medium;
        }

        if Regex::new(r"sudo\s+").unwrap().is_match(command) {
            reasons.push("Privileged operation requiring sudo".to_string());
            risk_level = RiskLevel::High;
        }

        if Regex::new(r"systemctl|service|shutdown|reboot").unwrap().is_match(command) {
            reasons.push("System-level operation".to_string());
            risk_level = RiskLevel::High;
        }

        if Regex::new(r"curl|wget|nc|netcat").unwrap().is_match(command) {
            reasons.push("Network operation".to_string());
            risk_level = RiskLevel::Medium;
        }

        if Regex::new(r"npm\s+(uninstall|publish)|yarn\s+(remove|publish)").unwrap().is_match(command) {
            reasons.push("Package management operation".to_string());
            risk_level = RiskLevel::Medium;
        }

        CommandSafetyResult {
            is_safe: matches!(risk_level, RiskLevel::Low),
            risk_level,
            reasons,
        }
    }
}
