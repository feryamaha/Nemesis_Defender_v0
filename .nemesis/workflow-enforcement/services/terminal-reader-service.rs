use crate::services::terminal_reader_logger::TerminalReaderLogger;
use crate::services::terminal_reader_types::{PathValidation, ReadOptions, ReadResult, SearchResult};
use regex::Regex;
use std::path::{Path, PathBuf};
use std::time::Instant;
use tokio::process::Command;
use tokio::time::timeout;

pub struct TerminalReaderService {
    project_root: PathBuf,
}

// Comandos por sistema operacional
struct MacCommands;
struct WindowsCommands;
struct LinuxCommands;

impl MacCommands {
    const CAT: &'static str = "cat";
    const HEAD: &'static str = "head -n";
    const TAIL: &'static str = "tail -n";
    const GREP: &'static str = "grep";
    const SED: &'static str = "sed";
    const WC: &'static str = "wc -l";
    const FIND: &'static str = "find";
}

impl WindowsCommands {
    const CAT: &'static str = "type";
    const HEAD: &'static str = "type | findstr /V \"\"";
    const TAIL: &'static str = "type | findstr /V \"\"";
    const GREP: &'static str = "findstr";
    const SED: &'static str = r#"powershell -Command "Get-Content -Path \"{file}\" | Select-String -Pattern \"{pattern}\" -Raw""#;
    const WC: &'static str = "find /V /C \"*\"";
    const FIND: &'static str = "dir /b /s";
}

impl LinuxCommands {
    const CAT: &'static str = "cat";
    const HEAD: &'static str = "head -n";
    const TAIL: &'static str = "tail -n";
    const GREP: &'static str = "grep";
    const SED: &'static str = "sed";
    const WC: &'static str = "wc -l";
    const FIND: &'static str = "find";
}

impl TerminalReaderService {
    pub fn new(project_root: Option<&str>) -> Self {
        let root = project_root.map(PathBuf::from).unwrap_or_else(|| {
            std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
        });
        Self { project_root: root }
    }

    fn detect_os() -> &'static str {
        match std::env::consts::OS {
            "macos" => "mac",
            "windows" => "windows",
            _ => "linux",
        }
    }

    async fn check_command_availability(command: &str) -> bool {
        match timeout(
            std::time::Duration::from_millis(1000),
            Command::new("sh").arg("-c").arg(command).output()
        ).await {
            Ok(Ok(_)) => true,
            _ => false,
        }
    }

    fn validate_path(&self, file_path: &str) -> anyhow::Result<PathValidation> {
        let normalized_path = self.project_root.join(file_path);
        let relative_path = normalized_path.strip_prefix(&self.project_root)
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|_| normalized_path.clone());

        // Verifica se o caminho tenta escapar do projeto
        let is_within_project = !relative_path.starts_with("..") && !normalized_path.is_absolute();
        let is_project_root = normalized_path == self.project_root;

        // Verifica se está no .gitignore
        let is_git_ignored = self.check_git_ignore_sync(&relative_path.to_string_lossy());

        // Nível de segurança
        let security_level = if !is_within_project {
            "danger"
        } else if is_git_ignored {
            "warning"
        } else {
            "safe"
        };

        Ok(PathValidation {
            is_project_root,
            is_within_project,
            is_git_ignored,
            normalized_path: normalized_path.to_string_lossy().to_string(),
            security_level: security_level.to_string(),
        })
    }

    fn check_git_ignore_sync(&self, relative_path: &str) -> bool {
        let git_ignore_path = self.project_root.join(".gitignore");
        
        if !git_ignore_path.exists() {
            return false;
        }

        match std::fs::read_to_string(&git_ignore_path) {
            Ok(git_ignore) => {
                let lines: Vec<&str> = git_ignore
                    .lines()
                    .filter(|line| !line.trim().is_empty() && !line.starts_with('#'))
                    .collect();

                lines.iter().any(|pattern| {
                    // Converte padrão .gitignore para regex
                    let regex_pattern = pattern
                        .replace('.', "\\.")
                        .replace('*', ".*")
                        .replace('?', "[^/]")
                        .replace("^/", "^")
                        .replace("/", "");

                    if let Ok(regex) = Regex::new(&regex_pattern) {
                        regex.is_match(relative_path) || 
                        regex.is_match(Path::new(relative_path).file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or(relative_path))
                    } else {
                        false
                    }
                })
            }
            Err(_) => false,
        }
    }

    pub async fn read_file(&self, file_path: &str) -> anyhow::Result<ReadResult> {
        let start_time = Instant::now();
        let validation = self.validate_path(file_path)?;
        let os = Self::detect_os();

        if !validation.is_within_project {
            return Err(anyhow::anyhow!("Caminho fora do projeto: {}", file_path));
        }

        let normalized = &validation.normalized_path;
        
        let fallback_chain = [
            format!("cat \"{}\"", normalized),
            format!("head -n 1000 \"{}\"", normalized),
            format!("node -e \"console.log(require('fs').readFileSync('{}', 'utf8'))\"", normalized),
            format!("python3 -c \"with open('{}', 'r') as f: print(f.read())\"", normalized),
            format!("echo 'ERROR: Não foi possível ler o arquivo: {}'", normalized),
        ];

        let mut used_fallbacks: Vec<String> = Vec::new();
        let mut last_error: Option<String> = None;

        for command in &fallback_chain {
            used_fallbacks.push(command.clone());
            
            let result = timeout(
                std::time::Duration::from_secs(5),
                Command::new("sh").arg("-c").arg(command).output()
            ).await;

            match result {
                Ok(Ok(output)) if output.status.success() => {
                    let duration = start_time.elapsed().as_millis() as u64;
                    let content = String::from_utf8_lossy(&output.stdout).to_string();

                    TerminalReaderLogger::log(
                        "info",
                        "readFile",
                        file_path,
                        used_fallbacks.last().unwrap_or(&String::new()),
                        true,
                        used_fallbacks.clone(),
                        Some(duration),
                        None,
                    );

                    return Ok(ReadResult {
                        content: content.trim().to_string(),
                        method: used_fallbacks.last().unwrap_or(&String::new()).clone(),
                        fallbacks: used_fallbacks.clone(),
                        duration,
                        success: true,
                        os: os.to_string(),
                    });
                }
                Ok(Ok(output)) => {
                    last_error = Some(String::from_utf8_lossy(&output.stderr).to_string());
                }
                Ok(Err(e)) => {
                    last_error = Some(e.to_string());
                }
                Err(_) => {
                    last_error = Some("Timeout".to_string());
                }
            }

            TerminalReaderLogger::log(
                "warn",
                "readFile",
                file_path,
                command,
                false,
                vec![command.clone()],
                None,
                last_error.clone(),
            );
        }

        let duration = start_time.elapsed().as_millis() as u64;
        TerminalReaderLogger::log(
            "error",
            "readFile",
            file_path,
            "all_fallbacks",
            false,
            fallback_chain.iter().map(|s| s.to_string()).collect(),
            Some(duration),
            last_error.clone(),
        );

        Err(anyhow::anyhow!(
            "Não foi possível ler o arquivo: {}. Último erro: {}",
            file_path,
            last_error.unwrap_or_default()
        ))
    }

    pub async fn read_lines(&self, file_path: &str, options: ReadOptions) -> anyhow::Result<Vec<String>> {
        let start_time = Instant::now();
        let validation = self.validate_path(file_path)?;
        let os = Self::detect_os();

        if !validation.is_within_project {
            return Err(anyhow::anyhow!("Caminho fora do projeto: {}", file_path));
        }

        let start = options.start.unwrap_or(0);
        let end = options.end.unwrap_or(start + 50);

        let line_commands: Vec<String> = if os == "windows" {
            vec![
                format!(r#"powershell -Command "Get-Content -Path \"{}\" | Select-Object -Index {} -Last {}""#, 
                    validation.normalized_path, start, end - start),
                format!(r#"powershell -Command "Get-Content -Path \"{}\" | Select-Object -Skip {} -First {}""#, 
                    validation.normalized_path, start, end - start),
            ]
        } else {
            vec![
                format!("sed -n {},{}p \"{}\"", start + 1, end, validation.normalized_path),
                format!("awk 'NR>={} && NR<={}' \"{}\"", start + 1, end, validation.normalized_path),
            ]
        };

        let mut used_fallbacks: Vec<String> = Vec::new();
        let mut last_error: Option<String> = None;

        for command in &line_commands {
            used_fallbacks.push(command.clone());
            
            let result = timeout(
                std::time::Duration::from_secs(5),
                Command::new("sh").arg("-c").arg(command).output()
            ).await;

            match result {
                Ok(Ok(output)) if output.status.success() => {
                    let duration = start_time.elapsed().as_millis() as u64;
                    let content = String::from_utf8_lossy(&output.stdout);
                    let lines: Vec<String> = content
                        .lines()
                        .filter(|l| !l.is_empty())
                        .take(end - start)
                        .map(|s| s.to_string())
                        .collect();

                    TerminalReaderLogger::log(
                        "info",
                        "readLines",
                        file_path,
                        used_fallbacks.last().unwrap_or(&String::new()),
                        true,
                        used_fallbacks.clone(),
                        Some(duration),
                        None,
                    );

                    return Ok(lines);
                }
                Ok(Ok(output)) => {
                    last_error = Some(String::from_utf8_lossy(&output.stderr).to_string());
                }
                Ok(Err(e)) => {
                    last_error = Some(e.to_string());
                }
                Err(_) => {
                    last_error = Some("Timeout".to_string());
                }
            }

            TerminalReaderLogger::log(
                "warn",
                "readLines",
                file_path,
                command,
                false,
                vec![command.clone()],
                None,
                last_error.clone(),
            );
        }

        // Fallback: ler arquivo inteiro e extrair linhas
        match self.read_file(file_path).await {
            Ok(file_result) => {
                let lines: Vec<String> = file_result.content
                    .lines()
                    .skip(start)
                .take(end - start)
                .map(|s| s.to_string())
                .collect();
                let duration = start_time.elapsed().as_millis() as u64;

                TerminalReaderLogger::log(
                    "info",
                    "readLines",
                    file_path,
                    "full_file_fallback",
                    true,
                    used_fallbacks.clone(),
                    Some(duration),
                    None,
                );

                Ok(lines)
            }
            Err(e) => {
                let duration = start_time.elapsed().as_millis() as u64;
                TerminalReaderLogger::log(
                    "error",
                    "readLines",
                    file_path,
                    "all_fallbacks",
                    false,
                    line_commands.clone(),
                    Some(duration),
                    Some(e.to_string()),
                );

                Err(anyhow::anyhow!(
                    "Não foi possível ler linhas do arquivo: {}. Último erro: {}",
                    file_path,
                    last_error.unwrap_or_default()
                ))
            }
        }
    }

    pub async fn search_in_file(&self, file_path: &str, pattern: &str) -> anyhow::Result<SearchResult> {
        let start_time = Instant::now();
        let validation = self.validate_path(file_path)?;
        let os = Self::detect_os();

        if !validation.is_within_project {
            return Err(anyhow::anyhow!("Caminho fora do projeto: {}", file_path));
        }

        let search_commands: Vec<String> = if os == "windows" {
            vec![
                format!("findstr /R /N \"{}\" \"{}\"", pattern, validation.normalized_path),
                format!(r#"powershell -Command "Select-String -Pattern \"{}\" -Path \"{}\" | Select-Object LineNumber, Line""#, 
                    pattern, validation.normalized_path),
            ]
        } else {
            vec![
                format!("grep -n \"{}\" \"{}\"", pattern, validation.normalized_path),
                format!("awk '/{}/ {{print NR \":\" $0}}' \"{}\"", pattern, validation.normalized_path),
            ]
        };

        let mut used_fallbacks: Vec<String> = Vec::new();
        let mut last_error: Option<String> = None;

        for command in &search_commands {
            used_fallbacks.push(command.clone());
            
            let result = timeout(
                std::time::Duration::from_secs(5),
                Command::new("sh").arg("-c").arg(command).output()
            ).await;

            match result {
                Ok(Ok(output)) if output.status.success() => {
                    let duration = start_time.elapsed().as_millis() as u64;
                    let content = String::from_utf8_lossy(&output.stdout);
                    let lines: Vec<String> = content
                        .lines()
                        .filter(|l| !l.trim().is_empty())
                        .map(|s| s.to_string())
                        .collect();

                    TerminalReaderLogger::log(
                        "info",
                        "searchInFile",
                        file_path,
                        used_fallbacks.last().unwrap_or(&String::new()),
                        true,
                        used_fallbacks.clone(),
                        Some(duration),
                        None,
                    );

                    return Ok(SearchResult {
                        lines,
                        method: used_fallbacks.last().unwrap_or(&String::new()).clone(),
                        fallbacks: used_fallbacks.clone(),
                        success: true,
                        os: os.to_string(),
                        duration,
                    });
                }
                Ok(Ok(output)) => {
                    last_error = Some(String::from_utf8_lossy(&output.stderr).to_string());
                }
                Ok(Err(e)) => {
                    last_error = Some(e.to_string());
                }
                Err(_) => {
                    last_error = Some("Timeout".to_string());
                }
            }

            TerminalReaderLogger::log(
                "warn",
                "searchInFile",
                file_path,
                command,
                false,
                vec![command.clone()],
                None,
                last_error.clone(),
            );
        }

        // Fallback: ler arquivo e buscar manualmente
        match self.read_file(file_path).await {
            Ok(file_result) => {
                let lines: Vec<String> = file_result.content
                    .lines()
                    .enumerate()
                    .filter(|(_, line)| line.contains(pattern))
                    .map(|(idx, line)| format!("{}:{}", idx + 1, line.trim()))
                    .collect();

                let duration = start_time.elapsed().as_millis() as u64;

                TerminalReaderLogger::log(
                    "info",
                    "searchInFile",
                    file_path,
                    "manual_search_fallback",
                    true,
                    used_fallbacks.clone(),
                    Some(duration),
                    None,
                );

                Ok(SearchResult {
                    lines,
                    method: "manual_search_fallback".to_string(),
                    fallbacks: used_fallbacks.clone(),
                    success: true,
                    os: os.to_string(),
                    duration,
                })
            }
            Err(e) => {
                let duration = start_time.elapsed().as_millis() as u64;
                TerminalReaderLogger::log(
                    "error",
                    "searchInFile",
                    file_path,
                    "all_fallbacks",
                    false,
                    search_commands.clone(),
                    Some(duration),
                    Some(e.to_string()),
                );

                Err(anyhow::anyhow!(
                    "Não foi possível buscar no arquivo: {}. Último erro: {}",
                    file_path,
                    last_error.unwrap_or_default()
                ))
            }
        }
    }

    pub async fn file_exists(&self, file_path: &str) -> bool {
        match self.validate_path(file_path) {
            Ok(validation) if !validation.is_within_project => false,
            Ok(validation) => {
                let existence_commands: Vec<String> = if Self::detect_os() == "windows" {
                    vec![
                        format!("if exist \"{}\" (echo exists) else (echo not_exists)", validation.normalized_path),
                        format!(r#"powershell -Command "Test-Path \"{}\" && echo exists || echo not_exists""#, 
                            validation.normalized_path),
                    ]
                } else {
                    vec![
                        format!("test -f \"{}\" && echo exists || echo not_exists", validation.normalized_path),
                        format!("ls \"{}\" 2>/dev/null && echo exists || echo not_exists", validation.normalized_path),
                    ]
                };

                for command in &existence_commands {
                    match timeout(
                        std::time::Duration::from_millis(3000),
                        Command::new("sh").arg("-c").arg(command).output()
                    ).await {
                        Ok(Ok(output)) if output.status.success() => {
                            let stdout = String::from_utf8_lossy(&output.stdout);
                            if stdout.contains("exists") {
                                TerminalReaderLogger::log(
                                    "info",
                                    "fileExists",
                                    file_path,
                                    command,
                                    true,
                                    vec![command.clone()],
                                    None,
                                    None,
                                );
                                return true;
                            }
                        }
                        _ => {}
                    }
                }

                // Fallback: usar fs
                match tokio::fs::metadata(&validation.normalized_path).await {
                    Ok(metadata) => {
                        TerminalReaderLogger::log(
                            "info",
                            "fileExists",
                            file_path,
                            "fs_fallback",
                            true,
                            vec!["fs_exists".to_string()],
                            None,
                            None,
                        );
                        metadata.is_file()
                    }
                    Err(e) => {
                        TerminalReaderLogger::log(
                            "error",
                            "fileExists",
                            file_path,
                            "all_fallbacks",
                            false,
                            existence_commands.clone(),
                            None,
                            Some(e.to_string()),
                        );
                        false
                    }
                }
            }
            Err(_) => false,
        }
    }

    pub async fn is_git_ignored(&self, file_path: &str) -> bool {
        match self.validate_path(file_path) {
            Ok(validation) => validation.is_git_ignored,
            Err(_) => false,
        }
    }

    pub fn get_logs(&self, level: Option<&str>) -> Vec<crate::services::terminal_reader_types::LogEntry> {
        TerminalReaderLogger::get_logs(level)
    }

    pub fn clear_logs(&self) {
        TerminalReaderLogger::clear_logs();
    }

    pub fn set_project_root(&mut self, root: &str) {
        self.project_root = PathBuf::from(root);
    }
}
