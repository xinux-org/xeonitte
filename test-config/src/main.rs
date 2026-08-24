use std::path::PathBuf;

use anyhow::{Context, Result};
use log::error;
use xeonitte::modules::{disk::*, nix::*};

const TMPDIR: &str = "/nix/var/nix/builds/xeonitte";


const GB: u64 = 1024 * 1024 * 1024;

fn simple_part(label: &str, gib: u64) -> PartitionDef {
    PartitionDef {
        label: Some(label.into()),
        size: PartitionSize::Gib(Size::new(gib)),
        type_code: None,
        content: Some(Content::Filesystem(Fs::linux(LinuxFs::Ext4, "/"))),
    }
}

fn disk_to_nix_mod(d: &Disk) -> NixModule {
    let rr: String = 'a'.to_string();
    DiskLayout::new(d.id_path(), d.size)
        .add_partition(simple_part("a", 42))
        .unwrap()
        .add_partition(simple_part("b", 42))
        .unwrap().to_nix_module()
}

fn disk_to_layout(d: &Disk) -> DiskLayout {
    DiskLayout::new(d.id_path(), d.size)
}

fn dvc_list() {
    // let r: Vec<NixModule> = Disk::list().iter().map(disk_to_layout).collect();
    // println!("Disks {:?}", r);
}

pub fn samp() {


    let total = 100 * GB;
    let layout = DiskLayout::new("/dev/sda", total)
        .add_partition(simple_part("a", 30))
        .unwrap()
        .add_partition(simple_part("b", 20))
        .unwrap();


    // layouts_to_nix_module


    let r: Vec<NixModule> = Disk::list().iter().map(disk_to_nix_mod).collect();
    let a = r.iter().fold(NixModule::new(), |x, y| x.merge(y.clone()));
    let bp =  match std::path::absolute("test-config/generated/simple.nix") {
        Ok(d) => d,
        Err(_) => PathBuf::new()
    };
    println!("data: {:?}", bp);
    

    let smp2: Vec<DiskLayout> = Disk::list().iter().map(disk_to_layout).collect();
    let sm = layouts_to_nix_module(&smp2);
    std::fs::write(bp, a.render());
    println!("done");
}

fn main() {
    samp()
}
