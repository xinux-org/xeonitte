use anyhow::{Context, Result};
use clap::{self, FromArgMatches, Subcommand};
use log::error;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs::{self, File},
    io::{self, Read, Write},
    os::unix::fs::OpenOptionsExt,
    path::Path,
    process::Command,
};

const TMPDIR: &str = "/nix/var/nix/builds/xeonitte";

#[derive(Serialize, Clone)]
struct Disk {
    name: String,
    size: u64,
    partitions: Vec<Partition>,
}

#[derive(Serialize, Clone)]
struct Partition {
    name: String,
    format: String,
    size: u64,
}

#[derive(Deserialize, Debug, Clone)]
pub struct FullDiskOptions {
    pub device: String,
    pub encryption: bool,
    pub passphrase: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct CustomOptions {
    pub partitions: HashMap<String, CustomPartition>,
    pub encryption: bool,
    pub passphrase: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
pub enum PartitionSchema {
    FullDisk(FullDiskOptions),
    Custom(CustomOptions),
}

#[derive(Deserialize, Debug, Clone)]
pub struct CustomPartition {
    pub format: Option<String>,
    pub mountpoint: Option<String>,
    pub device: String,
}

#[derive(Subcommand, Debug)]
enum SubCommands {
    GetPartitions {},
    WriteFile {
        #[clap(short, long)]
        path: String,
        #[clap(short, long)]
        contents: String,
    },
    WriteLuksKey {
        #[clap(short, long)]
        path: String,
    },
    Unmount {},
}

fn main() {
    let cli = SubCommands::augment_subcommands(clap::Command::new(
        "Helper binary for Xeonitte installer",
    ));
    let matches = cli.get_matches();
    let derived_subcommands = SubCommands::from_arg_matches(&matches)
        .map_err(|err| err.exit())
        .unwrap();

    if uzers::get_effective_uid() != 0 {
        error!("xeonitte-helper must be run as root");
        std::process::exit(1);
    }

    match derived_subcommands {
        SubCommands::GetPartitions {} => {
            let mut outdisks = vec![];

            let mut devicevec = vec![];
            let devices = libparted::Device::devices(true);
            for device in devices {
                devicevec.push(device);
            }
            devicevec.sort_by(|a, b| a.path().to_str().cmp(&b.path().to_str()));
            for mut device in devicevec {
                let sectorsize = device.sector_size();
                let mut disk = Disk {
                    name: device.path().to_str().unwrap().to_string(),
                    size: device.length() * sectorsize,
                    partitions: vec![],
                };
                if let Ok(partdisk) = libparted::Disk::new(&mut device) {
                    let mut partvec = vec![];
                    for part in partdisk.parts() {
                        if part.get_path().is_none() {
                            continue;
                        }
                        partvec.push(part);
                    }
                    partvec.sort_by(|a, b| a.get_path().cmp(&b.get_path()));
                    for part in partvec {
                        disk.partitions.push(Partition {
                            name: part.get_path().unwrap().to_string_lossy().to_string(),
                            format: part.fs_type_name().unwrap_or("unknown").to_string(),
                            size: (part.geom_length() as u64) * sectorsize,
                        });
                    }
                }
                outdisks.push(disk);
            }
            println!("{}", serde_json::to_string(&outdisks).unwrap());
        }
        SubCommands::WriteFile { path, contents } => {
            fs::create_dir_all(path.rsplitn(2, '/').last().unwrap()).unwrap();
            let mut file = File::create(path).unwrap();
            file.write_all(contents.as_bytes()).unwrap();
        }
        SubCommands::WriteLuksKey { path } => {
            let write = || -> Result<()> {
                // Read the passphrase from stdin.
                let mut passphrase = Vec::new();
                io::stdin()
                    .read_to_end(&mut passphrase)
                    .context("failed to read passphrase from stdin")?;
                if passphrase.last() == Some(&b'\n') {
                    passphrase.pop();
                }

                if let Some(parent) = Path::new(&path).parent() {
                    fs::create_dir_all(parent).context("failed to create key file directory")?;
                }
                let mut file = fs::OpenOptions::new()
                    .write(true)
                    .create(true)
                    .truncate(true)
                    .mode(0o600)
                    .open(&path)
                    .context("failed to create key file")?;
                file.write_all(&passphrase)
                    .context("failed to write passphrase")?;
                file.sync_all().context("failed to flush key file")?;
                Ok(())
            };
            if let Err(e) = write() {
                eprintln!("failed to write LUKS key file: {e:#}");
                std::process::exit(1);
            }
        }

        SubCommands::Unmount {} => {
            if let Err(e) = Command::new("umount")
                .arg("-R")
                .arg("-f")
                .arg(TMPDIR)
                .output()
            {
                error!("Failed to unmount: {}", e);
                std::process::exit(1);
            }
        }
    }
}
