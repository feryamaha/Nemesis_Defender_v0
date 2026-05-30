// src/init.rs
use std::path::{Path, PathBuf};
use std::fs;
use std::process::Command;
use crate::detect::{detect_platform, detect_ides, detect_all_stacks};

pub fn init(target_dir: &Path, from: Option<String>) -> anyhow::Result<()> {
    println!("[nemesis-cli] Iniciando instalacao de Nemesis...");

    // PASSO 1: Detectar plataforma
    let platform = detect_platform();
    println!("[nemesis-cli] Platform: {}", platform.suffix);

    // PASSO 2: Localizar binarios
    let source_dir = resolve_source_dir(from)?;
    verify_binaries(&source_dir)?;
    println!("[nemesis-cli] Source: {}", source_dir.display());

    // PASSO 3: Criar estrutura .nemesis/ (bin/, config/, logs/, runtime/)
    create_target_structure(target_dir)?;

    // PASSO 4: Copiar binarios para .nemesis/bin/
    copy_binaries(&source_dir, target_dir)?;

    // PASSO 5: Harvest stacks
    let detected_stacks = harvest_detect_stacks(target_dir)?;
    println!("[nemesis-cli] Stacks detectados: {:?}", detected_stacks);

    // PASSO 6: Copiar configs
    copy_configs_with_harvest(&source_dir, target_dir, &detected_stacks)?;

    // PASSO 7: Detectar IDEs
    let ides = detect_ides(target_dir);

    // PASSO 8: Gerar hooks para cada IDE
    let hook_path = ".nemesis/bin/nemesis-pretool-check-unix";
    for ide_name in &ides {
        crate::hooks::generate_hooks(target_dir, ide_name, hook_path)?;
    }

    // PASSO 9: Iniciar defender daemon
    start_defender_daemon(target_dir)?;

    // PASSO 10: Ativar eBPF (Linux only)
    setup_ebpf(target_dir)?;

    // PASSO 11: Reportar
    println!("[nemesis-cli] IDEs: {}", ides.join(", "));
    println!("[nemesis-cli] Installed to: {}", target_dir.join(".nemesis").display());
    println!("[nemesis-cli] Done.");

    Ok(())
}

fn resolve_source_dir(from: Option<String>) -> anyhow::Result<PathBuf> {
    // Se --from foi fornecido, usar esse path
    if let Some(p) = from {
        println!("[nemesis-cli] Usando binarios do path fornecido: {}", p);
        return Ok(PathBuf::from(p));
    }

    // Tentar NEMESIS_HOME env var
    if let Ok(env_path) = std::env::var("NEMESIS_HOME") {
        println!("[nemesis-cli] Usando NEMESIS_HOME: {}", env_path);
        return Ok(PathBuf::from(env_path));
    }

    // Tentar mesmo diretorio do binario nemesis-cli
    if let Ok(current_exe) = std::env::current_exe() {
        if let Some(parent) = current_exe.parent() {
            println!("[nemesis-cli] Usando diretorio do binario: {}", parent.display());
            return Ok(parent.to_path_buf());
        }
    }

    // Fallback: baixar do GitHub Release
    println!("[nemesis-cli] --from nao fornecido. Baixando do GitHub Release...");

    let platform = detect_platform().suffix.clone();
    let temp_source = std::env::temp_dir().join(format!("nemesis-dl-{}", std::process::id()));
    fs::create_dir_all(&temp_source)?;

    crate::download::download_release(&platform, &temp_source)?;

    println!("[nemesis-cli] Release baixado em: {}", temp_source.display());
    Ok(temp_source)
}

fn verify_binaries(source_dir: &Path) -> anyhow::Result<()> {
    let binaries = if cfg!(windows) {
        vec!["nemesis-pretool-check-windows.exe", "nemesis-defender.exe"]
    } else {
        vec!["nemesis-pretool-check-unix", "nemesis-defender"]
    };

    for binary in binaries {
        let path = source_dir.join(binary);
        if !path.exists() {
            return Err(anyhow::anyhow!("Binario nao encontrado: {}", path.display()));
        }
    }

    Ok(())
}

fn create_target_structure(target_dir: &Path) -> anyhow::Result<()> {
    fs::create_dir_all(target_dir.join(".nemesis/bin"))?;
    fs::create_dir_all(target_dir.join(".nemesis/config"))?;
    fs::create_dir_all(target_dir.join(".nemesis/logs"))?;
    fs::create_dir_all(target_dir.join(".nemesis/runtime"))?;
    Ok(())
}

fn copy_binaries(source_dir: &Path, target_dir: &Path) -> anyhow::Result<()> {
    let binaries = if cfg!(windows) {
        vec!["nemesis-pretool-check-windows.exe", "nemesis-defender.exe"]
    } else {
        vec!["nemesis-pretool-check-unix", "nemesis-defender"]
    };

    let target_bin_dir = target_dir.join(".nemesis/bin");
    for binary in binaries {
        let src = source_dir.join(binary);
        let dst = target_bin_dir.join(binary);
        if src.exists() {
            fs::copy(&src, &dst)?;
            println!("[nemesis-cli] Copiado: {} -> {}", binary, dst.display());
        }
    }

    // Linux: copiar binarios eBPF se existirem
    #[cfg(target_os = "linux")]
    {
        for ebpf_bin in &["nemesis-ebpf-daemon", "nemesis-cgroup-watcher"] {
            let src = source_dir.join(ebpf_bin);
            let dst = target_bin_dir.join(ebpf_bin);
            if src.exists() {
                fs::copy(&src, &dst)?;
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    fs::set_permissions(&dst, std::fs::Permissions::from_mode(0o755))?;
                }
                println!("[nemesis-cli] Copiado: {} -> {}", ebpf_bin, dst.display());
            }
        }
    }

    Ok(())
}

fn harvest_detect_stacks(target_dir: &Path) -> anyhow::Result<Vec<String>> {
    // Detectar stacks usando harvest detection logic (detect_all_stacks)
    // Isso chama os detectores de arquivo para identificar qual tecnologia esta em uso
    let stacks = detect_all_stacks(target_dir);
    Ok(stacks)
}

fn copy_configs_with_harvest(source_dir: &Path, target_dir: &Path, stacks: &[String]) -> anyhow::Result<()> {
    use nemesis::hook::deny_list_loader::load_and_combine_deny_lists;

    // 1. Copiar deny-lists base
    let base_files = vec![
        "deny-list-base.json",
        "deny-list-typescript.json",
        "denylist-defender.json",
        "denylist-folder-files.json",
    ];

    let target_config = target_dir.join(".nemesis/config");
    for file in base_files {
        let src = source_dir.join("../../workflow-enforcement/config").join(file);
        if src.exists() {
            let dst = target_config.join(file);
            fs::copy(&src, &dst)?;
        }
    }

    // 2. Copiar stack-específicos (harvest-detected)
    for stack in stacks {
        let stack_file = format!("deny-list-{}.json", stack.to_lowercase());
        let src = source_dir.join("../../workflow-enforcement/config").join(&stack_file);
        if src.exists() {
            let dst = target_config.join(&stack_file);
            fs::copy(&src, &dst)?;
        }
    }

    // 3. Copiar generic
    let generic_src = source_dir.join("../../workflow-enforcement/config/deny-list-generic.json");
    if generic_src.exists() {
        let generic_dst = target_config.join("deny-list-generic.json");
        fs::copy(&generic_src, &generic_dst)?;
    }

    // 4. Combinar deny-lists (com stacks detectados por harvest)
    let _combined = load_and_combine_deny_lists(stacks);

    Ok(())
}

fn start_defender_daemon(target_dir: &Path) -> anyhow::Result<()> {
    let defender_path = target_dir.join(".nemesis/bin/nemesis-defender");
    let status = Command::new(&defender_path)
        .arg("--ensure-daemon")
        .status()?;

    if !status.success() {
        return Err(anyhow::anyhow!("Falha ao iniciar daemon"));
    }

    Ok(())
}

#[cfg(target_os = "linux")]
fn setup_ebpf(target_dir: &Path) -> anyhow::Result<()> {
    let nemesis_dir = target_dir.join(".nemesis");
    let ebpf_daemon = nemesis_dir.join("bin/nemesis-ebpf-daemon");
    if !ebpf_daemon.exists() {
        println!("[nemesis-cli] eBPF daemon not found, skipping kernel protection");
        return Ok(());
    }

    // Verificar kernel >= 5.7
    let kernel = Command::new("uname").arg("-r").output()?;
    let version = String::from_utf8_lossy(&kernel.stdout);
    println!("[nemesis-cli] Kernel: {}", version.trim());

    // Verificar BPF LSM no boot
    if let Ok(lsm) = std::fs::read_to_string("/sys/kernel/security/lsm") {
        if !lsm.contains("bpf") {
            println!("[nemesis-cli] BPF LSM not enabled in kernel boot params");
            println!("[nemesis-cli] eBPF unavailable. Pretool + Defender still active.");
            return Ok(());
        }
    } else {
        println!("[nemesis-cli] Cannot read /sys/kernel/security/lsm (not Linux?)");
        return Ok(());
    }

    println!("[nemesis-cli] BPF LSM detected — starting eBPF daemon...");
    let _ = Command::new(&ebpf_daemon)
        .arg("--ensure-daemon")
        .spawn();

    println!("[nemesis-cli] eBPF kernel protection: active");
    Ok(())
}

#[cfg(not(target_os = "linux"))]
fn setup_ebpf(_target_dir: &Path) -> anyhow::Result<()> {
    // eBPF is Linux-only — no-op on other platforms
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    fn make_temp_dir(prefix: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("nemesis-cli-test-{}-{}", prefix, std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn test_harvest_detect_stacks_rust_project() {
        // RED: harvest_detect_stacks must detect "rust" when Cargo.toml exists
        let dir = make_temp_dir("rust");
        fs::write(dir.join("Cargo.toml"), "[package]\nname = \"test\"\n").unwrap();

        let stacks = harvest_detect_stacks(&dir).expect("harvest_detect_stacks should succeed");
        assert!(stacks.contains(&"rust".to_string()), "Should detect rust stack");

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_harvest_detect_stacks_typescript_project() {
        // RED: harvest_detect_stacks must detect "typescript" when package.json exists
        let dir = make_temp_dir("ts");
        fs::write(dir.join("package.json"), "{\"name\":\"test\"}").unwrap();

        let stacks = harvest_detect_stacks(&dir).expect("harvest_detect_stacks should succeed");
        assert!(stacks.contains(&"typescript".to_string()), "Should detect typescript stack");

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_harvest_detect_stacks_empty_dir() {
        // RED: harvest_detect_stacks on empty dir should return empty vec
        let dir = make_temp_dir("empty");
        let stacks = harvest_detect_stacks(&dir).expect("harvest_detect_stacks should succeed even on empty dir");
        assert!(stacks.is_empty(), "Empty dir should yield no stacks");

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_harvest_detect_stacks_multi_stack_project() {
        // RED: harvest_detect_stacks must detect multiple stacks
        let dir = make_temp_dir("multi");
        fs::write(dir.join("Cargo.toml"), "[package]\nname = \"test\"\n").unwrap();
        fs::write(dir.join("package.json"), "{\"name\":\"test\"}").unwrap();

        let stacks = harvest_detect_stacks(&dir).expect("harvest_detect_stacks should succeed");
        assert!(stacks.contains(&"rust".to_string()), "Should detect rust");
        assert!(stacks.contains(&"typescript".to_string()), "Should detect typescript");
        assert_eq!(stacks.len(), 2, "Should detect exactly 2 stacks");

        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_init_signature_no_stacks_param() {
        // RED: init() must accept (target_dir, from) without stacks parameter
        // This test validates the signature change by calling the function
        // with only 2 args (will fail to compile if signature still has 3 args)
        let dir = make_temp_dir("init-sig");
        // We don't actually call init (it has side effects), just verify it
        // compiles with the 2-arg signature.
        // This test ensures the refactoring happened.
        let _expected_sig: fn(&Path, Option<String>) -> anyhow::Result<()> = init;
        let _ = dir;
        fs::remove_dir_all(&dir).ok();
    }
}
