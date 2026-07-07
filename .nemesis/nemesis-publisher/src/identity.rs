//! Identidade e tokens do publisher: geracao, persistencia, carga.

use crate::config;
use anyhow::{Context, Result};
use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Identity {
    pub install_id: String,
    pub alias: String,
    pub project_token_hash: String,
    pub ingest_token: String,
    pub ingest_token_hash: String,
    pub opt_in: bool,
    pub environment: String,
    pub created_at: String,
    pub registered_at: Option<String>,
}

/// Gera 32 bytes aleatorios como string hex (64 chars).
fn random_hex_32() -> String {
    let bytes: [u8; 32] = rand::thread_rng().gen();
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

/// Calcula SHA-256 de uma string e retorna hex (64 chars).
pub fn sha256_hex(input: &str) -> String {
    let hash = Sha256::digest(input.as_bytes());
    hash.iter().map(|b| format!("{:02x}", b)).collect()
}

/// Gera uma nova identidade com opt_in.
pub fn create_identity() -> Identity {
    let install_id = uuid::Uuid::new_v4().to_string();
    let alias = format!(
        "inst-{}",
        install_id.replace('-', "").chars().take(4).collect::<String>()
    );
    let project_token = random_hex_32();
    let ingest_token = random_hex_32();
    let project_token_hash = sha256_hex(&project_token);
    let ingest_token_hash = sha256_hex(&ingest_token);
    let created_at = chrono::Local::now().to_rfc3339();
    let environment = crate::config::environment();

    Identity {
        install_id,
        alias,
        project_token_hash,
        ingest_token,
        ingest_token_hash,
        opt_in: true,
        environment,
        created_at,
        registered_at: None,
    }
}

/// Salva identidade em `.nemesis/telemetry/identity.json` com permissao 0600.
pub fn save(identity: &Identity) -> Result<()> {
    let dir = config::telemetry_dir();
    fs::create_dir_all(&dir).context("criar diretorio telemetry")?;
    let path = config::identity_path();
    let json = serde_json::to_string_pretty(identity).context("serializar identity")?;
    fs::write(&path, json).context("escrever identity.json")?;
    set_permissions_0600(&path);
    Ok(())
}

/// Carrega identidade de `.nemesis/telemetry/identity.json`.
pub fn load() -> Result<Identity> {
    let path = config::identity_path();
    let content = fs::read_to_string(&path)
        .with_context(|| format!("ler {}", path.display()))?;
    serde_json::from_str(&content).context("parsear identity.json")
}

/// Verifica se identity.json existe.
pub fn exists() -> bool {
    config::identity_path().exists()
}

/// Atualiza um campo da identidade e persiste.
pub fn update<F: FnOnce(&mut Identity)>(f: F) -> Result<()> {
    let mut id = load()?;
    f(&mut id);
    save(&id)
}

/// Seta permissao 0600 (owner-only read/write) em Unix.
fn set_permissions_0600(path: &Path) {
    #[cfg(unix)]
    {
        if let Ok(metadata) = fs::metadata(path) {
            let mut perms = metadata.permissions();
            perms.set_mode(0o600);
            let _ = fs::set_permissions(path, perms);
        }
    }
    #[cfg(not(unix))]
    {
        let _ = path;
    }
}
