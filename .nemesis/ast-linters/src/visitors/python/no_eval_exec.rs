use crate::parser::ParsedTree;
use crate::lint_rule::{Violation, RuleCategory, Severity};

/// Detect eval() and exec() calls - both execute arbitrary code
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
        check_dangerous_call(&node, source, violations);
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

fn check_dangerous_call(
    node: &tree_sitter::Node,
    source: &str,
    violations: &mut Vec<Violation>,
) {
    let mut cursor = node.walk();
    let children: Vec<_> = node.children(&mut cursor).collect();

    if let Some(first_child) = children.first() {
        if first_child.kind() == "identifier" {
            let func_name = &source[first_child.byte_range()];

            if func_name == "eval" || func_name == "exec" {
                let line = node.start_position().row + 1;

                violations.push(
                    Violation::new(
                        format!(
                            "{}() executa código arbitrário. Use ast.literal_eval() para JSON ou implemente parser seguro.",
                            func_name
                        ),
                        line,
                        RuleCategory::Security,
                    )
                    .with_rule_name("python-no-eval-exec")
                    .with_severity(Severity::Error),
                );
            }
        }
    }
}
