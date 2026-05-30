use crate::parser::ParsedTree;
use crate::lint_rule::{Violation, RuleCategory, Severity};

/// Detect SQL queries constructed with string interpolation or concatenation
/// Patterns: f"SELECT...{var}" or "SELECT " + var
pub fn visit(tree: &ParsedTree, source: &str) -> Vec<Violation> {
    let mut violations = Vec::new();
    let cursor = &mut tree.tree.walk();
    visit_node(cursor, source, &mut violations);
    violations
}

fn visit_node(
    cursor: &mut tree_sitter::TreeCursor,
    source: &str,
    violations: &mut Vec<Violation>,
) {
    let node = cursor.node();
    let text = &source[node.byte_range()];

    // Check for SQL keywords with interpolation patterns
    let is_sql = text.contains("SELECT")
        || text.contains("INSERT")
        || text.contains("UPDATE")
        || text.contains("DELETE");

    if is_sql {
        // Check for f-string pattern: f"... {var} ..."
        if text.starts_with("f\"") || text.starts_with("f'") {
            if text.contains("{") && text.contains("}") {
                let line = node.start_position().row + 1;
                violations.push(
                    Violation::new(
                        "SQL construída com f-string interpolation. Use parametrized queries.",
                        line,
                        RuleCategory::Security,
                    )
                    .with_rule_name("python-sql-injection")
                    .with_severity(Severity::Error),
                );
                if cursor.goto_first_child() {
                    loop {
                        visit_node(cursor, source, violations);
                        if !cursor.goto_next_sibling() {
                            break;
                        }
                    }
                    cursor.goto_parent();
                }
                return;
            }
        }

        // Check for concatenation pattern: "SELECT " + var
        if text.contains("+") && (text.contains("\"") || text.contains("'")) {
            let line = node.start_position().row + 1;
            violations.push(
                Violation::new(
                    "SQL construída com string concatenation. Use parametrized queries.",
                    line,
                    RuleCategory::Security,
                )
                .with_rule_name("python-sql-injection")
                .with_severity(Severity::Error),
            );
            if cursor.goto_first_child() {
                loop {
                    visit_node(cursor, source, violations);
                    if !cursor.goto_next_sibling() {
                        break;
                    }
                }
                cursor.goto_parent();
            }
            return;
        }
    }

    if cursor.goto_first_child() {
        loop {
            visit_node(cursor, source, violations);
            if !cursor.goto_next_sibling() {
                break;
            }
        }
        cursor.goto_parent();
    }
}

