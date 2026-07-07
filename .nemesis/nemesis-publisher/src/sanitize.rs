//! Sanitizacao de paths reais da maquina para formato ~/...

use regex::Regex;

/// Substitui `/home/<usuario>/` e `/Users/<usuario>/` por `~/`.
/// Paths absolutos que nao casam sao substituidos por `<redacted>`.
pub fn sanitize_path(path: &str) -> String {
    let home_re = Regex::new(r"^/home/[^/]+").unwrap();
    let users_re = Regex::new(r"^/Users/[^/]+").unwrap();
    let result = home_re.replace(path, "~").to_string();
    let result = users_re.replace(&result, "~").to_string();
    if result.starts_with("/") {
        "<redacted>".to_string()
    } else {
        result
    }
}

/// Extrai o target apos ` · ` na mensagem e sanitiza.
pub fn extract_target(message: &str) -> Option<String> {
    let idx = message.find(" \u{00b7} ")?;
    let raw = message[idx + 3..].trim();
    if raw.is_empty() {
        None
    } else {
        Some(sanitize_path(raw))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_home() {
        assert_eq!(sanitize_path("/home/fernando/devproj/foo"), "~/devproj/foo");
    }

    #[test]
    fn test_sanitize_users() {
        assert_eq!(sanitize_path("/Users/fernando/devproj/foo"), "~/devproj/foo");
    }

    #[test]
    fn test_sanitize_other_absolute() {
        assert_eq!(sanitize_path("/etc/passwd"), "<redacted>");
    }

    #[test]
    fn test_sanitize_relative() {
        assert_eq!(sanitize_path("relative/path"), "relative/path");
    }

    #[test]
    fn test_extract_target() {
        let msg = "NEMESIS SEC - COMANDO NAO PERMITIDO \u{00b7} /home/fernando/devproj/foo";
        assert_eq!(extract_target(msg), Some("~/devproj/foo".to_string()));
    }

    #[test]
    fn test_extract_target_no_separator() {
        assert_eq!(extract_target("sem separador"), None);
    }
}
