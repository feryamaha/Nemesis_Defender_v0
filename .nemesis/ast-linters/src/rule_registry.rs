/// RuleRegistry - Registro dinâmico de regras de lint.
///
/// Este módulo implementa um registro de regras que permite ativar/desativar
/// regras dinamicamente sem recompilar o crate, baseado em configuração
/// externa (arquivo JSON gerado pelo harvest).
use crate::lint_rule::{LintRule, Severity, Context};
use crate::parser::ParsedTree;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Configuração de regras lida de arquivo JSON.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct RuleConfig {
    /// Mapa de nome da regra para severidade configurada.
    pub rules: HashMap<String, String>,
}

/// Registry de regras de lint.
pub struct RuleRegistry {
    /// Todas as regras disponíveis (registradas).
    rules: HashMap<String, Box<dyn LintRule>>,
    /// Configuração de regras (sevridade por nome).
    config: Arc<RwLock<HashMap<String, Severity>>>,
}

impl RuleRegistry {
    /// Cria um novo RuleRegistry vazio.
    pub fn new() -> Self {
        Self {
            rules: HashMap::new(),
            config: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Registra uma nova regra no registry.
    pub fn register(&mut self, rule: Box<dyn LintRule>) {
        let name = rule.name().to_string();
        self.rules.insert(name, rule);
    }

    /// Atualiza a configuração de regras a partir de um HashMap.
    pub fn update_config(&self, config: HashMap<String, Severity>) {
        if let Ok(mut cfg) = self.config.write() {
            *cfg = config;
        }
    }

    /// Atualiza a configuração de regras a partir de um RuleConfig (JSON).
    pub fn update_config_from_json(&self, json_config: &RuleConfig) {
        let mut config = HashMap::new();
        for (name, severity_str) in &json_config.rules {
            let severity = Severity::from_str(severity_str);
            config.insert(name.clone(), severity);
        }
        self.update_config(config);
    }

    /// Retorna a severidade configurada para uma regra.
    ///
    /// Se a regra não estiver configurada, retorna a severidade padrão da regra.
    pub fn get_severity(&self, rule_name: &str) -> Severity {
        if let Ok(config) = self.config.read() {
            if let Some(&severity) = config.get(rule_name) {
                return severity;
            }
        }
        
        // Fallback para severidade padrão da regra
        if let Some(rule) = self.rules.get(rule_name) {
            return rule.default_severity();
        }
        
        Severity::Error // Default se a regra não existir
    }

    /// Executa todas as regras ativas para a linguagem dada.
    ///
    /// # Arguments
    ///
    /// * `tree` - Árvore parseada pelo tree-sitter
    /// * `ctx` - Contexto de execução
    ///
    /// # Returns
    ///
    /// Lista de violações detectadas pelas regras ativas.
    pub fn run_active_rules(&self, tree: &ParsedTree, ctx: &Context) -> Vec<crate::lint_rule::Violation> {
        let mut violations = Vec::new();
        
        for (name, rule) in &self.rules {
            // Verifica se a regra deve rodar para esta linguagem
            if !rule.should_run(ctx.language) {
                continue;
            }
            
            // Verifica se a regra está ativa (não "off")
            let severity = self.get_severity(name);
            if severity == Severity::Off {
                continue;
            }
            
            // Executa a regra
            for mut violation in rule.visit(tree, ctx) {
                // Adiciona o nome da regra e severidade configurada à violação
                violation.rule_name = name.clone();
                violation.severity = severity;
                violations.push(violation);
            }
        }
        
        violations
    }

    /// Retorna uma lista de todas as regras registradas.
    pub fn list_rules(&self) -> Vec<String> {
        self.rules.keys().cloned().collect()
    }

    /// Retorna o número de regras registradas.
    pub fn rule_count(&self) -> usize {
        self.rules.len()
    }
}

impl Default for RuleRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock rule para testes
    struct MockRule {
        name: String,
        severity: Severity,
    }

    impl MockRule {
        fn new(name: &str, severity: Severity) -> Self {
            Self {
                name: name.to_string(),
                severity,
            }
        }
    }

    impl LintRule for MockRule {
        fn name(&self) -> &str {
            &self.name
        }

        fn category(&self) -> crate::lint_rule::RuleCategory {
            crate::lint_rule::RuleCategory::Correctness
        }

        fn default_severity(&self) -> Severity {
            self.severity
        }

        fn visit(&self, _tree: &ParsedTree, _ctx: &Context) -> Vec<crate::lint_rule::Violation> {
            Vec::new()
        }
    }

    #[test]
    fn test_register_rule() {
        let mut registry = RuleRegistry::new();
        let rule = Box::new(MockRule::new("test-rule", Severity::Error));
        registry.register(rule);
        
        assert_eq!(registry.rule_count(), 1);
        assert!(registry.list_rules().contains(&"test-rule".to_string()));
    }

    #[test]
    fn test_get_severity_default() {
        let mut registry = RuleRegistry::new();
        let rule = Box::new(MockRule::new("test-rule", Severity::Warning));
        registry.register(rule);
        
        let severity = registry.get_severity("test-rule");
        assert_eq!(severity, Severity::Warning);
    }

    #[test]
    fn test_get_severity_configured() {
        let mut registry = RuleRegistry::new();
        let rule = Box::new(MockRule::new("test-rule", Severity::Warning));
        registry.register(rule);
        
        let mut config = HashMap::new();
        config.insert("test-rule".to_string(), Severity::Error);
        registry.update_config(config);
        
        let severity = registry.get_severity("test-rule");
        assert_eq!(severity, Severity::Error);
    }

    #[test]
    fn test_get_severity_off() {
        let mut registry = RuleRegistry::new();
        let rule = Box::new(MockRule::new("test-rule", Severity::Error));
        registry.register(rule);
        
        let mut config = HashMap::new();
        config.insert("test-rule".to_string(), Severity::Off);
        registry.update_config(config);
        
        let severity = registry.get_severity("test-rule");
        assert_eq!(severity, Severity::Off);
    }

    #[test]
    fn test_update_config_from_json() {
        let mut registry = RuleRegistry::new();
        let rule = Box::new(MockRule::new("test-rule", Severity::Warning));
        registry.register(rule);
        
        let json_config = RuleConfig {
            rules: {
                let mut map = HashMap::new();
                map.insert("test-rule".to_string(), "error".to_string());
                map
            },
        };
        
        registry.update_config_from_json(&json_config);
        
        let severity = registry.get_severity("test-rule");
        assert_eq!(severity, Severity::Error);
    }
}
