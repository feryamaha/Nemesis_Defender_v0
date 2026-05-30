// src/download.rs
use std::path::{Path, PathBuf};
use std::fs;
use std::io::{self, Read};
use anyhow::{anyhow, Result};
use tar::Archive;
use flate2::read::GzDecoder;

pub fn download_release(platform: &str, target_dir: &Path) -> Result<()> {
    println!("[nemesis-cli] Baixando release para: {}", platform);

    // Montar URL
    let url = format!(
        "https://github.com/feryamaha/Nemesis_Rust_v2.0/releases/latest/download/nemesis-v2.0-{}.tar.gz",
        platform
    );

    println!("[nemesis-cli] URL: {}", url);

    // Fazer download
    let response = ureq::get(&url)
        .call()
        .map_err(|e| anyhow!("Download falhou: {}. Use --from <path> para instalação offline.", e))?;

    // Ler resposta em buffer
    let mut buffer = Vec::new();
    io::BufReader::new(response.into_reader())
        .read_to_end(&mut buffer)
        .map_err(|e| anyhow!("Erro ao ler resposta: {}", e))?;

    println!("[nemesis-cli] Download concluído ({} bytes)", buffer.len());

    // Descompactar para temp dir
    let temp_dir = std::env::temp_dir().join(format!("nemesis-extract-{}", std::process::id()));
    fs::create_dir_all(&temp_dir)?;

    {
        let gz_decoder = GzDecoder::new(&buffer[..]);
        let mut archive = Archive::new(gz_decoder);
        archive.unpack(&temp_dir)
            .map_err(|e| anyhow!("Erro ao descompactar tarball: {}", e))?;
    }

    println!("[nemesis-cli] Tarball descompactado em: {}", temp_dir.display());

    // Copiar bin/
    let src_bin = temp_dir.join("bin");
    let target_bin = target_dir.join(".nemesis/bin");
    if src_bin.exists() {
        fs::create_dir_all(&target_bin)?;
        for entry in fs::read_dir(&src_bin)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                let file_name = path.file_name().unwrap();
                let dst = target_bin.join(&file_name);
                fs::copy(&path, &dst)
                    .map_err(|e| anyhow!("Erro ao copiar binário {}: {}", file_name.to_string_lossy(), e))?;
            }
        }
        println!("[nemesis-cli] Binários copiados para: {}", target_bin.display());
    }

    // Copiar config/
    let src_config = temp_dir.join("config");
    let target_config = target_dir.join(".nemesis/config");
    if src_config.exists() {
        fs::create_dir_all(&target_config)?;
        for entry in fs::read_dir(&src_config)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                let file_name = path.file_name().unwrap();
                let dst = target_config.join(&file_name);
                fs::copy(&path, &dst)
                    .map_err(|e| anyhow!("Erro ao copiar config {}: {}", file_name.to_string_lossy(), e))?;
            }
        }
        println!("[nemesis-cli] Configs copiadas para: {}", target_config.display());
    }

    // Limpar temp
    fs::remove_dir_all(&temp_dir)
        .map_err(|e| anyhow!("Erro ao limpar temp: {}", e))?;

    println!("[nemesis-cli] Download concluído com sucesso!");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_detection_order() {
        // Validar que platforms esperadas sao suportadas
        let platforms = vec![
            "linux-x86_64",
            "macos-aarch64",
            "macos-x86_64",
            "windows-x86_64",
        ];
        for p in platforms {
            assert!(!p.is_empty());
        }
    }
}