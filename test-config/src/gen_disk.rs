use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use xeonitte::modules::disk::{
    BtrfsContent, BtrfsSubvolume, Content, DiscardPolicy, DiskLayout, Fs, LUKS_PASSWORD_FILE,
    LinuxFs, LuksContent, PartitionDef, PartitionSize, Size, SwapContent, layouts_to_nix_module,
};

const OUT_DIR: &str = "test-config/generated";

const SDA: &str = "/dev/disk/by-id/ata-Samsung_990_EVO_500GB_MAIN";
const SDB: &str = "/dev/disk/by-id/ata-WDC_WD10EZEX_1TB_DATA";

const G500: u64 = 500 * 1024 * 1024 * 1024;
const G40: u64 = 40 * 1024 * 1024 * 1024;

pub fn run() {
    fs::create_dir_all(OUT_DIR).expect("failed to create output directory");

    // canonical (unencrypted)
    write("canonical-no-swap", canonical_no_swap());
    write("canonical-with-swap", canonical_with_swap());

    // LUKS encrypted
    write("luks-no-swap", luks_no_swap());
    write("luks-with-swap", luks_with_swap());

    // alternative root filesystems
    write("xfs-root", xfs_root());
    write("f2fs-root", f2fs_root());
    write("bcachefs-root", bcachefs_root());

    // btrfs
    write("btrfs-subvolumes", btrfs_subvolumes());
    write("btrfs-no-subvols", btrfs_no_subvols());
    write("luks-btrfs", luks_btrfs());

    // swap variants
    write("swap-random-enc", swap_random_enc());
    write("swap-discard-once", swap_discard_once());
    write("swap-discard-pages", swap_discard_pages());
    write("swap-priority", swap_priority());

    // LVM
    write("lvm-pv", lvm_pv());

    // multi-disk
    write("multi-disk-plain", multi_disk_plain());
    write("multi-disk-luks-root", multi_disk_luks_root());
}

fn write(name: &str, nix: String) {
    let path = Path::new(OUT_DIR).join(format!("{name}.nix"));
    fs::write(&path, &nix).unwrap_or_else(|e| panic!("failed to write {name}.nix: {e}"));
    println!("wrote {}", path.display());
}

// shared partition helpers

fn efi() -> PartitionDef {
    PartitionDef {
        label: Some("ESP".into()),
        size: PartitionSize::Gib(Size::new(2)),
        type_code: Some("EF00".into()),
        content: Some(Content::Filesystem(Fs::efi("/boot"))),
    }
}

fn root(fs: LinuxFs) -> PartitionDef {
    PartitionDef {
        label: Some("root".into()),
        size: PartitionSize::remaining(),
        type_code: None,
        content: Some(Content::Filesystem(Fs::linux(fs, "/"))),
    }
}

fn swap(gib: u64) -> PartitionDef {
    PartitionDef {
        label: Some("swap".into()),
        size: PartitionSize::Gib(Size::new(gib)),
        type_code: None,
        content: Some(Content::Swap(SwapContent {
            resume_device: true,
            ..Default::default()
        })),
    }
}

fn luks_root(fs: LinuxFs) -> PartitionDef {
    PartitionDef {
        label: Some("luks".into()),
        size: PartitionSize::remaining(),
        type_code: None,
        content: Some(Content::Luks(
            LuksContent::new(
                "crypted",
                Some(LUKS_PASSWORD_FILE.into()),
                Content::Filesystem(Fs::linux(fs, "/")),
            )
            .allow_discards(),
        )),
    }
}

fn luks_swap(gib: u64) -> PartitionDef {
    PartitionDef {
        label: Some("cryptswap".into()),
        size: PartitionSize::Gib(Size::new(gib)),
        type_code: None,
        content: Some(Content::Luks(
            LuksContent::new(
                "cryptswap",
                Some(LUKS_PASSWORD_FILE.into()),
                Content::Swap(SwapContent {
                    resume_device: true,
                    ..Default::default()
                }),
            )
            .allow_discards(),
        )),
    }
}

// (unencrypted)

fn canonical_no_swap() -> String {
    DiskLayout::canonical(SDA, G40, None)
        .to_nix_module()
        .render()
}

fn canonical_with_swap() -> String {
    DiskLayout::canonical(SDA, G500, Some(Size::new(8)))
        .to_nix_module()
        .render()
}

// LUKS encrypted

fn luks_no_swap() -> String {
    DiskLayout::luks_encrypted(SDA, G40, None, LUKS_PASSWORD_FILE)
        .to_nix_module()
        .render()
}

fn luks_with_swap() -> String {
    DiskLayout::luks_encrypted(SDA, G500, Some(Size::new(8)), LUKS_PASSWORD_FILE)
        .to_nix_module()
        .render()
}

// alternative root filesystems

fn xfs_root() -> String {
    DiskLayout::new(SDA, G500)
        .add_partition(efi())
        .unwrap()
        .add_partition(swap(8))
        .unwrap()
        .add_partition(root(LinuxFs::Xfs))
        .unwrap()
        .to_nix_module()
        .render()
}

fn f2fs_root() -> String {
    DiskLayout::new(SDA, G500)
        .add_partition(efi())
        .unwrap()
        .add_partition(swap(4))
        .unwrap()
        .add_partition(root(LinuxFs::F2fs))
        .unwrap()
        .to_nix_module()
        .render()
}

fn bcachefs_root() -> String {
    DiskLayout::new(SDA, G500)
        .add_partition(efi())
        .unwrap()
        .add_partition(swap(4))
        .unwrap()
        .add_partition(root(LinuxFs::Bcachefs))
        .unwrap()
        .to_nix_module()
        .render()
}

// btrfs

fn btrfs_subvols_content() -> Content {
    Content::Btrfs(BtrfsContent {
        extra_args: vec!["-f".into()],
        mountpoint: None,
        mount_options: vec![],
        subvolumes: BTreeMap::from([
            (
                "@".into(),
                BtrfsSubvolume {
                    mountpoint: Some("/".into()),
                    mount_options: vec!["noatime".into(), "compress=zstd".into()],
                },
            ),
            (
                "@home".into(),
                BtrfsSubvolume {
                    mountpoint: Some("/home".into()),
                    mount_options: vec!["noatime".into()],
                },
            ),
            (
                "@nix".into(),
                BtrfsSubvolume {
                    mountpoint: Some("/nix".into()),
                    mount_options: vec!["noatime".into()],
                },
            ),
            (
                "@var".into(),
                BtrfsSubvolume {
                    mountpoint: Some("/var".into()),
                    mount_options: vec!["noatime".into()],
                },
            ),
            (
                "@snapshots".into(),
                BtrfsSubvolume {
                    mountpoint: Some("/.snapshots".into()),
                    mount_options: vec![],
                },
            ),
        ]),
    })
}

fn btrfs_subvolumes() -> String {
    DiskLayout::new(SDA, G500)
        .add_partition(efi())
        .unwrap()
        .add_partition(swap(8))
        .unwrap()
        .add_partition(PartitionDef {
            label: Some("btrfs".into()),
            size: PartitionSize::remaining(),
            type_code: None,
            content: Some(btrfs_subvols_content()),
        })
        .unwrap()
        .to_nix_module()
        .render()
}

fn btrfs_no_subvols() -> String {
    DiskLayout::new(SDA, G40)
        .add_partition(efi())
        .unwrap()
        .add_partition(PartitionDef {
            label: Some("btrfs".into()),
            size: PartitionSize::remaining(),
            type_code: None,
            content: Some(Content::Btrfs(BtrfsContent {
                extra_args: vec![],
                mountpoint: Some("/".into()),
                mount_options: vec!["noatime".into()],
                subvolumes: BTreeMap::new(),
            })),
        })
        .unwrap()
        .to_nix_module()
        .render()
}

fn luks_btrfs() -> String {
    DiskLayout::new(SDA, G500)
        .add_partition(efi())
        .unwrap()
        .add_partition(PartitionDef {
            label: Some("luks-btrfs".into()),
            size: PartitionSize::remaining(),
            type_code: None,
            content: Some(Content::Luks(
                LuksContent::new(
                    "cryptbtrfs",
                    Some(LUKS_PASSWORD_FILE.into()),
                    btrfs_subvols_content(),
                )
                .allow_discards(),
            )),
        })
        .unwrap()
        .to_nix_module()
        .render()
}

// ── swap variants ─────

fn swap_random_enc() -> String {
    DiskLayout::new(SDA, G500)
        .add_partition(efi())
        .unwrap()
        .add_partition(PartitionDef {
            label: Some("swap".into()),
            size: PartitionSize::Gib(Size::new(4)),
            type_code: None,
            content: Some(Content::Swap(SwapContent {
                random_encryption: true,
                ..Default::default()
            })),
        })
        .unwrap()
        .add_partition(root(LinuxFs::Ext4))
        .unwrap()
        .to_nix_module()
        .render()
}

fn swap_discard_once() -> String {
    DiskLayout::new(SDA, G500)
        .add_partition(efi())
        .unwrap()
        .add_partition(PartitionDef {
            label: Some("swap".into()),
            size: PartitionSize::Gib(Size::new(8)),
            type_code: None,
            content: Some(Content::Swap(SwapContent {
                discard_policy: Some(DiscardPolicy::Once),
                resume_device: true,
                ..Default::default()
            })),
        })
        .unwrap()
        .add_partition(root(LinuxFs::Ext4))
        .unwrap()
        .to_nix_module()
        .render()
}

fn swap_discard_pages() -> String {
    DiskLayout::new(SDA, G500)
        .add_partition(efi())
        .unwrap()
        .add_partition(PartitionDef {
            label: Some("swap".into()),
            size: PartitionSize::Gib(Size::new(8)),
            type_code: None,
            content: Some(Content::Swap(SwapContent {
                discard_policy: Some(DiscardPolicy::Pages),
                ..Default::default()
            })),
        })
        .unwrap()
        .add_partition(root(LinuxFs::Ext4))
        .unwrap()
        .to_nix_module()
        .render()
}

fn swap_priority() -> String {
    DiskLayout::new(SDA, G500)
        .add_partition(efi())
        .unwrap()
        .add_partition(PartitionDef {
            label: Some("swap".into()),
            size: PartitionSize::Gib(Size::new(4)),
            type_code: None,
            content: Some(Content::Swap(SwapContent {
                priority: Some(10),
                resume_device: true,
                ..Default::default()
            })),
        })
        .unwrap()
        .add_partition(root(LinuxFs::Ext4))
        .unwrap()
        .to_nix_module()
        .render()
}

//LVM

fn lvm_pv() -> String {
    DiskLayout::new(SDA, G500)
        .add_partition(efi())
        .unwrap()
        .add_partition(PartitionDef {
            label: Some("lvm".into()),
            size: PartitionSize::remaining(),
            type_code: Some("8E00".into()),
            content: Some(Content::LvmPv { vg: "vg0".into() }),
        })
        .unwrap()
        .to_nix_module()
        .render()
}

// multi-disk

fn multi_disk_plain() -> String {
    let sda = DiskLayout::new(SDA, G500)
        .add_partition(efi())
        .unwrap()
        .add_partition(swap(8))
        .unwrap()
        .add_partition(PartitionDef {
            label: Some("root".into()),
            size: PartitionSize::Gib(Size::new(80)),
            type_code: None,
            content: Some(Content::Filesystem(Fs::linux(LinuxFs::Ext4, "/"))),
        })
        .unwrap();

    let sdb = DiskLayout::new(SDB, G500)
        .add_partition(PartitionDef {
            label: Some("home".into()),
            size: PartitionSize::remaining(),
            type_code: None,
            content: Some(Content::Filesystem(Fs::linux(LinuxFs::Ext4, "/home"))),
        })
        .unwrap();

    layouts_to_nix_module(&[sda, sdb]).render()
}

fn multi_disk_luks_root() -> String {
    let sda = DiskLayout::new(SDA, G500)
        .add_partition(efi())
        .unwrap()
        .add_partition(luks_swap(8))
        .unwrap()
        .add_partition(luks_root(LinuxFs::Ext4))
        .unwrap();

    let sdb = DiskLayout::new(SDB, G500)
        .add_partition(PartitionDef {
            label: Some("home".into()),
            size: PartitionSize::remaining(),
            type_code: None,
            content: Some(Content::Filesystem(Fs::linux(LinuxFs::Xfs, "/home"))),
        })
        .unwrap();

    layouts_to_nix_module(&[sda, sdb]).render()
}
