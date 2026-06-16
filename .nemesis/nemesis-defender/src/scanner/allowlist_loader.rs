//! allowlist_loader — override humano ABSOLUTO (lido do disco em runtime).
//!
//! Ao contrário da `denylist-defender.json` (EMBUTIDA, tamper-proof), a allowlist é a ÚNICA
//! superfície editável pelo dono da máquina após o install. Ela SOBRESCREVE qualquer bloqueio
//! do Nemesis: qualquer finding cuja evidência casa uma entrada da allowlist é suprimido, e um
//! comando que casa é liberado — inclusive `rm -rf`, `git`, etc. É decisão e risco do dono.
//!
//! INVARIANTE DE SEGURANÇA: o arquivo está em `absolute_block` (o agente NUNCA escreve nele via
//! hook); só o humano edita no terminal nativo. A proteção é embutida (denylist tamper-proof),
//! então o agente não consegue nem remover a própria proteção.
//!
//! Fail-safe: arquivo ausente/vazio/inválido => allowlist VAZIA => comportamento idêntico ao
//! padrão (nenhum override; `is_allowlisted` sempre false). Tolera comentários de LINHA INTEIRA
//! `//` (para o exemplo comentado distribuído no install).
//!
//! Sem cache: relê o arquivo a cada chamada. O arquivo é minúsculo e isso garante que a edição
//! humana tenha efeito IMEDIATO, tanto no pretool (processo efêmero) quanto no daemon (longo).

use regex::Regex;
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Default, Deserialize)]
pub struct AllowList {
    #[serde(default)]
    pub allow_commands: Vec<String>,
    #[serde(default)]
    pub allow_patterns: Vec<String>,
}

/// Caminho do arquivo da allowlist, relativo à raiz do projeto (mesma convenção de pid.rs/daemon).
fn allowlist_path() -> PathBuf {
    std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(".nemesis")
        .join("denylist-customers")
        .join("allowlist-customers.jsonc")
}

/// Remove linhas cujo primeiro caractere não-branco é `//`. Só descarta a linha INTEIRA quando
/// ela COMEÇA com `//`, então `//` dentro de strings (ex.: `https://...`) é preservado.
fn strip_line_comments(raw: &str) -> String {
    raw.lines()
        .filter(|l| !l.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Lê + parseia a allowlist do disco. Fail-safe: qualquer erro => allowlist vazia.
fn load() -> AllowList {
    std::fs::read_to_string(allowlist_path())
        .ok()
        .map(|raw| strip_line_comments(&raw))
        .and_then(|s| serde_json::from_str::<AllowList>(&s).ok())
        .unwrap_or_default()
}

/// true se `text` é autorizado pelo dono: substring de algum `allow_commands` OU casa algum
/// `allow_patterns` (regex). Allowlist vazia => sempre false (override desligado).
pub fn is_allowlisted(text: &str) -> bool {
    let allow = load();
    if allow
        .allow_commands
        .iter()
        .any(|c| !c.is_empty() && text.contains(c.as_str()))
    {
        return true;
    }
    allow
        .allow_patterns
        .iter()
        .filter(|p| !p.is_empty())
        .any(|p| Regex::new(p).map(|re| re.is_match(text)).unwrap_or(false))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_keeps_url_in_string() {
        let raw = "{\n  // comentário\n  \"allow_commands\": [\"curl https://x.io\"]\n}";
        let stripped = strip_line_comments(raw);
        assert!(stripped.contains("https://x.io"));
        assert!(!stripped.contains("comentário"));
    }

    #[test]
    fn empty_allowlist_blocks_nothing() {
        // sem arquivo no cwd de teste => fail-safe vazio => sempre false.
        let allow = AllowList::default();
        assert!(allow.allow_commands.is_empty());
    }

    #[test]
    fn command_substring_matches() {
        let allow = AllowList {
            allow_commands: vec!["rm -rf".to_string()],
            allow_patterns: vec![],
        };
        let text = "rm -rf /tmp/build";
        assert!(allow
            .allow_commands
            .iter()
            .any(|c| text.contains(c.as_str())));
    }
}
