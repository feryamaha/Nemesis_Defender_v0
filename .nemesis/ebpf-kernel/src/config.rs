use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandsConfig {
    pub blocked_commands: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathsConfig {
    pub blocked_write_prefixes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeConfig {
    pub enabled: bool,
    pub block_commands: bool,
    pub block_write_paths: bool,
    pub log_to_user_space: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LandlockConfig {
    pub allowed_exec: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EgressConfig {
    #[serde(default)]
    pub enforce: bool,
    #[serde(default)]
    pub allowlist: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct EbpfConfigBundle {
    pub commands: CommandsConfig,
    pub paths: PathsConfig,
    pub runtime: RuntimeConfig,
    pub landlock: LandlockConfig,
    pub egress: EgressConfig,
    pub root: PathBuf,
}

impl EbpfConfigBundle {
    pub fn load_from(root: impl AsRef<Path>) -> Result<Self> {
        let root = root.as_ref().to_path_buf();
        let denylist_root = root.join("denylist-ebpf");

        let commands = parse_toml::<CommandsConfig>(&denylist_root.join("commands.toml"))?;
        let paths = parse_toml::<PathsConfig>(&denylist_root.join("paths.toml"))?;
        let runtime = parse_toml::<RuntimeConfig>(&denylist_root.join("config.toml"))?;
        let landlock =
            parse_toml::<LandlockConfig>(&denylist_root.join("landlock-allowed-exec.toml"))?;
        // egress.toml é opcional (rollout incremental): ausência ⇒ default seguro
        // (enforce=false, allowlist vazia), sem falhar a carga.
        let egress = parse_toml_optional::<EgressConfig>(&denylist_root.join("egress.toml"))
            .unwrap_or_default();

        Ok(Self {
            commands,
            paths,
            runtime,
            landlock,
            egress,
            root,
        })
    }
}

fn parse_toml<T>(path: &Path) -> Result<T>
where
    T: for<'de> Deserialize<'de>,
{
    let raw = fs::read_to_string(path)
        .with_context(|| format!("failed to read {}", path.display()))?;
    toml::from_str(&raw).with_context(|| format!("failed to parse {}", path.display()))
}

fn parse_toml_optional<T>(path: &Path) -> Option<T>
where
    T: for<'de> Deserialize<'de>,
{
    let raw = fs::read_to_string(path).ok()?;
    toml::from_str(&raw).ok()
}
