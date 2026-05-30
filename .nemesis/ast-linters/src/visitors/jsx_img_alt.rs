use crate::parser::ParsedTree;
use crate::lint_rule::{Violation, RuleCategory, Severity};

pub fn visit(tree: &ParsedTree, source: &str) -> Vec<Violation> {
    let mut violations = Vec::new();
    let cursor = &mut tree.tree.walk();
    visit_node(cursor, source, &mut violations);
    violations
}

fn visit_node(cursor: &mut tree_sitter::TreeCursor, source: &str, violations: &mut Vec<Violation>) {
    let node = cursor.node();
    if node.kind() == "jsx_opening_element" || node.kind() == "jsx_self_closing_element" {
        check_img_element(&node, source, violations);
    }
    if cursor.goto_first_child() {
        loop {
            visit_node(cursor, source, violations);
            if !cursor.goto_next_sibling() { break; }
        }
        cursor.goto_parent();
    }
}

fn check_img_element(node: &tree_sitter::Node, source: &str, violations: &mut Vec<Violation>) {
    let mut cursor = node.walk();
    let children: Vec<_> = node.children(&mut cursor).collect();
    let mut is_img = false;
    let mut has_alt = false;
    for child in &children {
        if (child.kind() == "identifier" || child.kind() == "jsx_identifier")
            && &source[child.byte_range()] == "img"
        {
            is_img = true;
        }
        if child.kind() == "jsx_attribute" {
            let mut ac = child.walk();
            let acs: Vec<_> = child.children(&mut ac).collect();
            for a in &acs {
                if a.kind() == "property_identifier" || a.kind() == "jsx_identifier" {
                    if &source[a.byte_range()] == "alt" {
                        has_alt = true;
                    }
                }
            }
        }
    }
    if is_img && !has_alt {
        let line = node.start_position().row + 1;
        violations.push(Violation::new(
            "Elemento <img> sem atributo alt. Atributo alt eh essencial para acessibilidade.",
            line, RuleCategory::Correctness,
        ).with_suggestion("[STOP] Leia .windsurf/rules/react-hooks-patterns-rules.md antes de reescrever. Adicione alt descritivo: <img alt=\"Descricao\" />. Consulte: https://biomejs.dev/").with_severity(Severity::Warning));
    }
}
