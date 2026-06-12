#[cfg(target_os = "linux")]
pub mod cgroup;
pub mod config;
#[cfg(target_os = "linux")]
pub mod landlock;
#[cfg(target_os = "linux")]
pub mod egress;
#[cfg(target_os = "linux")]
pub mod loader;
pub mod logger;
#[cfg(target_os = "linux")]
pub mod seccomp;
pub mod transport;
pub mod violation;

use bytemuck::{Pod, Zeroable};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EbpfRuntimeStatus {
    Enabled,
    Disabled,
    Unsupported,
    Error,
}

impl EbpfRuntimeStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Enabled => "enabled",
            Self::Disabled => "disabled",
            Self::Unsupported => "unsupported",
            Self::Error => "error",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EbpfEventKind {
    CommandBlocked,
    WritePathBlocked,
    EgressBlocked,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EbpfBlockEvent {
    pub pid: u32,
    pub tgid: u32,
    pub kind: EbpfEventKind,
    pub subject: String,
    pub decision: String,
    pub timestamp: String,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct KernelCommandKey {
    pub name: [u8; 32],
}

impl KernelCommandKey {
    pub fn from_command(command: &str) -> Self {
        let mut key = Self { name: [0; 32] };
        let bytes = command.as_bytes();
        let len = bytes.len().min(31);
        key.name[..len].copy_from_slice(&bytes[..len]);
        key
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct KernelEvent {
    pub pid: u32,
    pub tgid: u32,
    pub kind: u32,
    pub subject: [u8; 256],
    pub decision: [u8; 32],
    pub timestamp_ns: u64,
}

impl KernelEvent {
    pub const BYTE_SIZE: usize = 308;

    pub fn from_bytes(data: &[u8]) -> Option<Self> {
        if data.len() < Self::BYTE_SIZE {
            return None;
        }

        let pid = u32::from_ne_bytes(data[0..4].try_into().ok()?);
        let tgid = u32::from_ne_bytes(data[4..8].try_into().ok()?);
        let kind = u32::from_ne_bytes(data[8..12].try_into().ok()?);

        let mut subject = [0u8; 256];
        subject.copy_from_slice(&data[12..268]);

        let mut decision = [0u8; 32];
        decision.copy_from_slice(&data[268..300]);

        let timestamp_ns = u64::from_ne_bytes(data[300..308].try_into().ok()?);

        Some(Self {
            pid,
            tgid,
            kind,
            subject,
            decision,
            timestamp_ns,
        })
    }

    pub fn subject_string(&self) -> String {
        c_buf_to_string(&self.subject)
    }

    pub fn decision_string(&self) -> String {
        c_buf_to_string(&self.decision)
    }
}

fn c_buf_to_string(buf: &[u8]) -> String {
    let end = buf.iter().position(|b| *b == 0).unwrap_or(buf.len());
    String::from_utf8_lossy(&buf[..end]).to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EbpfHealthResponse {
    pub status: EbpfRuntimeStatus,
    pub platform: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EbpfDoctorReport {
    pub platform: String,
    pub kernel_release: String,
    pub config_enabled: bool,
    pub active_lsms: Vec<String>,
    pub kernel_supports_bpf_lsm: bool,
    pub bpf_lsm_active: bool,
    pub unprivileged_bpf_disabled: Option<i32>,
    pub has_btf: bool,
    pub has_clang: bool,
    pub has_bpftool: bool,
    pub is_root: bool,
    pub has_cap_bpf: bool,
    pub has_cap_perfmon: bool,
    pub has_cap_sys_admin: bool,
    pub recommended_backend: String,
    pub can_attempt_load: bool,
    pub blocking_constraints: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EbpfSelfTestResult {
    pub command: String,
    pub attempted: bool,
    pub blocked: bool,
    pub status: Option<i32>,
    pub stderr: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum KernelLayerStatus {
    BpfLsmActive,
    LandlockActive,
    PretoolOnly,
    Unsupported,
    Error(String),
}
