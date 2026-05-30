use crate::parser::ParsedTree;
use crate::lint_rule::{Violation, RuleCategory, Severity};

/// Detect subprocess.run(shell=True) or subprocess.Popen(shell=True)
/// shell=True allows shell injection attacks
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
        check_subprocess_shell(&node, source, violations);
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

fn check_subprocess_shell(
    node: &tree_sitter::Node,
    source: &str,
    violations: &mut Vec<Violation>,
) {
    let full_text = &source[node.byte_range()];

    if (full_text.contains("subprocess.run") || full_text.contains("subprocess.Popen"))
        && full_text.contains("shell=True")
    {
        let line = node.start_position().row + 1;

        violations.push(
            Violation::new(
                "subprocess.run(shell=True) permite shell injection. Use shell=False.",
                line,
                RuleCategory::Security,
            )
            .with_rule_name("python-no-shell-true")
            .with_severity(Severity::Error),
        );
    }
}
