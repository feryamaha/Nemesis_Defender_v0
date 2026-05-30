use crate::types::{CommandResult, ExecutionOptions};
use std::path::Path;
use std::process::Stdio;
use std::time::Instant;
use tokio::process::Command;
use tokio::time::timeout;

pub struct BashToolAdapter {
    execution_options: ExecutionOptions,
}

impl BashToolAdapter {
    pub fn new(options: ExecutionOptions) -> Self {
        Self {
            execution_options: options,
        }
    }

    pub async fn execute_command(&self, command: &str) -> CommandResult {
        let start_time = Instant::now();
        let cwd = self.execution_options.cwd.clone().unwrap_or_else(|| ".".to_string());

        // Build command with shell
        let shell_command = format!("{} 2>&1", command);
        
        let output_result = timeout(
            std::time::Duration::from_secs(60),
            Command::new("sh")
                .arg("-c")
                .arg(&shell_command)
                .current_dir(&cwd)
                .envs(self.execution_options.env.iter().flatten().map(|(k, v)| (k.clone(), v.clone())))
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output()
        ).await;

        match output_result {
            Ok(Ok(output)) => {
                let execution_time = start_time.elapsed().as_millis() as f64;
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                let exit_code = output.status.code().unwrap_or(1);

                CommandResult {
                    stdout,
                    stderr,
                    exit_code,
                    execution_time,
                    command: command.to_string(),
                }
            }
            Ok(Err(e)) => {
                let execution_time = start_time.elapsed().as_millis() as f64;
                CommandResult {
                    stdout: String::new(),
                    stderr: format!("Failed to execute command: {}", e),
                    exit_code: 1,
                    execution_time,
                    command: command.to_string(),
                }
            }
            Err(_) => {
                let execution_time = start_time.elapsed().as_millis() as f64;
                CommandResult {
                    stdout: String::new(),
                    stderr: "Command timed out after 60 seconds".to_string(),
                    exit_code: 124,
                    execution_time,
                    command: command.to_string(),
                }
            }
        }
    }

    pub async fn execute_commands(&self, commands: &[String]) -> Vec<CommandResult> {
        let mut results = Vec::new();

        for command in commands {
            let result = self.execute_command(command).await;
            let should_stop = result.exit_code != 0;
            results.push(result);

            // Stop execution if a command fails
            if should_stop {
                eprintln!("Command failed with exit code {}: {}", results.last().unwrap().exit_code, command);
                break;
            }
        }

        results
    }

    pub async fn read_file(&self, file_path: &str) -> anyhow::Result<String> {
        let cwd = self.execution_options.cwd.clone().unwrap_or_else(|| ".".to_string());
        let full_path = Path::new(&cwd).join(file_path);
        
        tokio::fs::read_to_string(&full_path).await.map_err(|e| {
            anyhow::anyhow!("Failed to read file {}: {}", file_path, e)
        })
    }

    pub async fn write_file(&self, file_path: &str, content: &str) -> bool {
        let cwd = self.execution_options.cwd.clone().unwrap_or_else(|| ".".to_string());
        let full_path = Path::new(&cwd).join(file_path);
        
        // Create directory if it doesn't exist
        if let Some(parent) = full_path.parent() {
            if let Err(e) = tokio::fs::create_dir_all(parent).await {
                eprintln!("Failed to create directory for {}: {}", file_path, e);
                return false;
            }
        }

        match tokio::fs::write(&full_path, content).await {
            Ok(_) => true,
            Err(e) => {
                eprintln!("Failed to write file {}: {}", file_path, e);
                false
            }
        }
    }

    pub fn get_execution_statistics(&self) -> std::collections::HashMap<String, u64> {
        let mut stats = std::collections::HashMap::new();
        // Simplified statistics - in a full implementation, track history
        stats.insert("total_commands".to_string(), 0);
        stats.insert("average_execution_time".to_string(), 0);
        stats.insert("success_rate".to_string(), 0);
        stats.insert("total_execution_time".to_string(), 0);
        stats
    }

    pub fn clear_execution_history(&self) {
        // No-op for native implementation
    }
}
