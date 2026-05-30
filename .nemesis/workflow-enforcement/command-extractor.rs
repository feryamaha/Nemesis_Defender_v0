use crate::types::CodeBlock;
use regex::Regex;
use std::collections::HashMap;

pub struct CommandExtractor;

impl CommandExtractor {
    fn comment_patterns() -> Vec<Regex> {
        vec![
            Regex::new(r"^\s*#").unwrap(),           // Bash/Shell comments
            Regex::new(r"^\s*//").unwrap(),          // JavaScript/TypeScript comments
            Regex::new(r"^\s*/\*").unwrap(),         // Multi-line comment start
            Regex::new(r"^\s*\*").unwrap(),          // Multi-line comment middle/end
            Regex::new(r"^\s*<!--").unwrap(),        // HTML/XML comments
            Regex::new(r"^\s*;").unwrap(),           // Some config files
            Regex::new(r"^\s*--").unwrap(),          // Command line comments
        ]
    }

    pub fn extract_commands(code_blocks: &[CodeBlock]) -> Vec<String> {
        let mut commands = Vec::new();

        for block in code_blocks {
            if !block.is_executable {
                continue;
            }

            let block_commands = Self::extract_from_block(block);
            commands.extend(block_commands);
        }

        commands
    }

    fn extract_from_block(block: &CodeBlock) -> Vec<String> {
        let lines: Vec<&str> = block.content.lines().collect();
        let mut commands = Vec::new();

        for line in lines {
            let trimmed_line = line.trim();

            // Skip empty lines and comments
            if trimmed_line.is_empty() || Self::is_comment(trimmed_line) {
                continue;
            }

            // Skip variable assignments and exports
            if Self::is_variable_assignment(trimmed_line) {
                continue;
            }

            // Skip function definitions
            if Self::is_function_definition(trimmed_line) {
                continue;
            }

            commands.push(trimmed_line.to_string());
        }

        commands
    }

    fn is_comment(line: &str) -> bool {
        Self::comment_patterns().iter().any(|pattern| pattern.is_match(line))
    }

    fn is_variable_assignment(line: &str) -> bool {
        let assignment_pattern = Regex::new(r"^(export\s+)?[a-zA-Z_][a-zA-Z0-9_]*\s*=").unwrap();
        assignment_pattern.is_match(line)
    }

    fn is_function_definition(line: &str) -> bool {
        let function_patterns = vec![
            Regex::new(r"^(export\s+)?(async\s+)?function\s+\w+\s*\(").unwrap(),
            Regex::new(r"^(export\s+)?(const|let|var)\s+\w+\s*=\s*(async\s+)?\(").unwrap(),
            Regex::new(r"^\w+\s*\(\s*\)\s*\{").unwrap(),
            Regex::new(r"^class\s+\w+").unwrap(),
        ];
        function_patterns.iter().any(|pattern| pattern.is_match(line))
    }

    pub fn extract_executable_commands(code_blocks: &[CodeBlock]) -> Vec<String> {
        let all_commands = Self::extract_commands(code_blocks);

        // Filter for commands that actually execute something
        all_commands.into_iter().filter(|command| {
            // Skip echo/print statements that don't modify state
            let skip_pattern = Regex::new(r"^(echo|print|console\.log|console\.info|console\.warn|console\.error)\s").unwrap();
            if skip_pattern.is_match(command) {
                return false;
            }

            // Skip read-only commands
            let read_only_pattern = Regex::new(r"^(ls|cat|grep|find|which|where|type|pwd|whoami|date|uptime|free|df|du)\s").unwrap();
            if read_only_pattern.is_match(command) {
                return false;
            }

            true
        }).collect()
    }

    pub fn categorize_commands(commands: &[String]) -> HashMap<String, Vec<String>> {
        let mut categories: HashMap<String, Vec<String>> = HashMap::new();
        categories.insert("fileOperations".to_string(), Vec::new());
        categories.insert("networkOperations".to_string(), Vec::new());
        categories.insert("systemOperations".to_string(), Vec::new());
        categories.insert("packageOperations".to_string(), Vec::new());
        categories.insert("other".to_string(), Vec::new());

        let file_ops = vec!["cp", "mv", "rm", "mkdir", "rmdir", "touch", "chmod", "chown", "ln"];
        let network_ops = vec!["curl", "wget", "nc", "netcat", "ssh", "scp", "rsync"];
        let system_ops = vec!["systemctl", "service", "kill", "killall", "ps", "top", "htop", "reboot", "shutdown"];
        let package_ops = vec!["npm", "yarn", "pnpm", "pip", "conda", "brew", "apt", "yum", "dnf"];

        for command in commands {
            let first_word = command.split_whitespace().next().unwrap_or("");

            if file_ops.contains(&first_word) {
                categories.get_mut("fileOperations").unwrap().push(command.clone());
            } else if network_ops.contains(&first_word) {
                categories.get_mut("networkOperations").unwrap().push(command.clone());
            } else if system_ops.contains(&first_word) {
                categories.get_mut("systemOperations").unwrap().push(command.clone());
            } else if package_ops.contains(&first_word) {
                categories.get_mut("packageOperations").unwrap().push(command.clone());
            } else {
                categories.get_mut("other").unwrap().push(command.clone());
            }
        }

        categories
    }
}
