//! denylist-defender.json loader
//!
//! A deny-list de SEGURANÇA DE CONTEÚDO é EMBUTIDA no binário em tempo de compilação
//! (`include_str!`). Consequências (decisão arquitetural — 2026-06-13):
//!   - O usuário NÃO pode editar as regras de segurança: não há arquivo no disco para alterar.
//!   - O daemon NÃO tem um arquivo de config para escanear → elimina o falso-positivo de
//!     auto-scan ("cobra mordendo o rabo") na raiz.
//!   - A fonte da verdade é o JSON versionado no repo; para mudar regras: editar + recompilar.
//! Usado pelo regex_layer para detectar comandos hostis em conteúdo — qualquer OS.

use serde::Deserialize;
use std::collections::HashMap;
use std::sync::OnceLock;

#[derive(Debug, Deserialize, Clone)]
pub struct DenyListConfig {
    pub version: String,
    pub categories: HashMap<String, Category>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Category {
    pub description: String,
    pub severity: String,
    pub suggestion: Option<String>,
    pub patterns: Vec<String>,
}

static DENYLIST_CACHE: OnceLock<Option<DenyListConfig>> = OnceLock::new();

/// Conteúdo canônico do denylist-defender.json, embutido no binário em tempo de compilação.
/// Path relativo a este arquivo: src/scanner/ → ../../config/denylist-defender.json.
const EMBEDDED_DENYLIST: &str = include_str!("../../config/denylist-defender.json");

/// Load deny-list configuration (parsed once, cached).
/// Fonte ÚNICA: a string embutida. Não há leitura de disco — imune a CWD/layout/edição.
pub fn load() -> Option<&'static DenyListConfig> {
    DENYLIST_CACHE
        .get_or_init(|| serde_json::from_str::<DenyListConfig>(EMBEDDED_DENYLIST).ok())
        .as_ref()
}

/// Get all patterns flattened for a given severity
/// Returns: Vec<(category_name, pattern, description, suggestion)>
pub fn patterns_by_severity(severity: &str) -> Vec<(String, String, String, Option<String>)> {
    let mut result = Vec::new();

    if let Some(cfg) = load() {
        for (cat_name, cat) in &cfg.categories {
            if cat.severity == severity {
                for pattern in &cat.patterns {
                    result.push((
                        cat_name.clone(),
                        pattern.clone(),
                        cat.description.clone(),
                        cat.suggestion.clone(),
                    ));
                }
            }
        }
    }

    result
}
