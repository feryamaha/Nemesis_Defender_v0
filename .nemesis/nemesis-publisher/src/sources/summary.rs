//! Agregacao de summary. Portado de local-source.ts:147-190.

use super::doctor::DoctorRun;
use super::pentest::PentestRun;
use super::violations::Violation;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};

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
pub struct DailyCount {
    pub date: String,
    pub pretool: u64,
    #[serde(rename = "nemesis-defender")]
    pub nemesis_defender: u64,
    #[serde(rename = "ebpf-kernel")]
    pub ebpf_kernel: u64,
    pub posttool: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummaryDoctor {
    pub verdict: String,
    pub run_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummaryPentest {
    pub total: usize,
    pub blocked: usize,
    pub run_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Summary {
    pub total_violations: usize,
    pub malformed_count: u64,
    pub escalation_count: u64,
    pub by_layer: Vec<LayerCount>,
    pub by_kind: Vec<KindCount>,
    pub last_30_days: Vec<DailyCount>,
    pub doctor: SummaryDoctor,
    pub pentest: SummaryPentest,
}

pub fn build_summary(
    violations: &[Violation],
    doctor: &DoctorRun,
    pentest: &PentestRun,
) -> Summary {
    let mut by_layer: BTreeMap<String, u64> = BTreeMap::new();
    let mut by_kind: BTreeMap<String, u64> = BTreeMap::new();

    for v in violations {
        *by_layer.entry(v.layer.clone()).or_default() += 1;
        if v.layer != "malformed" {
            *by_kind.entry(v.kind.clone()).or_default() += 1;
        }
    }

    let mut unique_dates: Vec<String> = {
        let mut seen: HashSet<String> = HashSet::new();
        let mut sorted: Vec<String> = violations
            .iter()
            .map(|v| v.date.clone())
            .filter(|d| !d.is_empty() && seen.insert(d.clone()))
            .collect();
        sorted.sort();
        sorted
    };
    let last_30: Vec<String> = unique_dates
        .iter()
        .rev()
        .take(30)
        .cloned()
        .collect();
    unique_dates.clear();

    let mut by_day: BTreeMap<String, DailyCount> = BTreeMap::new();
    for date in &last_30 {
        by_day.insert(
            date.clone(),
            DailyCount {
                date: date.clone(),
                pretool: 0,
                nemesis_defender: 0,
                ebpf_kernel: 0,
                posttool: 0,
            },
        );
    }

    for v in violations {
        if let Some(day) = by_day.get_mut(&v.date) {
            if v.layer != "malformed" {
                match v.layer.as_str() {
                    "pretool" => day.pretool += 1,
                    "nemesis-defender" => day.nemesis_defender += 1,
                    "ebpf-kernel" => day.ebpf_kernel += 1,
                    "posttool" => day.posttool += 1,
                    _ => {}
                }
            }
        }
    }

    Summary {
        total_violations: violations.len(),
        malformed_count: by_layer.get("malformed").copied().unwrap_or(0),
        escalation_count: by_kind.get("behavioral_escalation").copied().unwrap_or(0),
        by_layer: by_layer
            .iter()
            .map(|(l, c)| LayerCount {
                layer: l.clone(),
                count: *c,
            })
            .collect(),
        by_kind: by_kind
            .iter()
            .map(|(k, c)| KindCount {
                kind: k.clone(),
                count: *c,
            })
            .collect(),
        last_30_days: by_day.into_values().collect(),
        doctor: SummaryDoctor {
            verdict: doctor.verdict.clone(),
            run_at: doctor.run_at.clone(),
        },
        pentest: SummaryPentest {
            total: pentest.total,
            blocked: pentest.blocked,
            run_at: pentest.run_at.clone(),
        },
    }
}
