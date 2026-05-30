use anyhow::{Context, Result};
use std::fs;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

pub const NEMESIS_CGROUP: &str = "/sys/fs/cgroup/nemesis-agent";

/// Cria o cgroup v2 para o agente Nemesis.
/// Retorna o cgroup_id (inode number do diretório).
pub fn create_agent_cgroup() -> Result<u64> {
    let path = Path::new(NEMESIS_CGROUP);
    if !path.exists() {
        fs::create_dir_all(path)
            .with_context(|| format!("failed to create cgroup at {}", NEMESIS_CGROUP))?;
    }
    let metadata = fs::metadata(path)?;
    Ok(metadata.ino())
}

/// Atribui um PID ao cgroup do agente (move o processo para o cgroup).
pub fn assign_pid_to_agent_cgroup(pid: u32) -> Result<()> {
    let procs_path = PathBuf::from(NEMESIS_CGROUP).join("cgroup.procs");
    fs::write(procs_path, pid.to_string())
        .with_context(|| format!("failed to assign PID {} to cgroup", pid))?;
    Ok(())
}

/// Remove o cgroup (após o daemon encerrar).
pub fn remove_agent_cgroup() -> Result<()> {
    let path = Path::new(NEMESIS_CGROUP);
    if path.exists() {
        fs::remove_dir(path)?;
    }
    Ok(())
}
