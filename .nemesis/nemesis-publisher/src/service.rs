//! Instalacao e remocao do service de sistema do publisher (systemd user / launchd).
//!
//! O service so e instalado com opt-in ativo: telemetria nunca vira persistencia por
//! padrao. Os templates sao versionados em `.nemesis/install/` e embutidos no binario
//! via `include_str!` (mesmo padrao das regras criticas do Defender): o que executa e
//! sempre o que foi compilado, nao um arquivo editavel no disco.

use crate::{config, identity};
use anyhow::{Context, Result};
use std::path::PathBuf;

// Ambos os templates sao embutidos em qualquer plataforma: o build de um OS carrega o
// template do outro apenas como dado (usado nos testes); o codigo por plataforma e cfg-gated.
#[allow(dead_code)]
const SYSTEMD_TEMPLATE: &str = include_str!("../../install/nemesis-publisher.service");
#[allow(dead_code)]
const LAUNCHD_TEMPLATE: &str = include_str!("../../install/com.nemesis.publisher.plist");

/// Substitui os placeholders do template pelos valores reais da instalacao.
fn render(template: &str, exec: &str, env_file: &str) -> String {
    template
        .replace("{{EXEC}}", exec)
        .replace("{{ENV_FILE}}", env_file)
}

/// O service so existe apos opt-in explicito (issue 010: nunca por padrao).
fn require_opt_in() -> Result<()> {
    if !identity::exists() {
        anyhow::bail!(
            "Telemetria sem identidade. O service so e instalado apos opt-in: \
             nemesis-publisher --opt-in"
        );
    }
    let id = identity::load()?;
    if !id.opt_in {
        anyhow::bail!(
            "Opt-in nao ativado (opt_in: false). O service so e instalado apos opt-in: \
             nemesis-publisher --opt-in"
        );
    }
    Ok(())
}

#[allow(dead_code)]
fn home_dir() -> Result<PathBuf> {
    std::env::var("HOME")
        .map(PathBuf::from)
        .context("variavel HOME nao definida")
}

fn exec_path() -> Result<String> {
    let exe = std::env::current_exe().context("resolver executavel atual")?;
    let exe = exe.canonicalize().unwrap_or(exe);
    Ok(exe.to_string_lossy().into_owned())
}

#[allow(dead_code)]
fn run_cmd(program: &str, args: &[&str]) -> Result<()> {
    let status = std::process::Command::new(program)
        .args(args)
        .status()
        .with_context(|| format!("executar {}", program))?;
    if !status.success() {
        anyhow::bail!("{} {:?} saiu com status {}", program, args, status);
    }
    Ok(())
}

#[cfg(target_os = "linux")]
fn unit_path() -> Result<PathBuf> {
    Ok(home_dir()?
        .join(".config")
        .join("systemd")
        .join("user")
        .join("nemesis-publisher.service"))
}

#[cfg(target_os = "linux")]
pub fn install_service() -> Result<()> {
    require_opt_in()?;
    let exec = exec_path()?;
    let env_file = config::publisher_env_path().to_string_lossy().into_owned();
    let unit = unit_path()?;
    if let Some(dir) = unit.parent() {
        std::fs::create_dir_all(dir)
            .with_context(|| format!("criar {}", dir.display()))?;
    }
    std::fs::write(&unit, render(SYSTEMD_TEMPLATE, &exec, &env_file))
        .with_context(|| format!("escrever {}", unit.display()))?;
    run_cmd("systemctl", &["--user", "daemon-reload"])?;
    run_cmd(
        "systemctl",
        &["--user", "enable", "--now", "nemesis-publisher.service"],
    )?;
    println!(
        "[nemesis-publisher] Service instalado e ativo: {}",
        unit.display()
    );
    println!(
        "[nemesis-publisher] Nota: unit de usuario inicia no login. \
         Para boot sem login: loginctl enable-linger $USER"
    );
    Ok(())
}

#[cfg(target_os = "linux")]
pub fn uninstall_service() -> Result<()> {
    // Best-effort e idempotente: service ja parado ou unit ausente nao e erro.
    let _ = std::process::Command::new("systemctl")
        .args(["--user", "disable", "--now", "nemesis-publisher.service"])
        .status();
    let unit = unit_path()?;
    if unit.exists() {
        std::fs::remove_file(&unit)
            .with_context(|| format!("remover {}", unit.display()))?;
        let _ = std::process::Command::new("systemctl")
            .args(["--user", "daemon-reload"])
            .status();
        println!("[nemesis-publisher] Service removido: {}", unit.display());
    } else {
        println!("[nemesis-publisher] Nenhum service instalado.");
    }
    Ok(())
}

#[cfg(target_os = "macos")]
fn plist_path() -> Result<PathBuf> {
    Ok(home_dir()?
        .join("Library")
        .join("LaunchAgents")
        .join("com.nemesis.publisher.plist"))
}

#[cfg(target_os = "macos")]
pub fn install_service() -> Result<()> {
    require_opt_in()?;
    let exec = exec_path()?;
    let env_file = config::publisher_env_path().to_string_lossy().into_owned();
    let plist = plist_path()?;
    if let Some(dir) = plist.parent() {
        std::fs::create_dir_all(dir)
            .with_context(|| format!("criar {}", dir.display()))?;
    }
    std::fs::write(&plist, render(LAUNCHD_TEMPLATE, &exec, &env_file))
        .with_context(|| format!("escrever {}", plist.display()))?;
    let plist_str = plist.to_string_lossy().into_owned();
    run_cmd("launchctl", &["load", "-w", &plist_str])?;
    println!(
        "[nemesis-publisher] LaunchAgent instalado e ativo: {}",
        plist.display()
    );
    Ok(())
}

#[cfg(target_os = "macos")]
pub fn uninstall_service() -> Result<()> {
    let plist = plist_path()?;
    if plist.exists() {
        let plist_str = plist.to_string_lossy().into_owned();
        let _ = std::process::Command::new("launchctl")
            .args(["unload", "-w", &plist_str])
            .status();
        std::fs::remove_file(&plist)
            .with_context(|| format!("remover {}", plist.display()))?;
        println!("[nemesis-publisher] LaunchAgent removido: {}", plist.display());
    } else {
        println!("[nemesis-publisher] Nenhum LaunchAgent instalado.");
    }
    Ok(())
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
pub fn install_service() -> Result<()> {
    anyhow::bail!("Service automatico suportado apenas em Linux (systemd) e macOS (launchd).")
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
pub fn uninstall_service() -> Result<()> {
    anyhow::bail!("Service automatico suportado apenas em Linux (systemd) e macOS (launchd).")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_substitui_todos_os_placeholders() {
        for template in [SYSTEMD_TEMPLATE, LAUNCHD_TEMPLATE] {
            let out = render(
                template,
                "/x/bin/nemesis-publisher",
                "/x/telemetry/publisher.env",
            );
            assert!(!out.contains("{{"), "placeholder sem substituicao:\n{}", out);
            assert!(out.contains("/x/bin/nemesis-publisher"));
        }
    }

    #[test]
    fn systemd_template_reinicia_sempre_e_serve() {
        assert!(SYSTEMD_TEMPLATE.contains("Restart=always"));
        assert!(SYSTEMD_TEMPLATE.contains("ExecStart={{EXEC}} --serve"));
        assert!(SYSTEMD_TEMPLATE.contains("EnvironmentFile=-{{ENV_FILE}}"));
    }

    #[test]
    fn launchd_template_keepalive_e_label() {
        assert!(LAUNCHD_TEMPLATE.contains("<key>KeepAlive</key>"));
        assert!(LAUNCHD_TEMPLATE.contains("com.nemesis.publisher"));
        assert!(LAUNCHD_TEMPLATE.contains("--serve"));
    }
}
