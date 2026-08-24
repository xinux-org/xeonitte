use super::fs::{Content, Fs, LuksContent, SwapContent};
use super::layout::{DiskLayout, PartitionDef, layouts_to_nix_module};
use super::ops::LUKS_PASSWORD_FILE;
use super::size::{GiB, PartitionSize, Size as DiskSize};
use crate::modules::nix::NixModule;
use crate::{
    get_memory_size, get_storage_size, get_storage_size_for_disko,
    ui::partitions::partition_model::{
        CustomOptions, CustomPartition, FullDiskOptions, PartitionSchema,
    },
};
use size::Size;
use std::collections::BTreeMap;

/// Convert a `PartitionSchema` (UI selection) into a `NixModule` ready to render.
pub fn disko_from_schema(schema: &PartitionSchema) -> NixModule {
    match schema {
        PartitionSchema::FullDisk(opts) => disko_from_full_disk(opts),
        PartitionSchema::Custom(opts) => disko_from_custom(opts),
    }
}

fn disko_from_full_disk(opts: &FullDiskOptions) -> NixModule {
    let swap = compute_swap(&opts.device);
    let layout = if opts.encryption {
        DiskLayout::luks_encrypted(
            opts.device.clone(),
            opts.disk_size,
            swap,
            LUKS_PASSWORD_FILE,
        )
    } else {
        DiskLayout::canonical(opts.device.clone(), opts.disk_size, swap)
    };
    layout.to_nix_module()
}

fn compute_swap(device: &str) -> Option<DiskSize<GiB>> {
    let storage_mb = get_storage_size(device, 512)?;
    let memory_kib = get_memory_size()?;

    let swap_gib: u64 = match storage_mb {
        256_000.. => (memory_kib / (1024 * 1024)).max(1),
        128_000..256_000 => 8,
        64_000..128_000 => 4,
        _ => return None,
    };

    Some(DiskSize::new(swap_gib))
}

fn disko_from_custom(opts: &CustomOptions) -> NixModule {
    let any_encrypted = opts.partitions.values().any(|p| p.encrypt);

    // Group partitions by parent disk device path.
    let mut by_disk: BTreeMap<String, Vec<(String, &CustomPartition)>> = BTreeMap::new();
    for (name, part) in &opts.partitions {
        by_disk
            .entry(part.device.clone())
            .or_default()
            .push((name.clone(), part));
    }

    let layouts: Vec<DiskLayout> = by_disk
        .into_iter()
        .map(|(device, parts)| {
            let partitions: Vec<PartitionDef> = parts
                .into_iter()
                .filter_map(|(name, part)| {
                    let part_key = name.rsplit('/').next().unwrap_or(&name).to_string();
                    let content = content_from_custom(part, &part_key, any_encrypted)?;

                    let size = if part.is_full {
                        PartitionSize::remaining()
                    } else {
                        PartitionSize::Exact {
                            bytes: part.size,
                            display: get_storage_size_for_disko(Size::from_bytes(part.size)),
                        }
                    };

                    Some(PartitionDef {
                        label: Some(part_key),
                        size,
                        type_code: type_code_for(part),
                        content: Some(content),
                    })
                })
                .collect();

            partitions
                .into_iter()
                .fold(DiskLayout::new(device, opts.disk_size), |layout, def| {
                    layout.add_partition(def).unwrap_or(layout)
                })
        })
        .collect();

    layouts_to_nix_module(&layouts)
}

fn type_code_for(part: &CustomPartition) -> Option<String> {
    match part.mountpoint.as_deref() {
        Some("/boot") => Some("EF00".into()),
        _ => None,
    }
}

fn content_from_custom(
    part: &CustomPartition,
    part_key: &str,
    any_encrypted: bool,
) -> Option<Content> {
    let content = match (part.mountpoint.as_deref(), part.format.as_deref()) {
        (Some("/boot"), fmt) => Content::Filesystem(Fs {
            format: fmt.unwrap_or("vfat").into(),
            mountpoint: Some("/boot".into()),
            mount_options: vec!["umask=0077".into()],
            extra_args: vec![],
        }),

        (None, Some("swap")) => Content::Swap(SwapContent {
            resume_device: true,
            ..Default::default()
        }),

        (Some(mount), fmt) => Content::Filesystem(Fs {
            format: fmt.unwrap_or("ext4").into(),
            mountpoint: Some(mount.into()),
            ..Default::default()
        }),

        (None, Some(fmt)) => Content::Filesystem(Fs {
            format: fmt.into(),
            mountpoint: None,
            ..Default::default()
        }),

        (None, None) => return None,
    };

    // Wrap with LUKS when the partition is marked for encryption
    let needs_luks = part.encrypt || (any_encrypted && matches!(content, Content::Swap(_)));

    if needs_luks {
        Some(Content::Luks(
            LuksContent::new(
                format!("crypted-{part_key}"),
                Some(LUKS_PASSWORD_FILE.into()),
                content,
            )
            .allow_discards(),
        ))
    } else {
        Some(content)
    }
}
