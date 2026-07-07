//! Leitura do ledger de violacoes. Portado de local-source.ts:71-143.
//!
//! SPEC-001: o ledger e append-only; o parse e INCREMENTAL por offset de bytes.
//! O custo de re-parse completo (~1s com 17k linhas, medido) so ocorre na primeira
//! leitura ou apos truncamento/rotacao; mudancas normais custam so o delta.

use crate::kind::{classify_kind, Kind};
use crate::sanitize::extract_target;
use serde::{Deserialize, Serialize};
use std::io::BufRead;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Violation {
    pub id: String,
    pub ts: String,
    pub date: String,
    pub time: String,
    pub layer: String,
    pub kind: String,
    pub message: String,
    pub target: Option<String>,
    pub raw_ok: bool,
}

const VALID_LAYERS: &[&str] = &["pretool", "nemesis-defender", "ebpf-kernel", "posttool", "malformed"];

/// Estado do parse incremental: bytes ja consumidos + violations em ORDEM DE ARQUIVO.
/// A ordenacao (ts desc) e responsabilidade de quem serve (server.rs).
pub struct LedgerState {
    pub offset: u64,
    pub next_idx: usize,
    pub items: Vec<Violation>,
}

impl LedgerState {
    fn empty() -> Self {
        LedgerState { offset: 0, next_idx: 0, items: Vec::new() }
    }
}

fn parse_line(line: &str, idx: usize) -> Violation {
    let parsed: Result<serde_json::Value, _> = serde_json::from_str(line);
    match parsed {
        Ok(v) => {
            let layer_str = v.get("layer").and_then(|x| x.as_str()).unwrap_or("");
            let is_valid = VALID_LAYERS.contains(&layer_str);
            let layer = if is_valid { layer_str } else { "malformed" };

            let ts = v.get("ts").and_then(|x| x.as_str()).unwrap_or("1970-01-01T00:00:00.000-03:00");
            let date = v.get("date").and_then(|x| x.as_str()).unwrap_or("1970-01-01");
            let time = v.get("time").and_then(|x| x.as_str()).unwrap_or("00:00:00");
            let message = v.get("message").and_then(|x| x.as_str()).unwrap_or("{linha corrompida no ledger}");

            let (kind, target, raw_ok) = if layer == "malformed" {
                (Kind::Other, None, false)
            } else {
                (classify_kind(message), extract_target(message), true)
            };

            Violation {
                id: format!("v-{}", idx),
                ts: ts.to_string(),
                date: date.to_string(),
                time: time.to_string(),
                layer: layer.to_string(),
                kind: kind.as_str().to_string(),
                message: if layer == "malformed" {
                    "{linha corrompida no ledger}".to_string()
                } else {
                    message.to_string()
                },
                target,
                raw_ok,
            }
        }
        Err(_) => Violation {
            id: format!("v-{}", idx),
            ts: "1970-01-01T00:00:00.000-03:00".to_string(),
            date: "1970-01-01".to_string(),
            time: "00:00:00".to_string(),
            layer: "malformed".to_string(),
            kind: "other".to_string(),
            message: "{linha corrompida no ledger}".to_string(),
            target: None,
            raw_ok: false,
        },
    }
}

/// Parse incremental: consome apenas os bytes novos desde `prev.offset`.
/// - Arquivo menor que o offset salvo (truncado/rotacionado) → re-parse completo.
/// - Ultima linha sem `\n` (escrita em andamento) → nao consumida; fica para a proxima.
pub fn parse_ledger_incremental(path: &Path, prev: Option<LedgerState>) -> LedgerState {
    let mut state = prev.unwrap_or_else(LedgerState::empty);

    let mut file = match std::fs::File::open(path) {
        Ok(f) => f,
        Err(_) => return LedgerState::empty(),
    };

    let len = file.metadata().map(|m| m.len()).unwrap_or(0);
    if len < state.offset {
        state = LedgerState::empty();
    }
    if len == state.offset {
        return state; // nada novo
    }

    use std::io::{Seek, SeekFrom};
    if file.seek(SeekFrom::Start(state.offset)).is_err() {
        return state;
    }

    let mut reader = std::io::BufReader::new(file);
    let mut buf = String::new();
    loop {
        buf.clear();
        let n = match reader.read_line(&mut buf) {
            Ok(0) | Err(_) => break,
            Ok(n) => n,
        };
        if !buf.ends_with('\n') {
            break; // linha parcial: consumir na proxima rodada
        }
        state.offset += n as u64;
        let line = buf.trim();
        if line.is_empty() {
            continue;
        }
        state.items.push(parse_line(line, state.next_idx));
        state.next_idx += 1;
    }

    state
}

/// Le o ledger JSONL COMPLETO e retorna violations ordenadas do mais recente para o
/// mais antigo. Mantida para os consumidores nao-incrementais (--sync/neon.rs).
pub fn parse_ledger(path: &Path) -> Vec<Violation> {
    let state = parse_ledger_incremental(path, None);
    let mut out = state.items;
    out.sort_by(|a, b| b.ts.cmp(&a.ts));
    out
}
