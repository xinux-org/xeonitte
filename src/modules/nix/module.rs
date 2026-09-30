use super::{expr::NixExpr, render::render_at};

// A composable NixOS module — a flat list of dotted option paths and their values.
//
// Renders as:
// ```nix
// {
//   disko.devices = { ... };
//   boot.initrd.systemd.enable = true;
// }
// ```
#[derive(Debug, Clone, Default)]
pub struct NixModule {
    entries: Vec<(String, NixExpr)>,
}

impl NixModule {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add or overwrite a NixOS option at the given dotted path.
    pub fn set(mut self, path: impl Into<String>, value: impl Into<NixExpr>) -> Self {
        self.entries.push((path.into(), value.into()));
        self
    }

    /// Append all entries from `other`. Entries with the same path are kept
    /// as separate bindings (Nix allows multiple assignments to the same path
    /// only if they are mergeable sets; use with care for scalar options).
    pub fn merge(mut self, other: NixModule) -> Self {
        self.entries.extend(other.entries);
        self
    }

    /// Render as a Nix module string ready to write to a `.nix` file.
    pub fn render(&self) -> String {
        if self.entries.is_empty() {
            return "{ }\n".to_owned();
        }
        let body: String = self
            .entries
            .iter()
            .map(|(path, val)| {
                let rendered = render_at(val, 1);
                format!("  {path} = {};\n", indent_tail(&rendered, 1))
            })
            .collect();
        format!("{{\n{body}}}\n")
    }
}

/// Indent every line after the first by `levels` levels (2 spaces each).
fn indent_tail(s: &str, levels: usize) -> String {
    let pad = "  ".repeat(levels);
    let mut lines = s.lines();
    let first = lines.next().unwrap_or("").to_owned();
    let rest: Vec<String> = lines.map(|l| format!("{pad}{l}")).collect();
    if rest.is_empty() {
        first
    } else {
        format!("{first}\n{}", rest.join("\n"))
    }
}
