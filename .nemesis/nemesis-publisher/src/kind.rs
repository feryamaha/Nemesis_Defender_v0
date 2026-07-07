//! Classificacao de kind a partir da mensagem do ledger.
//! Portado de local-source.ts:42-58.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    CommandBlocked,
    MaliciousContent,
    FileAccessDenied,
    FileReadDenied,
    BehavioralEscalation,
    WriteOutsideProject,
    WriteOutsideScope,
    GlobProtected,
    Other,
}

impl Kind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Kind::CommandBlocked => "command_blocked",
            Kind::MaliciousContent => "malicious_content",
            Kind::FileAccessDenied => "file_access_denied",
            Kind::FileReadDenied => "file_read_denied",
            Kind::BehavioralEscalation => "behavioral_escalation",
            Kind::WriteOutsideProject => "write_outside_project",
            Kind::WriteOutsideScope => "write_outside_scope",
            Kind::GlobProtected => "glob_protected",
            Kind::Other => "other",
        }
    }
}

/// Mapeia mensagem do ledger para Kind por casamento de substring.
/// Portado de local-source.ts:53-58.
pub fn classify_kind(message: &str) -> Kind {
    if message.contains("COMANDO NAO PERMITIDO") {
        Kind::CommandBlocked
    } else if message.contains("CONTEUDO MALICIOSO DETECTADO") {
        Kind::MaliciousContent
    } else if message.contains("ACESSO NEGADO - ARQUIVO PROTEGIDO") {
        Kind::FileAccessDenied
    } else if message.contains("LEITURA NEGADA - ARQUIVO PROTEGIDO") {
        Kind::FileReadDenied
    } else if message.contains("ESCALACAO COMPORTAMENTAL") {
        Kind::BehavioralEscalation
    } else if message.contains("ESCRITA FORA DO PROJETO") {
        Kind::WriteOutsideProject
    } else if message.contains("ESCRITA FORA DO ESCOPO") {
        Kind::WriteOutsideScope
    } else if message.contains("ACESSO NEGADO (glob") {
        Kind::GlobProtected
    } else {
        Kind::Other
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_blocked() {
        assert_eq!(classify_kind("NEMESIS SEC - COMANDO NAO PERMITIDO \u{00b7} foo"), Kind::CommandBlocked);
    }

    #[test]
    fn test_malicious_content() {
        assert_eq!(classify_kind("NEMESIS SEC - CONTEUDO MALICIOSO DETECTADO"), Kind::MaliciousContent);
    }

    #[test]
    fn test_other() {
        assert_eq!(classify_kind("mensagem desconhecida"), Kind::Other);
    }
}
