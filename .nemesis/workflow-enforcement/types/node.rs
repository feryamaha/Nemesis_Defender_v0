// Type declarations for Node.js built-ins
// Equivalente Rust: tipos fundamentais do sistema

use std::collections::HashMap;
use std::env;
use std::path::PathBuf;

/// Equivalente a process.env em Node.js
pub fn process_env() -> HashMap<String, Option<String>> {
    env::vars()
        .map(|(k, v)| (k, Some(v)))
        .collect()
}

/// Equivalente a process.cwd() em Node.js
pub fn process_cwd() -> PathBuf {
    env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

/// Equivalente a process.platform em Node.js
pub fn process_platform() -> &'static str {
    if cfg!(target_os = "windows") {
        "win32"
    } else if cfg!(target_os = "macos") {
        "darwin"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else {
        "unknown"
    }
}

/// Equivalente a process.pid em Node.js
pub fn process_pid() -> u32 {
    std::process::id()
}

/// Equivalente a process.ppid em Node.js
pub fn process_ppid() -> Option<u32> {
    // Rust não tem API padrão para obter PPID em todas as plataformas
    // Em plataformas Unix pode-se usar libc, mas aqui retornamos None
    None
}

/// Buffer equivalente simplificado
pub struct Buffer {
    pub data: Vec<u8>,
}

impl Buffer {
    pub fn from(data: &[u8]) -> Self {
        Self {
            data: data.to_vec(),
        }
    }

    pub fn from_string(s: &str, _encoding: Option<&str>) -> Self {
        Self {
            data: s.as_bytes().to_vec(),
        }
    }
}

/// Console logging equivalentes
pub mod console {
    pub fn log(args: &[&str]) {
        println!("{}", args.join(" "));
    }

    pub fn error(args: &[&str]) {
        eprintln!("{}", args.join(" "));
    }

    pub fn warn(args: &[&str]) {
        eprintln!("WARN: {}", args.join(" "));
    }

    pub fn info(args: &[&str]) {
        println!("INFO: {}", args.join(" "));
    }
}
