use regex::Regex;
use std::fs;
use std::path::Path;
use std::process::Command;

/// Environment Detector for Nemesis Enforcement Engine
/// Detecta OS e gerenciador de pacotes do PROJETO HOSPEDEIRO para adaptação

#[derive(Debug, Clone)]
pub struct EnvironmentInfo {
    pub os: String, // "mac" | "windows" | "linux"
    pub package_manager: String, // "yarn" | "bun" | "npm" | "pnpm" | "unknown"
    pub has_lock_file: bool,
    pub lock_file_type: String, // yarn.lock, bun.lockb, package-lock.json, pnpm-lock.yaml, none
    pub node_version: String,
    pub package_manager_version: String,
}

pub fn detect_os() -> String {
    if cfg!(target_os = "macos") {
        "mac".to_string()
    } else if cfg!(target_os = "windows") {
        "windows".to_string()
    } else if cfg!(target_os = "linux") {
        "linux".to_string()
    } else {
        panic!("Unsupported OS");
    }
}

/// Detecta o gerenciador de pacotes do PROJETO HOSPEDEIRO baseado no lockfile
/// Esta é a detecção principal para o adapter do Nemesis
pub fn detect_host_package_manager() -> String {
    if Path::new("yarn.lock").exists() {
        return "yarn".to_string();
    }
    if Path::new("bun.lockb").exists() {
        return "bun".to_string();
    }
    if Path::new("package-lock.json").exists() {
        return "npm".to_string();
    }
    if Path::new("pnpm-lock.yaml").exists() {
        return "pnpm".to_string();
    }

    // Prioridade 2: Se não tem lockfile, usa Bun (default do Nemesis)
    "bun".to_string()
}

/// Detecta gerenciadores disponíveis globalmente (fallback)
pub fn detect_available_package_manager() -> String {
    if Command::new("yarn").arg("--version").output().map(|o| o.status.success()).unwrap_or(false) {
        return "yarn".to_string();
    }

    if Command::new("bun").arg("--version").output().map(|o| o.status.success()).unwrap_or(false) {
        return "bun".to_string();
    }

    if Command::new("npm").arg("--version").output().map(|o| o.status.success()).unwrap_or(false) {
        return "npm".to_string();
    }

    if Command::new("pnpm").arg("--version").output().map(|o| o.status.success()).unwrap_or(false) {
        return "pnpm".to_string();
    }

    "unknown".to_string()
}

pub fn detect_lock_file() -> String {
    if Path::new("yarn.lock").exists() {
        return "yarn.lock".to_string();
    }
    if Path::new("bun.lockb").exists() {
        return "bun.lockb".to_string();
    }
    if Path::new("package-lock.json").exists() {
        return "package-lock.json".to_string();
    }
    if Path::new("pnpm-lock.yaml").exists() {
        return "pnpm-lock.yaml".to_string();
    }
    "none".to_string()
}

pub fn get_node_version() -> String {
    match Command::new("node").arg("--version").output() {
        Ok(output) => String::from_utf8_lossy(&output.stdout).trim().to_string(),
        Err(_) => "unknown".to_string(),
    }
}

pub fn get_package_manager_version(package_manager: &str) -> String {
    match Command::new(package_manager).arg("--version").output() {
        Ok(output) => String::from_utf8_lossy(&output.stdout).trim().to_string(),
        Err(_) => "unknown".to_string(),
    }
}

pub fn detect_environment() -> EnvironmentInfo {
    let os = detect_os();
    let package_manager = detect_host_package_manager();
    let lock_file_type = detect_lock_file();
    let has_lock_file = lock_file_type != "none";
    let node_version = get_node_version();
    let package_manager_version = get_package_manager_version(&package_manager);

    EnvironmentInfo {
        os,
        package_manager,
        has_lock_file,
        lock_file_type,
        node_version,
        package_manager_version,
    }
}

#[derive(Debug)]
pub struct CompatibilityResult {
    pub compatible: bool,
    pub issues: Vec<String>,
    pub recommendations: Vec<String>,
}

pub fn validate_environment_compatibility(env: &EnvironmentInfo) -> CompatibilityResult {
    let mut issues: Vec<String> = Vec::new();
    let mut recommendations: Vec<String> = Vec::new();

    // Verificar compatibilidade básica
    if env.package_manager == "unknown" {
        issues.push("Nenhum gerenciador de pacotes compatível encontrado".to_string());
        recommendations.push("Instalar Yarn, Bun, npm ou pnpm".to_string());
    }

    if !env.has_lock_file {
        issues.push("Nenhum arquivo de lock encontrado".to_string());
        recommendations.push("Executar install do gerenciador de pacotes para criar lock file".to_string());
    }

    // Verificar consistência entre lock file e package manager detectado
    if env.has_lock_file && env.package_manager != "unknown" {
        if env.package_manager == "bun" && env.lock_file_type != "bun.lockb" {
            issues.push(format!("Detectado Bun mas lock file é {}", env.lock_file_type));
            recommendations.push("Usar \"bun install\" para criar bun.lockb".to_string());
        }
        if env.package_manager == "yarn" && env.lock_file_type != "yarn.lock" {
            issues.push(format!("Detectado Yarn mas lock file é {}", env.lock_file_type));
            recommendations.push("Usar \"yarn install\" para criar yarn.lock".to_string());
        }
        if env.package_manager == "npm" && env.lock_file_type != "package-lock.json" {
            issues.push(format!("Detectado npm mas lock file é {}", env.lock_file_type));
            recommendations.push("Usar \"npm install\" para criar package-lock.json".to_string());
        }
        if env.package_manager == "pnpm" && env.lock_file_type != "pnpm-lock.yaml" {
            issues.push(format!("Detectado pnpm mas lock file é {}", env.lock_file_type));
            recommendations.push("Usar \"pnpm install\" para criar pnpm-lock.yaml".to_string());
        }
    }

    // Verificar versão do Node.js
    let re = Regex::new(r"^v(\d+)\.").unwrap();
    if let Some(caps) = re.captures(&env.node_version) {
        let major_version: u32 = caps[1].parse().unwrap_or(0);
        if major_version < 18 {
            issues.push(format!("Node.js v{} é muito antigo", major_version));
            recommendations.push("Atualizar para Node.js 18+ para compatibilidade completa".to_string());
        }
    }

    let compatible = issues.is_empty();

    CompatibilityResult {
        compatible,
        issues,
        recommendations,
    }
}
