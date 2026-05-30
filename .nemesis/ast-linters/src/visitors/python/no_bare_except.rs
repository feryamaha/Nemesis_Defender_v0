use crate::parser::ParsedTree;
use crate::lint_rule::{Violation, RuleCategory, Severity};

/// Detect bare except: without specific exception type
/// Bare except catches all exceptions including SystemExit, KeyboardInterrupt
pub fn visit(tree: &ParsedTree, source: &str) -> Vec<Violation> {
    let mut violations = Vec::new();

    // Simple pattern matching: search source for "except:" without exception type
    // Bare except: is "except:" immediately followed by ":" with no type name in between
    let lines: Vec<&str> = source.split('\n').collect();
    for (idx, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        // Match bare except: pattern
        // except: (bare, no exception type)
        // except Exception: (typed, OK)
        // except (ValueError, TypeError): (typed, OK)
        if trimmed.starts_with("except:") {
            violations.push(
                Violation::new(
                    "Bare except: captura todas as exceções (até SystemExit, KeyboardInterrupt). Especifique o tipo.",
                    idx + 1,
                    RuleCategory::Correctness,
                )
                .with_rule_name("python-no-bare-except")
                .with_severity(Severity::Error),
            );
        }
    }

    violations
}

