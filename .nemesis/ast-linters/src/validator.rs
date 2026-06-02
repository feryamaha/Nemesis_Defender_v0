/// Validador semântico — ponto de entrada público para o AST linter.
///
/// A função principal `validate_semantic()` detecta a linguagem do arquivo,
/// faz o parse via tree-sitter e executa todos os visitors disponíveis.
///
/// # Comportamento
///
/// - Se a linguagem não for suportada: retorna `Vec::empty()` (silencioso)
/// - Se o parse falhar: retorna `Vec::empty()` (log em debug)
/// - Se o cache tiver a entrada: usa o cache (Etapa 3)
/// - As violações incluem `message` e `line` para referência

use crate::cache;
use crate::language::detect_language;
use crate::parser::{parse_content, parse_content_with_previous_tree, TreeEdit};
use crate::lint_rule::Context;
use crate::rule_registry::RuleRegistry;
use crate::rule_wrappers;
use std::fs;
use std::path::Path;
use std::time::Instant;

/// Estrutura de violação retornada pelo validador semântico.
#[derive(Debug, Clone)]
pub struct SemanticViolation {
    pub message: String,
    pub line: usize,
    /// Camada de validação: sempre "ast"
    pub layer: &'static str,
    /// Sugestão de correção (opcional)
    pub suggestion: Option<String>,
    /// Categoria da regra (Correctness, Suspicious, Security, Style)
    pub category: String,
    /// Severidade: "critical" (bloqueia), "warn" (aviso), "info" (informativo)
    pub severity: String,
}

/// Configuração de regras AST lida do arquivo JSON.
#[derive(Debug, Clone, serde::Deserialize)]
struct AstRulesConfig {
    _version: String,
    _generated_at: String,
    rules: std::collections::HashMap<String, String>,
}

/// Carrega a configuração de regras do arquivo .nemesis/ast-rules.json.
///
/// Retorna None se o arquivo não existir ou houver erro na leitura.
fn load_ast_rules_config() -> Option<AstRulesConfig> {
    let config_path = Path::new(".nemesis/ast-rules.json");
    if !config_path.exists() {
        return None;
    }

    match fs::read_to_string(&config_path) {
        Ok(content) => match serde_json::from_str(&content) {
            Ok(config) => Some(config),
            Err(_) => None,
        },
        Err(_) => None,
    }
}

/// Cria um RuleRegistry com as regras padrão registradas.
fn create_default_registry() -> RuleRegistry {
    let mut registry = RuleRegistry::new();
    rule_wrappers::register_default_rules(&mut registry);
    registry
}

/// Cria um RuleRegistry com configuração carregada do arquivo.
///
/// Se a configuração não existir ou houver erro, usa configuração padrão.
fn create_configured_registry() -> RuleRegistry {
    let registry = create_default_registry();

    if let Some(config) = load_ast_rules_config() {
        // Converte as regras do JSON para severidades
        let mut severity_map = std::collections::HashMap::new();
        for (rule_name, severity_str) in &config.rules {
            let severity = crate::lint_rule::Severity::from_str(severity_str);
            severity_map.insert(rule_name.clone(), severity);
        }
        registry.update_config(severity_map);
    }

    registry
}

/// Valida o conteúdo de um arquivo usando AST.
///
/// # Arguments
///
/// * `content` - O conteúdo do arquivo a ser validado
/// * `file_path` - O caminho do arquivo (usado para detectar linguagem)
///
/// # Returns
///
/// Uma lista de violações semânticas encontradas. Vazia se a linguagem
/// não for suportada ou se o parse falhar.
pub fn validate_semantic(content: &str, file_path: &str) -> Vec<SemanticViolation> {
    // 0. Se o conteúdo estiver vazio, não há o que validar
    if content.is_empty() || file_path.is_empty() {
        return Vec::new();
    }

    // FASE 1 CIRÚRGICA: Documentação não passa pelo ast-linters
    // Markdown/txt/rst não é código-fonte, não tem "padrão de código" a violar
    if file_path.ends_with(".md") || file_path.ends_with(".txt") || file_path.ends_with(".rst")
        || file_path.ends_with(".markdown") {
        return Vec::new();
    }

    // 1. Detecta linguagem pela extensão do arquivo
    let language = match detect_language(file_path) {
        Some(lang) => lang,
        None => return Vec::new(), // linguagem não suportada — silencioso
    };

    // 2. Verifica cache (Etapa 3)
    if cache::has(file_path, content) {
        return Vec::new(); // Já validado, sem violações
    }

    // 3. Tenta parsing incremental se há árvore anterior no cache
    let tree = if let (Some(previous_tree), Some(previous_content)) = (
        cache::get_tree(file_path, content),
        cache::get_previous_content(file_path, content),
    ) {
        // Calcula diff para parsing incremental
        if let Some(edit) = TreeEdit::from_diff(&previous_content, content) {
            let start = Instant::now();
            let result = parse_content_with_previous_tree(content, &previous_tree, &edit);
            let duration = start.elapsed();
            
            #[cfg(debug_assertions)]
            eprintln!("[AST-LINTERS DEBUG] Incremental parse: {:?} ({}ms)", file_path, duration.as_millis());
            
            match result {
                Ok(t) => t,
                Err(_) => {
                    // Fallback para parse completo se incremental falhar
                    let start = Instant::now();
                    let result = parse_content(content, language);
                    let duration = start.elapsed();
                    
                    #[cfg(debug_assertions)]
                    eprintln!("[AST-LINTERS DEBUG] Full parse (fallback): {:?} ({}ms)", file_path, duration.as_millis());
                    
                    match result {
                        Ok(t) => t,
                        Err(e) => {
                            #[cfg(debug_assertions)]
                            eprintln!("[AST-LINTERS DEBUG] Parse failed for {}: {:?}", file_path, e);
                            return Vec::new();
                        }
                    }
                }
            }
        } else {
            // Sem diff detectado, usa parse completo
            let start = Instant::now();
            let result = parse_content(content, language);
            let duration = start.elapsed();
            
            #[cfg(debug_assertions)]
            eprintln!("[AST-LINTERS DEBUG] Full parse: {:?} ({}ms)", file_path, duration.as_millis());
            
            match result {
                Ok(t) => t,
                Err(e) => {
                    #[cfg(debug_assertions)]
                    eprintln!("[AST-LINTERS DEBUG] Parse failed for {}: {:?}", file_path, e);
                    return Vec::new();
                }
            }
        }
    } else {
        // Sem árvore anterior, usa parse completo
        let start = Instant::now();
        let result = parse_content(content, language);
        let duration = start.elapsed();
        
        #[cfg(debug_assertions)]
        eprintln!("[AST-LINTERS DEBUG] Full parse (no cache): {:?} ({}ms)", file_path, duration.as_millis());
        
        match result {
            Ok(t) => t,
            Err(e) => {
                #[cfg(debug_assertions)]
                eprintln!("[AST-LINTERS DEBUG] Parse failed for {}: {:?}", file_path, e);
                return Vec::new();
            }
        }
    };

    // 4. Armazena árvore no cache para parsing incremental futuro
    cache::put_tree(file_path, content, tree.clone());

    // 5. Cria registry configurado com regras do arquivo (ou padrão)
    let registry = create_configured_registry();

    // 6. Cria contexto de execução
    let ctx = Context::new(content, language, file_path);

    // 7. Executa regras ativas via registry
    let rule_violations = registry.run_active_rules(&tree, &ctx);

    // 8. Converte para SemanticViolation
    // Severidade padrão: qualidade=warn, segurança=critical
    let violations: Vec<SemanticViolation> = rule_violations
        .into_iter()
        .map(|v| {
            let severity = match format!("{:?}", v.category).as_str() {
                "Security" | "Correctness" => "critical",
                _ => "warn",
            };
            SemanticViolation {
                message: v.message,
                line: v.line,
                layer: "ast",
                suggestion: v.suggestion.map(|s| s.message),
                category: format!("{:?}", v.category),
                severity: severity.to_string(),
            }
        })
        .collect();

    // 9. Atualiza cache se não houver violações
    if violations.is_empty() {
        cache::put(file_path, content);
    }

    violations
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unsupported_language() {
        let violations = validate_semantic("print('hello')", "main.py");
        assert!(violations.is_empty(), "Python not supported yet");
    }

    #[test]
    fn test_empty_content() {
        let violations = validate_semantic("", "file.ts");
        assert!(violations.is_empty(), "Empty content should return empty");
    }

    #[test]
    fn test_valid_ts_no_violations() {
        let content = r#"
            function greet(name: string): string {
                return `Hello ${name}`;
            }
        "#;
        let violations = validate_semantic(content, "greet.ts");
        assert!(violations.is_empty(), "Clean TS should have no violations");
    }

    #[test]
    fn test_detects_any_via_alias() {
        let content = "type X = any;";
        let violations = validate_semantic(content, "types.ts");
        assert!(!violations.is_empty(), "Should detect type alias any");
        assert_eq!(violations[0].layer, "ast");
    }

    #[test]
    fn test_detects_floating_promise() {
        let content = r#"
            const url = "https://api.example.com";
            fetch(url);
        "#;
        let violations = validate_semantic(content, "test.ts");
        assert!(!violations.is_empty(), "Should detect floating promise");
    }

    #[test]
    fn test_awaited_promise_not_detected() {
        let content = r#"
            const url = "https://api.example.com";
            await fetch(url);
        "#;
        let violations = validate_semantic(content, "test.ts");
        assert!(violations.is_empty(), "Awaited promise should not be detected");
    }

    #[test]
    fn test_detects_unsafe_assignment() {
        let content = "const x: any = someValue;";
        let violations = validate_semantic(content, "test.ts");
        assert!(!violations.is_empty(), "Should detect unsafe assignment");
    }

    #[test]
    fn test_specific_type_not_detected() {
        let content = "const x: string = someValue;";
        let violations = validate_semantic(content, "test.ts");
        assert!(violations.is_empty(), "Specific type should not be detected");
    }

    #[test]
    fn test_detects_console_log() {
        let content = r#"
            console.log("debug message");
        "#;
        let violations = validate_semantic(content, "test.ts");
        assert!(!violations.is_empty(), "Should detect console.log");
    }

    #[test]
    fn test_console_error_not_detected() {
        let content = r#"
            console.error("error message");
        "#;
        let violations = validate_semantic(content, "test.ts");
        assert!(violations.is_empty(), "console.error should be allowed");
    }

    #[test]
    fn test_detects_jsx_target_blank_without_rel() {
        let content = r#"
            <a href="https://example.com" target="_blank">Link</a>
        "#;
        let violations = validate_semantic(content, "test.tsx");
        assert!(!violations.is_empty(), "Should detect target=_blank without rel");
    }

    #[test]
    fn test_jsx_target_blank_with_noreferrer_not_detected() {
        let content = r#"
            <a href="https://example.com" target="_blank" rel="noreferrer">Link</a>
        "#;
        let violations = validate_semantic(content, "test.tsx");
        assert!(violations.is_empty(), "target=_blank with noreferrer should be allowed");
    }
}
