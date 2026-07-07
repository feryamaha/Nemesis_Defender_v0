//! Configuracao do publisher: resolucao de paths, env vars, bootstrap secret.

use std::path::PathBuf;

/// Resolve o diretorio `.nemesis` subindo do executavel ate achar o ancestral.
/// Fallback: `.nemesis` relativo ao CWD.
pub fn nemesis_dir() -> PathBuf {
    if let Ok(exe) = std::env::current_exe() {
        for anc in exe.ancestors() {
            if anc.file_name().map(|n| n == ".nemesis").unwrap_or(false) {
                return anc.to_path_buf();
            }
        }
    }
    PathBuf::from(".nemesis")
}

/// Caminho do arquivo de identidade: `.nemesis/telemetry/identity.json`
pub fn identity_path() -> PathBuf {
    nemesis_dir().join("telemetry").join("identity.json")
}

/// Caminho do diretorio de telemetria: `.nemesis/telemetry/`
pub fn telemetry_dir() -> PathBuf {
    nemesis_dir().join("telemetry")
}

/// Caminho do ledger: `.nemesis/logs/nemesis-violations.log`
pub fn ledger_path() -> PathBuf {
    nemesis_dir().join("logs").join("nemesis-violations.log")
}

/// URL base da dashboard. Lida de env var `NEMESIS_DASHBOARD_URL`.
/// Default: `https://nemesis-defender.vercel.app`
pub fn dashboard_url() -> String {
    std::env::var("NEMESIS_DASHBOARD_URL")
        .unwrap_or_else(|_| "https://nemesis-defender.vercel.app".to_string())
}

/// Bootstrap secret para registro. Compilado em build-time via `option_env!`.
/// Se nao definido, retorna None e `--register` falha com mensagem clara.
pub fn bootstrap_secret() -> Option<&'static str> {
    option_env!("NEMESIS_BOOTSTRAP_SECRET")
}

/// Porta do servidor HTTP local (modo --serve). Default: 8080.
pub fn publisher_port() -> u16 {
    std::env::var("NEMESIS_PUBLISHER_PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(8080)
}

/// Environment: "official" ou "market". Default: "official".
pub fn environment() -> String {
    std::env::var("NEMESIS_ENVIRONMENT").unwrap_or_else(|_| "official".to_string())
}

/// URL de conexao com o Neon Postgres (modo --sync).
pub fn database_url() -> Option<String> {
    std::env::var("DATABASE_URL").ok()
}

/// Caminho do snapshot do doctor FULL: `.nemesis/telemetry/doctor-full.json`.
/// Fonte unica da observabilidade do doctor (SPEC-001: full-only).
pub fn doctor_full_snapshot_path() -> PathBuf {
    telemetry_dir().join("doctor-full.json")
}

/// Intervalo (segundos) de re-execucao do doctor full em background no modo --serve.
/// Env `NEMESIS_DOCTOR_FULL_INTERVAL`. Default: 1800 (30min). 0 = desativado
/// (o snapshot passa a ser atualizado apenas pelo --sync ou execucao manual).
pub fn doctor_full_interval() -> u64 {
    std::env::var("NEMESIS_DOCTOR_FULL_INTERVAL")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(1800)
}

/// Caminho raiz do repo Nemesis (para resolver paths das fontes).
pub fn nemesis_repo_root() -> PathBuf {
    let dir = nemesis_dir();
    if dir.file_name().map(|n| n == ".nemesis").unwrap_or(false) {
        dir.parent().map(|p| p.to_path_buf()).unwrap_or_else(|| PathBuf::from("."))
    } else {
        PathBuf::from(".")
    }
}
