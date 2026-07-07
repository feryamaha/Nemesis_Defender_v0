//! Testes de agregacao do ledger e mapeamento layer/nature.

use nemesis_publisher::ledger::{aggregate, Layer, Nature};
use std::io::Write;

fn write_temp_ledger(content: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir();
    let path = dir.join(format!(
        "nemesis-test-ledger-{}-{}.jsonl",
        std::process::id(),
        uuid::Uuid::new_v4()
    ));
    let mut f = std::fs::File::create(&path).unwrap();
    f.write_all(content.as_bytes()).unwrap();
    path
}

#[test]
fn test_layer_mapping() {
    let ledger = r#"{"ts":"2026-07-06T10:00:00+00:00","date":"2026-07-06","time":"10:00:00","layer":"pretool","message":"NEMESIS SEC - COMANDO NAO PERMITIDO"}
{"ts":"2026-07-06T10:01:00+00:00","date":"2026-07-06","time":"10:01:00","layer":"posttool","message":"NEMESIS SEC - ACESSO NEGADO"}
{"ts":"2026-07-06T10:02:00+00:00","date":"2026-07-06","time":"10:02:00","layer":"nemesis-defender","message":"NEMESIS SEC - CONTEUDO MALICIOSO DETECTADO"}
{"ts":"2026-07-06T10:03:00+00:00","date":"2026-07-06","time":"10:03:00","layer":"ebpf-kernel","message":"NEMESIS SEC - COMANDO NAO PERMITIDO"}
"#;
    let path = write_temp_ledger(ledger);
    let agg = aggregate(&path);
    std::fs::remove_file(&path).ok();

    assert_eq!(agg.total_blocks, 4);
    assert_eq!(*agg.by_layer.get(&Layer::Pretool).unwrap_or(&0), 1);
    assert_eq!(*agg.by_layer.get(&Layer::Posttool).unwrap_or(&0), 1);
    assert_eq!(*agg.by_layer.get(&Layer::NemesisDefender).unwrap_or(&0), 1);
    assert_eq!(*agg.by_layer.get(&Layer::EbpfKernel).unwrap_or(&0), 1);
}

#[test]
fn test_nature_mapping() {
    let ledger = r#"{"ts":"2026-07-06T10:00:00+00:00","date":"2026-07-06","time":"10:00:00","layer":"pretool","message":"NEMESIS SEC - COMANDO NAO PERMITIDO"}
{"ts":"2026-07-06T10:01:00+00:00","date":"2026-07-06","time":"10:01:00","layer":"nemesis-defender","message":"NEMESIS SEC - CONTEUDO MALICIOSO DETECTADO"}
{"ts":"2026-07-06T10:02:00+00:00","date":"2026-07-06","time":"10:02:00","layer":"nemesis-defender","message":"NEMESIS SEC - ESCALACAO COMPORTAMENTAL"}
{"ts":"2026-07-06T10:03:00+00:00","date":"2026-07-06","time":"10:03:00","layer":"pretool","message":"NEMESIS SEC - ACESSO NEGADO - ARQUIVO PROTEGIDO"}
"#;
    let path = write_temp_ledger(ledger);
    let agg = aggregate(&path);
    std::fs::remove_file(&path).ok();

    assert_eq!(*agg.by_nature.get(&Nature::Destructive).unwrap_or(&0), 1);
    assert_eq!(*agg.by_nature.get(&Nature::Malicious).unwrap_or(&0), 2);
    assert_eq!(*agg.by_nature.get(&Nature::Other).unwrap_or(&0), 1);
}

#[test]
fn test_malformed_lines() {
    let ledger = "linha invalida\n{nao e json}\n{\"ts\":\"2026-07-06T10:00:00+00:00\",\"layer\":\"pretool\",\"message\":\"NEMESIS SEC - COMANDO NAO PERMITIDO\"}\n";
    let path = write_temp_ledger(ledger);
    let agg = aggregate(&path);
    std::fs::remove_file(&path).ok();

    assert_eq!(agg.total_blocks, 3);
    assert_eq!(*agg.by_layer.get(&Layer::Malformed).unwrap_or(&0), 2);
    assert_eq!(*agg.by_layer.get(&Layer::Pretool).unwrap_or(&0), 1);
    assert_eq!(*agg.by_nature.get(&Nature::Other).unwrap_or(&0), 2);
    assert_eq!(*agg.by_nature.get(&Nature::Destructive).unwrap_or(&0), 1);
}

#[test]
fn test_window_days() {
    let ledger = r#"{"ts":"2026-07-01T10:00:00+00:00","date":"2026-07-01","time":"10:00:00","layer":"pretool","message":"NEMESIS SEC - COMANDO NAO PERMITIDO"}
{"ts":"2026-07-06T10:00:00+00:00","date":"2026-07-06","time":"10:00:00","layer":"pretool","message":"NEMESIS SEC - COMANDO NAO PERMITIDO"}
"#;
    let path = write_temp_ledger(ledger);
    let agg = aggregate(&path);
    std::fs::remove_file(&path).ok();

    assert_eq!(agg.window_days, 6);
}

#[test]
fn test_window_days_single_entry() {
    let ledger = r#"{"ts":"2026-07-06T10:00:00+00:00","date":"2026-07-06","time":"10:00:00","layer":"pretool","message":"NEMESIS SEC - COMANDO NAO PERMITIDO"}
"#;
    let path = write_temp_ledger(ledger);
    let agg = aggregate(&path);
    std::fs::remove_file(&path).ok();

    assert_eq!(agg.window_days, 1);
}

#[test]
fn test_payload_shape() {
    let ledger = r#"{"ts":"2026-07-06T10:00:00+00:00","date":"2026-07-06","time":"10:00:00","layer":"pretool","message":"NEMESIS SEC - COMANDO NAO PERMITIDO"}
"#;
    let path = write_temp_ledger(ledger);
    let agg = aggregate(&path);
    std::fs::remove_file(&path).ok();

    let payload = agg.to_payload("test-install-id");
    assert_eq!(payload["installId"], "test-install-id");
    assert_eq!(payload["environment"], "official");
    assert_eq!(payload["optIn"], true);
    assert_eq!(payload["totalBlocks"], 1);
    assert!(payload["byLayer"].is_array());
    assert!(payload["byNature"].is_array());
}

#[test]
fn test_empty_ledger() {
    let path = std::path::PathBuf::from("/tmp/nemesis-nonexistent-ledger-test.jsonl");
    let agg = aggregate(&path);
    assert_eq!(agg.total_blocks, 0);
    assert_eq!(agg.window_days, 1);
}
