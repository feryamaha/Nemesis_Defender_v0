use crate::parser::ParsedTree;
use crate::lint_rule::{Violation, RuleCategory, Severity};

/// Detect mutable default arguments in function definitions
/// Pattern: def func(x=[]) or def func(x={})
/// Dangerous because the default is created once and shared across calls
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

    if node.kind() == "function_definition" {
        check_mutable_defaults(&node, source, violations);
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

fn check_mutable_defaults(
    node: &tree_sitter::Node,
    source: &str,
    violations: &mut Vec<Violation>,
) {
    let text = &source[node.byte_range()];

    // Check for mutable default patterns: =[] or ={}
    // Must be careful to check parameters, not function body
    if text.contains("def ") && (text.contains("=[]") || text.contains("={}")) {
        // Simple heuristic: if =[] or ={} appears before the first colon (function body)
        if let Some(colon_pos) = text.find(':') {
            let params = &text[..colon_pos];
            if params.contains("=[]") || params.contains("={}") {
                let line = node.start_position().row + 1;
                violations.push(
                    Violation::new(
                        "Mutable default argument. Criado uma vez na definição, compartilhado entre chamadas. Use None como padrão.",
                        line,
                        RuleCategory::Correctness,
                    )
                    .with_rule_name("python-no-mutable-default")
                    .with_severity(Severity::Warning),
                );
            }
        }
    }
}
