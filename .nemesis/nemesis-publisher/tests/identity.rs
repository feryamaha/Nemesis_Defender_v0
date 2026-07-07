//! Testes de geracao de identidade e tokens.

use nemesis_publisher::identity::{create_identity, sha256_hex};

#[test]
fn test_identity_install_id_is_uuid_v4() {
    let id = create_identity();
    let uuid = uuid::Uuid::parse_str(&id.install_id);
    assert!(uuid.is_ok(), "install_id nao e UUID valido: {}", id.install_id);
    assert_eq!(uuid.unwrap().get_version_num(), 4, "install_id nao e UUID v4");
}

#[test]
fn test_identity_token_hashes_are_sha256_hex() {
    let id = create_identity();
    assert_eq!(
        id.project_token_hash.len(),
        64,
        "project_token_hash deve ter 64 chars hex"
    );
    assert!(
        id.project_token_hash.chars().all(|c| c.is_ascii_hexdigit()),
        "project_token_hash deve ser hex"
    );
    assert_eq!(
        id.ingest_token_hash.len(),
        64,
        "ingest_token_hash deve ter 64 chars hex"
    );
    assert!(
        id.ingest_token_hash.chars().all(|c| c.is_ascii_hexdigit()),
        "ingest_token_hash deve ser hex"
    );
}

#[test]
fn test_identity_ingest_token_is_hex_64() {
    let id = create_identity();
    assert_eq!(id.ingest_token.len(), 64, "ingest_token deve ter 64 chars hex");
    assert!(
        id.ingest_token.chars().all(|c| c.is_ascii_hexdigit()),
        "ingest_token deve ser hex"
    );
}

#[test]
fn test_identity_alias_format() {
    let id = create_identity();
    assert!(
        id.alias.starts_with("inst-"),
        "alias deve comecar com 'inst-': {}",
        id.alias
    );
    assert_eq!(id.alias.len(), 9, "alias deve ter 9 chars (inst- + 4)");
}

#[test]
fn test_identity_opt_in_default_true() {
    let id = create_identity();
    assert!(id.opt_in, "opt_in deve ser true na criacao");
}

#[test]
fn test_identity_registered_at_none_on_create() {
    let id = create_identity();
    assert!(id.registered_at.is_none(), "registered_at deve ser None na criacao");
}

#[test]
fn test_sha256_hex_known_value() {
    let hash = sha256_hex("hello");
    assert_eq!(
        hash,
        "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824",
        "SHA-256 de 'hello' deve ser o valor conhecido"
    );
}

#[test]
fn test_sha256_hex_length() {
    let hash = sha256_hex("test input");
    assert_eq!(hash.len(), 64, "SHA-256 hex deve ter 64 chars");
    assert!(
        hash.chars().all(|c| c.is_ascii_hexdigit()),
        "SHA-256 hex deve ser apenas hex digits"
    );
}
