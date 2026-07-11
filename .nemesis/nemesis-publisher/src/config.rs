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
/// Default: `https://dashboard-nemesis-defender.vercel.app`
pub fn dashboard_url() -> String {
    std::env::var("NEMESIS_DASHBOARD_URL")
        .unwrap_or_else(|_| "https://dashboard-nemesis-defender.vercel.app".to_string())
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

/// Environment: "official" ou "market". Default: "market".
pub fn environment() -> String {
    std::env::var("NEMESIS_ENVIRONMENT").unwrap_or_else(|_| "market".to_string())
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

/// Intervalo (segundos) de sincronizacao com o Neon no modo --serve.
/// Env `NEMESIS_SYNC_INTERVAL`. Default: 1800 (30min). 0 = desativado
/// (o sync passa a ser apenas manual via --sync).
pub fn sync_interval() -> u64 {
    std::env::var("NEMESIS_SYNC_INTERVAL")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(1800)
}

/// Caminho do arquivo de env do publisher: `.nemesis/telemetry/publisher.env`.
/// Consumido pelo proprio publisher no startup (fallback de DATABASE_URL e afins quando o
/// service do sistema nao injeta ambiente; launchd nao tem equivalente de EnvironmentFile).
pub fn publisher_env_path() -> PathBuf {
    telemetry_dir().join("publisher.env")
}

/// Parseia linhas KEY=VALUE de um publisher.env (linhas vazias e `#` sao ignoradas;
/// split no PRIMEIRO `=`, valores podem conter `=`).
fn parse_env_lines(content: &str) -> Vec<(String, String)> {
    content
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .filter_map(|l| {
            let (key, value) = l.split_once('=')?;
            let key = key.trim();
            if key.is_empty() {
                return None;
            }
            Some((key.to_string(), value.trim().to_string()))
        })
        .collect()
}

/// Define a var de ambiente somente se ela ainda nao existe (o ambiente real, ex. a unit
/// systemd via EnvironmentFile, sempre vence o fallback in-process).
fn set_if_absent(key: &str, value: &str) {
    if std::env::var_os(key).is_none() {
        std::env::set_var(key, value);
    }
}

/// Carrega `publisher.env` no ambiente do processo, sem sobrescrever o que ja existe.
/// Best-effort: arquivo ausente ou ilegivel nao falha (telemetria e opt-in).
pub fn load_env_file() {
    let Ok(content) = std::fs::read_to_string(publisher_env_path()) else {
        return;
    };
    for (key, value) in parse_env_lines(&content) {
        set_if_absent(&key, &value);
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_env_lines_ignora_comentarios_e_vazias() {
        let content = "# comentario\n\nDATABASE_URL=postgres://x/y?a=b\n  NEMESIS_PUBLISHER_PORT = 9090\nsem_igual\n=sem_chave\n";
        let vars = parse_env_lines(content);
        assert_eq!(vars.len(), 2);
        assert_eq!(vars[0], ("DATABASE_URL".to_string(), "postgres://x/y?a=b".to_string()));
        assert_eq!(vars[1], ("NEMESIS_PUBLISHER_PORT".to_string(), "9090".to_string()));
    }

    #[test]
    fn set_if_absent_nao_sobrescreve_existente() {
        std::env::set_var("NEMESIS_TEST_ENVFILE_KEEP", "original");
        set_if_absent("NEMESIS_TEST_ENVFILE_KEEP", "novo");
        assert_eq!(std::env::var("NEMESIS_TEST_ENVFILE_KEEP").unwrap(), "original");

        std::env::remove_var("NEMESIS_TEST_ENVFILE_SET");
        set_if_absent("NEMESIS_TEST_ENVFILE_SET", "valor");
        assert_eq!(std::env::var("NEMESIS_TEST_ENVFILE_SET").unwrap(), "valor");
    }
}
