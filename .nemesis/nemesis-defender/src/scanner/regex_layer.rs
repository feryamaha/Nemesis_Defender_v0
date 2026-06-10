//! Regex fast-path scanner — pre-AST pattern matching
//!
//! Catches well-known attack patterns that don't require AST context:
//! - Credential file paths (~/.npmrc, ~/.ssh/, ~/.aws/)
//! - Known C2 infrastructure patterns
//! - Download-and-execute one-liners
//! - Self-deletion patterns

use crate::{DefenderViolation, Language};

/// Avisa (uma vez por pattern, no stderr) quando um regex falha em compilar, em vez de
/// pular em SILÊNCIO. A falha silenciosa (`Err(_) => continue`) escondia patterns mortos
/// — ex.: lookahead `(?!...)`, que o crate `regex` não suporta. Tornar a falha visível
/// garante que regra quebrada apareça em vez de virar proteção fantasma.
fn warn_invalid_pattern(source: &str, pattern: &str, err: &regex::Error) {
    use std::collections::HashSet;
    use std::sync::{Mutex, OnceLock};
    static WARNED: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();
    let set = WARNED.get_or_init(|| Mutex::new(HashSet::new()));
    if let Ok(mut guard) = set.lock() {
        if guard.insert(pattern.to_string()) {
            eprintln!(
                "[nemesis-defender] AVISO: regex invalido ignorado ({source}): {pattern:?} — {err}. \
                 O engine `regex` nao suporta lookahead/backreference; reescreva sem (?!...)/(?=...) \
                 ou implemente a verificacao em codigo. Esta regra esta INATIVA ate ser corrigida."
            );
        }
    }
}

struct RegexPattern {
    visitor: &'static str,
    pattern: &'static str,
    message: &'static str,
}

/// Patterns that apply to ALL languages
const UNIVERSAL_PATTERNS: &[RegexPattern] = &[
    RegexPattern {
        visitor: "credential_harvest",
        pattern: r#"(~/\.npmrc|~/\.pypirc|~/\.netrc|home.*\.npmrc)"#,
        message: "Access to npm/pypi credential file. Shai-Hulud 2.0 attack pattern — reads tokens for exfiltration.",
    },
    RegexPattern {
        visitor: "credential_harvest",
        pattern: r#"(~/\.ssh/|/root/\.ssh/|home.*\.ssh/)(id_rsa|id_ed25519|authorized_keys|known_hosts)"#,
        message: "Access to SSH private key. Supply chain credential theft pattern.",
    },
    RegexPattern {
        visitor: "credential_harvest",
        pattern: r#"(~/\.aws/credentials|AWS_SECRET_ACCESS_KEY|AWS_ACCESS_KEY_ID)"#,
        message: "Access to AWS credentials. Cloud credential exfiltration pattern (Shai-Hulud 2.0).",
    },
    RegexPattern {
        visitor: "credential_harvest",
        pattern: r#"(GITHUB_TOKEN|GH_TOKEN|NPM_TOKEN|PYPI_TOKEN|npm_config_[a-z_]*token)"#,
        message: "Access to VCS/registry token. Used by self-propagating supply chain worms.",
    },
    RegexPattern {
        visitor: "url_in_exec",
        pattern: r#"(https?://[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}[/:])"#,
        message: "Raw IP address in HTTP URL. Known C2 infrastructure pattern (e.g., ClawHub attack: 91.92.242.30).",
    },
    RegexPattern {
        visitor: "decode_exec",
        pattern: r#"(eval|exec|execSync|spawnSync)\s*\(\s*(Buffer\.from|atob|btoa)"#,
        message: "decode-then-exec pattern detected. Base64 decode result passed directly to code execution.",
    },
    RegexPattern {
        visitor: "decode_exec",
        pattern: r#"String\.fromCharCode\s*\(\s*[0-9]+\s*,"#,
        message: "String.fromCharCode array detected. Known obfuscation technique to reconstruct commands at runtime.",
    },
    RegexPattern {
        visitor: "decode_exec",
        pattern: r#"Buffer\.from\s*\(\s*['"][A-Za-z0-9+/=]+['"]\s*,\s*['"]base64['"]\s*\)\s*\.\s*toString"#,
        message: "Base64 string literal decoded to text via Buffer.from(...).toString(). Runtime obfuscation primitive — the decoded string is typically passed to eval/exec in a later statement.",
    },
    RegexPattern {
        visitor: "self_clean",
        pattern: r#"(fs\.unlink|fs\.unlinkSync|require\('fs'\)\.unlink).*__filename"#,
        message: "Self-deletion pattern: unlinks own file (__filename). Forensic evasion — malware deletes itself after execution.",
    },
    RegexPattern {
        visitor: "self_clean",
        pattern: r#"(rm\s+-f|unlink).*\$0"#,
        message: "Shell self-deletion: rm -f $0. Script deletes itself after execution to evade forensic analysis.",
    },
    RegexPattern {
        visitor: "url_in_exec",
        pattern: r#"(curl|wget)\s+.*\|\s*(bash|sh|zsh|python|node)"#,
        message: "Download-and-execute: curl/wget piped to shell interpreter. Classic one-liner malware delivery.",
    },
    RegexPattern {
        visitor: "prompt_injection",
        pattern: r#"(?i)(ignore\s+previous\s+instructions|disregard\s+(all|previous)|forget\s+your\s+(rules|instructions))"#,
        message: "Indirect prompt injection detected. Instruction to override AI agent safety rules.",
    },
    RegexPattern {
        visitor: "prompt_injection",
        pattern: r#"(?i)(<\|im_start\||<\|system\||<<SYS>>|\[INST\]|###\s*System:)"#,
        message: "AI instruction format token detected in source/skill file. Hidden system prompt injection.",
    },
    // ── Prompt injection / jailbreak (DAN, role-play override) ──
    RegexPattern {
        visitor: "prompt_injection",
        pattern: r#"(?i)\bDAN\b.{0,40}(?:Do\s+Anything\s+Now|mode)"#,
        message: "DAN jailbreak pattern detected. Attempts to force AI to operate without safety constraints.",
    },
    RegexPattern {
        visitor: "prompt_injection",
        pattern: r#"(?i)you\s+are\s+now\s+(?:unrestricted|free|god|DAN|evil|unfiltered)"#,
        message: "Role-override jailbreak detected. Attempts to replace AI identity to bypass restrictions.",
    },
    RegexPattern {
        visitor: "prompt_injection",
        pattern: r#"(?i)(?:bypass|ignore|disable)\s+(?:your\s+)?(?:safety|restrictions|filter|alignment|guidelines)"#,
        message: "Safety bypass directive detected in file content. Prompt injection attack.",
    },
    RegexPattern {
        visitor: "prompt_injection",
        pattern: r#"(?i)(?:do\.?anything\.?now|no\.?restrictions|no\.?limits|no\.?rules|no\.?boundaries)"#,
        message: "Unrestricted operation directive detected. AI jailbreak pattern.",
    },
    RegexPattern {
        visitor: "prompt_injection",
        pattern: r#"(?i)(?:jailbreak(?:ed|ing)?|developer\s+mode\s+enabled|maintenance\s+mode\s+on)"#,
        message: "Jailbreak activation phrase detected in file content.",
    },
    // ── Reverse shells (hardcoded — works without denylist) ──
    RegexPattern {
        visitor: "decode_exec",
        pattern: r#"socket\.socket\b.{0,60}\.connect\s*\("#,
        message: "Reverse shell pattern: socket.connect. Active exploitation technique — connect-back to attacker.",
    },
    RegexPattern {
        visitor: "decode_exec",
        pattern: r#"import\s+socket\s*;.{0,80}\.connect\s*\("#,
        message: "Python reverse shell pattern: import socket + connect. Remove immediately.",
    },
    RegexPattern {
        visitor: "decode_exec",
        pattern: r#"(?:/dev/tcp/|/dev/udp/|bash\s+-i\s+>&|nc\s+-e\s+/bin|socat\s+exec:)"#,
        message: "Reverse shell infrastructure pattern detected (bash /dev/tcp, netcat -e, socat). Active exploitation.",
    },
    // ── Supply chain: registry/index redirect ──
    // NOTA: a detecção de registry-redirect é feita pelo manifest_scanner, com ESCOPO
    // DE PATH e sem lookahead (scan_npmrc / scan_pypirc / scan_requirements_txt /
    // scan_ruby_gemfile). Os padrões que existiam aqui usavam lookahead `(?!...)`, que
    // o crate `regex` do Rust NÃO suporta — falhavam em Regex::new e eram pulados em
    // silêncio (proteção INERTE e redundante). Removidos. Fazê-los em regex_layer (sem
    // path) seria perigoso: casaria `"repository": "https://github.com"` em package.json
    // ou `const registry = "https://cdn"` em JS → falso-positivo destrutivo.
];

pub fn scan(content: &[u8], _lang: &Language) -> Vec<DefenderViolation> {
    let mut violations = Vec::new();

    let text = match std::str::from_utf8(content) {
        Ok(s) => s,
        Err(_) => return violations,
    };

    for pattern_def in UNIVERSAL_PATTERNS {
        // Build regex (compiled once per call — for Phase 3+ this will be cached)
        let re = match regex::Regex::new(pattern_def.pattern) {
            Ok(r) => r,
            Err(e) => {
                warn_invalid_pattern("regex_layer/builtin", pattern_def.pattern, &e);
                continue;
            }
        };

        for m in re.find_iter(text) {
            // Calculate line/col from byte offset
            let before = &text[..m.start()];
            let line = before.chars().filter(|&c| c == '\n').count() as u32 + 1;
            let last_newline = before.rfind('\n').map(|p| p + 1).unwrap_or(0);
            let col = (m.start() - last_newline) as u32 + 1;

            violations.push(DefenderViolation {
                visitor: pattern_def.visitor.to_string(),
                line,
                col,
                evidence: m.as_str().to_string(),
                decoded: None,
                message: pattern_def.message.to_string(),
                suggestion: None,
            });
        }
    }

    // ── Layer 3.5: External deny-list (denylist-defender.json) ──
    // Loads patterns from config file, applies on top of hardcoded patterns above
    for severity in &["malicious", "suspicious"] {
        for (category, pattern_str, description, suggestion) in
            super::denylist_loader::patterns_by_severity(severity)
        {
            let re = match regex::Regex::new(&pattern_str) {
                Ok(r) => r,
                Err(e) => {
                    warn_invalid_pattern("denylist-defender.json", &pattern_str, &e);
                    continue;
                }
            };

            for m in re.find_iter(text) {
                let before = &text[..m.start()];
                let line = before.chars().filter(|&c| c == '\n').count() as u32 + 1;
                let last_newline = before.rfind('\n').map(|p| p + 1).unwrap_or(0);
                let col = (m.start() - last_newline) as u32 + 1;

                let visitor = if *severity == "malicious" {
                    "denylist_malicious"
                } else {
                    "denylist_suspicious"
                };

                violations.push(DefenderViolation {
                    visitor: visitor.to_string(),
                    line,
                    col,
                    evidence: m.as_str().to_string(),
                    decoded: None,
                    message: format!(
                        "Hostile command detected (denylist-defender / {}): {}",
                        category, description
                    ),
                    suggestion: suggestion.clone(),
                });
            }
        }
    }

    violations
}
