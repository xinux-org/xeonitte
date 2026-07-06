use serde::{Deserialize, Serialize};
use size::Size;
use std::collections::BTreeMap;

use crate::{get_memory_size, get_storage_size, get_storage_size_for_disko};

pub type Attrs<T> = BTreeMap<String, T>;

pub const LUKS_PASSWORD_FILE: &str = "/run/xeonitte-luks.key";

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Devices {
    #[serde(default, skip_serializing_if = "Attrs::is_empty")]
    pub disk: Attrs<Disk>,
}

impl Devices {
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).expect("Devices serialization is total")
    }

    pub fn to_nix_devices(&self) -> String {
        let v = serde_json::to_value(self).expect("total");
        nix::render(&v, 0)
    }

    // importable module: `{ disko.devices = { ... }; }`.
    pub fn to_nix_module(&self) -> String {
        let mut body = format!("  disko.devices = {};\n", indent(&self.to_nix_devices(), 1));
        if self.has_luks() {
            // boot.initrd.systemd.enable - saves passphrase to keyring and open all encrypted partitions
            body.push_str("  boot.initrd.systemd.enable = true;\n");
        }
        format!("{{\n{body}}}\n")
    }

    // True if any device in the tree is LUKS-encrypted
    fn has_luks(&self) -> bool {
        self.disk
            .values()
            .filter_map(|d| d.content.as_ref())
            .any(device_has_luks)
    }
}

fn device_has_luks(content: &DeviceContent) -> bool {
    match content {
        DeviceContent::Luks(_) => true,
        DeviceContent::Gpt(gpt) => gpt
            .partitions
            .values()
            .filter_map(|p| p.content.as_ref())
            .any(partition_has_luks),
        DeviceContent::Filesystem(_) | DeviceContent::Swap(_) => false,
    }
}

fn partition_has_luks(content: &PartitionContent) -> bool {
    match content {
        PartitionContent::Luks(_) => true,
        PartitionContent::Filesystem(_) | PartitionContent::Swap(_) => false,
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Disk {
    // `/dev/...` path. Required.
    pub device: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<DeviceContent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DeviceContent {
    Gpt(Gpt),
    // Legacy MBR/GPT table driven by `parted` GPT.
    Luks(Luks),
    Filesystem(Filesystem),
    Swap(Swap),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PartitionContent {
    Luks(Luks),
    Filesystem(Filesystem),
    Swap(Swap),
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Gpt {
    #[serde(default, skip_serializing_if = "Attrs::is_empty")]
    pub partitions: Attrs<Partition>,
    // Place the EFI GPT (0xEE) partition first in the MBR (default `true`).
    #[serde(
        rename = "efiGptPartitionFirst",
        skip_serializing_if = "Option::is_none"
    )]
    pub efi_gpt_partition_first: Option<bool>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Partition {
    // sgdisk typecode (e.g. `"EF00"`, `"8300"`, `"8200"`) or a full type GUID.
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub type_code: Option<String>,
    // `"512M"`, `"100%"`, ... (sgdisk size; `"0"` means "auto").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    // Partition GUID; `None` lets disko generate one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uuid: Option<String>,
    // GPT partition-entry attribute bit numbers (see UEFI 2.10).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub attributes: Vec<i64>,
    //Smaller is created first; defaults are derived from size/type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<PartitionContent>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FsFormat {
    Ext2,
    Ext3,
    Ext4,
    Vfat,
    Xfs,
    Btrfs,
    F2fs,
    Bcachefs,
    Exfat,
    Ntfs,
    // Unkown
    Other(String),
}

impl FsFormat {
    pub fn as_str(&self) -> &str {
        match self {
            FsFormat::Ext2 => "ext2",
            FsFormat::Ext3 => "ext3",
            FsFormat::Ext4 => "ext4",
            FsFormat::Vfat => "vfat",
            FsFormat::Xfs => "xfs",
            FsFormat::Btrfs => "btrfs",
            FsFormat::F2fs => "f2fs",
            FsFormat::Bcachefs => "bcachefs",
            FsFormat::Exfat => "exfat",
            FsFormat::Ntfs => "ntfs",
            FsFormat::Other(s) => s,
        }
    }

    pub fn is_known(&self) -> bool {
        !matches!(self, FsFormat::Other(_))
    }
    pub fn values() -> Vec<&'static str> {
        vec![
            "ext2", "ext3", "ext4", "vfat", "xfs", "btrfs", "f2fs", "bcachefs", "exfat", "ntfs",
        ]
    }
}

impl Default for FsFormat {
    fn default() -> Self {
        FsFormat::Ext4
    }
}

impl std::fmt::Display for FsFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl From<&str> for FsFormat {
    fn from(s: &str) -> Self {
        match s.to_ascii_lowercase().as_str() {
            "ext2" => FsFormat::Ext2,
            "ext3" => FsFormat::Ext3,
            "ext4" => FsFormat::Ext4,
            // The UI presents the friendly label "fat32"; disko's format is "vfat"
            "vfat" | "fat32" => FsFormat::Vfat,
            "xfs" => FsFormat::Xfs,
            "btrfs" => FsFormat::Btrfs,
            "f2fs" => FsFormat::F2fs,
            "bcachefs" => FsFormat::Bcachefs,
            "exfat" => FsFormat::Exfat,
            "ntfs" => FsFormat::Ntfs,
            _ => FsFormat::Other(s.to_owned()),
        }
    }
}

impl From<String> for FsFormat {
    fn from(s: String) -> Self {
        FsFormat::from(s.as_str())
    }
}

impl std::str::FromStr for FsFormat {
    type Err = std::convert::Infallible;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(FsFormat::from(s))
    }
}

impl Serialize for FsFormat {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for FsFormat {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        Ok(FsFormat::from(String::deserialize(d)?))
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Filesystem {
    pub format: FsFormat,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mountpoint: Option<String>,
    #[serde(
        rename = "mountOptions",
        default,
        skip_serializing_if = "Vec::is_empty"
    )]
    pub mount_options: Vec<String>,
    #[serde(rename = "extraArgs", default, skip_serializing_if = "Vec::is_empty")]
    pub extra_args: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Luks {
    pub name: String,

    #[serde(rename = "passwordFile", skip_serializing_if = "Option::is_none")]
    pub password_file: Option<String>,

    #[serde(rename = "askPassword", skip_serializing_if = "Option::is_none")]
    pub ask_password: Option<bool>,
    //`allowDiscards`, `bypassWorkqueues`, `fallbackToPassword`, ...
    #[serde(default, skip_serializing_if = "Attrs::is_empty")]
    pub settings: Attrs<NixValue>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<Box<DeviceContent>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Swap {
    #[serde(rename = "randomEncryption", skip_serializing_if = "Option::is_none")]
    pub random_encryption: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<i64>,
    #[serde(rename = "discardPolicy", skip_serializing_if = "Option::is_none")]
    pub discard_policy: Option<DiscardPolicy>,
    #[serde(rename = "resumeDevice", skip_serializing_if = "Option::is_none")]
    pub resume_device: Option<bool>,
    #[serde(
        rename = "mountOptions",
        default,
        skip_serializing_if = "Vec::is_empty"
    )]
    pub mount_options: Vec<String>,
    #[serde(rename = "extraArgs", default, skip_serializing_if = "Vec::is_empty")]
    pub extra_args: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DiscardPolicy {
    Once,
    Pages,
    Both,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum NixValue {
    Bool(bool),
    Int(i64),
    Str(String),
    List(Vec<NixValue>),
    Attrs(Attrs<NixValue>),
}

impl From<bool> for NixValue {
    fn from(b: bool) -> Self {
        NixValue::Bool(b)
    }
}
impl From<i64> for NixValue {
    fn from(i: i64) -> Self {
        NixValue::Int(i)
    }
}
impl From<&str> for NixValue {
    fn from(s: &str) -> Self {
        NixValue::Str(s.to_owned())
    }
}
impl From<String> for NixValue {
    fn from(s: String) -> Self {
        NixValue::Str(s)
    }
}

pub fn to_nix<T: Serialize>(value: &T) -> String {
    let v = serde_json::to_value(value).expect("serialization is total");
    nix::render(&v, 0)
}

mod nix {
    use serde_json::Value;

    const STEP: usize = 2;

    pub fn render(v: &Value, depth: usize) -> String {
        match v {
            Value::Null => "null".to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Number(n) => n.to_string(),
            Value::String(s) => escape_str(s),
            Value::Array(items) => render_list(items, depth),
            Value::Object(map) => render_attrs(map, depth),
        }
    }

    fn render_list(items: &[Value], depth: usize) -> String {
        if items.is_empty() {
            return "[ ]".to_string();
        }
        let inner = " ".repeat((depth + 1) * STEP);
        let close = " ".repeat(depth * STEP);
        let body: String = items
            .iter()
            .map(|it| format!("{inner}{}", render(it, depth + 1)))
            .collect::<Vec<_>>()
            .join("\n");
        format!("[\n{body}\n{close}]")
    }

    fn render_attrs(map: &serde_json::Map<String, Value>, depth: usize) -> String {
        if map.is_empty() {
            return "{ }".to_string();
        }
        let inner = " ".repeat((depth + 1) * STEP);
        let close = " ".repeat(depth * STEP);
        let body: String = map
            .iter()
            .map(|(k, val)| format!("{inner}{} = {};", render_key(k), render(val, depth + 1)))
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
        if valid { k.to_string() } else { escape_str(k) }
    }

    // escape `\`, `"`, `${` chars.
    fn escape_str(s: &str) -> String {
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
                // `${` begins antiquotation; escape the dollar.
                '$' if chars.peek() == Some(&'{') => out.push_str("\\$"),
                _ => out.push(c),
            }
        }
        out.push('"');
        out
    }
}

fn indent(s: &str, levels: usize) -> String {
    let pad = "  ".repeat(levels);
    let mut lines = s.lines();
    let first = lines.next().unwrap_or("").to_string();
    let rest: Vec<String> = lines.map(|l| format!("{pad}{l}")).collect();
    if rest.is_empty() {
        first
    } else {
        format!("{first}\n{}", rest.join("\n"))
    }
}

// nix version: https://gist.github.com/lambdajon/1946c9585c997a2615f5386a5f222c6f
pub fn luks_encrypted(device: String, password_file: impl Into<String>) -> Devices {
    // Shared by the swap and LUKS containers
    let password_file = password_file.into();
    let mut partitions = Attrs::new();
    let storage_size: Option<u64> = get_storage_size(&device, 512);
    let memory_size = get_memory_size();
    let swap_size: Option<String> = match (storage_size, memory_size) {
        (Some(256_000..), Some(memory_size)) => Some(get_storage_size_for_disko(
            Size::from_kib(memory_size).bytes() as u64,
        )),
        (Some(128_000..256_000), _) => Some(get_storage_size_for_disko(
            Size::from_gigabytes(8).bytes() as u64,
        )),
        (Some(64_000..128_000), _) => Some(get_storage_size_for_disko(
            Size::from_gigabytes(4).bytes() as u64,
        )),
        _ => None,
    };

    println!("storage size: {:?}", storage_size);
    println!("memory size: {:?}", memory_size);
    println!("swap size: {:?}", swap_size);

    partitions.insert(
        "BOOT".into(),
        Partition {
            type_code: Some("EF02".into()),
            size: Some("1000M".into()),
            content: Some(PartitionContent::Filesystem(Filesystem {
                format: "vfat".into(),
                mountpoint: Some("/boot".into()),
                mount_options: vec!["umask=0077".into()],
                ..Default::default()
            })),
            ..Default::default()
        },
    );

    let mut luks_settings = Attrs::new();
    luks_settings.insert("allowDiscards".into(), NixValue::Bool(true));

    // Encrypt swap with the same passphrase as LUKS
    if swap_size.is_some() {
        partitions.insert(
            "SWAP".into(),
            Partition {
                size: swap_size,
                content: Some(PartitionContent::Luks(Luks {
                    name: "cryptswap".into(),
                    password_file: Some(password_file.clone()),
                    settings: luks_settings.clone(),
                    content: Some(Box::new(DeviceContent::Swap(Swap {
                        resume_device: Some(true),
                        ..Default::default()
                    }))),
                    ..Default::default()
                })),
                ..Default::default()
            },
        );
    }

    partitions.insert(
        "luks".into(),
        Partition {
            size: Some("100%".into()),
            content: Some(PartitionContent::Luks(Luks {
                name: "crypted".into(),
                password_file: Some(password_file.into()),
                settings: luks_settings,
                content: Some(Box::new(DeviceContent::Filesystem(Filesystem {
                    format: "ext4".into(),
                    mountpoint: Some("/".into()),
                    ..Default::default()
                }))),
                ..Default::default()
            })),
            ..Default::default()
        },
    );

    let mut disk = Attrs::new();
    disk.insert(
        "main".into(),
        Disk {
            device: device,
            content: Some(DeviceContent::Gpt(Gpt {
                partitions,
                ..Default::default()
            })),
            ..Default::default()
        },
    );

    Devices {
        disk,
        ..Default::default()
    }
}

pub fn canonical(device: String) -> Devices {
    let mut partitions = Attrs::new();

    let storage_size: Option<u64> = get_storage_size(&device, 512);
    let memory_size = get_memory_size();
    let swap_size: Option<String> = match (storage_size, memory_size) {
        (Some(256_000..), Some(memory_size)) => Some(get_storage_size_for_disko(
            Size::from_kib(memory_size).bytes() as u64,
        )),
        (Some(128_000..256_000), _) => Some(get_storage_size_for_disko(
            Size::from_gigabytes(8).bytes() as u64,
        )),
        (Some(64_000..128_000), _) => Some(get_storage_size_for_disko(
            Size::from_gigabytes(4).bytes() as u64,
        )),
        _ => None,
    };
    partitions.insert(
        "ESP".into(),
        Partition {
            type_code: Some("EF02".into()),
            size: Some("512M".into()),
            content: Some(PartitionContent::Filesystem(Filesystem {
                format: "vfat".into(),
                mountpoint: Some("/boot".into()),
                mount_options: vec!["umask=0077".into()],
                ..Default::default()
            })),
            ..Default::default()
        },
    );

    if swap_size.is_some() {
        partitions.insert(
            "swap".into(),
            Partition {
                size: swap_size,
                content: Some(PartitionContent::Swap(Swap {
                    resume_device: Some(true),
                    ..Default::default()
                })),
                ..Default::default()
            },
        );
    }
    partitions.insert(
        "root".into(),
        Partition {
            size: Some("100%".into()),
            content: Some(PartitionContent::Filesystem(Filesystem {
                format: "ext4".into(),
                mountpoint: Some("/".into()),
                ..Default::default()
            })),
            ..Default::default()
        },
    );

    let mut disk = Attrs::new();
    disk.insert(
        "main".into(),
        Disk {
            device: device,
            content: Some(DeviceContent::Gpt(Gpt {
                partitions,
                ..Default::default()
            })),
            ..Default::default()
        },
    );
    Devices {
        disk,
        ..Default::default()
    }
}
