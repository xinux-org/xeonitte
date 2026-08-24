#[path = "disk/device.rs"]
mod device;
#[path = "disk/fs.rs"]
pub mod fs;
#[path = "disk/generate.rs"]
pub mod generate;
#[path = "disk/layout.rs"]
pub mod layout;
#[path = "disk/ops.rs"]
pub mod ops;
#[path = "disk/size.rs"]
pub mod size;

pub use device::{Disk, Partition};
pub use fs::{
    BtrfsContent, BtrfsSubvolume, Content, DiscardPolicy, Fs, LinuxFs, LuksContent, SwapContent,
};
pub use generate::disko_from_schema;
pub use layout::{DiskLayout, LayoutError, PartitionDef, SizeOp, layouts_to_nix_module};
pub use ops::{LUKS_PASSWORD_FILE, unmount, write_file, write_luks_key};
pub use size::{GiB, KiB, MiB, PartitionSize, Size, SizeUnit, TiB};
