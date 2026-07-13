//! P2 (SPEC_022): logica pura da restored-allowlist (sem disco).
use nemesis_defender::quarantine::{entry_matches, sha256_hex, RestoredEntry};
use std::path::Path;

fn entry(path: &str, content: &[u8]) -> RestoredEntry {
    RestoredEntry {
        original_path: path.to_string(),
        sha256: sha256_hex(content),
        restored_at: "2026-07-13T00:00:00-03:00".to_string(),
    }
}

#[test]
fn sha256_known_vector() {
    // SHA-256("hello") valor conhecido.
    assert_eq!(
        sha256_hex(b"hello"),
        "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
    );
}

#[test]
fn matches_same_path_and_content() {
    let content = b"const x = eval(atob('payload'));";
    let entries = vec![entry("/proj/.next/x.js", content)];
    assert!(entry_matches(&entries, Path::new("/proj/.next/x.js"), content));
}

#[test]
fn tampered_content_does_not_match() {
    // Mesmo path, conteudo diferente => NAO isento (novo conteudo pode ser malware real).
    let entries = vec![entry("/proj/a.js", b"original")];
    assert!(!entry_matches(&entries, Path::new("/proj/a.js"), b"tampered"));
}

#[test]
fn different_path_does_not_match() {
    let content = b"same bytes";
    let entries = vec![entry("/proj/a.js", content)];
    assert!(!entry_matches(&entries, Path::new("/proj/OTHER.js"), content));
}

#[test]
fn empty_allowlist_matches_nothing() {
    assert!(!entry_matches(&[], Path::new("/proj/a.js"), b"x"));
}
