use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

// ============================================================
// NEMESIS SCOPE VALIDATOR
// Valida se um arquivo esta dentro do escopo autorizado pelo RAG
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScopeConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rag_reference: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_files: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_patterns: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocked_files: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ScopeValidationResult {
    pub valid: bool,
    pub reason: Option<String>,
    pub rule: Option<String>,
    pub suggestion: Option<String>,
}

/// Caminho padrao do arquivo de escopo
fn get_scope_path() -> PathBuf {
    std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(".nemesis")
        .join("scope.json")
}

/// Verifica se existe um escopo ativo
pub fn has_scope_active() -> bool {
    get_scope_path().exists()
}

/// Le o escopo atual
pub fn read_scope() -> Option<ScopeConfig> {
    let scope_path = get_scope_path();
    if !scope_path.exists() {
        return None;
    }

    match fs::read_to_string(&scope_path) {
        Ok(content) => serde_json::from_str(&content).ok(),
        Err(_) => None,
    }
}

/// Valida se um arquivo esta dentro do escopo autorizado
pub fn validate_file_scope(file_path: &str) -> ScopeValidationResult {
    let scope = read_scope();

    // Sem scope = modo aberto (permite tudo)
    if scope.is_none() {
        return ScopeValidationResult {
            valid: true,
            reason: None,
            rule: None,
            suggestion: None,
        };
    }

    let scope = scope.unwrap();

    // Normalizar o path para comparacao
    let absolute_path = std::path::absolute(file_path).unwrap_or_else(|_| PathBuf::from(file_path));
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let relative_path = pathdiff::diff_paths(&absolute_path, &cwd)
        .unwrap_or_else(|| PathBuf::from(file_path))
        .to_string_lossy()
        .replace('\\', "/");

    // Verificar blocked_files primeiro (prioridade maxima)
    if let Some(ref blocked_files) = scope.blocked_files {
        if !blocked_files.is_empty() {
            for blocked in blocked_files {
                let normalized_blocked = blocked.replace('\\', "/");
                if relative_path == normalized_blocked || relative_path.ends_with(&normalized_blocked) {
                    return ScopeValidationResult {
                        valid: false,
                        reason: Some(format!("Arquivo explicitamente bloqueado pelo escopo: {}", relative_path)),
                        rule: Some(".nemesis/scope.json - blocked_files".to_string()),
                        suggestion: Some("Este arquivo foi bloqueado pelo usuario. Re-leia o workflow e siga as instrucoes voce esta pulando etapas".to_string()),
                    };
                }
            }
        }
    }

    // Se nao ha allowed_files nem allowed_patterns, modo aberto
    let has_allowed_files = scope.allowed_files.as_ref().map(|v| !v.is_empty()).unwrap_or(false);
    let has_allowed_patterns = scope.allowed_patterns.as_ref().map(|v| !v.is_empty()).unwrap_or(false);

    if !has_allowed_files && !has_allowed_patterns {
        return ScopeValidationResult {
            valid: true,
            reason: None,
            rule: None,
            suggestion: None,
        };
    }

    // Verificar allowed_files (match exato ou por sufixo)
    if has_allowed_files {
        if let Some(ref allowed_files) = scope.allowed_files {
            for allowed in allowed_files {
                let normalized_allowed = allowed.replace('\\', "/");
                if relative_path == normalized_allowed || relative_path.ends_with(&normalized_allowed) {
                    return ScopeValidationResult {
                        valid: true,
                        reason: None,
                        rule: None,
                        suggestion: None,
                    };
                }
            }
        }
    }

    // Verificar allowed_patterns (glob simples)
    if has_allowed_patterns {
        if let Some(ref allowed_patterns) = scope.allowed_patterns {
            for pattern in allowed_patterns {
                if match_glob(&relative_path, pattern) {
                    return ScopeValidationResult {
                        valid: true,
                        reason: None,
                        rule: None,
                        suggestion: None,
                    };
                }
            }
        }
    }

    // Arquivo nao esta no escopo
    let allowed_list = scope
        .allowed_files
        .as_ref()
        .map(|v| v.join(", "))
        .unwrap_or_else(|| "nenhum especificado".to_string());

    ScopeValidationResult {
        valid: false,
        reason: Some(format!("Arquivo fora do escopo autorizado: {}", relative_path)),
        rule: Some(".nemesis/scope.json - Escopo definido pelo RAG".to_string()),
        suggestion: Some(format!(
            "Arquivos permitidos: {}. Este arquivo foi bloqueado pelo Nemesis. Re-leia o workflow e siga as instrucoes voce esta pulando etapas.",
            allowed_list
        )),
    }
}

/// Glob matching simples (sem dependencia externa)
/// Suporta: *, **, ?
///
/// Exemplos:
///   src/types/*.types.ts  ->  match src/types/ui.types.ts
///   src/hooks/**/\*.hook.ts  ->  match src/hooks/deep/nested/use.hook.ts
///   src/components/ui/*.tsx  ->  match src/components/ui/Button.tsx
fn match_glob(file_path: &str, pattern: &str) -> bool {
    // Normalizar
    let normalized_pattern = pattern.replace('\\', "/");
    let normalized_path = file_path.replace('\\', "/");

    // Converter glob para regex
    let mut regex_str = normalized_pattern
        // Escapar caracteres especiais de regex (exceto * e ?)
        .chars()
        .map(|c| match c {
            '.' | '+' | '^' | '$' | '{' | '}' | '(' | ')' | '|' | '[' | ']' => format!("\\{}", c),
            _ => c.to_string(),
        })
        .collect::<String>()
        // ** = qualquer coisa incluindo /
        .replace("**", "___DOUBLESTAR___")
        // * = qualquer coisa exceto /
        .replace('*', "[^/]*")
        // ? = qualquer caractere exceto /
        .replace('?', "[^/]")
        // Restaurar **
        .replace("___DOUBLESTAR___", ".*");

    // Adicionar anchors
    regex_str = format!("^{}$", regex_str);

    match Regex::new(&regex_str) {
        Ok(regex) => regex.is_match(&normalized_path),
        Err(_) => false,
    }
}
