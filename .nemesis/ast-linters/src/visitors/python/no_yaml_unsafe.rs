use crate::parser::ParsedTree;
use crate::lint_rule::{Violation, RuleCategory, Severity};

/// Detect yaml.load() without SafeLoader
/// yaml.load() with unsafe loader can execute arbitrary code
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

    if node.kind() == "call" {
        check_yaml_unsafe(&node, source, violations);
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

fn check_yaml_unsafe(
    node: &tree_sitter::Node,
    source: &str,
    violations: &mut Vec<Violation>,
) {
    let text = &source[node.byte_range()];

    // Detect yaml.load() without SafeLoader
    if text.contains("yaml.load") && !text.contains("SafeLoader") && !text.contains("safe_load") {
        let line = node.start_position().row + 1;
        violations.push(
            Violation::new(
                "yaml.load() sem SafeLoader permite execução de código arbitrário. Use yaml.safe_load().",
                line,
                RuleCategory::Security,
            )
            .with_rule_name("python-no-yaml-unsafe")
            .with_severity(Severity::Error),
        );
    }
}
