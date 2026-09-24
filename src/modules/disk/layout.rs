use super::{
    fs::Content,
    size::{PartitionSize, Size, SizeUnit},
};
use crate::modules::nix::{NixExpr, NixModule, ToNix};
use std::collections::BTreeMap;

// ── Error ─────────────────────────────────────────────────────────────────────

#[derive(Debug, thiserror::Error)]
pub enum LayoutError {
    /// New partition exceeds available space.
    #[error("insufficient space: requested {requested} bytes, {available} bytes available")]
    InsufficientSpace { requested: u64, available: u64 },
    /// A fill-remaining (`Percent(100)`) partition already occupies the tail.
    #[error("a fill-remaining (100%) partition already exists")]
    FillPartitionExists,
    /// No partition with the given label was found.
    #[error("partition {0:?} not found")]
    PartitionNotFound(String),
}

// ── SizeOp ───────────────────────────────────────────────────────────────────

/// An append-only record of how partitions affect used space.
/// `Fill`/`Unfill` are sentinels for `Percent(100)` — "use all remaining".
#[derive(Debug, Clone)]
pub enum SizeOp {
    Add(u64),
    Sub(u64),
    /// A fill-remaining partition was added: used = total.
    Fill,
    /// The fill-remaining partition was removed: used reverts to concrete sum.
    Unfill,
}

impl SizeOp {
    /// Evaluate a log, returning total used bytes.
    pub fn eval(ops: &[Self], total: u64) -> u64 {
        let mut used = 0u64;
        let mut filled = false;
        for op in ops {
            match op {
                SizeOp::Add(n) => used = used.saturating_add(*n),
                SizeOp::Sub(n) => used = used.saturating_sub(*n),
                SizeOp::Fill => filled = true,
                SizeOp::Unfill => filled = false,
            }
        }
        if filled { total } else { used }
    }
}

// ── PartitionDef ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct PartitionDef {
    pub label: Option<String>,
    pub size: PartitionSize,
    pub type_code: Option<String>,
    /// `None` is BIOS-boot compatibility slot (no filesystem content).
    pub content: Option<Content>,
}

// ── DiskLayout ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct DiskLayout {
    /// stable by-id device path, e.g. `/dev/disk/by-id/ata-Samsung_860_EVO_xxx`.
    pub device: String,
    /// Physical capacity in bytes. `0` = unconstrained (no size validation).
    /// maybe u64 overflowed...
    pub total_bytes: u64,
    partitions: Vec<PartitionDef>,
    ops: Vec<SizeOp>,
}

impl DiskLayout {
    pub fn new(device: impl Into<String>, total_bytes: u64) -> Self {
        DiskLayout {
            device: device.into(),
            total_bytes,
            partitions: vec![],
            ops: vec![],
        }
    }

    // ── Space accounting ─────────────────────────────────────────────────────

    pub fn total_space<U: SizeUnit>(&self) -> Size<U> {
        Size::new(self.total_bytes / U::BYTES)
    }

    pub fn used_space<U: SizeUnit>(&self) -> Size<U> {
        Size::new(SizeOp::eval(&self.ops, self.total_bytes) / U::BYTES)
    }

    pub fn free_space<U: SizeUnit>(&self) -> Size<U> {
        let free = self
            .total_bytes
            .saturating_sub(SizeOp::eval(&self.ops, self.total_bytes));
        Size::new(free / U::BYTES)
    }

    pub fn partitions(&self) -> &[PartitionDef] {
        &self.partitions
    }

    pub fn add_partition(&self, def: PartitionDef) -> Result<Self, LayoutError> {
        if self.has_fill() {
            return Err(LayoutError::FillPartitionExists);
        }

        let is_fill = matches!(def.size, PartitionSize::Percent(100));

        let op = if is_fill {
            SizeOp::Fill
        } else {
            let requested = def.size.to_bytes(self.total_bytes);
            let available = self
                .total_bytes
                .saturating_sub(SizeOp::eval(&self.ops, self.total_bytes));
            if self.total_bytes > 0 && requested > available {
                return Err(LayoutError::InsufficientSpace {
                    requested,
                    available,
                });
            }
            SizeOp::Add(requested)
        };

        let mut next = self.clone();
        next.ops.push(op);
        next.partitions.push(def);
        Ok(next)
    }

    pub fn remove_partition(&self, label: &str) -> Result<Self, LayoutError> {
        let pos = self
            .partitions
            .iter()
            .position(|p| p.label.as_deref() == Some(label))
            .ok_or_else(|| LayoutError::PartitionNotFound(label.to_string()))?;

        let def = &self.partitions[pos];
        let op = if matches!(def.size, PartitionSize::Percent(100)) {
            SizeOp::Unfill
        } else {
            SizeOp::Sub(def.size.to_bytes(self.total_bytes))
        };

        let mut next = self.clone();
        next.ops.push(op);
        next.partitions.remove(pos);
        Ok(next)
    }

    // ── Nix generation ───────────────────────────────────────────────────────

    pub fn to_nix_module(&self) -> NixModule {
        let has_luks = self
            .partitions
            .iter()
            .any(|p| p.content.as_ref().is_some_and(Content::has_luks));

        let mut module = NixModule::new().set("disko.devices", self.to_nix());
        if has_luks {
            module = module.set("boot.initrd.systemd.enable", NixExpr::Bool(true));
        }
        module
    }

    fn to_nix(&self) -> NixExpr {
        let partitions: BTreeMap<String, NixExpr> = self
            .partitions
            .iter()
            .map(|p| {
                let label = p.label.clone().unwrap_or_else(|| "part".into());
                (label, p.to_nix())
            })
            .collect();

        let gpt = NixExpr::Attrs(BTreeMap::from([
            ("type".into(), "gpt".into()),
            ("partitions".into(), NixExpr::Attrs(partitions)),
        ]));

        let disk = NixExpr::Attrs(BTreeMap::from([
            ("type".into(), "disk".into()),
            ("device".into(), NixExpr::Str(self.device.clone())),
            ("content".into(), gpt),
        ]));

        let disk_name = self.device.rsplit('/').next().unwrap_or("main").to_string();

        NixExpr::Attrs(BTreeMap::from([(
            "disk".into(),
            NixExpr::Attrs(BTreeMap::from([(disk_name, disk)])),
        )]))
    }

    // ── helpers

    fn has_fill(&self) -> bool {
        for op in self.ops.iter().rev() {
            match op {
                SizeOp::Fill => return true,
                SizeOp::Unfill => return false,
                _ => {}
            }
        }
        false
    }
}

// Multi-disk
// Merge multiple layouts (one per physical disk) into a single `NixModule`.
// Used by the advanced/custom partition path.
pub fn layouts_to_nix_module(layouts: &[DiskLayout]) -> NixModule {
    let has_luks = layouts
        .iter()
        .flat_map(|l| l.partitions())
        .any(|p| p.content.as_ref().is_some_and(Content::has_luks));

    let disks: BTreeMap<String, NixExpr> = layouts
        .iter()
        .map(|l| {
            let name = l.device.rsplit('/').next().unwrap_or("main").to_string();
            let partitions: BTreeMap<String, NixExpr> = l
                .partitions()
                .iter()
                .map(|p| (p.label.clone().unwrap_or_else(|| "part".into()), p.to_nix()))
                .collect();
            let gpt = NixExpr::Attrs(BTreeMap::from([
                ("type".into(), "gpt".into()),
                ("partitions".into(), NixExpr::Attrs(partitions)),
            ]));
            let disk = NixExpr::Attrs(BTreeMap::from([
                ("type".into(), "disk".into()),
                ("device".into(), NixExpr::Str(l.device.clone())),
                ("content".into(), gpt),
            ]));
            (name, disk)
        })
        .collect();

    let devices = NixExpr::Attrs(BTreeMap::from([("disk".into(), NixExpr::Attrs(disks))]));

    let mut module = NixModule::new().set("disko.devices", devices);
    if has_luks {
        module = module.set("boot.initrd.systemd.enable", NixExpr::Bool(true));
    }
    module
}

impl ToNix for PartitionDef {
    fn to_nix(&self) -> NixExpr {
        let type_code = self.type_code.clone().or_else(|| {
            self.content
                .as_ref()?
                .default_type_code()
                .map(|s| s.to_string())
        });

        let mut m = BTreeMap::new();
        if let Some(code) = type_code {
            m.insert("type".into(), NixExpr::Str(code));
        }
        m.insert("size".into(), NixExpr::Str(self.size.to_disko_str()));
        if let Some(label) = &self.label {
            m.insert("label".into(), NixExpr::Str(label.clone()));
        }
        if let Some(content) = &self.content {
            m.insert("content".into(), content.to_nix());
        }
        NixExpr::Attrs(m)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::disk::{
        fs::{Fs, LinuxFs},
        size::{GiB, MiB},
    };

    const GB: u64 = 1024 * 1024 * 1024;

    fn simple_part(label: &str, gib: u64) -> PartitionDef {
        PartitionDef {
            label: Some(label.into()),
            size: PartitionSize::Gib(Size::new(gib)),
            type_code: None,
            content: Some(Content::Filesystem(Fs::linux(LinuxFs::Ext4, "/"))),
        }
    }

    #[test]
    fn add_fits() {
        let layout = DiskLayout::new("/dev/sda", 100 * GB);
        let layout = layout.add_partition(simple_part("root", 50)).unwrap();
        assert_eq!(layout.free_space::<GiB>().value, 50);
        assert_eq!(layout.used_space::<GiB>().value, 50);
    }

    #[test]
    fn add_too_large() {
        let layout = DiskLayout::new("/dev/sda", 10 * GB);
        let err = layout.add_partition(simple_part("root", 20)).unwrap_err();
        assert!(matches!(err, LayoutError::InsufficientSpace { .. }));
    }

    #[test]
    fn add_after_fill_rejected() {
        let layout = DiskLayout::new("/dev/sda", 100 * GB);
        let layout = layout
            .add_partition(PartitionDef {
                label: Some("root".into()),
                size: PartitionSize::remaining(),
                type_code: None,
                content: None,
            })
            .unwrap();
        let err = layout.add_partition(simple_part("extra", 1)).unwrap_err();
        assert!(matches!(err, LayoutError::FillPartitionExists));
    }

    #[test]
    fn remove_restores_space() {
        let layout = DiskLayout::new("/dev/sda", 100 * GB);
        let layout = layout.add_partition(simple_part("root", 40)).unwrap();
        let layout = layout.remove_partition("root").unwrap();
        assert_eq!(layout.free_space::<GiB>().value, 100);
    }

    #[test]
    fn remove_fill_restores_space() {
        let layout = DiskLayout::new("/dev/sda", 100 * GB)
            .add_partition(simple_part("data", 10))
            .unwrap()
            .add_partition(PartitionDef {
                label: Some("root".into()),
                size: PartitionSize::remaining(),
                type_code: None,
                content: None,
            })
            .unwrap();
        assert_eq!(layout.free_space::<MiB>().value, 0);

        let layout = layout.remove_partition("root").unwrap();
        assert_eq!(layout.free_space::<GiB>().value, 90);
    }

    #[test]
    fn remove_not_found() {
        let layout = DiskLayout::new("/dev/sda", 100 * GB);
        assert!(matches!(
            layout.remove_partition("ghost"),
            Err(LayoutError::PartitionNotFound(_))
        ));
    }

    #[test]
    fn total_used_free_consistent() {
        let total = 100 * GB;
        let layout = DiskLayout::new("/dev/sda", total)
            .add_partition(simple_part("a", 30))
            .unwrap()
            .add_partition(simple_part("b", 20))
            .unwrap();

        let used = layout.used_space::<GiB>().value * GB;
        let free = layout.free_space::<GiB>().value * GB;
        assert_eq!(used + free, total);
    }
}
