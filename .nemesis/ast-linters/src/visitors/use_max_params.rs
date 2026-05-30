use crate::parser::ParsedTree;
use crate::lint_rule::{Violation, RuleCategory, Severity};

const MAX_PARAMS: usize = 4;

pub fn visit(tree: &ParsedTree, source: &str) -> Vec<Violation> {
    let mut violations = Vec::new();
    let cursor = &mut tree.tree.walk();
    visit_node(cursor, source, &mut violations);
    violations
}

fn visit_node(cursor: &mut tree_sitter::TreeCursor, source: &str, violations: &mut Vec<Violation>) {
    let node = cursor.node();
    if node.kind() == "formal_parameters" {
        let count = count_params(&node);
        if count > MAX_PARAMS {
            let line = node.start_position().row + 1;
            violations.push(Violation::new(
                format!("Funcao com {} parametros. Maximo recomendado: {}.", count, MAX_PARAMS),
                line, RuleCategory::Suspicious,
            ).with_suggestion("Agrupe parametros em objeto de opcoes.").with_severity(Severity::Warning));
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

fn count_params(node: &tree_sitter::Node) -> usize {
    let mut cursor = node.walk();
    let children: Vec<_> = node.children(&mut cursor).collect();
    children.iter().filter(|c| {
        matches!(c.kind(), "required_parameter" | "optional_parameter")
    }).count()
}
