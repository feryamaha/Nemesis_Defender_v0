use crate::ast_types::{ParseError, WorkflowAST, WorkflowPhase};
use std::collections::{HashMap, HashSet};

pub struct ASTValidator;

#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<ParseError>,
    pub warnings: Vec<ParseError>,
}

#[derive(Debug, Clone)]
pub struct ExecutionReadinessResult {
    pub is_ready: bool,
    pub blockers: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct DSLSyntaxResult {
    pub is_valid: bool,
    pub errors: Vec<ParseError>,
    pub warnings: Vec<ParseError>,
}

impl ASTValidator {
    /// Validate complete AST structure
    pub fn validate_ast(ast: &WorkflowAST) -> ValidationResult {
        let mut errors: Vec<ParseError> = Vec::new();
        let mut warnings: Vec<ParseError> = Vec::new();

        // Basic structure validation
        if ast.name.is_empty() {
            errors.push(ParseError {
                line: 1,
                message: "Workflow name is required".to_string(),
                error_type: "structure".to_string(),
            });
        }

        if ast.phases.is_empty() {
            errors.push(ParseError {
                line: 1,
                message: "At least one phase is required".to_string(),
                error_type: "structure".to_string(),
            });
        }

        // Phase validation
        for (index, phase) in ast.phases.iter().enumerate() {
            let phase_errors = Self::validate_phase(phase, index + 1);
            errors.extend(phase_errors.errors);
            warnings.extend(phase_errors.warnings);
        }

        // Global validation
        let global_errors = Self::validate_global_structure(ast);
        errors.extend(global_errors);

        ValidationResult {
            is_valid: errors.is_empty(),
            errors,
            warnings,
        }
    }

    /// Validate individual phase
    pub fn validate_phase(phase: &WorkflowPhase, phase_number: usize) -> ValidationResult {
        let mut errors: Vec<ParseError> = Vec::new();
        let mut warnings: Vec<ParseError> = Vec::new();

        if phase.id.is_empty() {
            errors.push(ParseError {
                line: phase.line_number,
                message: format!("Phase {} missing ID", phase_number),
                error_type: "structure".to_string(),
            });
        }

        // Validate critical phases have actions
        if phase.id.contains("MODEL") && phase.actions.is_empty() {
            warnings.push(ParseError {
                line: phase.line_number,
                message: format!("Phase {} has no actions - may be incomplete", phase.id),
                error_type: "style".to_string(),
            });
        }

        // Validate gate dependencies
        for (index, gate) in phase.gates.iter().enumerate() {
            if gate.name.is_empty() {
                errors.push(ParseError {
                    line: gate.line_number,
                    message: format!("Gate {} in phase {} missing name", index + 1, phase.id),
                    error_type: "structure".to_string(),
                });
            }
        }

        // Validate restriction syntax
        for (index, restriction) in phase.restrictions.iter().enumerate() {
            if restriction.rule.is_empty() {
                errors.push(ParseError {
                    line: restriction.line_number,
                    message: format!("Restriction {} in phase {} missing rule", index + 1, phase.id),
                    error_type: "structure".to_string(),
                });
            }
        }

        // Validate verify targets
        for (index, verify) in phase.verifies.iter().enumerate() {
            if verify.target.is_empty() {
                errors.push(ParseError {
                    line: verify.line_number,
                    message: format!("Verify {} in phase {} missing target", index + 1, phase.id),
                    error_type: "structure".to_string(),
                });
            }
        }

        // Validate action commands
        for (index, action) in phase.actions.iter().enumerate() {
            if action.command.is_empty() {
                errors.push(ParseError {
                    line: action.line_number,
                    message: format!("Action {} in phase {} missing command", index + 1, phase.id),
                    error_type: "structure".to_string(),
                });
            }
        }

        ValidationResult {
            is_valid: errors.is_empty(),
            errors,
            warnings,
        }
    }

    /// Validate global AST structure
    pub fn validate_global_structure(ast: &WorkflowAST) -> Vec<ParseError> {
        let mut errors: Vec<ParseError> = Vec::new();

        // Check for duplicate phase IDs
        let mut seen_ids: HashSet<String> = HashSet::new();
        let mut duplicates: HashSet<String> = HashSet::new();
        
        for phase in &ast.phases {
            if !seen_ids.insert(phase.id.clone()) {
                duplicates.insert(phase.id.clone());
            }
        }

        for duplicate_id in duplicates {
            errors.push(ParseError {
                line: 1,
                message: format!("Duplicate phase ID: {}", duplicate_id),
                error_type: "structure".to_string(),
            });
        }

        // Check for required phases in specific workflows
        if ast.name.contains("work-01-rag") || ast.name.contains("work-02-main") {
            let has_model_phase = ast.phases.iter().any(|p| p.id.contains("MODEL"));
            if !has_model_phase {
                errors.push(ParseError {
                    line: 1,
                    message: "Workflow requires a MODEL declaration phase".to_string(),
                    error_type: "structure".to_string(),
                });
            }
        }

        // Validate gate sequence
        let mut gate_names_seen: HashSet<String> = HashSet::new();
        let mut gate_duplicates: HashSet<String> = HashSet::new();
        
        for phase in &ast.phases {
            for gate in &phase.gates {
                if !gate_names_seen.insert(gate.name.clone()) {
                    gate_duplicates.insert(gate.name.clone());
                }
            }
        }

        for duplicate_gate in gate_duplicates {
            errors.push(ParseError {
                line: 1,
                message: format!("Duplicate gate name: {}", duplicate_gate),
                error_type: "structure".to_string(),
            });
        }

        errors
    }

    /// Validate execution readiness
    pub fn validate_execution_readiness(ast: &WorkflowAST) -> ExecutionReadinessResult {
        let mut blockers: Vec<String> = Vec::new();
        let mut warnings: Vec<String> = Vec::new();

        // Check for critical missing elements
        if ast.phases.is_empty() {
            blockers.push("No phases to execute".to_string());
        }

        // Check for phases with no executable content
        let empty_phases: Vec<String> = ast.phases.iter()
            .filter(|p| p.actions.is_empty() && p.gates.is_empty() && p.verifies.is_empty())
            .map(|p| p.id.clone())
            .collect();

        if !empty_phases.is_empty() {
            warnings.push(format!(
                "Phases with no executable content: {}",
                empty_phases.join(", ")
            ));
        }

        // Check for unsupported action commands
        let supported_commands = [
            "WRITE_ARTEFATO",
            "STEP_DECLARED",
            "MAP",
            "VERIFY_CWD",
            "ENSURE_DIRS",
            "FETCH_NATIVE",
            "ANALYZE_REQUEST",
            "GENERATE_RAG",
            "COMMIT_FILENAME_DECLARATION",
            "PRESENT_RAG_RESULT",
            "WRITE_NATIVE",
            "DECLARE_COMPLETE",
            "CLEANUP_ARTEFATO",
        ];

        let all_actions: Vec<_> = ast.phases.iter().flat_map(|p| &p.actions).collect();
        let unsupported_actions: Vec<String> = all_actions.iter()
            .filter(|a| !supported_commands.contains(&a.command.as_str()))
            .map(|a| a.command.clone())
            .collect();

        if !unsupported_actions.is_empty() {
            warnings.push(format!(
                "Unsupported action commands: {}",
                unsupported_actions.join(", ")
            ));
        }

        // Check for circular dependencies in gates
        let dependencies = Self::analyze_gate_dependencies(ast);
        let circular_deps = Self::detect_circular_dependencies(&dependencies);

        if !circular_deps.is_empty() {
            blockers.push(format!("Circular gate dependencies: {}", circular_deps.join(" -> ")));
        }

        ExecutionReadinessResult {
            is_ready: blockers.is_empty(),
            blockers,
            warnings,
        }
    }

    /// Analyze gate dependencies
    fn analyze_gate_dependencies(ast: &WorkflowAST) -> HashMap<String, Vec<String>> {
        let mut dependencies: HashMap<String, Vec<String>> = HashMap::new();

        for phase in &ast.phases {
            for gate in &phase.gates {
                let gate_deps = dependencies.entry(gate.name.clone()).or_insert_with(Vec::new);

                // Look for dependencies in restrictions and actions
                for restriction in &phase.restrictions {
                    if restriction.rule.contains(&gate.name) {
                        gate_deps.push(format!("RESTRICTION:{}", restriction.rule));
                    }
                }

                for action in &phase.actions {
                    if action.command.contains(&gate.name) {
                        gate_deps.push(format!("ACTION:{}", action.command));
                    }
                }
            }
        }

        dependencies
    }

    /// Detect circular dependencies
    fn detect_circular_dependencies(dependencies: &HashMap<String, Vec<String>>) -> Vec<String> {
        let mut visited: HashSet<String> = HashSet::new();
        let mut recursion_stack: HashSet<String> = HashSet::new();
        let mut cycles: Vec<String> = Vec::new();

        fn dfs(
            node: &str,
            path: &mut Vec<String>,
            visited: &mut HashSet<String>,
            recursion_stack: &mut HashSet<String>,
            dependencies: &HashMap<String, Vec<String>>,
            cycles: &mut Vec<String>,
        ) -> bool {
            if recursion_stack.contains(node) {
                let cycle_start = path.iter().position(|p| p == node).unwrap_or(0);
                let cycle_path: Vec<_> = path[cycle_start..].iter().cloned().collect();
                cycles.push(format!("{} -> {}", cycle_path.join(" -> "), node));
                return true;
            }

            if visited.contains(node) {
                return false;
            }

            visited.insert(node.to_string());
            recursion_stack.insert(node.to_string());
            path.push(node.to_string());

            if let Some(deps) = dependencies.get(node) {
                for dep in deps {
                    let dep_node = dep.split(':').nth(1).unwrap_or(dep);
                    if dfs(dep_node, path, visited, recursion_stack, dependencies, cycles) {
                        return true;
                    }
                }
            }

            path.pop();
            recursion_stack.remove(node);
            false
        }

        for node in dependencies.keys() {
            if !visited.contains(node) {
                let mut path = Vec::new();
                dfs(node, &mut path, &mut visited, &mut recursion_stack, dependencies, &mut cycles);
            }
        }

        cycles
    }

    /// Validate DSL syntax
    pub fn validate_dsl_syntax(content: &str) -> DSLSyntaxResult {
        let mut errors: Vec<ParseError> = Vec::new();
        let mut warnings: Vec<ParseError> = Vec::new();
        let lines: Vec<&str> = content.lines().collect();

        for (index, line) in lines.iter().enumerate() {
            let line_number = (index + 1) as u32;
            let trimmed = line.trim();

            // Check for malformed DSL blocks
            if trimmed.starts_with('[') && !trimmed.ends_with(']') {
                errors.push(ParseError {
                    line: line_number,
                    message: "Malformed DSL block - missing closing bracket".to_string(),
                    error_type: "syntax".to_string(),
                });
            }

            // Check for empty DSL blocks
            let empty_dsl_pattern = regex::Regex::new(r"^\[\w+:\s*\]$").unwrap();
            if empty_dsl_pattern.is_match(trimmed) {
                warnings.push(ParseError {
                    line: line_number,
                    message: "Empty DSL block".to_string(),
                    error_type: "style".to_string(),
                });
            }

            // Check for invalid DSL types
            let dsl_match_pattern = regex::Regex::new(r"^\[(\w+):").unwrap();
            if let Some(captures) = dsl_match_pattern.captures(trimmed) {
                if let Some(dsl_type) = captures.get(1) {
                    let valid_types = [
                        "ACTION", "GATE", "VERIFY", "RESTRICTION", "CONSTANT",
                        "DICTIONARY", "DICT", "SCHEMA", "MAP",
                    ];

                    if !valid_types.contains(&dsl_type.as_str()) {
                        warnings.push(ParseError {
                            line: line_number,
                            message: format!("Unknown DSL type: {}", dsl_type.as_str()),
                            error_type: "style".to_string(),
                        });
                    }
                }
            }
        }

        DSLSyntaxResult {
            is_valid: errors.is_empty(),
            errors,
            warnings,
        }
    }

    /// Generate validation report
    pub fn generate_validation_report(ast: &WorkflowAST) -> String {
        let structure_validation = Self::validate_ast(ast);
        let execution_validation = Self::validate_execution_readiness(ast);

        let mut report = "# AST Validation Report\n\n".to_string();
        report.push_str(&format!("## Workflow: {}\n\n", ast.name));

        report.push_str("### Structure Validation\n");
        report.push_str(&format!("- Valid: {}\n", if structure_validation.is_valid { "✅" } else { "❌" }));
        report.push_str(&format!("- Errors: {}\n", structure_validation.errors.len()));
        report.push_str(&format!("- Warnings: {}\n\n", structure_validation.warnings.len()));

        if !structure_validation.errors.is_empty() {
            report.push_str("#### Errors\n");
            for error in &structure_validation.errors {
                report.push_str(&format!("- Line {}: {}\n", error.line, error.message));
            }
            report.push('\n');
        }

        if !structure_validation.warnings.is_empty() {
            report.push_str("#### Warnings\n");
            for warning in &structure_validation.warnings {
                report.push_str(&format!("- Line {}: {}\n", warning.line, warning.message));
            }
            report.push('\n');
        }

        report.push_str("### Execution Readiness\n");
        report.push_str(&format!("- Ready: {}\n", if execution_validation.is_ready { "✅" } else { "❌" }));
        report.push_str(&format!("- Blockers: {}\n", execution_validation.blockers.len()));
        report.push_str(&format!("- Warnings: {}\n\n", execution_validation.warnings.len()));

        if !execution_validation.blockers.is_empty() {
            report.push_str("#### Blockers\n");
            for blocker in &execution_validation.blockers {
                report.push_str(&format!("- {}\n", blocker));
            }
            report.push('\n');
        }

        if !execution_validation.warnings.is_empty() {
            report.push_str("#### Execution Warnings\n");
            for warning in &execution_validation.warnings {
                report.push_str(&format!("- {}\n", warning));
            }
            report.push('\n');
        }

        report.push_str("### Summary\n");
        report.push_str(&format!("- Total Phases: {}\n", ast.phases.len()));
        let total_actions: usize = ast.phases.iter().map(|p| p.actions.len()).sum();
        let total_gates: usize = ast.phases.iter().map(|p| p.gates.len()).sum();
        let total_restrictions: usize = ast.phases.iter().map(|p| p.restrictions.len()).sum();
        let total_verifies: usize = ast.phases.iter().map(|p| p.verifies.len()).sum();
        report.push_str(&format!("- Total Actions: {}\n", total_actions));
        report.push_str(&format!("- Total Gates: {}\n", total_gates));
        report.push_str(&format!("- Total Restrictions: {}\n", total_restrictions));
        report.push_str(&format!("- Total Verifies: {}\n", total_verifies));

        report
    }
}
