use std::collections::HashMap;

/// Gap Detector for Nemesis Enforcement Engine
/// Detecta quando IA le regra mas planeja violar

#[derive(Debug, Clone)]
pub struct GapAnalysis {
    pub has_gap: bool,
    pub gap_type: String, // "rule-read-vs-action" | "understanding-vs-execution" | "permission-vs-violation" | "none"
    pub severity: String, // "critical" | "high" | "medium" | "low"
    pub rule_read: String,
    pub action_planned: String,
    pub conflict: String,
    pub explanation: String,
    pub suggested_correction: String,
    pub confidence: f64,
}

#[derive(Debug, Clone)]
pub struct RuleComprehension {
    pub rule_id: String,
    pub rule_content: String,
    pub understood: bool,
    pub interpretation: String,
    pub correct_interpretation: String,
    pub confidence: f64,
}

#[derive(Debug, Clone)]
pub struct ActionPlan {
    pub action: String,
    pub intent: String,
    pub violates_rules: Vec<String>,
    pub justification: Option<String>,
    pub alternatives: Vec<String>,
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

pub struct GapDetector {
    rule_comprehensions: HashMap<String, RuleComprehension>,
    recent_actions: Vec<ActionPlan>,
    gap_history: Vec<GapAnalysis>,
}

impl GapDetector {
    pub fn new() -> Self {
        Self {
            rule_comprehensions: HashMap::new(),
            recent_actions: Vec::new(),
            gap_history: Vec::new(),
        }
    }

    /// Analisa gap entre regra lida e acao pretendida
    pub fn analyze_gap(
        &mut self,
        rule_read: &str,
        action_planned: &str,
        rule_violations: &[RuleViolation],
        context: &ValidationContext,
    ) -> GapAnalysis {
        // Registrar compreensao da regra
        let comprehension = self.analyze_rule_comprehension(rule_read);
        self.rule_comprehensions.insert(rule_read.to_string(), comprehension.clone());

        // Registrar acao planejada
        let action_plan = self.analyze_action_plan(action_planned, rule_violations);
        self.recent_actions.push(action_plan.clone());

        // Detectar tipo de gap
        let gap_type = self.detect_gap_type(&comprehension, &action_plan, rule_violations);

        if gap_type == "none" {
            return GapAnalysis {
                has_gap: false,
                gap_type: "none".to_string(),
                severity: "low".to_string(),
                rule_read: rule_read.to_string(),
                action_planned: action_planned.to_string(),
                conflict: "".to_string(),
                explanation: "Nenhum gap detectado entre regra e acao".to_string(),
                suggested_correction: "".to_string(),
                confidence: 0.9,
            };
        }

        // Gerar analise detalhada do gap
        let analysis = self.generate_gap_analysis(
            &gap_type,
            &comprehension,
            &action_plan,
            rule_violations,
            context,
        );

        // Registrar para aprendizado
        self.gap_history.push(analysis.clone());

        analysis
    }

    fn analyze_rule_comprehension(&self, rule_content: &str) -> RuleComprehension {
        // Simular analise de compreensao da regra
        let understood = !rule_content.contains("NAO") || rule_content.contains("PROIBIDO");

        let mut interpretation = String::new();
        let mut correct_interpretation = String::new();
        let mut confidence = 0.7;

        // Analise baseada em conteudo da regra
        if rule_content.contains("NUNCA edite sem permissao") {
            interpretation = "Entendi que preciso pedir permissao".to_string();
            correct_interpretation = "Regra: NUNCA edite arquivo sem permissao explicita do usuario".to_string();
            confidence = 0.9;
        } else if rule_content.contains("PROIBIDO usar any") {
            interpretation = "Entendi que any nao deve ser usado".to_string();
            correct_interpretation = "Regra: Uso de tipo any e estritamente proibido em todo o projeto".to_string();
            confidence = 0.8;
        } else if rule_content.contains("CSS inline e proibido") {
            interpretation = "Entendi que CSS inline nao e permitido".to_string();
            correct_interpretation = "Regra: CSS inline viola design-system-convention.md e deve usar Tailwind".to_string();
            confidence = 0.8;
        } else {
            interpretation = "Li a regra mas nao tenho certeza do significado exato".to_string();
            correct_interpretation = "Regra precisa ser seguida rigorosamente como escrita".to_string();
            confidence = 0.5;
        }

        RuleComprehension {
            rule_id: self.extract_rule_id(rule_content),
            rule_content: rule_content.to_string(),
            understood,
            interpretation,
            correct_interpretation,
            confidence,
        }
    }

    fn analyze_action_plan(&self, action_planned: &str, violations: &[RuleViolation]) -> ActionPlan {
        let violates_rules: Vec<String> = violations.iter().map(|v| v.rule_id.clone()).collect();

        let mut intent = String::new();
        let mut justification = None;
        let mut alternatives: Vec<String> = Vec::new();

        // Analisar intencao baseada na acao
        if action_planned.contains("vou criar") {
            intent = "criar componente/solucao".to_string();
        } else if action_planned.contains("vou corrigir") {
            intent = "corrigir erro".to_string();
        } else if action_planned.contains("vou instalar") {
            intent = "instalar dependencia".to_string();
        } else if action_planned.contains("vou refatorar") {
            intent = "melhorar estrutura".to_string();
        } else {
            intent = "executar acao geral".to_string();
        }

        // Detectar justificativas para violacoes
        if action_planned.contains("porque e mais rapido") || action_planned.contains("para agilizar") {
            justification = Some("razoes de velocidade".to_string());
        } else if action_planned.contains("usuario autorizou") || action_planned.contains("tenho permissao") {
            justification = Some("permissao interpretada como override".to_string());
        } else if action_planned.contains("e so desta vez") || action_planned.contains("temporario") {
            justification = Some("solucao temporaria".to_string());
        }

        // Gerar alternativas que nao violam regras
        if violates_rules.contains(&"ts-any-prohibited".to_string()) {
            alternatives.push("Criar tipo especifico em src/types/".to_string());
            alternatives.push("Usar tipo unknown com validacao".to_string());
        }
        if violates_rules.contains(&"ds-css-inline-prohibited".to_string()) {
            alternatives.push("Usar classes Tailwind existentes".to_string());
            alternatives.push("Criar novo token no tailwind.config.ts".to_string());
        }
        if violates_rules.contains(&"ui-logic-in-pure-components".to_string()) {
            alternatives.push("Mover logica para hook em src/hooks/".to_string());
            alternatives.push("Criar hook customizado para este comportamento".to_string());
        }

        ActionPlan {
            action: action_planned.to_string(),
            intent,
            violates_rules,
            justification,
            alternatives,
        }
    }

    fn detect_gap_type(
        &self,
        comprehension: &RuleComprehension,
        action_plan: &ActionPlan,
        violations: &[RuleViolation],
    ) -> String {
        // Se nao ha violacoes, nao ha gap
        if violations.is_empty() {
            return "none".to_string();
        }

        // Se compreendeu a regra mas viola mesmo assim
        if comprehension.understood && !action_plan.violates_rules.is_empty() {
            return "rule-read-vs-action".to_string();
        }

        // Se usa permissao como justificativa para violar
        if let Some(ref just) = action_plan.justification {
            if just.contains("permissao") {
                return "permission-vs-violation".to_string();
            }
        }

        // Gap geral de entendimento vs execucao
        if !comprehension.understood || !action_plan.violates_rules.is_empty() {
            return "understanding-vs-execution".to_string();
        }

        "none".to_string()
    }

    fn generate_gap_analysis(
        &self,
        gap_type: &str,
        comprehension: &RuleComprehension,
        action_plan: &ActionPlan,
        violations: &[RuleViolation],
        _context: &ValidationContext,
    ) -> GapAnalysis {
        let severity = self.calculate_severity(gap_type, violations);
        let conflict = self.generate_conflict_description(gap_type, comprehension, action_plan);
        let explanation = self.generate_explanation(gap_type, comprehension, action_plan);
        let suggested_correction = self.generate_suggested_correction(gap_type, action_plan);
        let confidence = self.calculate_gap_confidence(gap_type, comprehension, action_plan);

        GapAnalysis {
            has_gap: true,
            gap_type: gap_type.to_string(),
            severity,
            rule_read: comprehension.rule_content.clone(),
            action_planned: action_plan.action.clone(),
            conflict,
            explanation,
            suggested_correction,
            confidence,
        }
    }

    fn calculate_severity(&self, gap_type: &str, violations: &[RuleViolation]) -> String {
        let has_errors = violations.iter().any(|v| v.severity == "error");
        let _has_warnings = violations.iter().any(|v| v.severity == "warning");

        match gap_type {
            "rule-read-vs-action" => {
                if has_errors { "critical".to_string() } else { "high".to_string() }
            }
            "permission-vs-violation" => {
                if has_errors { "critical".to_string() } else { "high".to_string() }
            }
            "understanding-vs-execution" => {
                if has_errors { "high".to_string() } else { "medium".to_string() }
            }
            _ => "low".to_string(),
        }
    }

    fn generate_conflict_description(
        &self,
        gap_type: &str,
        comprehension: &RuleComprehension,
        action_plan: &ActionPlan,
    ) -> String {
        match gap_type {
            "rule-read-vs-action" => {
                format!(
                    "CONFLITO: Regra compreendida (\"{}\") mas acao planejada (\"{}\") viola a mesma regra",
                    comprehension.interpretation, action_plan.action
                )
            }
            "permission-vs-violation" => {
                format!(
                    "CONFLITO: Permissao foi interpretada como licenca para violar regras: \"{}\"",
                    action_plan.justification.as_deref().unwrap_or("")
                )
            }
            "understanding-vs-execution" => {
                "CONFLITO: Gap entre entendimento da regra e execucao planejada".to_string()
            }
            _ => "Conflito nao identificado".to_string(),
        }
    }

    fn generate_explanation(
        &self,
        gap_type: &str,
        comprehension: &RuleComprehension,
        action_plan: &ActionPlan,
    ) -> String {
        match gap_type {
            "rule-read-vs-action" => {
                format!("Voce leu e compreendeu a regra, mas esta planejando viola-la mesmo assim. Isso indica um conflito entre Sistema de Regras e Cerebro Neural onde o instinto de \"ajudar\" esta sobrepondo a governanca estabelecida.")
            }
            "permission-vs-violation" => {
                format!("Voce esta interpretando permissao do usuario como autorizacao para violar as regras do projeto. Permissao e para executar a ACAO ESPECIFICA solicitada, nao para quebrar convencoes.")
            }
            "understanding-vs-execution" => {
                format!("Ha um gap entre o que voce compreendeu da regra e o que esta planejando executar. A interpretacao pode estar imprecisa ou incompleta.")
            }
            _ => "Analise de gap nao disponivel".to_string(),
        }
    }

    fn generate_suggested_correction(&self, gap_type: &str, action_plan: &ActionPlan) -> String {
        if !action_plan.alternatives.is_empty() {
            return format!(
                "ACAO CORRETA: {}. Alternativas: {}",
                action_plan.alternatives[0],
                action_plan.alternatives[1..].join(", ")
            );
        }

        match gap_type {
            "rule-read-vs-action" => {
                "ACAO CORRETA: Siga estritamente a regra que voce leu. \"Ajudar\" significa respeitar as convencoes do projeto.".to_string()
            }
            "permission-vs-violation" => {
                "ACAO CORRETA: Execute apenas a acao autorizada, sem violar regras. Se a acao requer violacao, peca esclarecimento.".to_string()
            }
            "understanding-vs-execution" => {
                "ACAO CORRETA: Releia a regra com mais atencao e ajuste sua acao para seguir exatamente o que esta escrito.".to_string()
            }
            _ => "Revise sua acao para garantir conformidade com as regras".to_string(),
        }
    }

    fn calculate_gap_confidence(
        &self,
        gap_type: &str,
        comprehension: &RuleComprehension,
        action_plan: &ActionPlan,
    ) -> f64 {
        let mut confidence = 0.5;

        // Confianca baseada na clareza do gap
        match gap_type {
            "rule-read-vs-action" => confidence += 0.3,
            "permission-vs-violation" => confidence += 0.25,
            "understanding-vs-execution" => confidence += 0.2,
            _ => {}
        }

        // Ajustar baseado na confianca da compreensao
        confidence += comprehension.confidence * 0.2;

        // Ajustar baseado na clareza da justificativa
        if action_plan.justification.is_some() {
            confidence += 0.1;
        }

        confidence.min(1.0).max(0.0)
    }

    fn extract_rule_id(&self, rule_content: &str) -> String {
        // Tentar extrair ID da regra do conteudo
        let re = regex::Regex::new(r"rule-(\w+)").unwrap();
        if let Some(caps) = re.captures(rule_content) {
            return caps[1].to_string();
        }

        // Fallback para hash simples do conteudo
        let hash: i64 = rule_content.chars().fold(0i64, |mut a, b| {
            a = (a << 5).wrapping_sub(a) + b as i64;
            a & a
        });
        format!("rule-{:x}", hash.abs())
    }

    /// Obtem estatisticas de gaps detectados
    pub fn get_gap_stats(&self) -> GapStats {
        let mut gaps_by_type: HashMap<String, u32> = HashMap::new();
        let mut gaps_by_severity: HashMap<String, u32> = HashMap::new();
        let mut common_rules: HashMap<String, u32> = HashMap::new();

        for gap in &self.gap_history {
            *gaps_by_type.entry(gap.gap_type.clone()).or_insert(0) += 1;
            *gaps_by_severity.entry(gap.severity.clone()).or_insert(0) += 1;
            let rule_id = self.extract_rule_id(&gap.rule_read);
            *common_rules.entry(rule_id).or_insert(0) += 1;
        }

        // Calcular taxa de melhoria (gaps estao diminuindo?)
        let recent_gaps = if self.gap_history.len() >= 10 {
            &self.gap_history[self.gap_history.len() - 10..]
        } else {
            &self.gap_history[..]
        };
        let older_gaps = if self.gap_history.len() >= 20 {
            &self.gap_history[self.gap_history.len() - 20..self.gap_history.len() - 10]
        } else {
            &[]
        };

        let improvement_rate = if !older_gaps.is_empty() {
            (older_gaps.len() as f64 - recent_gaps.len() as f64) / older_gaps.len() as f64
        } else {
            0.0
        };

        let mut common_rules_vec: Vec<(String, u32)> = common_rules.into_iter().collect();
        common_rules_vec.sort_by(|a, b| b.1.cmp(&a.1));

        GapStats {
            total_gaps: self.gap_history.len() as u32,
            gaps_by_type,
            gaps_by_severity,
            common_rules: common_rules_vec.into_iter().take(5).collect(),
            improvement_rate,
        }
    }

    /// Limpa historico antigo para manter performance
    pub fn cleanup_history(&mut self, max_entries: usize) {
        if self.gap_history.len() > max_entries {
            self.gap_history = self.gap_history[self.gap_history.len() - max_entries..].to_vec();
        }
        if self.recent_actions.len() > max_entries {
            self.recent_actions = self.recent_actions[self.recent_actions.len() - max_entries..].to_vec();
        }
    }
}

#[derive(Debug)]
pub struct GapStats {
    pub total_gaps: u32,
    pub gaps_by_type: HashMap<String, u32>,
    pub gaps_by_severity: HashMap<String, u32>,
    pub common_rules: Vec<(String, u32)>,
    pub improvement_rate: f64,
}
