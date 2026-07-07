//! Leitura de coverage de denylists e AST linters. Portado de local-source.ts:314-411.

use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bilingual {
    pub pt: String,
    pub en: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageLayer {
    pub layer: String,
    pub label: Bilingual,
    pub patterns: usize,
    pub source: String,
    pub description: Bilingual,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageReport {
    pub layers: Vec<CoverageLayer>,
}

/// Le os arquivos de configuracao de regras e retorna CoverageReport com 6 layers.
pub fn read_coverage(nemesis_path: &Path) -> CoverageReport {
    let defender_config_path =
        nemesis_path.join(".nemesis/nemesis-defender/config/denylist-defender.json");
    let folder_files_path = nemesis_path.join(".nemesis/denylist/denylist-folder-files.json");
    let rules_toml_path = nemesis_path.join(".nemesis/ast-linters/rules.toml");

    let defender_total = count_defender_patterns(&defender_config_path);
    let (absolute_block, write_block) = count_folder_files(&folder_files_path);
    let total_folder = absolute_block + write_block;
    let ast_rules = count_ast_rules(&rules_toml_path);

    let layers = vec![
        CoverageLayer {
            layer: "embedded-denylist".to_string(),
            label: Bilingual {
                pt: "Denylist embutida".to_string(),
                en: "Embedded denylist".to_string(),
            },
            patterns: 0,
            source: ".nemesis/denylist/deny-list-base.json (via include_str!)".to_string(),
            description: Bilingual {
                pt: "Comandos bloqueados + padroes de evasao do Nemesis, compilados no binario."
                    .to_string(),
                en: "Blocked commands + Nemesis evasion patterns, compiled into the binary."
                    .to_string(),
            },
        },
        CoverageLayer {
            layer: "protected-paths".to_string(),
            label: Bilingual {
                pt: "Paths protegidos".to_string(),
                en: "Protected paths".to_string(),
            },
            patterns: total_folder,
            source: ".nemesis/denylist/denylist-folder-files.json".to_string(),
            description: Bilingual {
                pt: format!(
                    "Bloqueio absoluto ({}) + bloqueio de escrita ({}).",
                    absolute_block, write_block
                ),
                en: format!(
                    "Absolute block ({}) + write block ({}).",
                    absolute_block, write_block
                ),
            },
        },
        CoverageLayer {
            layer: "ast-visitors".to_string(),
            label: Bilingual {
                pt: "Visitors AST".to_string(),
                en: "AST visitors".to_string(),
            },
            patterns: ast_rules,
            source: ".nemesis/ast-linters/rules.toml".to_string(),
            description: Bilingual {
                pt: format!(
                    "Regras semanticas via tree-sitter (padrao Visitor); {} regras.",
                    ast_rules
                ),
                en: format!(
                    "Semantic rules via tree-sitter (Visitor pattern); {} rules.",
                    ast_rules
                ),
            },
        },
        CoverageLayer {
            layer: "scanner-heuristics".to_string(),
            label: Bilingual {
                pt: "Heuristicas do scanner".to_string(),
                en: "Scanner heuristics".to_string(),
            },
            patterns: 6,
            source: ".nemesis/nemesis-defender/src/scanner/".to_string(),
            description: Bilingual {
                pt: "Pipeline em camadas: entropia de Shannon, taint tracking (3 passadas), decode-exec.".to_string(),
                en: "Layered pipeline: Shannon entropy, taint tracking (3 passes), decode-exec.".to_string(),
            },
        },
        CoverageLayer {
            layer: "command-denylists".to_string(),
            label: Bilingual {
                pt: "Denylists de comando".to_string(),
                en: "Command denylists".to_string(),
            },
            patterns: defender_total,
            source: ".nemesis/nemesis-defender/config/denylist-defender.json".to_string(),
            description: Bilingual {
                pt: format!(
                    "Comandos hostis em conteudo de arquivo ({} padroes).",
                    defender_total
                ),
                en: format!(
                    "Hostile commands in file content ({} patterns).",
                    defender_total
                ),
            },
        },
        CoverageLayer {
            layer: "ebpf-lsm".to_string(),
            label: Bilingual {
                pt: "eBPF / BPF-LSM".to_string(),
                en: "eBPF / BPF-LSM".to_string(),
            },
            patterns: 4,
            source: ".nemesis/ebpf-kernel/".to_string(),
            description: Bilingual {
                pt: "Hook bprm_check_security + mapas (hash/ringbuf/array/LPM trie). Linux, opt-in.".to_string(),
                en: "bprm_check_security hook + maps (hash/ringbuf/array/LPM trie). Linux, opt-in.".to_string(),
            },
        },
    ];

    CoverageReport { layers }
}

fn count_defender_patterns(path: &Path) -> usize {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return 0,
    };
    let v: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(_) => return 0,
    };
    if let Some(cats) = v.get("categories").and_then(|c| c.as_object()) {
        let mut total = 0usize;
        for (_, cat_val) in cats {
            if let Some(patterns) = cat_val
                .as_object()
                .and_then(|o| o.get("patterns"))
                .and_then(|p| p.as_array())
            {
                total += patterns.len();
            }
        }
        return total;
    }
    0
}

fn count_folder_files(path: &Path) -> (usize, usize) {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return (0, 0),
    };
    let v: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(_) => return (0, 0),
    };
    let absolute_block = v
        .get("absolute_block")
        .and_then(|x| x.get("paths"))
        .and_then(|x| x.as_array())
        .map(|a| a.len())
        .unwrap_or(0);
    let write_block = v
        .get("write_block")
        .and_then(|x| x.get("paths"))
        .and_then(|x| x.as_array())
        .map(|a| a.len())
        .unwrap_or(0);
    (absolute_block, write_block)
}

fn count_ast_rules(path: &Path) -> usize {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return 0,
    };
    let parsed: toml::Value = match toml::from_str(&content) {
        Ok(v) => v,
        Err(_) => return 0,
    };
    if let Some(rule_table) = parsed.get("rule").and_then(|r| r.as_table()) {
        return rule_table.len();
    }
    0
}
