use nemesis_publisher::kind::{classify_kind, Kind};

#[test]
fn test_classify_command_blocked() {
    assert_eq!(
        classify_kind("NEMESIS SEC - COMANDO NAO PERMITIDO \u{00b7} foo"),
        Kind::CommandBlocked
    );
}

#[test]
fn test_classify_malicious_content() {
    assert_eq!(
        classify_kind("NEMESIS SEC - CONTEUDO MALICIOSO DETECTADO"),
        Kind::MaliciousContent
    );
}

#[test]
fn test_classify_file_access_denied() {
    assert_eq!(
        classify_kind("NEMESIS SEC - ACESSO NEGADO - ARQUIVO PROTEGIDO"),
        Kind::FileAccessDenied
    );
}

#[test]
fn test_classify_file_read_denied() {
    assert_eq!(
        classify_kind("NEMESIS SEC - LEITURA NEGADA - ARQUIVO PROTEGIDO"),
        Kind::FileReadDenied
    );
}

#[test]
fn test_classify_behavioral_escalation() {
    assert_eq!(
        classify_kind("NEMESIS SEC - ESCALACAO COMPORTAMENTAL"),
        Kind::BehavioralEscalation
    );
}

#[test]
fn test_classify_write_outside_project() {
    assert_eq!(
        classify_kind("NEMESIS SEC - ESCRITA FORA DO PROJETO"),
        Kind::WriteOutsideProject
    );
}

#[test]
fn test_classify_write_outside_scope() {
    assert_eq!(
        classify_kind("NEMESIS SEC - ESCRITA FORA DO ESCOPO"),
        Kind::WriteOutsideScope
    );
}

#[test]
fn test_classify_glob_protected() {
    assert_eq!(
        classify_kind("NEMESIS SEC - ACESSO NEGADO (glob pattern)"),
        Kind::GlobProtected
    );
}

#[test]
fn test_classify_other() {
    assert_eq!(classify_kind("mensagem desconhecida"), Kind::Other);
}
