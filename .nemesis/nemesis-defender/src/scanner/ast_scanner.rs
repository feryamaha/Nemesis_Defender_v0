//! AST scanner — tree-sitter semantic analysis
//!
//! Uses tree-sitter parsers to traverse the Concrete Syntax Tree (CST)
//! and detect malicious patterns that require AST context.
//!
//! Supported languages:
//! - JavaScript/TypeScript (tree-sitter-javascript)
//! - Bash/Shell (tree-sitter-bash)
//! - Python (tree-sitter-python)

use crate::visitors;
use crate::{DefenderViolation, Language};
use std::path::Path;

pub fn scan(path: &Path, content: &[u8], lang: &Language) -> Vec<DefenderViolation> {
    let mut all_violations = Vec::new();
    let file_path = path.to_string_lossy().to_string();

    match lang {
        Language::JavaScript | Language::TypeScript => {
            if let Some(violations) = scan_javascript(content, &file_path) {
                all_violations.extend(violations);
            }
        }
        Language::Bash => {
            if let Some(violations) = scan_bash(content, &file_path) {
                all_violations.extend(violations);
            }
        }
        Language::Python => {
            if let Some(violations) = scan_python(content, &file_path) {
                all_violations.extend(violations);
            }
        }
        _ => {}
    }

    all_violations
}

fn scan_javascript(content: &[u8], file_path: &str) -> Option<Vec<DefenderViolation>> {
    let text = std::str::from_utf8(content).ok()?;

    // Initialize tree-sitter JavaScript parser
    let mut parser = tree_sitter::Parser::new();
    let language = tree_sitter_javascript::LANGUAGE.into();
    parser.set_language(&language).ok()?;

    let tree = parser.parse(text, None)?;
    let root_node = tree.root_node();

    let mut violations = Vec::new();

    // Traverse the CST and call visitors
    traverse_javascript_node(&root_node, text, &mut violations, file_path);

    // Taint tracking: full-content pass (intra-file data-flow analysis)
    violations.extend(visitors::taint_tracker::scan_js_content(content));

    Some(violations)
}

fn traverse_javascript_node(
    node: &tree_sitter::Node,
    source: &str,
    violations: &mut Vec<DefenderViolation>,
    file_path: &str,
) {
    // Call visitors for this node
    violations.extend(visitors::decode_exec::visit_js_node(node, source));
    violations.extend(visitors::dynamic_cmd::visit_js_node(node, source));
    violations.extend(visitors::url_in_exec::visit_js_node(node, source));
    violations.extend(visitors::unicode_steg::visit_js_node(node, source));
    violations.extend(visitors::prompt_injection::visit_js_node(node, source));
    violations.extend(visitors::credential_harvest::visit_js_node(node, source, file_path));
    violations.extend(visitors::time_gated::visit_js_node(node, source));
    violations.extend(visitors::self_clean::visit_js_node(node, source));
    violations.extend(visitors::persistence_patterns::visit_js_node(node, source));
    violations.extend(visitors::nemesis_bypass::visit_js_node(node, source));

    // Recursively traverse children
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        traverse_javascript_node(&child, source, violations, file_path);
    }
}

fn scan_bash(content: &[u8], file_path: &str) -> Option<Vec<DefenderViolation>> {
    let text = std::str::from_utf8(content).ok()?;

    let mut parser = tree_sitter::Parser::new();
    let language = tree_sitter_bash::language();
    parser.set_language(&language).ok()?;

    let tree = parser.parse(text, None)?;
    let root_node = tree.root_node();

    let mut violations = Vec::new();

    traverse_bash_node(&root_node, text, &mut violations, file_path);

    Some(violations)
}

fn traverse_bash_node(
    node: &tree_sitter::Node,
    source: &str,
    violations: &mut Vec<DefenderViolation>,
    file_path: &str,
) {
    violations.extend(visitors::decode_exec::visit_bash_node(node, source));
    violations.extend(visitors::dynamic_cmd::visit_bash_node(node, source));
    violations.extend(visitors::url_in_exec::visit_bash_node(node, source));
    violations.extend(visitors::credential_harvest::visit_bash_node(node, source, file_path));
    violations.extend(visitors::self_clean::visit_bash_node(node, source));
    violations.extend(visitors::persistence_patterns::visit_bash_node(
        node, source,
    ));
    violations.extend(visitors::time_gated::visit_bash_node(node, source));
    violations.extend(visitors::nemesis_bypass::visit_bash_node(node, source));

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        traverse_bash_node(&child, source, violations, file_path);
    }
}

fn scan_python(content: &[u8], file_path: &str) -> Option<Vec<DefenderViolation>> {
    let text = std::str::from_utf8(content).ok()?;

    let mut parser = tree_sitter::Parser::new();
    let language = tree_sitter_python::language();
    parser.set_language(&language).ok()?;

    let tree = parser.parse(text, None)?;
    let root_node = tree.root_node();

    let mut violations = Vec::new();

    traverse_python_node(&root_node, text, &mut violations, file_path);

    // Taint tracking: full-content pass (intra-file data-flow analysis)
    violations.extend(visitors::taint_tracker::scan_py_content(content));

    Some(violations)
}

fn traverse_python_node(
    node: &tree_sitter::Node,
    source: &str,
    violations: &mut Vec<DefenderViolation>,
    file_path: &str,
) {
    violations.extend(visitors::decode_exec::visit_python_node(node, source));
    violations.extend(visitors::dynamic_cmd::visit_python_node(node, source));
    violations.extend(visitors::url_in_exec::visit_python_node(node, source));
    violations.extend(visitors::credential_harvest::visit_python_node(
        node, source, file_path,
    ));
    violations.extend(visitors::self_clean::visit_python_node(node, source));
    violations.extend(visitors::persistence_patterns::visit_python_node(
        node, source,
    ));
    violations.extend(visitors::time_gated::visit_python_node(node, source));
    violations.extend(visitors::nemesis_bypass::visit_python_node(node, source));
    violations.extend(visitors::python_import_injection::visit_python_node(
        node, source,
    ));

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        traverse_python_node(&child, source, violations, file_path);
    }
}
