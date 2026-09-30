use super::expr::NixExpr;
use std::collections::BTreeMap;

const STEP: usize = 2;

pub fn render(expr: &NixExpr) -> String {
    render_at(expr, 0)
}

/// Render with explicit indentation depth. Used by NixModule to start values
/// at the correct nesting level.
pub(crate) fn render_at(expr: &NixExpr, depth: usize) -> String {
    match expr {
        NixExpr::Null => "null".to_owned(),
        NixExpr::Bool(b) => b.to_string(),
        NixExpr::Int(i) => i.to_string(),
        NixExpr::Float(f) => format!("{f}"),
        NixExpr::Str(s) => escape_str(s),
        NixExpr::Path(p) => p.clone(),
        NixExpr::List(items) => render_list(items, depth),
        NixExpr::Attrs(map) => render_attrs(map, depth),
    }
}

fn render_list(items: &[NixExpr], depth: usize) -> String {
    if items.is_empty() {
        return "[ ]".to_owned();
    }
    let inner = " ".repeat((depth + 1) * STEP);
    let close = " ".repeat(depth * STEP);
    let body = items
        .iter()
        .map(|it| format!("{inner}{}", render_at(it, depth + 1)))
        .collect::<Vec<_>>()
        .join("\n");
    format!("[\n{body}\n{close}]")
}

fn render_attrs(map: &BTreeMap<String, NixExpr>, depth: usize) -> String {
    if map.is_empty() {
        return "{ }".to_owned();
    }
    let inner = " ".repeat((depth + 1) * STEP);
    let close = " ".repeat(depth * STEP);
    let body = map
        .iter()
        .map(|(k, v)| format!("{inner}{} = {};", render_key(k), render_at(v, depth + 1)))
        .collect::<Vec<_>>()
        .join("\n");
    format!("{{\n{body}\n{close}}}")
}

fn render_key(k: &str) -> String {
    let valid = !k.is_empty()
        && k.chars()
            .next()
            .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
        && k.chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '\''));
    if valid { k.to_owned() } else { escape_str(k) }
}

pub(crate) fn escape_str(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '$' if chars.peek() == Some(&'{') => out.push_str("\\$"),
            _ => out.push(c),
        }
    }
    out.push('"');
    out
}
