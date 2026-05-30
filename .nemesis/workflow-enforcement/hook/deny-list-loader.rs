use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

// ============================================================
// DENY-LIST LOADER
// Carrega e gerencia padrões de bloqueio do deny-list.json
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum PatternType {
    Regex,
    String,
    Description,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ContextType {
    PathContains,
    PathEndsWith,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Source {
    Harvest,
    Rules,
    Manual,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DenyPattern {
    pub id: String,
    pub pattern: String,
    #[serde(rename = "type")]
    pub pattern_type: PatternType,
    pub severity: Severity,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_type: Option<ContextType>,
    pub message: String,
    pub suggestion: String,
    pub rule: String,
    pub source: Source,
    pub enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub needs_manual_pattern: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Layer {
    pub description: String,
    pub patterns: Vec<DenyPattern>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DenyList {
    pub version: String,
    pub last_updated: String,
    pub project_stack: Vec<String>,
    pub layers: HashMap<String, Layer>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tailwind_allow_list: Option<Vec<String>>,
}

// Cache estático para evitar recarregamentos
lazy_static::lazy_static! {
    static ref CACHE: Mutex<Option<DenyList>> = Mutex::new(None);
    static ref CACHE_TIMESTAMP: Mutex<u128> = Mutex::new(0);
}

const CACHE_TTL: u128 = 5000; // 5 segundos em milissegundos

fn deny_list_path() -> PathBuf {
    std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(".nemesis")
        .join("workflow-enforcement")
        .join("config")
        .join("deny-list.json")
}

/// Carrega o deny-list.json com cache baseado em mtime
pub fn load_deny_list() -> DenyList {
    let path = deny_list_path();

    if !path.exists() {
        return DenyList {
            version: "0.0.0".to_string(),
            last_updated: "".to_string(),
            project_stack: vec![],
            layers: HashMap::new(),
            tailwind_allow_list: None,
        };
    }

    let mtime = match fs::metadata(&path) {
        Ok(metadata) => match metadata.modified() {
            Ok(modified) => {
                modified
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis()
            }
            Err(_) => 0,
        },
        Err(_) => 0,
    };

    {
        let cache = CACHE.lock().unwrap();
        let cache_timestamp = CACHE_TIMESTAMP.lock().unwrap();
        if cache.is_some() && mtime == *cache_timestamp {
            return cache.as_ref().unwrap().clone();
        }
    }

    match fs::read_to_string(&path) {
        Ok(content) => match serde_json::from_str::<DenyList>(&content) {
            Ok(deny_list) => {
                let mut cache = CACHE.lock().unwrap();
                let mut cache_timestamp = CACHE_TIMESTAMP.lock().unwrap();
                *cache = Some(deny_list.clone());
                *cache_timestamp = mtime;
                deny_list
            }
            Err(_) => DenyList {
                version: "0.0.0".to_string(),
                last_updated: "".to_string(),
                project_stack: vec![],
                layers: HashMap::new(),
                tailwind_allow_list: None,
            },
        },
        Err(_) => DenyList {
            version: "0.0.0".to_string(),
            last_updated: "".to_string(),
            project_stack: vec![],
            layers: HashMap::new(),
            tailwind_allow_list: None,
        },
    }
}

/// Retorna padrões de uma camada específica (apenas regex habilitados)
pub fn get_patterns_for_layer(layer: &str) -> Vec<DenyPattern> {
    load_deny_list()
        .layers
        .get(layer)
        .map(|l| {
            l.patterns
                .iter()
                .filter(|p| p.enabled && p.pattern_type == PatternType::Regex)
                .cloned()
                .collect()
        })
        .unwrap_or_default()
}

/// Retorna todos os padrões de código (typescript, react, css, nextjs, api, security, bypass)
pub fn get_all_code_patterns() -> Vec<DenyPattern> {
    let dl = load_deny_list();
    let code_layers = [
        "typescript",
        "react",
        "css",
        "nextjs",
        "api",
        "security",
        "bypass",
    ];

    code_layers
        .iter()
        .flat_map(|layer| {
            dl.layers
                .get(*layer)
                .map(|l| {
                    l.patterns
                        .iter()
                        .filter(|p| p.enabled && p.pattern_type == PatternType::Regex)
                        .cloned()
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default()
        })
        .collect()
}

/// Retorna padrões para comandos shell
pub fn get_command_patterns() -> Vec<DenyPattern> {
    get_patterns_for_layer("commands")
}

/// Retorna padrões aplicáveis a um arquivo específico (filtra por context)
pub fn get_patterns_for_file(file_path: &str) -> Vec<DenyPattern> {
    get_all_code_patterns()
        .into_iter()
        .filter(|p| {
            if let Some(ref context) = p.context {
                if let Some(ref context_type) = p.context_type {
                    match context_type {
                        ContextType::PathContains => file_path.contains(context),
                        ContextType::PathEndsWith => file_path.ends_with(context),
                    }
                } else {
                    true
                }
            } else {
                true
            }
        })
        .collect()
}

/// Compila um padrão em regex (se for do tipo regex)
pub fn compile_pattern(p: &DenyPattern) -> Option<Regex> {
    if p.pattern_type != PatternType::Regex {
        return None;
    }
    Regex::new(&p.pattern).ok()
}

/// Verifica se o conteúdo de um arquivo viola algum padrão
pub fn check_content(file_path: &str, content: &str) -> Option<DenyPattern> {
    for p in get_patterns_for_file(file_path) {
        if let Some(regex) = compile_pattern(&p) {
            if regex.is_match(content) {
                return Some(p);
            }
        }
    }
    None
}

/// Verifica se um comando viola algum padrão de comandos bloqueados
pub fn check_command(command: &str) -> Option<DenyPattern> {
    for p in get_command_patterns() {
        if let Some(regex) = compile_pattern(&p) {
            if regex.is_match(command) {
                return Some(p);
            }
        }
    }
    None
}

/// Combina deny-lists em ordem: base + stack + generic
/// Retorna DenyList final com patterns de todos
pub fn load_and_combine_deny_lists(stacks: &[String]) -> DenyList {
    let mut combined = DenyList {
        version: "2.0.0".to_string(),
        last_updated: chrono::Utc::now().to_rfc3339(),
        project_stack: stacks.to_vec(),
        layers: HashMap::new(),
        tailwind_allow_list: None,
    };

    // 1. Carregar base
    if let Ok(base_content) = std::fs::read_to_string(
        std::env::current_dir().unwrap_or_default()
            .join(".nemesis/workflow-enforcement/config/deny-list-base.json")
    ) {
        if let Ok(base) = serde_json::from_str::<DenyList>(&base_content) {
            for (layer_name, layer) in base.layers {
                combined.layers.insert(layer_name, layer);
            }
        }
    }

    // 2. Carregar stack-específicos
    for stack in stacks {
        let stack_file = format!("deny-list-{}.json", stack.to_lowercase());
        let path = std::env::current_dir().unwrap_or_default()
            .join(format!(".nemesis/workflow-enforcement/config/{}", stack_file));

        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(stack_deny_list) = serde_json::from_str::<DenyList>(&content) {
                    for (layer_name, mut stack_layer) in stack_deny_list.layers {
                        if let Some(existing_layer) = combined.layers.get_mut(&layer_name) {
                            existing_layer.patterns.extend(stack_layer.patterns);
                        } else {
                            combined.layers.insert(layer_name, stack_layer);
                        }
                    }
                }
            }
        }
    }

    // 3. Carregar generic (sempre)
    let generic_path = std::env::current_dir().unwrap_or_default()
        .join(".nemesis/workflow-enforcement/config/deny-list-generic.json");

    if generic_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&generic_path) {
            if let Ok(generic_deny_list) = serde_json::from_str::<DenyList>(&content) {
                for (layer_name, mut generic_layer) in generic_deny_list.layers {
                    if let Some(existing_layer) = combined.layers.get_mut(&layer_name) {
                        existing_layer.patterns.extend(generic_layer.patterns);
                    } else {
                        combined.layers.insert(layer_name, generic_layer);
                    }
                }
            }
        }
    }

    // Gerar arquivo deny-list.json final
    if let Ok(json) = serde_json::to_string_pretty(&combined) {
        let deny_list_path = std::env::current_dir().unwrap_or_default()
            .join(".nemesis/workflow-enforcement/config/deny-list.json");
        let _ = std::fs::write(&deny_list_path, json);
    }

    combined
}

/// Combina deny-lists com dedup por pattern ID e resolução de conflitos de severity
/// Implementa lógica avançada:
/// - Dedup por pattern ID
/// - Merge patterns com mesmo ID (usa mais restritivo)
/// - Adiciona novos layers
/// - Conflito severity → Critical > High > Medium
pub fn combine_deny_lists_advanced(
    base: &DenyList,
    stack_lists: &[DenyList],
    generic: &DenyList,
) -> DenyList {
    let mut combined = DenyList {
        version: "3.0.0".to_string(),
        last_updated: chrono::Utc::now().to_rfc3339(),
        project_stack: vec![], // Será preenchido com stacks detectados
        layers: HashMap::new(),
        tailwind_allow_list: None,
    };

    // 1. Adicionar base
    for (layer_name, layer) in &base.layers {
        combined.layers.insert(layer_name.clone(), layer.clone());
    }

    // 2. Mergear stack-specific (dedup por ID)
    for stack_list in stack_lists {
        for (layer_name, stack_layer) in &stack_list.layers {
            if let Some(existing_layer) = combined.layers.get_mut(layer_name) {
                // Merge patterns: dedup por ID
                let mut id_map: HashMap<String, DenyPattern> = existing_layer
                    .patterns
                    .iter()
                    .map(|p| (p.id.clone(), p.clone()))
                    .collect();

                for pattern in &stack_layer.patterns {
                    id_map
                        .entry(pattern.id.clone())
                        .and_modify(|existing| {
                            // Conflito severity: mais restritivo
                            if pattern.severity > existing.severity {
                                existing.severity = pattern.severity.clone();
                            }
                        })
                        .or_insert_with(|| pattern.clone());
                }

                existing_layer.patterns = id_map.values().cloned().collect();
            } else {
                combined.layers.insert(layer_name.clone(), stack_layer.clone());
            }
        }
    }

    // 3. Mergear generic (sempre adiciona)
    for (layer_name, generic_layer) in &generic.layers {
        if let Some(existing_layer) = combined.layers.get_mut(layer_name) {
            // Merge com dedup
            let mut id_map: HashMap<String, DenyPattern> = existing_layer
                .patterns
                .iter()
                .map(|p| (p.id.clone(), p.clone()))
                .collect();

            for pattern in &generic_layer.patterns {
                id_map.entry(pattern.id.clone()).or_insert_with(|| pattern.clone());
            }

            existing_layer.patterns = id_map.values().cloned().collect();
        } else {
            combined.layers.insert(layer_name.clone(), generic_layer.clone());
        }
    }

    combined
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_and_combine_deny_lists_empty_stacks() {
        let result = load_and_combine_deny_lists(&[]);
        assert_eq!(result.version, "2.0.0");
        assert_eq!(result.project_stack, vec![String::new(); 0]);
    }

    #[test]
    fn test_load_and_combine_deny_lists_with_stacks() {
        let stacks = vec!["typescript".to_string(), "rust".to_string()];
        let result = load_and_combine_deny_lists(&stacks);
        assert_eq!(result.version, "2.0.0");
        assert_eq!(result.project_stack, stacks);
    }

    // Tests for combine_deny_lists_advanced()

    #[test]
    fn test_combine_deny_lists_advanced_basic_merge() {
        // Create base deny list with one pattern
        let base = DenyList {
            version: "1.0.0".to_string(),
            last_updated: "2024-01-01T00:00:00Z".to_string(),
            project_stack: vec![],
            layers: {
                let mut layers = HashMap::new();
                let pattern = DenyPattern {
                    id: "base-001".to_string(),
                    pattern: "test_pattern".to_string(),
                    pattern_type: PatternType::Regex,
                    severity: Severity::Medium,
                    context: None,
                    context_type: None,
                    message: "Base pattern".to_string(),
                    suggestion: "Fix it".to_string(),
                    rule: "rule-base".to_string(),
                    source: Source::Manual,
                    enabled: true,
                    needs_manual_pattern: None,
                };
                layers.insert(
                    "test-layer".to_string(),
                    Layer {
                        description: "Test layer".to_string(),
                        patterns: vec![pattern],
                    },
                );
                layers
            },
            tailwind_allow_list: None,
        };

        let stack_lists: Vec<DenyList> = vec![];
        let generic = DenyList {
            version: "1.0.0".to_string(),
            last_updated: "2024-01-01T00:00:00Z".to_string(),
            project_stack: vec![],
            layers: HashMap::new(),
            tailwind_allow_list: None,
        };

        let result = combine_deny_lists_advanced(&base, &stack_lists, &generic);

        assert_eq!(result.version, "3.0.0");
        assert!(result.layers.contains_key("test-layer"));
        let layer = result.layers.get("test-layer").unwrap();
        assert_eq!(layer.patterns.len(), 1);
        assert_eq!(layer.patterns[0].id, "base-001");
    }

    #[test]
    fn test_combine_deny_lists_advanced_dedup_by_id() {
        // Create base with pattern ID "dup-001"
        let base = DenyList {
            version: "1.0.0".to_string(),
            last_updated: "2024-01-01T00:00:00Z".to_string(),
            project_stack: vec![],
            layers: {
                let mut layers = HashMap::new();
                let pattern = DenyPattern {
                    id: "dup-001".to_string(),
                    pattern: "pattern_v1".to_string(),
                    pattern_type: PatternType::Regex,
                    severity: Severity::Medium,
                    context: None,
                    context_type: None,
                    message: "Base version".to_string(),
                    suggestion: "Fix v1".to_string(),
                    rule: "rule-base".to_string(),
                    source: Source::Manual,
                    enabled: true,
                    needs_manual_pattern: None,
                };
                layers.insert(
                    "test-layer".to_string(),
                    Layer {
                        description: "Test layer".to_string(),
                        patterns: vec![pattern],
                    },
                );
                layers
            },
            tailwind_allow_list: None,
        };

        // Create stack list with same ID but different pattern
        let stack_lists = vec![DenyList {
            version: "1.0.0".to_string(),
            last_updated: "2024-01-01T00:00:00Z".to_string(),
            project_stack: vec!["stack1".to_string()],
            layers: {
                let mut layers = HashMap::new();
                let pattern = DenyPattern {
                    id: "dup-001".to_string(),
                    pattern: "pattern_v2".to_string(),
                    pattern_type: PatternType::Regex,
                    severity: Severity::High,
                    context: None,
                    context_type: None,
                    message: "Stack version".to_string(),
                    suggestion: "Fix v2".to_string(),
                    rule: "rule-stack".to_string(),
                    source: Source::Harvest,
                    enabled: true,
                    needs_manual_pattern: None,
                };
                layers.insert(
                    "test-layer".to_string(),
                    Layer {
                        description: "Test layer".to_string(),
                        patterns: vec![pattern],
                    },
                );
                layers
            },
            tailwind_allow_list: None,
        }];

        let generic = DenyList {
            version: "1.0.0".to_string(),
            last_updated: "2024-01-01T00:00:00Z".to_string(),
            project_stack: vec![],
            layers: HashMap::new(),
            tailwind_allow_list: None,
        };

        let result = combine_deny_lists_advanced(&base, &stack_lists, &generic);

        // Should only have 1 pattern (dedup by ID)
        let layer = result.layers.get("test-layer").unwrap();
        assert_eq!(layer.patterns.len(), 1);
        assert_eq!(layer.patterns[0].id, "dup-001");
        // Should use more restrictive severity (High > Medium)
        assert_eq!(layer.patterns[0].severity, Severity::High);
    }

    #[test]
    fn test_combine_deny_lists_advanced_severity_conflict_critical_wins() {
        // Base with Medium severity
        let base = DenyList {
            version: "1.0.0".to_string(),
            last_updated: "2024-01-01T00:00:00Z".to_string(),
            project_stack: vec![],
            layers: {
                let mut layers = HashMap::new();
                let pattern = DenyPattern {
                    id: "sev-001".to_string(),
                    pattern: "pattern".to_string(),
                    pattern_type: PatternType::Regex,
                    severity: Severity::Medium,
                    context: None,
                    context_type: None,
                    message: "Base".to_string(),
                    suggestion: "Fix".to_string(),
                    rule: "rule".to_string(),
                    source: Source::Manual,
                    enabled: true,
                    needs_manual_pattern: None,
                };
                layers.insert(
                    "test-layer".to_string(),
                    Layer {
                        description: "Test layer".to_string(),
                        patterns: vec![pattern],
                    },
                );
                layers
            },
            tailwind_allow_list: None,
        };

        // Stack with Critical severity
        let stack_lists = vec![DenyList {
            version: "1.0.0".to_string(),
            last_updated: "2024-01-01T00:00:00Z".to_string(),
            project_stack: vec!["stack1".to_string()],
            layers: {
                let mut layers = HashMap::new();
                let pattern = DenyPattern {
                    id: "sev-001".to_string(),
                    pattern: "pattern".to_string(),
                    pattern_type: PatternType::Regex,
                    severity: Severity::Critical,
                    context: None,
                    context_type: None,
                    message: "Stack".to_string(),
                    suggestion: "Fix".to_string(),
                    rule: "rule".to_string(),
                    source: Source::Harvest,
                    enabled: true,
                    needs_manual_pattern: None,
                };
                layers.insert(
                    "test-layer".to_string(),
                    Layer {
                        description: "Test layer".to_string(),
                        patterns: vec![pattern],
                    },
                );
                layers
            },
            tailwind_allow_list: None,
        }];

        let generic = DenyList {
            version: "1.0.0".to_string(),
            last_updated: "2024-01-01T00:00:00Z".to_string(),
            project_stack: vec![],
            layers: HashMap::new(),
            tailwind_allow_list: None,
        };

        let result = combine_deny_lists_advanced(&base, &stack_lists, &generic);

        let layer = result.layers.get("test-layer").unwrap();
        assert_eq!(layer.patterns.len(), 1);
        // Critical should win
        assert_eq!(layer.patterns[0].severity, Severity::Critical);
    }

    #[test]
    fn test_combine_deny_lists_advanced_add_new_layer() {
        let base = DenyList {
            version: "1.0.0".to_string(),
            last_updated: "2024-01-01T00:00:00Z".to_string(),
            project_stack: vec![],
            layers: HashMap::new(),
            tailwind_allow_list: None,
        };

        // Stack adds a new layer
        let stack_lists = vec![DenyList {
            version: "1.0.0".to_string(),
            last_updated: "2024-01-01T00:00:00Z".to_string(),
            project_stack: vec!["stack1".to_string()],
            layers: {
                let mut layers = HashMap::new();
                let pattern = DenyPattern {
                    id: "new-layer-001".to_string(),
                    pattern: "pattern".to_string(),
                    pattern_type: PatternType::Regex,
                    severity: Severity::High,
                    context: None,
                    context_type: None,
                    message: "New layer pattern".to_string(),
                    suggestion: "Fix".to_string(),
                    rule: "rule".to_string(),
                    source: Source::Harvest,
                    enabled: true,
                    needs_manual_pattern: None,
                };
                layers.insert(
                    "new-layer".to_string(),
                    Layer {
                        description: "New layer".to_string(),
                        patterns: vec![pattern],
                    },
                );
                layers
            },
            tailwind_allow_list: None,
        }];

        let generic = DenyList {
            version: "1.0.0".to_string(),
            last_updated: "2024-01-01T00:00:00Z".to_string(),
            project_stack: vec![],
            layers: HashMap::new(),
            tailwind_allow_list: None,
        };

        let result = combine_deny_lists_advanced(&base, &stack_lists, &generic);

        // Should have the new layer
        assert!(result.layers.contains_key("new-layer"));
        let layer = result.layers.get("new-layer").unwrap();
        assert_eq!(layer.patterns.len(), 1);
        assert_eq!(layer.patterns[0].id, "new-layer-001");
    }

    #[test]
    fn test_combine_deny_lists_advanced_generic_always_added() {
        let base = DenyList {
            version: "1.0.0".to_string(),
            last_updated: "2024-01-01T00:00:00Z".to_string(),
            project_stack: vec![],
            layers: HashMap::new(),
            tailwind_allow_list: None,
        };

        let stack_lists: Vec<DenyList> = vec![];

        // Generic adds a pattern to a non-existing layer
        let generic = DenyList {
            version: "1.0.0".to_string(),
            last_updated: "2024-01-01T00:00:00Z".to_string(),
            project_stack: vec![],
            layers: {
                let mut layers = HashMap::new();
                let pattern = DenyPattern {
                    id: "generic-001".to_string(),
                    pattern: "generic_pattern".to_string(),
                    pattern_type: PatternType::Regex,
                    severity: Severity::Medium,
                    context: None,
                    context_type: None,
                    message: "Generic pattern".to_string(),
                    suggestion: "Fix".to_string(),
                    rule: "rule".to_string(),
                    source: Source::Rules,
                    enabled: true,
                    needs_manual_pattern: None,
                };
                layers.insert(
                    "generic-layer".to_string(),
                    Layer {
                        description: "Generic layer".to_string(),
                        patterns: vec![pattern],
                    },
                );
                layers
            },
            tailwind_allow_list: None,
        };

        let result = combine_deny_lists_advanced(&base, &stack_lists, &generic);

        // Should have the generic layer
        assert!(result.layers.contains_key("generic-layer"));
        let layer = result.layers.get("generic-layer").unwrap();
        assert_eq!(layer.patterns.len(), 1);
        assert_eq!(layer.patterns[0].id, "generic-001");
    }

    #[test]
    fn test_combine_deny_lists_advanced_multiple_stack_layers() {
        let base = DenyList {
            version: "1.0.0".to_string(),
            last_updated: "2024-01-01T00:00:00Z".to_string(),
            project_stack: vec![],
            layers: {
                let mut layers = HashMap::new();
                let pattern = DenyPattern {
                    id: "base-001".to_string(),
                    pattern: "pattern".to_string(),
                    pattern_type: PatternType::Regex,
                    severity: Severity::Medium,
                    context: None,
                    context_type: None,
                    message: "Base".to_string(),
                    suggestion: "Fix".to_string(),
                    rule: "rule".to_string(),
                    source: Source::Manual,
                    enabled: true,
                    needs_manual_pattern: None,
                };
                layers.insert(
                    "base-layer".to_string(),
                    Layer {
                        description: "Base layer".to_string(),
                        patterns: vec![pattern],
                    },
                );
                layers
            },
            tailwind_allow_list: None,
        };

        // Two stack lists
        let stack_lists = vec![
            DenyList {
                version: "1.0.0".to_string(),
                last_updated: "2024-01-01T00:00:00Z".to_string(),
                project_stack: vec!["stack1".to_string()],
                layers: {
                    let mut layers = HashMap::new();
                    let pattern = DenyPattern {
                        id: "stack1-001".to_string(),
                        pattern: "pattern".to_string(),
                        pattern_type: PatternType::Regex,
                        severity: Severity::High,
                        context: None,
                        context_type: None,
                        message: "Stack1".to_string(),
                        suggestion: "Fix".to_string(),
                        rule: "rule".to_string(),
                        source: Source::Harvest,
                        enabled: true,
                        needs_manual_pattern: None,
                    };
                    layers.insert(
                        "base-layer".to_string(),
                        Layer {
                            description: "Base layer".to_string(),
                            patterns: vec![pattern],
                        },
                    );
                    layers
                },
                tailwind_allow_list: None,
            },
            DenyList {
                version: "1.0.0".to_string(),
                last_updated: "2024-01-01T00:00:00Z".to_string(),
                project_stack: vec!["stack2".to_string()],
                layers: {
                    let mut layers = HashMap::new();
                    let pattern = DenyPattern {
                        id: "stack2-001".to_string(),
                        pattern: "pattern".to_string(),
                        pattern_type: PatternType::Regex,
                        severity: Severity::Critical,
                        context: None,
                        context_type: None,
                        message: "Stack2".to_string(),
                        suggestion: "Fix".to_string(),
                        rule: "rule".to_string(),
                        source: Source::Harvest,
                        enabled: true,
                        needs_manual_pattern: None,
                    };
                    layers.insert(
                        "base-layer".to_string(),
                        Layer {
                            description: "Base layer".to_string(),
                            patterns: vec![pattern],
                        },
                    );
                    layers
                },
                tailwind_allow_list: None,
            },
        ];

        let generic = DenyList {
            version: "1.0.0".to_string(),
            last_updated: "2024-01-01T00:00:00Z".to_string(),
            project_stack: vec![],
            layers: HashMap::new(),
            tailwind_allow_list: None,
        };

        let result = combine_deny_lists_advanced(&base, &stack_lists, &generic);

        // Should have 3 unique patterns (base, stack1, stack2)
        let layer = result.layers.get("base-layer").unwrap();
        assert_eq!(layer.patterns.len(), 3);

        let ids: std::collections::HashSet<_> = layer.patterns.iter().map(|p| p.id.clone()).collect();
        assert!(ids.contains("base-001"));
        assert!(ids.contains("stack1-001"));
        assert!(ids.contains("stack2-001"));
    }

    #[test]
    fn test_combine_deny_lists_advanced_version_updated() {
        let base = DenyList {
            version: "1.0.0".to_string(),
            last_updated: "2024-01-01T00:00:00Z".to_string(),
            project_stack: vec![],
            layers: HashMap::new(),
            tailwind_allow_list: None,
        };

        let result = combine_deny_lists_advanced(&base, &[], &DenyList {
            version: "1.0.0".to_string(),
            last_updated: "2024-01-01T00:00:00Z".to_string(),
            project_stack: vec![],
            layers: HashMap::new(),
            tailwind_allow_list: None,
        });

        // Version should be updated to 3.0.0
        assert_eq!(result.version, "3.0.0");
        // last_updated should be recent (just check it's not empty and different from old)
        assert!(!result.last_updated.is_empty());
        assert_ne!(result.last_updated, "2024-01-01T00:00:00Z");
    }

    #[test]
    fn test_severity_ordering() {
        assert!(Severity::Critical > Severity::High);
        assert!(Severity::High > Severity::Medium);
        assert!(Severity::Critical > Severity::Medium);

        // Test equality
        assert_eq!(Severity::Critical, Severity::Critical);
        assert_ne!(Severity::Critical, Severity::High);
    }
}
