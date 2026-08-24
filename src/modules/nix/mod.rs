pub mod expr;
pub mod module;
pub mod render;

pub use expr::{NixExpr, ToNix};
pub use module::NixModule;
pub use render::render;

// Convert any `serde::Serialize` type to a `NixExpr` via JSON intermediate.
// intermediate approach gives some ergonomics when we working structs
pub fn from_serde<T: serde::Serialize>(value: &T) -> NixExpr {
    let v = serde_json::to_value(value).unwrap_or(serde_json::Value::Null);
    json_to_expr(&v)
}

fn json_to_expr(v: &serde_json::Value) -> NixExpr {
    use serde_json::Value;
    match v {
        Value::Null => NixExpr::Null,
        Value::Bool(b) => NixExpr::Bool(*b),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                NixExpr::Int(i)
            } else {
                NixExpr::Float(n.as_f64().unwrap_or(0.0))
            }
        }
        Value::String(s) => NixExpr::Str(s.clone()),
        Value::Array(items) => NixExpr::List(items.iter().map(json_to_expr).collect()),
        Value::Object(map) => NixExpr::Attrs(
            map.iter()
                .map(|(k, v)| (k.clone(), json_to_expr(v)))
                .collect(),
        ),
    }
}
