use anyhow::{anyhow, Context, Result};
use clap::{self, FromArgMatches, Subcommand};
use disk_types::{BlockDeviceExt, FileSystem, PartitionTable, PartitionType, Sector, SectorExt};
use distinst_disks::{DiskExt, PartitionBuilder, PartitionFlag};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs::{self, File},
    io::{self, Read, Write},
    process::{Command, Stdio},
};

#[derive(Serialize)]
struct Disk {
    name: String,
    size: u64,
    partitions: Vec<Partition>,
}

#[derive(Serialize)]
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
    Partition {},
    WriteFile {
        #[clap(short, long)]
        path: String,
        #[clap(short, long)]
        contents: String,
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

    if users::get_effective_uid() != 0 {
        eprintln!("xeonitte-helper must be run as root");
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
        SubCommands::Partition {} => {
            if let Err(e) = partition() {
                eprintln!("Partitioning failed: {:#}", e);
                std::process::exit(1);
            }
        }
        SubCommands::WriteFile { path, contents } => {
            fs::create_dir_all(path.rsplitn(2, '/').last().unwrap()).unwrap();
            let mut file = File::create(path).unwrap();
            file.write_all(contents.as_bytes()).unwrap();
        }
        SubCommands::Unmount {} => {
            // Close LUKS container if it exists
            // when you close the installation GUI or if the installation fails
            let _ = Command::new("cryptsetup")
                .args(["close", "cryptroot"])
                .output();

            if let Err(e) = Command::new("umount")
                .arg("-R")
                .arg("-f")
                .arg("/tmp/xeonitte")
                .output()
            {
                eprintln!("Failed to unmount: {}", e);
                std::process::exit(1);
            }
        }
    }
}

fn partition() -> Result<()> {
    let stdin = io::stdin();
    let mut buf = String::new();
    stdin.lock().read_to_string(&mut buf)?;

    let schema: PartitionSchema = serde_json::from_str(&buf)?;

    match schema {
        PartitionSchema::FullDisk(full_disk_options) => {
            let start_sector = Sector::Start;
            let end_sector = Sector::End;
            let boot_sector = Sector::Unit(2_097_152);

            println!("Partition: Finding disk");
            let mut dev = distinst_disks::Disk::from_name(&full_disk_options.device).map_or_else(
                |disk_error| Err(anyhow!("Failed to find disk {disk_error:?}")),
                |disk| Ok(disk),
            )?;
            let efi = distinst_disks::Bootloader::detect() == distinst_disks::Bootloader::Efi;

            // Create partition table and partitions
            if efi {
                println!("Partition: Creating GPT partition table");
                dev.mklabel(PartitionTable::Gpt)
                    .ok()
                    .ok_or_else(|| anyhow!("Failed to create GPT partition table"))?;

                println!("Partition: Creating EFI partition");
                dev.add_partition(
                    PartitionBuilder::new(
                        dev.get_sector(start_sector),
                        dev.get_sector(boot_sector),
                        FileSystem::Fat32,
                    )
                    .partition_type(PartitionType::Primary)
                    .flag(PartitionFlag::PED_PARTITION_ESP),
                )
                .ok()
                .ok_or_else(|| anyhow!("Failed to create EFI partition"))?;
            } else {
                println!("Partition: Creating MBR partition table");
                dev.mklabel(PartitionTable::Msdos)
                    .ok()
                    .ok_or_else(|| anyhow!("Failed to create MBR partition table"))?;
            }

            println!("Partition: Creating root partition");
            // Add root partition
            dev.add_partition(
                PartitionBuilder::new(
                    dev.get_sector(if efi { boot_sector } else { start_sector }),
                    dev.get_sector(end_sector),
                    FileSystem::Ext4,
                )
                .partition_type(PartitionType::Primary),
            )
            .ok()
            .ok_or_else(|| anyhow!("Failed to create root partition"))?;

            println!("Partition: Committing changes");
            dev.commit()
                .ok()
                .ok_or_else(|| anyhow!("Failed to commit changes"))?
                .context("Failed to get partitions")?;

            // Update kernel partition table
            println!("Partition: Updating kernel partition table");
            let _ = Command::new("partprobe")
                .arg(&full_disk_options.device)
                .output();

            let _ = Command::new("udevadm")
                .args(["settle", "--timeout=10"])
                .output();

            // Determine partition paths early
            let partition_val = if full_disk_options.device.contains("nvme")
                || full_disk_options.device.contains("mmcblk")
            {
                "p"
            } else {
                ""
            };

            let (efi_partition, root_partition) = if efi {
                (
                    Some(format!("{}{}1", &full_disk_options.device, partition_val)),
                    format!("{}{}2", &full_disk_options.device, partition_val),
                )
            } else {
                (
                    None,
                    format!("{}{}1", &full_disk_options.device, partition_val),
                )
            };

            // Format EFI partition
            if let Some(efi_part) = &efi_partition {
                println!("Partition: Formatting EFI partition: {}", efi_part);
                let output = Command::new("mkfs.vfat")
                    .arg("-F32")
                    .arg("-I")
                    .arg(efi_part)
                    .output()
                    .context("Failed to format EFI partition")?;
                if !output.status.success() {
                    return Err(anyhow!(
                        "Failed to format EFI: {}",
                        String::from_utf8_lossy(&output.stderr)
                    ));
                }
            }

            let root_mount_device = if full_disk_options.encryption {
                let passphrase = full_disk_options
                    .passphrase
                    .as_deref()
                    .ok_or_else(|| anyhow!("Encryption enabled but no passphrase provided"))?;
                println!(
                    "Partition: Setting up LUKS on root partition: {}",
                    root_partition
                );
                setup_luks(&root_partition, passphrase)?;

                println!("Partition: Formatting LUKS container");
                let output = Command::new("mkfs.ext4")
                    .arg("-F")
                    .arg("/dev/mapper/cryptroot")
                    .output()
                    .context("Failed to format LUKS container")?;
                if !output.status.success() {
                    return Err(anyhow!(
                        "Failed to format LUKS: {}",
                        String::from_utf8_lossy(&output.stderr)
                    ));
                }

                "/dev/mapper/cryptroot".to_string()
            } else {
                println!("Partition: Formatting root partition: {}", root_partition);
                let output = Command::new("mkfs.ext4")
                    .arg("-F")
                    .arg(&root_partition)
                    .output()
                    .context("Failed to format root partition")?;
                if !output.status.success() {
                    return Err(anyhow!(
                        "Failed to format root: {}",
                        String::from_utf8_lossy(&output.stderr)
                    ));
                }
                root_partition
            };

            // Mount root
            println!("Partition: Mounting root: {}", root_mount_device);
            fs::create_dir_all("/tmp/xeonitte")?;
            let output = Command::new("mount")
                .arg(&root_mount_device)
                .arg("/tmp/xeonitte")
                .output()
                .context("Failed to mount root")?;
            if !output.status.success() {
                return Err(anyhow!(
                    "Failed to mount root: {}",
                    String::from_utf8_lossy(&output.stderr)
                ));
            }

            // Mount EFI
            if let Some(efi_part) = &efi_partition {
                println!("Partition: Mounting EFI: {}", efi_part);
                fs::create_dir_all("/tmp/xeonitte/boot")?;
                let output = Command::new("mount")
                    .arg("-o")
                    .arg("umask=0077")
                    .arg(efi_part)
                    .arg("/tmp/xeonitte/boot")
                    .output()
                    .context("Failed to mount EFI")?;
                if !output.status.success() {
                    return Err(anyhow!(
                        "Failed to mount EFI: {}",
                        String::from_utf8_lossy(&output.stderr)
                    ));
                }
            }
        }
        PartitionSchema::Custom(custom_disk_options) => {
            let partitions = &custom_disk_options.partitions;

            let mut devices = HashMap::new();
            for (path, custom_partition) in partitions {
                if !devices.contains_key(&custom_partition.device) {
                    let dev = distinst_disks::Disk::from_name(&custom_partition.device)
                        .map_or_else(
                            |disk_error| {
                                Err(anyhow!(
                                    "Failed to find disk {} -> {disk_error:?}",
                                    custom_partition.device
                                ))
                            },
                            |disk| Ok(disk),
                        )?;

                    devices.insert(custom_partition.device.to_string(), (dev, vec![]));
                }
                let partvec = &mut devices.get_mut(&custom_partition.device).unwrap().1;
                partvec.push((path, custom_partition));
                partvec.sort_by(|a, b| a.0.cmp(b.0));
            }

            // Loop through each modified disk
            for (device, (mut dev, disk_partitions)) in devices {
                println!("Partitions: Partitioning disk {}", device);
                for (part, custom_partition) in &disk_partitions {
                    let partition = dev
                        .partitions
                        .iter()
                        .find(|x| x.get_device_path().to_str() == Some(*part))
                        .ok_or_else(|| anyhow!("Failed to find partition {}", part))?;
                    let num = &partition.number;

                    // Skip formatting root if encryption is enabled (LUKS will handle it)
                    let is_root = custom_partition.mountpoint.as_deref() == Some("/");
                    if is_root && custom_disk_options.encryption {
                        continue;
                    }

                    if let Some(format) =
                        &custom_partition
                            .format
                            .as_ref()
                            .and_then(|x| match x.as_str() {
                                "btrfs" => Some(FileSystem::Btrfs),
                                "ext4" => Some(FileSystem::Ext4),
                                "ext3" => Some(FileSystem::Ext3),
                                "fat32" => Some(FileSystem::Fat32),
                                "ntfs" => Some(FileSystem::Ntfs),
                                "xfs" => Some(FileSystem::Xfs),
                                "swap" => Some(FileSystem::Swap),
                                _ => None,
                            })
                    {
                        dev.format_partition(*num, *format).map_or_else(
                            |disk_error| {
                                Err(anyhow!(
                                    "Failed to format partition {} -> {disk_error:?}",
                                    part
                                ))
                            },
                            |_| Ok(()),
                        )?;
                        if let Some(mountpoint) = &custom_partition.mountpoint {
                            if mountpoint == "/boot" {
                                let partition = dev
                                    .partitions
                                    .iter_mut()
                                    .find(|x| x.get_device_path().to_str() == Some(*part))
                                    .ok_or_else(|| anyhow!("Failed to find partition {}", part))?;
                                partition.flags.push(PartitionFlag::PED_PARTITION_ESP);
                            }
                        }
                    }
                }

                println!("Partitions: Committing changes");
                dev.commit().map_or_else(
                    |disk_error| {
                        Err(anyhow!(
                            "Failed to commit changes to disk: {} - {disk_error:?}",
                            device
                        ))
                    },
                    |_| Ok(()),
                )?;
                // .context("Failed to commit")?;

                println!("Partitions: Updating kernel partition table");
                let _ = Command::new("partprobe").arg(&device).output()?;

                let _ = Command::new("udevadm")
                    .args(["settle", "--timeout=10"])
                    .output()?;

                dev.reload().map_or_else(
                    |disk_error| Err(anyhow!("Failed to reload disk {} {disk_error:?}", device)),
                    |_| Ok(()),
                )?;
            }

            // Find root partition before formatting to handle LUKS
            let root_entry = partitions
                .iter()
                .find(|(_, p)| p.mountpoint.as_deref() == Some("/"));
            let root_partition_path = root_entry.map(|(path, _)| path.clone());

            // Setup LUKS on root partition if encryption is enabled
            if custom_disk_options.encryption {
                if let Some(root_path) = &root_partition_path {
                    let passphrase = custom_disk_options
                        .passphrase
                        .as_deref()
                        .ok_or_else(|| anyhow!("Encryption enabled but no passphrase provided"))?;
                    println!(
                        "Partitions: Setting up LUKS on root partition: {}",
                        root_path
                    );
                    setup_luks(root_path, passphrase)?;

                    let root_format = partitions
                        .get(root_path)
                        .and_then(|p| p.format.as_deref())
                        .unwrap_or("ext4");

                    println!("Partitions: Formatting LUKS container as {}", root_format);
                    let mkfs_cmd = match root_format {
                        "btrfs" => "mkfs.btrfs",
                        "ext3" => "mkfs.ext3",
                        "xfs" => "mkfs.xfs",
                        _ => "mkfs.ext4",
                    };
                    let output = Command::new(mkfs_cmd)
                        .arg("-f")
                        .arg("/dev/mapper/cryptroot")
                        .output()
                        .context("Failed to format LUKS container")?;
                    if !output.status.success() {
                        return Err(anyhow!(
                            "Failed to format LUKS: {}",
                            String::from_utf8_lossy(&output.stderr)
                        ));
                    }
                }
            }

            println!("Partitions: Mounting partitions");
            let mut mountvec: Vec<_> = partitions.iter().collect();
            mountvec.sort_by(|a, b| {
                // Sort by mountpoint length, shortest first
                let a = a.1.mountpoint.as_ref().map(|x| x.len()).unwrap_or(0);
                let b = b.1.mountpoint.as_ref().map(|x| x.len()).unwrap_or(0);
                a.cmp(&b)
            });

            for (part, custom) in mountvec {
                if custom.format == Some("swap".to_string()) {
                    println!("Partitions: Enabling swap: {}", part);
                    let _output = Command::new("swapon")
                        .arg(part)
                        .output()
                        .context("Failed to enable swap")?;
                    continue;
                }

                if let Some(target) = &custom.mountpoint {
                    fs::create_dir_all(format!("/tmp/xeonitte{}", target))
                        .context("Failed to create mountpoint")?;

                    // Use encrypted device for root if encryption is enabled
                    let mount_device = if target == "/" && custom_disk_options.encryption {
                        "/dev/mapper/cryptroot".to_string()
                    } else {
                        part.clone()
                    };

                    println!("Partitions: Mounting {} to {}", mount_device, target);

                    let output = if target == "/boot" {
                        Command::new("mount")
                            .arg("-o")
                            .arg("umask=0077")
                            .arg(&mount_device)
                            .arg(format!("/tmp/xeonitte{}", target))
                            .output()
                            .context("Failed to mount partition")?
                    } else {
                        Command::new("mount")
                            .arg(&mount_device)
                            .arg(format!("/tmp/xeonitte{}", target))
                            .output()
                            .context("Failed to mount partition")?
                    };

                    if !output.status.success() {
                        return Err(anyhow!(
                            "Failed to mount {}: {}",
                            target,
                            String::from_utf8_lossy(&output.stderr)
                        ));
                    }
                }
            }
        }
    }
    Ok(())
}

fn setup_luks(device: &str, passphrase: &str) -> Result<()> {
    // Checking if device already have LUKS container
    println!("LUKS: Checking if {} has existing LUKS container", device);
    let is_luks = Command::new("cryptsetup")
        .args(["isLuks", device])
        .output()
        .context("Failed to check for LUKS container")?;

    // Wipe LUKS container if it exists
    if is_luks.status.success() {
        println!("LUKS: Found existing LUKS container on {}", device);
        let _output = Command::new("cryptsetup")
            .args(["close", "cryptroot"])
            .output()?;

        println!("LUKS: Wiping signatures from {}", device);
        let wipe_output = Command::new("wipefs")
            .args(["-a", device])
            .output()
            .context("Failed to wipe device signatures")?;

        if !wipe_output.status.success() {
            return Err(anyhow!(
                "Failed to wipe device {}: {}",
                device,
                String::from_utf8_lossy(&wipe_output.stderr)
            ));
        }
        println!("LUKS: Device {} cleaned successfully", device);
    }

    println!("LUKS: Formatting {} as LUKS2", device);
    let mut child = Command::new("cryptsetup")
        .args(["luksFormat", "--type", "luks2", "-q", device])
        .stdin(Stdio::piped())
        .spawn()
        .context("Failed to start cryptsetup luksFormat")?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(passphrase.as_bytes())?;
    }

    let status = child.wait()?;
    if !status.success() {
        return Err(anyhow!("cryptsetup luksFormat failed"));
    }

    println!("LUKS: Opening {} as {}", device, "cryptroot");
    let mut child = Command::new("cryptsetup")
        .args(["open", device, "cryptroot"])
        .stdin(Stdio::piped())
        .spawn()
        .context("Failed to start cryptsetup open")?;

    // why this is the theme write action above line in 586 ?
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(passphrase.as_bytes())?;
    }

    let status = child.wait()?;
    if !status.success() {
        return Err(anyhow!("cryptsetup open failed"));
    }

    Ok(())
}
