use crate::parser::ParsedTree;
use crate::lint_rule::{Violation, RuleCategory, Severity};

const MAX_COMPLEXITY: u32 = 15;

pub fn visit(tree: &ParsedTree, source: &str) -> Vec<Violation> {
    let mut violations = Vec::new();
    let cursor = &mut tree.tree.walk();
    visit_node(cursor, source, &mut violations);
    violations
}

fn visit_node(cursor: &mut tree_sitter::TreeCursor, source: &str, violations: &mut Vec<Violation>) {
    let node = cursor.node();
    if node.kind() == "function_declaration" || node.kind() == "function_expression"
        || node.kind() == "arrow_function" || node.kind() == "method_definition"
    {
        let score = compute_complexity(&node, source, 0);
        if score > MAX_COMPLEXITY {
            let line = node.start_position().row + 1;
            violations.push(Violation::new(
                format!("Complexidade cognitiva excessiva ({}). Maximo recomendado: {}.", score, MAX_COMPLEXITY),
                line, RuleCategory::Suspicious,
            ).with_suggestion("[STOP] Leia .windsurf/rules/typescript-typing-convention.md antes de reescrever. Extraia funcoes menores para reduzir complexidade. Consulte: https://biomejs.dev/").with_severity(Severity::Warning));
        }
    }
    if cursor.goto_first_child() {
        loop {
            visit_node(cursor, source, violations);
            if !cursor.goto_next_sibling() { break; }
        }
        cursor.goto_parent();
    }
}

fn compute_complexity(node: &tree_sitter::Node, source: &str, nesting: u32) -> u32 {
    let mut score: u32 = 0;
    let nesting_bonus = if nesting > 0 { nesting } else { 1 };
    let mut cursor = node.walk();
    let children: Vec<_> = node.children(&mut cursor).collect();
    for child in &children {
        match child.kind() {
            "if_statement" | "else_clause" | "switch_statement"
            | "for_statement" | "for_in_statement" | "while_statement"
            | "do_statement" | "catch_clause" => {
                score += 1 * nesting_bonus;
                score += compute_complexity(child, source, nesting + 1);
            }
            "ternary_expression" => {
                score += 1 * nesting_bonus;
            }
            "binary_expression" => {
                let text = &source[child.byte_range()];
                if text.contains("&&") || text.contains("||") {
                    score += 1 * nesting_bonus;
                }
            }
            _ => {
                score += compute_complexity(child, source, nesting);
            }
        }
    }
    score
}
