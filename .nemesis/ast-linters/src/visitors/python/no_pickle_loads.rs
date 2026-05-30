use crate::parser::ParsedTree;
use crate::lint_rule::{Violation, RuleCategory, Severity};

/// Detect pickle.loads() from untrusted sources
/// Pickle deserialization can execute arbitrary code (RCE risk)
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
        check_pickle_loads(&node, source, violations);
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

fn check_pickle_loads(
    node: &tree_sitter::Node,
    source: &str,
    violations: &mut Vec<Violation>,
) {
    let text = &source[node.byte_range()];

    if text.contains("pickle.loads") {
        let line = node.start_position().row + 1;
        violations.push(
            Violation::new(
                "pickle.loads() deserializa código arbitrário (RCE risk). Use json.loads() ou considere outro formato seguro.",
                line,
                RuleCategory::Security,
            )
            .with_rule_name("python-no-pickle-loads")
            .with_severity(Severity::Error),
        );
    }
}
