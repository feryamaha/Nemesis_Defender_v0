//! Nemesis Workflow Enforcement Framework - Library
//! 
//! Biblioteca principal contendo todos os módulos do sistema Nemesis.
//! Esta lib.rs mantém a estrutura de pastas legada conforme aprovado.

// Módulos raiz (sem main()) - usando #[path] para arquivos com hífen
#[path = "types.rs"]
pub mod types;

#[path = "ast-types.rs"]
pub mod ast_types;

#[path = "ast-builder.rs"]
pub mod ast_builder;

#[path = "ast-validator.rs"]
pub mod ast_validator;

#[path = "bash-tool-adapter.rs"]
pub mod bash_tool_adapter;

#[path = "command-extractor.rs"]
pub mod command_extractor;

#[path = "permission-gate.rs"]
pub mod permission_gate;

#[path = "violation-logger.rs"]
pub mod violation_logger;

#[path = "workflow-catalog.rs"]
pub mod workflow_catalog;

#[path = "workflow-enforcer.rs"]
pub mod workflow_enforcer;

#[path = "workflow-parser.rs"]
pub mod workflow_parser;

#[path = "workflow-runner.rs"]
pub mod workflow_runner;

#[path = "workflow-validators.rs"]
pub mod workflow_validators;

#[path = "index.rs"]
pub mod index;

// Submódulos - adapters
pub mod adapters {
    #[path = "package-manager-adapter.rs"]
    pub mod package_manager_adapter;
}

// Submódulos - analysis
pub mod analysis {
    #[path = "gap-detector.rs"]
    pub mod gap_detector;
}

// Submódulos - behavioral
pub mod behavioral {
    #[path = "override-system.rs"]
    pub mod override_system;
}

// Submódulos - cli (bibliotecas compartilhadas, não binários)
pub mod cli {
    // Os binários são declarados no Cargo.toml, não aqui
}

// Submódulos - detectors
pub mod detectors {
    #[path = "environment-detector.rs"]
    pub mod environment_detector;
}

// Submódulos - engine
pub mod engine {
    #[path = "rule-engine.rs"]
    pub mod rule_engine;
}

// Submódulos - harvest
pub mod harvest {
    #[path = "nemesis-harvest.rs"]
    pub mod nemesis_harvest;
    // nemesis-install.rs é binário, declarado no Cargo.toml
}

// Re-export para acesso direto à run_harvest()
pub use harvest::nemesis_harvest::{run_harvest, HarvestResult, HarvestOutput};

/// Configuração para modo CLI de harvest
#[derive(Debug, Clone)]
pub struct HarvestCliMode {
    pub json_output: bool,
    pub pretty: bool,
}

/// Executa harvest e retorna resultado em JSON (strings formatadas)
pub async fn harvest_as_json() -> Result<String, Box<dyn std::error::Error>> {
    let result = run_harvest().await;
    let json = serde_json::to_string_pretty(&result.output)?;
    Ok(json)
}

/// Executa harvest com modo CLI (JSON ou debug output)
pub async fn run_harvest_cli(mode: HarvestCliMode) -> Result<String, Box<dyn std::error::Error>> {
    let result = run_harvest().await;

    if mode.json_output {
        if mode.pretty {
            serde_json::to_string_pretty(&result.output).map_err(|e| e.into())
        } else {
            serde_json::to_string(&result.output).map_err(|e| e.into())
        }
    } else {
        Ok(format!("{:#?}", result.output))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_harvest_cli_mode_creation() {
        let mode_pretty = HarvestCliMode {
            json_output: true,
            pretty: true,
        };
        assert!(mode_pretty.json_output);
        assert!(mode_pretty.pretty);

        let mode_compact = HarvestCliMode {
            json_output: true,
            pretty: false,
        };
        assert!(mode_compact.json_output);
        assert!(!mode_compact.pretty);

        let mode_debug = HarvestCliMode {
            json_output: false,
            pretty: false,
        };
        assert!(!mode_debug.json_output);
    }

    #[tokio::test]
    async fn test_harvest_as_json_returns_valid_json() {
        // RED: This test verifies that harvest_as_json() returns a valid JSON string
        let result = harvest_as_json().await;

        // Should be Ok (no error)
        assert!(result.is_ok(), "harvest_as_json should succeed");

        let json_str = result.unwrap();

        // Verify it's valid JSON by parsing it back
        let parsed: Result<serde_json::Value, _> = serde_json::from_str(&json_str);
        assert!(parsed.is_ok(), "harvest_as_json should return valid JSON");

        // Verify it's formatted (contains newlines for pretty-print)
        assert!(json_str.contains('\n'), "harvest_as_json should return pretty-printed JSON");
    }

    #[tokio::test]
    async fn test_run_harvest_cli_json_pretty() {
        // Test that run_harvest_cli with json_output=true and pretty=true works
        let mode = HarvestCliMode {
            json_output: true,
            pretty: true,
        };

        let result = run_harvest_cli(mode).await;
        assert!(result.is_ok(), "run_harvest_cli should succeed with pretty JSON mode");

        let output = result.unwrap();

        // Verify it's valid JSON
        let parsed: Result<serde_json::Value, _> = serde_json::from_str(&output);
        assert!(parsed.is_ok(), "output should be valid JSON");

        // Verify it's pretty-printed
        assert!(output.contains('\n'), "pretty JSON should contain newlines");
    }

    #[tokio::test]
    async fn test_run_harvest_cli_json_compact() {
        // Test that run_harvest_cli with json_output=true and pretty=false works
        let mode = HarvestCliMode {
            json_output: true,
            pretty: false,
        };

        let result = run_harvest_cli(mode).await;
        assert!(result.is_ok(), "run_harvest_cli should succeed with compact JSON mode");

        let output = result.unwrap();

        // Verify it's valid JSON
        let parsed: Result<serde_json::Value, _> = serde_json::from_str(&output);
        assert!(parsed.is_ok(), "output should be valid JSON");
    }

    #[tokio::test]
    async fn test_run_harvest_cli_debug_output() {
        // Test that run_harvest_cli with json_output=false returns debug format
        let mode = HarvestCliMode {
            json_output: false,
            pretty: false,
        };

        let result = run_harvest_cli(mode).await;
        assert!(result.is_ok(), "run_harvest_cli should succeed with debug mode");

        let output = result.unwrap();

        // Debug output should contain debug formatting (should have {, :, etc.)
        assert!(!output.is_empty(), "debug output should not be empty");
    }

    #[tokio::test]
    async fn test_harvest_result_has_output_field() {
        // Verify that HarvestResult has the expected structure
        let result = run_harvest().await;

        // Verify it has stack_detected
        assert!(!result.stack_detected.is_empty(), "stack_detected should not be empty");

        // Verify it has patterns_generated (field is present)
        let _ = result.patterns_generated;

        // Verify it has output (HarvestOutput)
        assert!(!result.output.version.is_empty(), "HarvestOutput should have version");
        assert!(!result.output.project_stack.is_empty(), "HarvestOutput should have project_stack");
    }
}

// Submódulos - hook
pub mod hook {
    #[path = "code-validator.rs"]
    pub mod code_validator;
    
    #[path = "deny-list-loader.rs"]
    pub mod deny_list_loader;
    
    #[path = "scope-validator.rs"]
    pub mod scope_validator;
}

// Submódulos - services
pub mod services {
    #[path = "terminal-reader-logger.rs"]
    pub mod terminal_reader_logger;
    
    #[path = "terminal-reader-service.rs"]
    pub mod terminal_reader_service;
    
    #[path = "terminal-reader-types.rs"]
    pub mod terminal_reader_types;
}

// Módulo node (declarações de tipos Node.js para compatibilidade)
#[path = "types/node.rs"]
pub mod node_types;

// Submódulos - validators
pub mod validators {
    #[path = "ia-action-validator.rs"]
    pub mod ia_action_validator;
}

// Re-exports principais para conveniência
// pub use types::*;
// pub use violation_logger::ViolationLogger;
// pub use permission_gate::PermissionGate;
// pub use workflow_runner::WorkflowRunner;
// pub use workflow_catalog::WorkflowCatalog;
// pub use workflow_enforcer::WorkflowEnforcer;
// pub use workflow_parser::WorkflowParser;
