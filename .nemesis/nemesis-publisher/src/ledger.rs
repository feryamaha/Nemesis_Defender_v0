//! Leitura e agregacao do ledger unificado de bloqueios.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::io::BufRead;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Layer {
    Pretool,
    Posttool,
    NemesisDefender,
    EbpfKernel,
    Malformed,
}

impl Layer {
    pub fn as_str(&self) -> &'static str {
        match self {
            Layer::Pretool => "pretool",
            Layer::Posttool => "posttool",
            Layer::NemesisDefender => "nemesis-defender",
            Layer::EbpfKernel => "ebpf-kernel",
            Layer::Malformed => "malformed",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Nature {
    Destructive,
    Malicious,
    Other,
}

impl Nature {
    pub fn as_str(&self) -> &'static str {
        match self {
            Nature::Destructive => "destructive",
            Nature::Malicious => "malicious",
            Nature::Other => "other",
        }
    }
}

#[derive(Debug, Serialize)]
pub struct LayerCount {
    pub layer: String,
    pub count: u64,
}

#[derive(Debug, Serialize)]
pub struct NatureCount {
    pub nature: String,
    pub count: u64,
}

#[derive(Debug)]
pub struct Aggregation {
    pub total_blocks: u64,
    pub by_layer: BTreeMap<Layer, u64>,
    pub by_nature: BTreeMap<Nature, u64>,
    pub window_days: u32,
}

impl Aggregation {
    pub fn to_payload(&self, install_id: &str) -> serde_json::Value {
        let by_layer: Vec<LayerCount> = self
            .by_layer
            .iter()
            .filter(|(_, &c)| c > 0)
            .map(|(layer, &count)| LayerCount {
                layer: layer.as_str().to_string(),
                count,
            })
            .collect();

        let by_nature: Vec<NatureCount> = self
            .by_nature
            .iter()
            .filter(|(_, &c)| c > 0)
            .map(|(nature, &count)| NatureCount {
                nature: nature.as_str().to_string(),
                count,
            })
            .collect();

        serde_json::json!({
            "installId": install_id,
            "environment": crate::config::environment(),
            "optIn": true,
            "windowDays": self.window_days,
            "totalBlocks": self.total_blocks,
            "byLayer": by_layer,
            "byNature": by_nature
        })
    }
}

/// Mapeia string de layer do ledger para enum do contrato.
fn map_layer(s: &str) -> Layer {
    match s {
        "pretool" => Layer::Pretool,
        "posttool" => Layer::Posttool,
        "nemesis-defender" => Layer::NemesisDefender,
        "ebpf-kernel" => Layer::EbpfKernel,
        _ => Layer::Malformed,
    }
}

/// Mapeia mensagem do ledger para nature do contrato (por prefixo).
fn map_nature(msg: &str) -> Nature {
    if msg.contains("NEMESIS SEC - COMANDO NAO PERMITIDO") {
        Nature::Destructive
    } else if msg.contains("NEMESIS SEC - CONTEUDO MALICIOSO DETECTADO") {
        Nature::Malicious
    } else if msg.contains("NEMESIS SEC - ESCALACAO COMPORTAMENTAL") {
        Nature::Malicious
    } else {
        Nature::Other
    }
}

/// Le o ledger JSONL e agrega contadores.
pub fn aggregate(path: &Path) -> Aggregation {
    let mut total_blocks = 0u64;
    let mut by_layer: BTreeMap<Layer, u64> = BTreeMap::new();
    let mut by_nature: BTreeMap<Nature, u64> = BTreeMap::new();
    let mut first_ts: Option<chrono::DateTime<chrono::FixedOffset>> = None;
    let mut last_ts: Option<chrono::DateTime<chrono::FixedOffset>> = None;

    let file = match std::fs::File::open(path) {
        Ok(f) => f,
        Err(_) => {
            return Aggregation {
                total_blocks: 0,
                by_layer,
                by_nature,
                window_days: 1,
            };
        }
    };

    for line in std::io::BufReader::new(file).lines().map_while(Result::ok) {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        total_blocks += 1;

        let v: serde_json::Value = match serde_json::from_str(line) {
            Ok(v) => v,
            Err(_) => {
                *by_layer.entry(Layer::Malformed).or_insert(0) += 1;
                *by_nature.entry(Nature::Other).or_insert(0) += 1;
                continue;
            }
        };

        let layer_str = v.get("layer").and_then(|x| x.as_str()).unwrap_or("");
        let layer = map_layer(layer_str);
        *by_layer.entry(layer).or_insert(0) += 1;

        let msg = v.get("message").and_then(|x| x.as_str()).unwrap_or("");
        let nature = map_nature(msg);
        *by_nature.entry(nature).or_insert(0) += 1;

        if let Some(ts_str) = v.get("ts").and_then(|x| x.as_str()) {
            if let Ok(ts) = chrono::DateTime::parse_from_rfc3339(ts_str) {
                if first_ts.is_none() || ts < first_ts.unwrap() {
                    first_ts = Some(ts);
                }
                if last_ts.is_none() || ts > last_ts.unwrap() {
                    last_ts = Some(ts);
                }
            }
        }
    }

    let window_days = match (first_ts, last_ts) {
        (Some(first), Some(last)) => {
            let diff = last - first;
            let days = diff.num_days().max(0) as u32 + 1;
            days.min(365)
        }
        _ => 1,
    };

    Aggregation {
        total_blocks,
        by_layer,
        by_nature,
        window_days,
    }
}
