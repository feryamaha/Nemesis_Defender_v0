//! Telemetria do ledger unificado de bloqueios (`.nemesis/logs/nemesis-violations.log`).
//!
//! Lê o ledger JSONL e agrega: total, por CAMADA (com a leitura de prioridade — eBPF, a
//! última camada, deve ter o MENOR volume), por TIPO (prefixo da mensagem padrão) e por DIA.
//! Serve para validar o projeto: quantas proteções, quais os tipos mais incidentes, qual
//! camada mais bloqueia, e candidatos a falso-positivo.

use std::collections::BTreeMap;
use std::io::BufRead;

const LAYER_ORDER: [&str; 4] = ["pretool", "posttool", "nemesis-defender", "ebpf-kernel"];

pub fn print_stats() {
    let path = crate::violations_log::ledger_path();
    let file = match std::fs::File::open(&path) {
        Ok(f) => f,
        Err(_) => {
            println!("Nenhum bloqueio registrado ainda ({}).", path.display());
            return;
        }
    };

    let mut total = 0usize;
    let mut by_layer: BTreeMap<String, usize> = BTreeMap::new();
    let mut by_type: BTreeMap<String, usize> = BTreeMap::new();
    let mut by_date: BTreeMap<String, usize> = BTreeMap::new();
    let mut first_ts = String::new();
    let mut last_ts = String::new();

    for line in std::io::BufReader::new(file).lines().map_while(Result::ok) {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let v: serde_json::Value = match serde_json::from_str(line) {
            Ok(v) => v,
            Err(_) => continue,
        };
        total += 1;
        let layer = v.get("layer").and_then(|x| x.as_str()).unwrap_or("unknown");
        *by_layer.entry(layer.to_string()).or_insert(0) += 1;

        let msg = v.get("message").and_then(|x| x.as_str()).unwrap_or("");
        // Categoria = prefixo padrão antes do separador de alvo `·`.
        let cat = msg.split('·').next().unwrap_or(msg).trim();
        *by_type.entry(cat.to_string()).or_insert(0) += 1;

        if let Some(d) = v.get("date").and_then(|x| x.as_str()) {
            if !d.is_empty() {
                *by_date.entry(d.to_string()).or_insert(0) += 1;
            }
        }
        if let Some(ts) = v.get("ts").and_then(|x| x.as_str()) {
            if first_ts.is_empty() {
                first_ts = ts.to_string();
            }
            last_ts = ts.to_string();
        }
    }

    if total == 0 {
        println!("Nenhum bloqueio registrado ainda.");
        return;
    }

    let pct = |n: usize| (n as f64) * 100.0 / (total as f64);

    println!("==============================================================");
    println!(" NEMESIS — Telemetria de Bloqueios");
    println!("==============================================================");
    println!("Arquivo : {}", path.display());
    println!("Periodo : {}  ->  {}", first_ts, last_ts);
    println!("TOTAL de bloqueios: {}", total);
    println!();

    println!("-- Por CAMADA (prioridade: eBPF e a ULTIMA camada -> deve ser o MENOR) --");
    for layer in LAYER_ORDER {
        let n = by_layer.get(layer).copied().unwrap_or(0);
        println!("  {:<18} {:>7}  ({:>5.1}%)", layer, n, pct(n));
    }
    for (l, n) in &by_layer {
        if !LAYER_ORDER.contains(&l.as_str()) {
            println!("  {:<18} {:>7}  ({:>5.1}%)", l, n, pct(*n));
        }
    }
    println!();

    println!("-- Por TIPO (mais incidente primeiro) --");
    let mut types: Vec<(&String, &usize)> = by_type.iter().collect();
    types.sort_by(|a, b| b.1.cmp(a.1));
    for (cat, n) in types {
        println!("  {:>7}  ({:>5.1}%)  {}", n, pct(*n), cat);
    }
    println!();

    println!("-- Por DIA --");
    for (d, n) in &by_date {
        println!("  {}  {:>7}", d, n);
    }
}
