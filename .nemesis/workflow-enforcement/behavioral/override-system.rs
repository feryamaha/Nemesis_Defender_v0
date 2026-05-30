use std::collections::HashMap;

/// Behavioral Override System for Nemesis Enforcement Engine
/// Forca compliance quando regra e clara e detecta conflitos

#[derive(Debug, Clone)]
pub struct ComplianceResult {
    pub compliant: bool,
    pub action: String, // "allow" | "block" | "warn"
    pub reason: String,
    pub overridden_rules: Vec<String>,
    pub original_intent: String,
    pub corrected_action: String,
    pub confidence: f64, // 0-1
}

#[derive(Debug, Clone)]
pub struct BehavioralPattern {
    pub id: String,
    pub name: String,
    pub description: String,
    pub triggers: Vec<String>, // padroes que indicam este comportamento
    pub override_level: String, // "strict" | "moderate" | "lenient"
    pub corrective_action: String,
    pub examples: Vec<PatternExample>,
}

#[derive(Debug, Clone)]
pub struct PatternExample {
    pub violation: String,
    pub correction: String,
    pub explanation: String,
}

#[derive(Debug, Clone)]
pub struct RuleViolation {
    pub rule_id: String,
    pub severity: String, // "error" | "warning"
    pub category: String,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct ValidationContext {
    pub file_path: String,
    pub operation: String,
}

pub struct BehavioralOverride {
    patterns: HashMap<String, BehavioralPattern>,
    rule_violations: Vec<RuleViolation>,
    context: Option<ValidationContext>,
}

impl BehavioralOverride {
    pub fn new() -> Self {
        let mut instance = Self {
            patterns: HashMap::new(),
            rule_violations: Vec::new(),
            context: None,
        };
        instance.initialize_patterns();
        instance
    }

    fn initialize_patterns(&mut self) {
        let patterns = vec![
            BehavioralPattern {
                id: "helping-urges-override".to_string(),
                name: "Instinto de Ajudar Imediato".to_string(),
                description: "IA sente urgencia de ajudar e ignora regras".to_string(),
                triggers: vec![
                    "vou resolver isso agora".to_string(),
                    "deixe-me corrigir".to_string(),
                    "posso arrumar rapido".to_string(),
                    "vamos fazer assim".to_string(),
                    "e mais facil assim".to_string(),
                ],
                override_level: "strict".to_string(),
                corrective_action: "PARAR! Siga as regras primeiro. \"Ajudar\" significa seguir convencoes, nao quebra-las.".to_string(),
                examples: vec![
                    PatternExample {
                        violation: "Vou adicionar estado local para resolver o problema".to_string(),
                        correction: "O problema deve ser resolvido movendo logica para hooks, nao adicionando estado local".to_string(),
                        explanation: "Estado local em UI pura viola ui-separation-convention.md".to_string(),
                    },
                    PatternExample {
                        violation: "Posso usar any so desta vez para funcionar".to_string(),
                        correction: "Crie tipo especifico em src/types/ mesmo que demore mais".to_string(),
                        explanation: "Uso de any quebra contrato de tipagem do projeto".to_string(),
                    },
                ],
            },
            BehavioralPattern {
                id: "permission-interpretation-error".to_string(),
                name: "Erro de Interpretacao de Permissao".to_string(),
                description: "IA interpreta permissao como licenca para violar regras".to_string(),
                triggers: vec![
                    "usuario autorizou".to_string(),
                    "tenho permissao".to_string(),
                    "usuario disse sim".to_string(),
                    "esta autorizado".to_string(),
                ],
                override_level: "strict".to_string(),
                corrective_action: "PERMISSAO NAO E OVERRIDE! Permissao e para executar ACAO ESPECIFICA, nao para violar regras.".to_string(),
                examples: vec![
                    PatternExample {
                        violation: "Usuario autorizou edicao, entao posso usar CSS inline".to_string(),
                        correction: "Permissao foi para editar o arquivo, nao para violar design-system-convention.md".to_string(),
                        explanation: "Permissao nao anula regras do projeto".to_string(),
                    },
                    PatternExample {
                        violation: "Como autorizou, posso instalar dependencias".to_string(),
                        correction: "Permissao foi para corrigir o bug, nao para instalar pacotes sem planejamento".to_string(),
                        explanation: "Instalacoes requerem analise de impacto e seguem workflow especifico".to_string(),
                    },
                ],
            },
            BehavioralPattern {
                id: "urgency-perceived-override".to_string(),
                name: "Urgencia Percebida".to_string(),
                description: "IA sente pressao e atalha processos".to_string(),
                triggers: vec![
                    "rapido".to_string(),
                    "urgente".to_string(),
                    "pressa".to_string(),
                    "imediatamente".to_string(),
                    "sem demora".to_string(),
                ],
                override_level: "moderate".to_string(),
                corrective_action: "URGENCIA NAO JUSTIFICA VIOLACAO! Processos existem para garantir qualidade.".to_string(),
                examples: vec![
                    PatternExample {
                        violation: "Como e urgente, vou pular validacao".to_string(),
                        correction: "Mesmo urgente, execute yarn tsc --noEmit antes de prosseguir".to_string(),
                        explanation: "Validacao previne regressoes que custam mais tempo depois".to_string(),
                    },
                    PatternExample {
                        violation: "Vou usar solucao rapida com any".to_string(),
                        correction: "Crie tipagem adequada mesmo que leve mais tempo".to_string(),
                        explanation: "Divida tecnica custa mais no longo prazo".to_string(),
                    },
                ],
            },
            BehavioralPattern {
                id: "creative-solution-override".to_string(),
                name: "Solucao Criativa".to_string(),
                description: "IA cria solucao \"engenhosa\" que viola padroes".to_string(),
                triggers: vec![
                    "solucao criativa".to_string(),
                    "abordagem diferente".to_string(),
                    "jeito inteligente".to_string(),
                    "workaround inteligente".to_string(),
                ],
                override_level: "moderate".to_string(),
                corrective_action: "CRIATIVIDADE SIGUE PADROES! Solucoes devem respeitar arquitetura estabelecida.".to_string(),
                examples: vec![
                    PatternExample {
                        violation: "Criei componente hibrido que resolve tudo".to_string(),
                        correction: "Siga pipeline UI → shared → main-content conforme arquitetura".to_string(),
                        explanation: "Componentes monoliticos quebram reusabilidade e manutenibilidade".to_string(),
                    },
                    PatternExample {
                        violation: "Usei tecnica avancada para contornar limitacao".to_string(),
                        correction: "Use patterns estabelecidos mesmo que parecam mais simples".to_string(),
                        explanation: "Tecnicas nao-padronizadas criam divida de conhecimento".to_string(),
                    },
                ],
            },
            BehavioralPattern {
                id: "frustration-response-override".to_string(),
                name: "Resposta a Frustracao".to_string(),
                description: "IA responde a frustracao do usuario com atalhos".to_string(),
                triggers: vec![
                    "usuario frustrado".to_string(),
                    "esta chateado".to_string(),
                    "impaciente".to_string(),
                    "irritado".to_string(),
                ],
                override_level: "lenient".to_string(),
                corrective_action: "FRUSTRACAO NAO JUSTIFICA VIOLACAO! Mantenha qualidade mesmo sob pressao.".to_string(),
                examples: vec![
                    PatternExample {
                        violation: "Usuario esta frustrado, vou fazer gambiarra rapida".to_string(),
                        correction: "Explique o processo necessario e execute corretamente".to_string(),
                        explanation: "Gambiarras criam mais frustracao futura".to_string(),
                    },
                ],
            },
        ];

        for pattern in patterns {
            self.patterns.insert(pattern.id.clone(), pattern);
        }
    }

    /// Analisa acao pretendida e detecta conflitos comportamentais
    pub fn analyze_behavioral_conflict(
        &mut self,
        intended_action: &str,
        rule_violations: &[RuleViolation],
        context: &ValidationContext,
    ) -> ComplianceResult {
        self.rule_violations = rule_violations.to_vec();
        self.context = Some(context.clone());

        // Detectar padroes comportamentais na intencao
        let detected_patterns = self.detect_behavioral_patterns(intended_action);

        if detected_patterns.is_empty() {
            return ComplianceResult {
                compliant: true,
                action: "allow".to_string(),
                reason: "Nenhum conflito comportamental detectado".to_string(),
                overridden_rules: vec![],
                original_intent: intended_action.to_string(),
                corrected_action: intended_action.to_string(),
                confidence: 0.9,
            };
        }

        // Ordenar por nivel de override
        let mut patterns_sorted = detected_patterns.clone();
        patterns_sorted.sort_by(|a, b| {
            let levels = [("strict", 3), ("moderate", 2), ("lenient", 1)];
            let level_a = levels.iter().find(|(k, _)| k == &a.override_level).map(|(_, v)| *v).unwrap_or(0);
            let level_b = levels.iter().find(|(k, _)| k == &b.override_level).map(|(_, v)| *v).unwrap_or(0);
            level_b.cmp(&level_a)
        });

        let primary_pattern = &patterns_sorted[0];
        let has_critical_violations = rule_violations.iter().any(|v| v.severity == "error");

        // Decidir acao baseada no padrao e violacoes
        let mut action = "allow".to_string();
        let mut compliant = true;

        if primary_pattern.override_level == "strict" && has_critical_violations {
            action = "block".to_string();
            compliant = false;
        } else if primary_pattern.override_level == "moderate" && has_critical_violations {
            action = "warn".to_string();
            compliant = false;
        } else if primary_pattern.override_level == "lenient" && has_critical_violations {
            action = "warn".to_string();
            compliant = false;
        }

        // Gerar acao corrigida
        let corrected_action = self.generate_corrected_action(
            intended_action,
            primary_pattern,
            rule_violations,
        );

        let reason = self.generate_reason(primary_pattern, rule_violations, &action);

        ComplianceResult {
            compliant,
            action,
            reason,
            overridden_rules: rule_violations.iter().map(|v| v.rule_id.clone()).collect(),
            original_intent: intended_action.to_string(),
            corrected_action,
            confidence: self.calculate_confidence(&detected_patterns, rule_violations),
        }
    }

    fn detect_behavioral_patterns(&self, intended_action: &str) -> Vec<BehavioralPattern> {
        let mut detected: Vec<BehavioralPattern> = vec![];
        let normalized_action = intended_action.to_lowercase();

        for (_id, pattern) in &self.patterns {
            let has_trigger = pattern.triggers.iter().any(|trigger| {
                normalized_action.contains(&trigger.to_lowercase())
            });

            if has_trigger {
                detected.push(pattern.clone());
            }
        }

        detected
    }

    fn generate_corrected_action(
        &self,
        original: &str,
        pattern: &BehavioralPattern,
        violations: &[RuleViolation],
    ) -> String {
        // Encontrar exemplo relevante baseado nas violacoes
        let relevant_example = pattern.examples.iter().find(|example| {
            violations.iter().any(|violation| {
                example.violation.to_lowercase().contains(&violation.rule_id.to_lowercase())
                    || example.explanation.to_lowercase().contains(&violation.category.to_lowercase())
            })
        });

        if let Some(example) = relevant_example {
            return example.correction.clone();
        }

        // Fallback para acao corretiva generica do padrao
        format!("{} Acao corrigida: {}", pattern.corrective_action, original)
    }

    fn generate_reason(
        &self,
        pattern: &BehavioralPattern,
        violations: &[RuleViolation],
        action: &str,
    ) -> String {
        let violation_list: Vec<String> = violations.iter().map(|v| v.message.clone()).collect();

        match action {
            "block" => {
                format!(
                    "BLOQUEADO: {}. {} Violacoes: {}",
                    pattern.name, pattern.corrective_action, violation_list.join(", ")
                )
            }
            "warn" => {
                format!(
                    "ALERTA: {}. {} Violacoes detectadas: {}",
                    pattern.name, pattern.corrective_action, violation_list.join(", ")
                )
            }
            "allow" => {
                format!(
                    "PERMITIDO: {} detectado, mas sem violacoes criticas.",
                    pattern.name
                )
            }
            _ => "Analise comportamental concluida".to_string(),
        }
    }

    fn calculate_confidence(
        &self,
        patterns: &[BehavioralPattern],
        violations: &[RuleViolation],
    ) -> f64 {
        let mut confidence = 0.5; // base confidence

        // Mais padroes detectados = mais confianca
        confidence += patterns.len() as f64 * 0.1;

        // Violacoes criticas aumentam confianca do bloqueio
        let critical_violations = violations.iter().filter(|v| v.severity == "error").count();
        confidence += critical_violations as f64 * 0.2;

        // Padroes strict aumentam confianca
        let has_strict_pattern = patterns.iter().any(|p| p.override_level == "strict");
        if has_strict_pattern {
            confidence += 0.2;
        }

        // Limitar entre 0 e 1
        confidence.min(1.0).max(0.0)
    }

    /// Registra nova violacao para aprendizado
    pub fn register_violation(&mut self, violation: RuleViolation, context: &ValidationContext) {
        self.rule_violations.push(violation.clone());
        self.context = Some(context.clone());

        // TODO: Implementar aprendizado baseado em violacoes recorrentes
        println!("🧠 Behavioral learning: {} in {}", violation.rule_id, context.file_path);
    }

    /// Obtem estatisticas de violacoes comportamentais
    pub fn get_behavioral_stats(&self) -> BehavioralStats {
        // TODO: Implementar estatisticas reais
        BehavioralStats {
            total_violations: self.rule_violations.len() as u32,
            patterns_detected: self.patterns.keys().cloned().collect(),
            common_violations: vec![],
            override_effectiveness: vec![],
        }
    }

    /// Adiciona novo padrao comportamental
    pub fn add_pattern(&mut self, pattern: BehavioralPattern) {
        self.patterns.insert(pattern.id.clone(), pattern);
    }

    /// Lista todos os padroes comportamentais
    pub fn list_patterns(&self) -> Vec<BehavioralPattern> {
        self.patterns.values().cloned().collect()
    }
}

#[derive(Debug)]
pub struct BehavioralStats {
    pub total_violations: u32,
    pub patterns_detected: Vec<String>,
    pub common_violations: Vec<(String, u32)>,
    pub override_effectiveness: Vec<(String, f64)>,
}
