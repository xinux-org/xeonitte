use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// A typed Nix expression value.
///
/// Variants are tried in declaration order by serde's untagged deserializer,
/// so `Str` always wins over `Path` for JSON strings — `Path` is only ever
/// constructed in Rust code, never round-tripped through JSON.
#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum NixExpr {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    /// A quoted Nix string: `"hello"`
    Str(String),
    /// An unquoted Nix path literal: `./hardware-configuration.nix`
    Path(String),
    List(Vec<NixExpr>),
    Attrs(BTreeMap<String, NixExpr>),
}

/// Types that can be represented as a Nix expression.
pub trait ToNix {
    fn to_nix(&self) -> NixExpr;
}

impl From<bool> for NixExpr {
    fn from(b: bool) -> Self {
        NixExpr::Bool(b)
    }
}
impl From<i64> for NixExpr {
    fn from(i: i64) -> Self {
        NixExpr::Int(i)
    }
}
impl From<i32> for NixExpr {
    fn from(i: i32) -> Self {
        NixExpr::Int(i64::from(i))
    }
}
impl From<u64> for NixExpr {
    fn from(u: u64) -> Self {
        NixExpr::Int(u as i64)
    }
}
impl From<f64> for NixExpr {
    fn from(f: f64) -> Self {
        NixExpr::Float(f)
    }
}
impl From<&str> for NixExpr {
    fn from(s: &str) -> Self {
        NixExpr::Str(s.to_owned())
    }
}
impl From<String> for NixExpr {
    fn from(s: String) -> Self {
        NixExpr::Str(s)
    }
}
impl<T: Into<NixExpr>> From<Vec<T>> for NixExpr {
    fn from(v: Vec<T>) -> Self {
        NixExpr::List(v.into_iter().map(Into::into).collect())
    }
}
impl From<BTreeMap<String, NixExpr>> for NixExpr {
    fn from(m: BTreeMap<String, NixExpr>) -> Self {
        NixExpr::Attrs(m)
    }
}
