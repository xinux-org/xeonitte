use crate::modules::nix::NixExpr;
use serde::Serialize;
use std::collections::BTreeMap;

/// Linux filesystem formats for use with `Content::Filesystem`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LinuxFs {
    Ext4,
    Btrfs,
    Xfs,
    F2fs,
    Bcachefs,
    Exfat,
    Ntfs,
}

impl LinuxFs {
    pub fn as_str(&self) -> &'static str {
        match self {
            LinuxFs::Ext4 => "ext4",
            LinuxFs::Btrfs => "btrfs",
            LinuxFs::Xfs => "xfs",
            LinuxFs::F2fs => "f2fs",
            LinuxFs::Bcachefs => "bcachefs",
            LinuxFs::Exfat => "exfat",
            LinuxFs::Ntfs => "ntfs",
        }
    }
}

/// Disko content type for a partition or encrypted/virtual device.
/// Variants map 1:1 to disko's content type system.
#[derive(Debug, Clone)]
pub enum Content {
    /// A formatted filesystem — any format, optional mountpoint.
    Filesystem(Fs),
    /// Btrfs with subvolume support.
    Btrfs(BtrfsContent),
    /// Swap partition.
    Swap(SwapContent),
    /// LUKS encryption wrapping inner content.
    Luks(LuksContent),
    /// LVM physical volume contributing to a named volume group.
    LvmPv { vg: String },
}

/// Simple formatted filesystem (`type = "filesystem"`).
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Fs {
    pub format: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mountpoint: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub mount_options: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub extra_args: Vec<String>,
}

impl Fs {
    /// EFI System Partition: vfat + umask=0077.
    pub fn efi(mountpoint: impl Into<String>) -> Self {
        Fs {
            format: "vfat".into(),
            mountpoint: Some(mountpoint.into()),
            mount_options: vec!["umask=0077".into()],
            extra_args: vec![],
        }
    }

    /// Plain Linux filesystem with a mountpoint.
    pub fn linux(format: LinuxFs, mountpoint: impl Into<String>) -> Self {
        Fs {
            format: format.as_str().into(),
            mountpoint: Some(mountpoint.into()),
            ..Default::default()
        }
    }
}

/// Btrfs filesystem with subvolumes (`type = "btrfs"`).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BtrfsContent {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub extra_args: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mountpoint: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub mount_options: Vec<String>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub subvolumes: BTreeMap<String, BtrfsSubvolume>,
}

/// A single Btrfs subvolume.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BtrfsSubvolume {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mountpoint: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub mount_options: Vec<String>,
}

/// Swap space (`type = "swap"`).
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SwapContent {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discard_policy: Option<DiscardPolicy>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub extra_args: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub mount_options: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<i64>,
    #[serde(skip_serializing_if = "is_false")]
    pub random_encryption: bool,
    #[serde(skip_serializing_if = "is_false")]
    pub resume_device: bool,
}

/// LUKS encryption (`type = "luks"`).
#[derive(Debug, Clone)]
pub struct LuksContent {
    pub name: String,
    pub password_file: Option<String>,
    pub settings: BTreeMap<String, NixExpr>,
    pub extra_format_args: Vec<String>,
    pub extra_open_args: Vec<String>,
    pub content: Box<Content>,
}

impl LuksContent {
    pub fn new(name: impl Into<String>, password_file: Option<String>, content: Content) -> Self {
        LuksContent {
            name: name.into(),
            password_file,
            settings: BTreeMap::new(),
            extra_format_args: vec![],
            extra_open_args: vec![],
            content: Box::new(content),
        }
    }

    pub fn allow_discards(mut self) -> Self {
        self.settings
            .insert("allowDiscards".into(), NixExpr::Bool(true));
        self
    }
}

/// Swap discard policy.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum DiscardPolicy {
    Once,
    Pages,
    Both,
}

impl Content {
    /// Default GPT type code for this content variant.
    /// Explicit `PartitionDef::type_code` takes precedence over this.
    pub fn default_type_code(&self) -> Option<&'static str> {
        match self {
            Content::Filesystem(fs) if fs.format == "vfat" => Some("EF00"),
            Content::Filesystem(_) | Content::Btrfs(_) => Some("8300"),
            Content::Swap(_) => Some("8200"),
            Content::Luks(_) => Some("8309"),
            Content::LvmPv { .. } => Some("8E00"),
        }
    }

    /// True if this content tree contains a LUKS layer anywhere.
    pub fn has_luks(&self) -> bool {
        matches!(self, Content::Luks(_))
    }

    pub(super) fn to_nix(&self) -> NixExpr {
        match self {
            Content::Filesystem(fs) => typed_attrs("filesystem", fs),
            Content::Btrfs(b) => typed_attrs("btrfs", b),
            Content::Swap(s) => typed_attrs("swap", s),
            Content::Luks(l) => luks_to_nix(l),
            Content::LvmPv { vg } => NixExpr::Attrs(BTreeMap::from([
                ("type".into(), "lvm_pv".into()),
                ("vg".into(), NixExpr::Str(vg.clone())),
            ])),
        }
    }
}

/// Serialize `val` to a `NixExpr::Attrs` and inject `"type"`.
fn typed_attrs<T: Serialize>(type_str: &'static str, val: &T) -> NixExpr {
    let mut m = match json_to_nix(serde_json::to_value(val).expect("serialization is infallible")) {
        NixExpr::Attrs(m) => m,
        _ => unreachable!(),
    };
    m.insert("type".into(), type_str.into());
    NixExpr::Attrs(m)
}

/// Convert a JSON value to NixExpr — works because NixExpr implements Deserialize.
fn json_to_nix(v: serde_json::Value) -> NixExpr {
    serde_json::from_value(v).expect("NixExpr covers all JSON value types")
}

// FIXME: Maybe we can simplfy
fn luks_to_nix(luks: &LuksContent) -> NixExpr {
    let mut m = BTreeMap::from([
        ("type".into(), "luks".into()),
        ("name".into(), NixExpr::Str(luks.name.clone())),
        ("content".into(), luks.content.to_nix()),
    ]);
    if let Some(pf) = &luks.password_file {
        m.insert("passwordFile".into(), NixExpr::Str(pf.clone()));
    }
    if !luks.settings.is_empty() {
        m.insert("settings".into(), NixExpr::Attrs(luks.settings.clone()));
    }
    if !luks.extra_format_args.is_empty() {
        m.insert("extraFormatArgs".into(), str_list(&luks.extra_format_args));
    }
    if !luks.extra_open_args.is_empty() {
        m.insert("extraOpenArgs".into(), str_list(&luks.extra_open_args));
    }
    NixExpr::Attrs(m)
}

fn str_list(v: &[String]) -> NixExpr {
    NixExpr::List(v.iter().map(|s| NixExpr::Str(s.clone())).collect())
}

fn is_false(b: &bool) -> bool {
    !*b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn type_codes() {
        assert_eq!(
            Content::Filesystem(Fs::efi("/boot")).default_type_code(),
            Some("EF00")
        );
        assert_eq!(
            Content::Filesystem(Fs::linux(LinuxFs::Ext4, "/")).default_type_code(),
            Some("8300")
        );
        assert_eq!(
            Content::Swap(SwapContent::default()).default_type_code(),
            Some("8200")
        );
        assert_eq!(
            Content::Luks(LuksContent::new(
                "x",
                None,
                Content::Swap(SwapContent::default())
            ))
            .default_type_code(),
            Some("8309")
        );
        assert_eq!(
            Content::LvmPv { vg: "vg0".into() }.default_type_code(),
            Some("8E00")
        );
    }

    #[test]
    fn btrfs_subvols_to_nix() {
        let content = Content::Btrfs(BtrfsContent {
            extra_args: vec!["-f".into()],
            mountpoint: None,
            mount_options: vec![],
            subvolumes: BTreeMap::from([
                (
                    "@".into(),
                    BtrfsSubvolume {
                        mountpoint: Some("/".into()),
                        mount_options: vec!["noatime".into()],
                    },
                ),
                (
                    "@home".into(),
                    BtrfsSubvolume {
                        mountpoint: Some("/home".into()),
                        mount_options: vec!["noatime".into()],
                    },
                ),
            ]),
        });
        // smoke test: should not panic
        let _ = content.to_nix();
    }

    #[test]
    fn luks_allow_discards() {
        let inner = Content::Filesystem(Fs::linux(LinuxFs::Ext4, "/"));
        let luks = LuksContent::new("crypted", Some("/run/key".into()), inner).allow_discards();
        assert!(luks.settings.contains_key("allowDiscards"));
    }

    #[test]
    fn swap_skips_false_fields() {
        let swap = Content::Swap(SwapContent::default());
        let nix = swap.to_nix();
        if let NixExpr::Attrs(m) = nix {
            assert_eq!(m["type"], NixExpr::Str("swap".into()));
            assert!(
                !m.contains_key("randomEncryption"),
                "false bool must be omitted"
            );
            assert!(
                !m.contains_key("resumeDevice"),
                "false bool must be omitted"
            );
            assert!(!m.contains_key("discardPolicy"), "None must be omitted");
        } else {
            panic!("expected Attrs");
        }
    }

    #[test]
    fn fs_to_nix_efi() {
        let fs = Content::Filesystem(Fs::efi("/boot/efi"));
        if let NixExpr::Attrs(m) = fs.to_nix() {
            assert_eq!(m["type"], NixExpr::Str("filesystem".into()));
            assert_eq!(m["format"], NixExpr::Str("vfat".into()));
            assert_eq!(m["mountpoint"], NixExpr::Str("/boot/efi".into()));
        } else {
            panic!("expected Attrs");
        }
    }
}
