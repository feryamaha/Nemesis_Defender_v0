use crate::checks::{binaries_dir, project_root};
use crate::report::{CheckResult, CheckStatus};

const SCAFFOLD_CONFIGS: &[&str] = &[
    ".devin/hooks.json",
    ".claude/settings.json",
    ".cursor/hooks.json",
    ".codex/hooks.json",
    ".github/hooks.json",
    ".grok/hooks/nemesis-pretool-hook.json",
];

pub fn run() -> CheckResult {
    let mut res = CheckResult::new(
        "G4 - Scaffold da IDE (hooks pretool/posttool)",
        "G4 - IDE scaffold (pretool/posttool hooks)",
    );
    let root = project_root();
    // Diretório real dos binários no layout ativo (distro `.nemesis/bin/` ou fonte
    // `target/release/`) — NÃO assumir target/release, senão o scaffold dá laudo falso no distro.
    let bin_dir = binaries_dir();

    let mut found_any = false;
    let mut any_valid_pretool = false;

    for rel in SCAFFOLD_CONFIGS {
        let path = root.join(rel);
        if !path.exists() {
            continue;
        }
        found_any = true;
        let content = std::fs::read_to_string(&path).unwrap_or_default();
        let trimmed = content.trim();

        if trimmed.is_empty() || trimmed == "{}" {
            res.push(
                format!("VAZIO {} - daemon NAO sobe automaticamente (sem hook pretool).", rel),
                format!("EMPTY {} - daemon does NOT start automatically (no pretool hook).", rel),
            );
            continue;
        }

        if serde_json::from_str::<serde_json::Value>(trimmed).is_err() {
            res.push(
                format!("JSON INVALIDO {} - corrija a sintaxe.", rel),
                format!("INVALID JSON {} - fix the syntax.", rel),
            );
            continue;
        }

        let has_pre = content.contains("pretool");
        let has_post = content.contains("posttool");
        let pre_bin_exists = bin_dir
            .as_ref()
            .map(|d| d.join("nemesis-pretool-check-unix").exists())
            .unwrap_or(false);

        if has_pre && pre_bin_exists {
            any_valid_pretool = true;
            res.push(
                format!("OK    {} - pretool configurado.", rel),
                format!("OK    {} - pretool configured.", rel),
            );
        } else if has_pre {
            res.push(
                format!(
                    "ATENCAO {} - referencia pretool mas binario nemesis-pretool-check-unix nao \
                     encontrado em nenhum layout (.nemesis/bin/ nem target/release/).",
                    rel
                ),
                format!(
                    "WARNING {} - references pretool but nemesis-pretool-check-unix binary not \
                     found in any layout (.nemesis/bin/ nor target/release/).",
                    rel
                ),
            );
        } else {
            res.push(
                format!("ATENCAO {} - sem referencia a pretool.", rel),
                format!("WARNING {} - no pretool reference.", rel),
            );
        }

        if !has_post {
            res.push(
                format!("NOTA  {} - sem referencia a posttool (scan pos-escrita inativo).", rel),
                format!("NOTE  {} - no posttool reference (post-write scan inactive).", rel),
            );
        }
    }

    res.push(
        "Por que importa: sem pretool no scaffold, a IDE nao dispara 'nemesis-defender --ensure-daemon' e o daemon nao sobe sozinho (no Linux so o eBPF protege).",
        "Why it matters: without pretool in the scaffold, the IDE does not trigger 'nemesis-defender --ensure-daemon' and the daemon does not start on its own (on Linux only eBPF protects).",
    );

    if !found_any {
        res.push(
            "Nenhum scaffold de IDE (.devin/.claude/.cursor/.codex/.github) com config encontrado.",
            "No IDE scaffold (.devin/.claude/.cursor/.codex/.github) with config found.",
        );
        res.status(CheckStatus::Fail)
    } else if any_valid_pretool {
        res.status(CheckStatus::Ok)
    } else {
        res.status(CheckStatus::Fail)
    }
}
