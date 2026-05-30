use crate::ast_types::{
    ActionNode, ConstantNode, DictionaryNode, DSLParseResult, DSLNode, GateNode, MapNode,
    ParseError, ParseWarning, RestrictionNode, SchemaNode, VerifyNode, WorkflowAST,
    WorkflowPhase, DSL_PATTERNS,
};
use crate::types::{CodeBlock, WorkflowDefinition};
use regex::Regex;
use std::collections::HashMap;
use std::path::Path;

pub struct WorkflowParser;

lazy_static::lazy_static! {
    static ref CODE_BLOCK_REGEX: Regex = Regex::new(r"```(\w+)?\n([\s\S]*?)```").unwrap();
    static ref METADATA_REGEX: Regex = Regex::new(r"^---\n([\s\S]*?)\n---").unwrap();
}

impl WorkflowParser {
    pub async fn parse_workflow(file_path: &str) -> anyhow::Result<WorkflowDefinition> {
        let content = tokio::fs::read_to_string(file_path).await?;
        let name = Self::extract_workflow_name(file_path);
        let code_blocks = Self::extract_code_blocks(&content);
        let metadata = Self::extract_metadata(&content);

        Ok(WorkflowDefinition {
            name,
            path: file_path.to_string(),
            content,
            code_blocks,
            metadata,
        })
    }

    /// Parse DSL workflow file into executable AST
    pub async fn parse_workflow_to_ast(file_path: &str) -> anyhow::Result<DSLParseResult> {
        match tokio::fs::read_to_string(file_path).await {
            Ok(content) => {
                let name = Self::extract_workflow_name(file_path);
                let metadata = Self::extract_metadata(&content);

                let phases = Self::extract_phases(&content);
                let mut errors: Vec<ParseError> = Vec::new();
                let warnings: Vec<ParseWarning> = Vec::new();

                // Validate structure
                if phases.is_empty() {
                    errors.push(ParseError {
                        line: 1,
                        message: "No phases found in workflow".to_string(),
                        error_type: "structure".to_string(),
                    });
                }

                let ast = WorkflowAST {
                    name,
                    path: file_path.to_string(),
                    phases,
                    metadata: metadata.iter().map(|(k, v)| (k.clone(), v.clone())).collect(),
                    raw_content: content,
                };

                Ok(DSLParseResult {
                    success: errors.is_empty(),
                    ast: Some(ast),
                    errors,
                    warnings,
                })
            }
            Err(e) => Ok(DSLParseResult {
                success: false,
                ast: None,
                errors: vec![ParseError {
                    line: 1,
                    message: e.to_string(),
                    error_type: "syntax".to_string(),
                }],
                warnings: vec![],
            }),
        }
    }

    fn extract_workflow_name(file_path: &str) -> String {
        Path::new(file_path)
            .file_name()
            .and_then(|n| n.to_str())
            .map(|n| n.trim_end_matches(".md").trim_end_matches(".markdown").to_string())
            .unwrap_or_else(|| file_path.to_string())
    }

    fn extract_code_blocks(content: &str) -> Vec<CodeBlock> {
        let mut blocks: Vec<CodeBlock> = Vec::new();
        let mut line_number = 1;

        for cap in CODE_BLOCK_REGEX.captures_iter(content) {
            let language = cap.get(1).map(|m| m.as_str()).unwrap_or("text").to_string();
            let block_content = cap.get(2).map(|m| m.as_str()).unwrap_or("").to_string();
            let start_index = cap.get(0).map(|m| m.start()).unwrap_or(0);

            // Calculate line number
            let before_block = &content[..start_index];
            line_number = before_block.lines().count() + 1;

            blocks.push(CodeBlock {
                language: language.clone(),
                content: block_content,
                line_number: line_number as u32,
                is_executable: Self::is_executable_language(&language),
            });
        }

        blocks
    }

    fn extract_metadata(content: &str) -> HashMap<String, String> {
        let mut metadata: HashMap<String, String> = HashMap::new();

        if let Some(captures) = METADATA_REGEX.captures(content) {
            if let Some(metadata_section) = captures.get(1) {
                for line in metadata_section.as_str().lines() {
                    if let Some(colon_index) = line.find(':') {
                        let key = line[..colon_index].trim().to_string();
                        let value = line[colon_index + 1..].trim().to_string();
                        metadata.insert(key, value);
                    }
                }
            }
        }

        metadata
    }

    fn is_executable_language(language: &str) -> bool {
        let executable_languages = [
            "bash", "sh", "shell", "powershell", "ps1", "javascript", "js",
            "typescript", "ts", "python", "py", "node", "npm", "yarn",
        ];

        executable_languages.contains(&language.to_lowercase().as_str())
    }

    pub async fn parse_multiple_workflows(file_paths: &[String]) -> Vec<WorkflowDefinition> {
        let mut workflows: Vec<WorkflowDefinition> = Vec::new();

        for file_path in file_paths {
            match Self::parse_workflow(file_path).await {
                Ok(workflow) => workflows.push(workflow),
                Err(e) => {
                    eprintln!("Failed to parse workflow {}: {}", file_path, e);
                }
            }
        }

        workflows
    }

    /// Parse multiple workflow files into ASTs
    pub async fn parse_multiple_workflows_to_ast(file_paths: &[String]) -> Vec<DSLParseResult> {
        let mut results: Vec<DSLParseResult> = Vec::new();

        for file_path in file_paths {
            let result = Self::parse_workflow_to_ast(file_path).await;
            match result {
                Ok(r) => results.push(r),
                Err(e) => results.push(DSLParseResult {
                    success: false,
                    ast: None,
                    errors: vec![ParseError {
                        line: 1,
                        message: e.to_string(),
                        error_type: "syntax".to_string(),
                    }],
                    warnings: vec![],
                }),
            }
        }

        results
    }

    /// Extract phases from workflow content
    fn extract_phases(content: &str) -> Vec<WorkflowPhase> {
        let lines: Vec<&str> = content.lines().collect();
        let mut phases: Vec<WorkflowPhase> = Vec::new();
        let mut current_phase: Option<WorkflowPhase> = None;
        let mut code_block_content = String::new();
        let mut in_code_block = false;

        for (i, line) in lines.iter().enumerate() {
            let line_number = i + 1;

            // Handle code blocks
            if line.starts_with("```") {
                if in_code_block {
                    // End of code block
                    in_code_block = false;
                    if let Some(ref mut phase) = current_phase {
                        Self::process_code_block(&code_block_content, phase);
                        code_block_content.clear();
                    }
                } else {
                    // Start of code block
                    in_code_block = true;
                }
                continue;
            }

            if in_code_block {
                code_block_content.push_str(line);
                code_block_content.push('\n');
                continue;
            }

            // Check for phase header
            if let Some(phase_match) = DSL_PATTERNS.phase_header.find(line) {
                // Save previous phase
                if let Some(phase) = current_phase.take() {
                    phases.push(phase);
                }

                // Start new phase
                let phase_id = phase_match.as_str()
                    .trim_start_matches("## **")
                    .trim_start_matches("## ")
                    .trim_end_matches("**")
                    .trim()
                    .to_string();

                current_phase = Some(WorkflowPhase {
                    id: phase_id.clone(),
                    title: phase_id,
                    actions: Vec::new(),
                    gates: Vec::new(),
                    verifies: Vec::new(),
                    restrictions: Vec::new(),
                    constants: Vec::new(),
                    dictionaries: Vec::new(),
                    schemas: Vec::new(),
                    maps: Vec::new(),
                    line_number: line_number as u32,
                });
                continue;
            }

            // Parse DSL nodes if we're in a phase
            if let Some(ref mut phase) = current_phase {
                Self::parse_dsl_node(line, line_number, phase);
            }
        }

        // Don't forget the last phase
        if let Some(phase) = current_phase {
            phases.push(phase);
        }

        phases
    }

    /// Parse individual DSL nodes
    fn parse_dsl_node(line: &str, line_number: usize, phase: &mut WorkflowPhase) {
        let trimmed_line = line.trim();

        // Action nodes
        if let Some(action_match) = DSL_PATTERNS.action.find(trimmed_line) {
            let command = action_match.as_str()
                .trim_start_matches("[ACTION:")
                .trim_end_matches(']')
                .trim()
                .to_string();
            
            let node = ActionNode {
                node_type: "ACTION".to_string(),
                command: command.clone(),
                args: Self::extract_args(&command),
                target: None,
                content: None,
                line_number: line_number as u32,
                raw_text: trimmed_line.to_string(),
            };
            phase.actions.push(node);
            return;
        }

        // Gate nodes
        if let Some(gate_match) = DSL_PATTERNS.gate.find(trimmed_line) {
            let name = gate_match.as_str()
                .trim_start_matches("[GATE:")
                .trim_end_matches(']')
                .trim()
                .to_string();
            
            let node = GateNode {
                node_type: "GATE".to_string(),
                name,
                condition: None,
                line_number: line_number as u32,
                raw_text: trimmed_line.to_string(),
            };
            phase.gates.push(node);
            return;
        }

        // Verify nodes
        if let Some(verify_match) = DSL_PATTERNS.verify.find(trimmed_line) {
            let target = verify_match.as_str()
                .trim_start_matches("[VERIFY:")
                .trim_end_matches(']')
                .trim()
                .to_string();
            
            let node = VerifyNode {
                node_type: "VERIFY".to_string(),
                target,
                checks: None,
                condition: None,
                line_number: line_number as u32,
                raw_text: trimmed_line.to_string(),
            };
            phase.verifies.push(node);
            return;
        }

        // Restriction nodes
        if let Some(restriction_match) = DSL_PATTERNS.restriction.find(trimmed_line) {
            let rule = restriction_match.as_str()
                .trim_start_matches("[RESTRICTION:")
                .trim_end_matches(']')
                .trim()
                .to_string();
            
            let node = RestrictionNode {
                node_type: "RESTRICTION".to_string(),
                rule,
                condition: None,
                line_number: line_number as u32,
                raw_text: trimmed_line.to_string(),
            };
            phase.restrictions.push(node);
            return;
        }

        // Constant nodes
        if let Some(constant_match) = DSL_PATTERNS.constant.find(trimmed_line) {
            let inner = constant_match.as_str()
                .trim_start_matches("[CONSTANT:")
                .trim_end_matches(']')
                .trim()
                .to_string();
            
            let parts: Vec<&str> = inner.splitn(2, '=').collect();
            let name = parts.get(0).map(|s| s.trim()).unwrap_or("").to_string();
            let value = parts.get(1).map(|s| s.trim()).unwrap_or("").to_string();
            
            let node = ConstantNode {
                node_type: "CONSTANT".to_string(),
                name,
                value,
                line_number: line_number as u32,
                raw_text: trimmed_line.to_string(),
            };
            phase.constants.push(node);
            return;
        }

        // Dictionary nodes
        if let Some(dict_match) = DSL_PATTERNS.dictionary.find(trimmed_line) {
            let inner = dict_match.as_str()
                .trim_start_matches("[DICTIONARY:")
                .trim_start_matches("[DICT:")
                .trim_end_matches(']')
                .trim()
                .to_string();
            
            let node = DictionaryNode {
                node_type: "DICTIONARY".to_string(),
                name: inner,
                entries: HashMap::new(),
                line_number: line_number as u32,
                raw_text: trimmed_line.to_string(),
            };
            phase.dictionaries.push(node);
            return;
        }

        // Schema nodes
        if let Some(schema_match) = DSL_PATTERNS.schema.find(trimmed_line) {
            let name = schema_match.as_str()
                .trim_start_matches("[SCHEMA:")
                .trim_end_matches(']')
                .trim()
                .to_string();
            
            let node = SchemaNode {
                node_type: "SCHEMA".to_string(),
                name,
                structure: String::new(),
                line_number: line_number as u32,
                raw_text: trimmed_line.to_string(),
            };
            phase.schemas.push(node);
            return;
        }

        // Map nodes
        if let Some(map_match) = DSL_PATTERNS.map.find(trimmed_line) {
            let inner = map_match.as_str()
                .trim_start_matches("[MAP:")
                .trim_end_matches(']')
                .trim()
                .to_string();
            
            let parts: Vec<&str> = inner.split("->").collect();
            let source = parts.get(0).map(|s| s.trim()).unwrap_or("").to_string();
            let target = parts.get(1).map(|s| s.trim()).unwrap_or("").to_string();
            
            let node = MapNode {
                node_type: "MAP".to_string(),
                source,
                target,
                transformation: None,
                line_number: line_number as u32,
                raw_text: trimmed_line.to_string(),
            };
            phase.maps.push(node);
        }
    }

    /// Extract arguments from action command
    fn extract_args(command: &str) -> Vec<String> {
        let args_pattern = Regex::new(r"^([^:]+):\s*(.+)$").unwrap();
        if let Some(captures) = args_pattern.captures(command) {
            vec![
                captures.get(1).map(|m| m.as_str().trim().to_string()).unwrap_or_default(),
                captures.get(2).map(|m| m.as_str().trim().to_string()).unwrap_or_default(),
            ]
        } else {
            vec![command.trim().to_string()]
        }
    }

    /// Process code block content for dictionaries and schemas
    fn process_code_block(content: &str, phase: &mut WorkflowPhase) {
        let trimmed = content.trim();

        // Try to parse as JSON for dictionaries
        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(trimmed) {
            if let Some(last_dict) = phase.dictionaries.last_mut() {
                if let serde_json::Value::Object(map) = parsed {
                    for (k, v) in map {
                        last_dict.entries.insert(k, v.to_string());
                    }
                }
            }
        } else {
            // Not JSON, treat as schema structure
            if let Some(last_schema) = phase.schemas.last_mut() {
                last_schema.structure = trimmed.to_string();
            }
        }
    }
}
