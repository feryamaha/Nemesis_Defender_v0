//! Estado de sincronizacao incremental para modo --sync.

use crate::config;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SyncState {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub violations_last_ts: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub doctor_last_run_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pentest_last_run_at: Option<String>,
}

fn sync_state_path() -> std::path::PathBuf {
    config::telemetry_dir().join("sync-state.json")
}

pub fn load() -> Result<SyncState> {
    let path = sync_state_path();
    if !path.exists() {
        return Ok(SyncState::default());
    }
    let content = std::fs::read_to_string(&path)
        .with_context(|| format!("ler {}", path.display()))?;
    serde_json::from_str(&content).context("parsear sync-state.json")
}

pub fn save(state: &SyncState) -> Result<()> {
    let dir = config::telemetry_dir();
    std::fs::create_dir_all(&dir).context("criar diretorio telemetry")?;
    let path = sync_state_path();
    let json = serde_json::to_string_pretty(state).context("serializar sync-state")?;
    std::fs::write(&path, json).context("escrever sync-state.json")?;
    Ok(())
}
