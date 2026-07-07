//! Derivacao de sessoes a partir de violations. Portado de local-source.ts:465-516.

use super::violations::Violation;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerCount {
    pub layer: String,
    pub count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KindCount {
    pub kind: String,
    pub count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    #[serde(rename = "type")]
    pub session_type: String,
    pub environment: String,
    pub started_at: String,
    pub ended_at: String,
    pub total_blocks: u64,
    pub by_layer: Vec<LayerCount>,
    pub by_kind: Vec<KindCount>,
}

/// Agrupa violations por dia e deriva sessoes.
pub fn derive_sessions(violations: &[Violation], environment: &str) -> Vec<Session> {
    let mut by_day: BTreeMap<String, Vec<&Violation>> = BTreeMap::new();
    for v in violations {
        if v.layer == "malformed" {
            continue;
        }
        by_day.entry(v.date.clone()).or_default().push(v);
    }

    let mut out: Vec<Session> = Vec::new();
    let mut idx = 0u64;

    for (_date, day_violations) in by_day.iter().rev() {
        let mut layer_map: BTreeMap<String, u64> = BTreeMap::new();
        let mut kind_map: BTreeMap<String, u64> = BTreeMap::new();

        for v in day_violations {
            *layer_map.entry(v.layer.clone()).or_default() += 1;
            *kind_map.entry(v.kind.clone()).or_default() += 1;
        }

        let total = day_violations.len() as u64;
        let session_type = if total > 200 {
            "redteam"
        } else if total > 100 {
            "validation"
        } else if total > 50 {
            "pentest"
        } else {
            "dev"
        };

        let first = day_violations.last();
        let last = day_violations.first();

        idx += 1;
        out.push(Session {
            id: format!("sess-{:02}", idx),
            session_type: session_type.to_string(),
            environment: environment.to_string(),
            started_at: first.map(|v| v.ts.clone()).unwrap_or_default(),
            ended_at: last.map(|v| v.ts.clone()).unwrap_or_default(),
            total_blocks: total,
            by_layer: layer_map
                .iter()
                .map(|(l, c)| LayerCount {
                    layer: l.clone(),
                    count: *c,
                })
                .collect(),
            by_kind: kind_map
                .iter()
                .map(|(k, c)| KindCount {
                    kind: k.clone(),
                    count: *c,
                })
                .collect(),
        });
    }

    out
}
