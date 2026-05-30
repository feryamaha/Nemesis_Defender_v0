//! Manifest scanner — structured file analysis
//!
//! Analyzes package manifests for supply chain attack vectors:
//! - package.json: postinstall / preinstall / prepare script abuse (Vetor 1)
//! - Cargo.toml: build.rs + custom build scripts
//! - pyproject.toml: setup hooks
//!
//! Reference attacks:
//! - Shai-Hulud 2.0 (Nov 2025): moved from postinstall → preinstall for wider blast radius
//! - axios compromise: postinstall dropped cross-platform RAT
//! - Self-cleaning: script deletes itself + rewrites package.json with clean version

use std::path::Path;
use crate::DefenderViolation;

/// Non-trivial command indicators in install scripts
/// If these appear in lifecycle scripts → flag as suspicious/malicious
const EXEC_INDICATORS: &[&str] = &[
    "curl", "wget", "fetch", "http", "https",
    "exec", "eval", "spawn", "fork",
    "base64", "atob", "Buffer.from",
    "require(", "import(",
    "rm ", "unlink", "rmdir",
    "chmod", "chown",
    "sudo",
    "python", "python3", "ruby", "perl",
    "bash -c", "sh -c",
    "process.env",
    "os.homedir", "homedir",
    "readFile", "readFileSync",
    "writeFile", "writeFileSync",
];

pub fn scan(path: &Path, content: &[u8]) -> Vec<DefenderViolation> {
    let filename = path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");

    match filename {
        "package.json" => scan_package_json(content),
        "Cargo.toml"   => scan_cargo_toml(content),
        "pyproject.toml" | "setup.py" | "setup.cfg" => scan_python_manifest(content),
        "requirements.txt" | "constraints.txt" | "requirements-dev.txt" => scan_requirements_txt(content),
        "build.rs" => scan_build_rs(content),
        "Gemfile" => scan_ruby_gemfile(content),
        "composer.json" => scan_php_composer(content),
        _ => vec![],
    }
}

fn scan_package_json(content: &[u8]) -> Vec<DefenderViolation> {
    let mut violations = Vec::new();

    let text = match std::str::from_utf8(content) {
        Ok(s) => s,
        Err(_) => return violations,
    };

    let json: serde_json::Value = match serde_json::from_str(text) {
        Ok(v) => v,
        Err(_) => return violations, // Malformed JSON — regex_layer will catch patterns
    };

    // Check lifecycle scripts
    let lifecycle_hooks = &["preinstall", "install", "postinstall", "prepare", "prepublish"];

    if let Some(scripts) = json.get("scripts").and_then(|s| s.as_object()) {
        for hook in lifecycle_hooks {
            if let Some(cmd) = scripts.get(*hook).and_then(|v| v.as_str()) {
                let cmd_lower = cmd.to_lowercase();

                let found_indicators: Vec<&str> = EXEC_INDICATORS
                    .iter()
                    .filter(|&&ind| cmd_lower.contains(ind))
                    .copied()
                    .collect();

                if !found_indicators.is_empty() {
                    violations.push(DefenderViolation {
                        visitor: "manifest_postinstall_exec".to_string().to_string(),
                        line: 0, // JSON doesn't give us line numbers easily
                        col: 0,
                        evidence: format!("scripts.{}: \"{}\"", hook, cmd),
                        decoded: None,
                        message: format!(
                            "Lifecycle script '{}' contains execution indicators: [{}]. \
                             Shai-Hulud 2.0 pattern: preinstall runs even when install fails. \
                             axios compromise pattern: postinstall dropped cross-platform RAT.",
                            hook,
                            found_indicators.join(", ")
                        ),
                        suggestion: Some("Remove execution commands from lifecycle scripts. Use CI/CD for builds and installations.".to_string()),
                    });
                }
            }
        }
    }

    // Check for suspiciously minimal structure (malware minimizes file count)
    // Low file count + no repository + install scripts = high risk
    let has_repository = json.get("repository").is_some();
    let has_scripts_with_hooks = json.get("scripts")
        .and_then(|s| s.as_object())
        .map(|s| lifecycle_hooks.iter().any(|h| s.contains_key(*h)))
        .unwrap_or(false);

    if has_scripts_with_hooks && !has_repository {
        violations.push(DefenderViolation {
            visitor: "manifest_postinstall_exec".to_string(),
            line: 0,
            col: 0,
            evidence: "package.json has lifecycle hooks but no repository field".to_string(),
            decoded: None,
            message: "Package has lifecycle install scripts but no linked repository. \
                     FortiGuard Q2 2025: malware authors minimize file count and omit \
                     repository links to reduce traceability.".to_string(),
            suggestion: Some("Add the 'repository' field to package.json and remove suspicious lifecycle scripts.".to_string()),
        });
    }

    violations
}

fn scan_cargo_toml(content: &[u8]) -> Vec<DefenderViolation> {
    let mut violations = Vec::new();

    let text = match std::str::from_utf8(content) {
        Ok(s) => s,
        Err(_) => return violations,
    };

    // Check for build.rs reference with network indicators in build script name
    if text.contains("build = ") && text.contains("build.rs") {
        // build.rs is legitimate but flag for manual review if combined with network deps
        let has_network_deps = text.contains("reqwest")
            || text.contains("hyper")
            || text.contains("ureq")
            || text.contains("curl-sys");

        if has_network_deps {
            violations.push(DefenderViolation {
                visitor: "manifest_postinstall_exec".to_string(),
                line: 0,
                col: 0,
                evidence: "Cargo.toml: build.rs + network dependency".to_string(),
                decoded: None,
                message: "Cargo.toml has build.rs + HTTP client dependency. \
                         Build scripts execute at compile time with full system access. \
                         Verify build.rs does not make outbound connections.".to_string(),
                suggestion: Some("Review build.rs manually. Build scripts must not make network connections at compile time.".to_string()),
            });
        }
    }

    // Check for suspicious [build-dependencies]
    // TrapDoor attack: malware disguised as build-only packages
    let build_deps_start = text.find("[build-dependencies]");
    if let Some(start) = build_deps_start {
        let build_section = &text[start..];
        let section_end = build_section.find("\n[")
            .map(|i| start + i)
            .unwrap_or(text.len());

        let build_deps_content = &text[start..section_end];

        // Check for unknown/suspicious build-dependencies
        let suspicious_build_deps = &[
            "fake-reqwest",
            "build-utils-evil",
            "compile-time-exec",
        ];

        for dep in suspicious_build_deps {
            if build_deps_content.contains(dep) {
                violations.push(DefenderViolation {
                    visitor: "manifest_supply_chain".to_string(),
                    line: 0,
                    col: 0,
                    evidence: format!("Cargo.toml: [build-dependencies] contains {}", dep),
                    decoded: None,
                    message: format!(
                        "Suspicious build-dependency detected: {}. \
                         Build-time dependencies execute arbitrary code during compilation.",
                        dep
                    ),
                    suggestion: Some("Verify build-dependency authenticity. Use 'cargo tree' to inspect transitive dependencies.".to_string()),
                });
            }
        }
    }

    violations
}

fn scan_python_manifest(content: &[u8]) -> Vec<DefenderViolation> {
    let mut violations = Vec::new();

    let text = match std::str::from_utf8(content) {
        Ok(s) => s,
        Err(_) => return violations,
    };

    // Check for cmdclass with install hooks
    if text.contains("cmdclass") && (text.contains("install") || text.contains("build")) {
        violations.push(DefenderViolation {
            visitor: "manifest_postinstall_exec".to_string(),
            line: 0,
            col: 0,
            evidence: "setup.py: cmdclass with install/build hook".to_string(),
            decoded: None,
            message: "Python setup.py defines custom install/build commands via cmdclass. \
                     These run during pip install with user privileges. \
                     Verify the custom command does not fetch or execute remote code.".to_string(),
            suggestion: Some("Review the cmdclass manually. Install hooks must not download or execute remote code.".to_string()),
        });
    }

    violations
}

fn scan_requirements_txt(content: &[u8]) -> Vec<DefenderViolation> {
    let mut violations = Vec::new();

    let text = match std::str::from_utf8(content) {
        Ok(s) => s,
        Err(_) => return violations,
    };

    // TrapDoor attack (May 2026): Check for suspicious package patterns in requirements
    let suspicious_packages = &[
        "tzlocal",      // TrapDoor used variant of tzlocal
        "requests-fork", // Fake fork of requests library
        "cryptography-backup", // Fake backup of cryptography
        "setuptools-fake", // Fake setuptools variant
    ];

    let suspicious_urls = &[
        "attacker.com",
        "evil.com",
        "127.0.0.1",
        "localhost",
        "malware",
        "phishing",
    ];

    let mut line_no = 1u32;
    for line in text.lines() {
        let trimmed = line.trim();

        // Skip comments and empty lines
        if trimmed.is_empty() || trimmed.starts_with('#') {
            line_no += 1;
            continue;
        }

        // Check for suspicious URLs in any directive (--extra-index-url, http/https links, etc)
        let line_lower = trimmed.to_lowercase();
        if (trimmed.starts_with("--extra-index-url") || trimmed.starts_with("--index-url") ||
            trimmed.contains("http://") || trimmed.contains("https://")) {
            for susp_url in suspicious_urls {
                if line_lower.contains(susp_url) {
                    violations.push(DefenderViolation {
                        visitor: "manifest_supply_chain".to_string(),
                        line: line_no,
                        col: 1,
                        evidence: format!("requirements.txt:{}: {}", line_no, trimmed),
                        decoded: None,
                        message: format!(
                            "Suspicious PyPI index URL detected: {}. \
                             TrapDoor attack (May 2026): custom PyPI index hijacking.",
                            trimmed
                        ),
                        suggestion: Some("Use only official PyPI (https://pypi.org/simple). Remove custom index URLs.".to_string()),
                    });
                    break;
                }
            }
        }

        // Extract package name (before ==, >=, etc.)
        let pkg_name = trimmed
            .split(|c: char| c == '=' || c == '>' || c == '<' || c == '!' || c == '~')
            .next()
            .unwrap_or("")
            .trim();

        // Check against suspicious packages
        for susp in suspicious_packages {
            if pkg_name.to_lowercase().contains(susp) {
                violations.push(DefenderViolation {
                    visitor: "manifest_supply_chain".to_string(),
                    line: line_no,
                    col: 1,
                    evidence: format!("requirements.txt:{}: {}", line_no, trimmed),
                    decoded: None,
                    message: format!(
                        "Suspicious package name detected: '{}'. \
                         TrapDoor attack (May 2026): malware disguised as legitimate packages.",
                        pkg_name
                    ),
                    suggestion: Some("Verify package authenticity in PyPI. Use pip index search or inspect source repository.".to_string()),
                });
                break;
            }
        }

        line_no += 1;
    }

    violations
}

fn scan_build_rs(content: &[u8]) -> Vec<DefenderViolation> {
    let mut violations = Vec::new();

    let text = match std::str::from_utf8(content) {
        Ok(s) => s,
        Err(_) => return violations,
    };

    // Check for Command::new() with suspicious commands
    let dangerous_commands = &["curl", "wget", "bash", "sh", "python", "perl", "ruby"];

    for cmd in dangerous_commands {
        let pattern = format!("Command::new(\"{}\")", cmd);
        if text.contains(&pattern) {
            violations.push(DefenderViolation {
                visitor: "manifest_build_exec".to_string(),
                line: 0,
                col: 0,
                evidence: format!("build.rs: Command::new(\"{}\")", cmd),
                decoded: None,
                message: format!(
                    "build.rs spawns dangerous command: {}. \
                     Build scripts execute at compile time with full system access. \
                     Verify no remote code is downloaded or executed.",
                    cmd
                ),
                suggestion: Some("Review build.rs. Avoid executing shell commands during build. Use build-dependencies instead of runtime execution.".to_string()),
            });
        }
    }

    // Check for downloading remote code in build script
    if text.contains("http://") || text.contains("https://") {
        if text.contains("Command::new") || text.contains("std::process") {
            violations.push(DefenderViolation {
                visitor: "manifest_build_exec".to_string(),
                line: 0,
                col: 0,
                evidence: "build.rs: network URL + process execution".to_string(),
                decoded: None,
                message: "build.rs contains both network URLs and process execution. \
                         Potential remote code download at compile time.".to_string(),
                suggestion: Some("Remove network operations from build.rs. Use cargo features or build-dependencies instead.".to_string()),
            });
        }
    }

    violations
}

fn scan_ruby_gemfile(content: &[u8]) -> Vec<DefenderViolation> {
    let mut violations = Vec::new();

    let text = match std::str::from_utf8(content) {
        Ok(s) => s,
        Err(_) => return violations,
    };

    let mut line_no = 1u32;
    for line in text.lines() {
        let trimmed = line.trim();

        // Check for suspicious source declarations
        if trimmed.starts_with("source ") && trimmed.contains("http") {
            let is_suspicious = trimmed.contains("attacker")
                || trimmed.contains("evil.com")
                || trimmed.contains("127.0.0.1")
                || trimmed.contains("localhost")
                || (!trimmed.contains("rubygems.org") && !trimmed.contains("ruby-gems.org"));

            if is_suspicious {
                violations.push(DefenderViolation {
                    visitor: "manifest_supply_chain".to_string(),
                    line: line_no,
                    col: 1,
                    evidence: format!("Gemfile:{}: {}", line_no, trimmed),
                    decoded: None,
                    message: format!(
                        "Suspicious gem source detected: {}. \
                         Gem server hijacking allows supply chain compromise.",
                        trimmed
                    ),
                    suggestion: Some("Only use official rubygems.org. Verify custom sources are trusted.".to_string()),
                });
            }
        }

        line_no += 1;
    }

    violations
}

fn scan_php_composer(content: &[u8]) -> Vec<DefenderViolation> {
    let mut violations = Vec::new();

    let text = match std::str::from_utf8(content) {
        Ok(s) => s,
        Err(_) => return violations,
    };

    // Parse as JSON
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(text) {
        // Check for post-install-cmd scripts
        if let Some(scripts) = json.get("scripts").and_then(|s| s.as_object()) {
            let dangerous_hooks = &["post-install-cmd", "post-update-cmd", "post-package-install"];

            for hook in dangerous_hooks {
                if let Some(cmd) = scripts.get(*hook).and_then(|v| v.as_str()) {
                    let cmd_lower = cmd.to_lowercase();

                    // Check for execution indicators
                    if cmd_lower.contains("curl")
                        || cmd_lower.contains("wget")
                        || cmd_lower.contains("exec")
                        || cmd_lower.contains("eval")
                        || cmd_lower.contains("shell_exec")
                        || cmd_lower.contains("passthru")
                    {
                        violations.push(DefenderViolation {
                            visitor: "manifest_postinstall_exec".to_string(),
                            line: 0,
                            col: 0,
                            evidence: format!("composer.json: {}: \"{}\"", hook, cmd),
                            decoded: None,
                            message: format!(
                                "PHP composer.json {} script executes code: {}. \
                                 Runs during composer install/update with user privileges.",
                                hook, cmd
                            ),
                            suggestion: Some("Remove execution commands from composer hooks. Use post-install scripts in src/ directory instead.".to_string()),
                        });
                    }
                }
            }
        }
    }

    violations
}
