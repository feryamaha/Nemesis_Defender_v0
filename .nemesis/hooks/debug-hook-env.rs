// =============================================================================
// NEMESIS HOOK DEBUG
// =============================================================================
//
// Script de debugging que intercepta e registra o ambiente de execucao dos hooks.
// Escreve metadados em /tmp/nemesis-hook-debug.log e encadeia para o hook real.
//
// Uso: Configurado temporariamente no lugar do nemesis-pretool-check.sh
//       para diagnosticar problemas de ambiente.
//
// =============================================================================

use std::env;
use std::fs::OpenOptions;
use std::io::{self, Read, Write};
use std::process::{Command, Stdio};

fn main() {
    let log_path = "/tmp/nemesis-hook-debug.log";
    
    // Abrir arquivo de log em modo append
    let mut log_file = match OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path) 
    {
        Ok(file) => file,
        Err(e) => {
            eprintln!("NEMESIS DEBUG ERROR: Falha ao abrir log: {}", e);
            std::process::exit(1);
        }
    };

    // Escreve header
    let _ = writeln!(log_file, "=== NEMESIS HOOK DEBUG ===");
    
    // Timestamp
    let timestamp = chrono::Local::now().format("%a %b %e %H:%M:%S %Z %Y").to_string();
    let _ = writeln!(log_file, "Timestamp: {}", timestamp);
    
    // PWD
    let pwd = env::current_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| "unknown".to_string());
    let _ = writeln!(log_file, "PWD: {}", pwd);
    
    // Whoami
    let whoami = env::var("USER")
        .or_else(|_| env::var("USERNAME"))
        .unwrap_or_else(|_| "unknown".to_string());
    let _ = writeln!(log_file, "Whoami: {}", whoami);
    
    // PATH
    let path = env::var("PATH").unwrap_or_else(|_| "not set".to_string());
    let _ = writeln!(log_file, "PATH: {}", path);
    
    // NODE_ENV
    let node_env = env::var("NODE_ENV").unwrap_or_else(|_| "not set".to_string());
    let _ = writeln!(log_file, "NODE_ENV: {}", node_env);
    
    // PROJECT_DIR
    let project_dir = env::var("PROJECT_DIR").unwrap_or_else(|_| "not set".to_string());
    let _ = writeln!(log_file, "PROJECT_DIR: {}", project_dir);
    
    // Linha em branco
    let _ = writeln!(log_file, "");
    
    // Capturar stdin
    let _ = writeln!(log_file, "=== STDIN RECEBIDO ===");
    
    let mut stdin_content = String::new();
    if io::stdin().read_to_string(&mut stdin_content).is_ok() {
        let _ = writeln!(log_file, "{}", stdin_content);
    }
    
    let _ = writeln!(log_file, "");
    let _ = writeln!(log_file, "=== ENCADEANDO NEMESIS ===");
    
    // Encadear para o hook real (mesmo caminho hardcoded do original)
    let hook_path = "/Users/fernandomoreira/devproj/Portal-Dental-UNI_Auclan-Design/.nemesis/hooks/nemesis-pretool-check.sh";
    
    let output = Command::new("bash")
        .arg(hook_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn();

    match output {
        Ok(mut child) => {
            // Passar o mesmo stdin para o hook
            if let Some(mut stdin) = child.stdin.take() {
                let _ = stdin.write_all(stdin_content.as_bytes());
            }
            
            match child.wait_with_output() {
                Ok(result) => {
                    let stdout = String::from_utf8_lossy(&result.stdout);
                    let stderr = String::from_utf8_lossy(&result.stderr);
                    
                    // Escreve stdout no log
                    let _ = writeln!(log_file, "{}", stdout);
                    // Escreve stderr no log
                    let _ = writeln!(log_file, "{}", stderr);
                    
                    // Tambem imprime na saida (tee behavior)
                    print!("{}", stdout);
                    eprint!("{}", stderr);
                    
                    let exit_code = result.status.code().unwrap_or(0);
                    let _ = writeln!(log_file, "=== FIM DO DEBUG ===");
                    let _ = writeln!(log_file, "");
                    
                    // Propagar exit code
                    std::process::exit(exit_code);
                }
                Err(e) => {
                    let _ = writeln!(log_file, "Erro ao executar hook: {}", e);
                    let _ = writeln!(log_file, "=== FIM DO DEBUG ===");
                    std::process::exit(1);
                }
            }
        }
        Err(e) => {
            let _ = writeln!(log_file, "Erro ao spawnar hook: {}", e);
            let _ = writeln!(log_file, "=== FIM DO DEBUG ===");
            std::process::exit(1);
        }
    }
}
