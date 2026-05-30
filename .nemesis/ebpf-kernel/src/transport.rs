use crate::EbpfHealthResponse;
use anyhow::{Context, Result};
use serde_json::to_vec;
use std::path::PathBuf;

#[cfg(unix)]
use tokio::io::AsyncWriteExt;
#[cfg(unix)]
use tokio::net::UnixListener;

pub fn default_status_socket_path() -> Option<PathBuf> {
    #[cfg(unix)]
    {
        Some(PathBuf::from("/tmp/nemesis-ebpf.sock"))
    }

    #[cfg(not(unix))]
    {
        None
    }
}

#[cfg(unix)]
pub async fn run_status_server(socket_path: PathBuf, response: EbpfHealthResponse) -> Result<()> {
    if socket_path.exists() {
        std::fs::remove_file(&socket_path)
            .with_context(|| format!("failed to remove stale socket {}", socket_path.display()))?;
    }

    let listener = UnixListener::bind(&socket_path)
        .with_context(|| format!("failed to bind {}", socket_path.display()))?;

    loop {
        let (mut stream, _) = listener.accept().await?;
        let payload = to_vec(&response)?;
        stream.write_all(&payload).await?;
    }
}

#[cfg(not(unix))]
pub async fn run_status_server(_socket_path: PathBuf, _response: EbpfHealthResponse) -> Result<()> {
    Ok(())
}
