use super::parse::ConfigType;
use super::report::ErrorPhase;
use crate::flow::Choice;
use crate::{
    config::{LIBEXECDIR, SYSCONFDIR, TMPDIR},
    ui::{
        install::install_model::{INSTALL_BROKER, InstallMsg},
        partitions::partition_model::PartitionSchema,
        window::{AppMsg, UserConfig},
    },
    utils::disko::{Devices, LUKS_PASSWORD_FILE},
};
use anyhow::{Context, Result, anyhow};
use log::{debug, error, info};
use relm4::*;
use std::{
    collections::HashMap,
    fs::{self},
    io::Write,
    process::{Command, Stdio},
};

pub struct InstallAsyncModel {
    username: Option<String>,
    password: Option<String>,
    rootpassword: Option<String>,
    postinstall_commands: Vec<String>,
}

#[derive(Debug)]
pub enum InstallAsyncMsg {
    Install(
        String,
        Option<String>,
        Option<String>,
        Option<String>,
        Box<Option<PartitionSchema>>,
        Box<Option<UserConfig>>,
        HashMap<String, HashMap<String, Choice>>, // Listconfig
        ConfigType,
        bool,
        Devices,
    ),
    FinishInstall(
        Option<String>, //timezone,
        bool,           // Imperative timezone
        Vec<String>,    // Commands
    ),
    RunNextCommand,
}

impl Worker for InstallAsyncModel {
    type Init = ();
    type Input = InstallAsyncMsg;
    type Output = AppMsg;

    fn init(_parent_window: Self::Init, _sender: ComponentSender<Self>) -> Self {
        InstallAsyncModel {
            username: None,
            password: None,
            rootpassword: None,
            postinstall_commands: vec![],
        }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>) {
        match msg {
            InstallAsyncMsg::Install(
                id,
                language,
                timezone,
                keyboard,
                partitions,
                user,
                listconfig,
                configtype,
                imperative_timezone,
                disko_config,
            ) => {
                self.username = user.as_ref().as_ref().map(|u| u.username.clone());
                self.password = user.as_ref().as_ref().map(|u| u.password.clone());
                self.rootpassword = user.as_ref().as_ref().and_then(|u| u.rootpassword.clone());
                let hostname = user
                    .as_ref()
                    .as_ref()
                    .map(|u| u.hostname.clone())
                    .unwrap_or_else(|| "nixos".to_string());
                let archout = match Command::new("uname")
                    .arg("-m")
                    .output()
                    .context("Failed to get architecture")
                {
                    Ok(o) => o,
                    Err(e) => {
                        sender.output(AppMsg::error(
                            ErrorPhase::Setup,
                            format!("Failed to get architecture: {e}"),
                        ));
                        return;
                    }
                };
                let arch = String::from_utf8_lossy(&archout.stdout).trim().to_string();

                // Step 0: Clear TMPDIR
                info!("Step 0: Clear {}", TMPDIR);
                INSTALL_BROKER.send(InstallMsg::ProgressbarTitle(
                    "Step 0: clearing up /nix/var/nix/builds/xeonitte folder".to_string(),
                ));
                fn clear() -> Result<()> {
                    Command::new("pkexec")
                        .arg("umount")
                        .arg("-R")
                        .arg(TMPDIR)
                        .output()?;
                    Command::new("pkexec")
                        .arg("rm")
                        .arg("-rf")
                        .arg(TMPDIR)
                        .output()?;
                    Ok(())
                }
                if let Err(e) = clear() {
                    sender.output(AppMsg::error(
                        ErrorPhase::Setup,
                        format!("Failed to clear {TMPDIR}: {e}"),
                    ));
                    return;
                }

                // Step 2: Generate base config
                let Ok(_) = Command::new("pkexec")
                    .arg("mkdir")
                    .arg("-p")
                    .arg(format!("{}/etc/nixos", TMPDIR))
                    .output()
                    .context("cannot create etc/nixos")
                else {
                    sender.output(AppMsg::error(
                        ErrorPhase::Configuration,
                        format!("Failed to create {TMPDIR}/etc/nixos directory"),
                    ));
                    return;
                };

                info!("Step 2: Generate base config");
                INSTALL_BROKER.send(InstallMsg::ProgressbarTitle(
                    "Step 2: Generate base config".to_string(),
                ));
                if let Err(e) = Command::new("pkexec")
                    .arg("nixos-generate-config")
                    .arg("--root")
                    .arg(TMPDIR)
                    .output()
                {
                    sender.output(AppMsg::error(
                        ErrorPhase::Configuration,
                        format!("Failed to generate base config: {e}"),
                    ));
                    return;
                }

                if configtype == ConfigType::Xinux {
                    // Move /nix/var/nix/builds/xeonitte/etc/nixos/hardware-configuration.nix to /nix/var/nix/builds/xeonitte/etc/nixos/systems/{ARCH}-linux/{HOSTNAME}/hardware.nix
                    let Ok(_) = Command::new("pkexec")
                        .arg("mkdir")
                        .arg("-p")
                        .arg(format!(
                            "{}/etc/nixos/systems/{}-linux/{}",
                            TMPDIR, arch, hostname
                        ))
                        .output()
                    else {
                        sender.output(AppMsg::error(
                            ErrorPhase::Configuration,
                            "Failed to create nixos config directory",
                        ));
                        return;
                    };
                    let Ok(_) = Command::new("pkexec")
                        .arg("mv")
                        .arg(format!("{}/etc/nixos/hardware-configuration.nix", TMPDIR))
                        .arg(format!(
                            "{}/etc/nixos/systems/{}-linux/{}/hardware.nix",
                            TMPDIR, arch, hostname
                        ))
                        .output()
                    else {
                        sender.output(AppMsg::error(
                            ErrorPhase::Configuration,
                            "Failed to move nixos hardware config file",
                        ));
                        return;
                    };

                    // Remove /tmp/xeonitte/etc/nixos/configuration.nix
                    let Ok(_) = Command::new("pkexec")
                        .arg("rm")
                        .arg(format!("{}/etc/nixos/configuration.nix", TMPDIR))
                        .output()
                    else {
                        sender.output(AppMsg::error(
                            ErrorPhase::Configuration,
                            "Failed to remove default configuration.nix",
                        ));
                        return;
                    };
                }

                // Step 3: Make configuration base on language, timezone, keyboard, and user
                info!("Step 3: Make configuration");
                INSTALL_BROKER.send(InstallMsg::ProgressbarTitle(
                    "Step 3: Make configuration".to_string(),
                ));
                let mut mbrdisk = None;
                if let Some(partitions) = partitions.as_ref() {
                    match partitions {
                        PartitionSchema::FullDisk(disk) => {
                            mbrdisk = Some(disk.device.clone());
                        }
                        PartitionSchema::Custom(options) => {
                            for part in options.partitions.values() {
                                if part.mountpoint == Some("/".to_string()) {
                                    mbrdisk = Some(part.device.to_string());
                                }
                            }
                        }
                    }
                }

                if let Err(e) = makeconfig(MakeConfig {
                    id,
                    language,
                    timezone,
                    keyboard,
                    user: *user.clone(),
                    list: listconfig,
                    bootdisk: mbrdisk,
                    imperative_timezone,
                    disko: disko_config.to_nix_module(),
                }) {
                    sender.output(AppMsg::error(
                        ErrorPhase::Configuration,
                        format!("Failed to make config: {e}"),
                    ));
                    return;
                }

                info!("Step 3.1: Backup xeonitte");
                INSTALL_BROKER.send(InstallMsg::ProgressbarTitle(
                    "Step 3.1: Backup xeonitte generated configs into /xeonitte".to_string(),
                ));
                if let Err(e) = backup_config() {
                    sender.output(AppMsg::error(
                        ErrorPhase::Configuration,
                        format!("Failed to create backup flake: {e}"),
                    ));
                    return;
                }
                // Step 4: Install NixOS
                info!("Step 4: Install Xinux");
                INSTALL_BROKER.send(InstallMsg::ProgressbarTitle(
                    "Step 4: Install Xinux".to_string(),
                ));
                if let Some(hostname) = user.as_ref().as_ref().map(|u| u.hostname.clone()) {
                    let flake_dir = format!("{}/etc/nixos", TMPDIR);
                    let flake_uri = format!("{}#{}", flake_dir, hostname);

                    let disko_path = format!(
                        "{}/etc/nixos/systems/{}-linux/{}/disko.nix",
                        TMPDIR, arch, hostname
                    );

                    let luks_passphrase =
                        partitions
                            .as_ref()
                            .as_ref()
                            .and_then(|schema| match schema {
                                PartitionSchema::FullDisk(opts) => opts.passphrase.clone(),
                                PartitionSchema::Custom(opts) => opts.passphrase.clone(),
                            });
                    if let Some(passphrase) = &luks_passphrase {
                        info!("Step 4.1: Write LUKS key file");
                        fn write_luks_key(passphrase: &str) -> Result<()> {
                            let mut child = Command::new("pkexec")
                                .arg(format!("{}/xeonitte-helper", LIBEXECDIR))
                                .arg("write-luks-key")
                                .arg("--path")
                                .arg(LUKS_PASSWORD_FILE)
                                .stdin(Stdio::piped())
                                .spawn()?;
                            child
                                .stdin
                                .as_mut()
                                .context("Failed to open helper stdin")?
                                .write_all(passphrase.as_bytes())?;
                            if !child.wait()?.success() {
                                return Err(anyhow!("xeonitte-helper write-luks-key failed"));
                            }
                            Ok(())
                        }
                        if let Err(e) = write_luks_key(passphrase) {
                            sender.output(AppMsg::error(
                                ErrorPhase::Installation,
                                format!("Failed to write LUKS key file: {e}"),
                            ));
                            return;
                        }
                    }

                    // TODO: make better way to write this shell command
                    let cmd = if luks_passphrase.is_some() {
                        format!(
                            "swapoff -a || true; nix run https://git.oss.uzinfocom.uz/mirrors/disko/archive/latest.tar.gz -- --mode destroy,format,mount {disko_path} --yes-wipe-all-disks; disko_rc=$?; shred -u -z -n 0 {key}; [ \"$disko_rc\" -eq 0 ] && nix flake lock {flake_dir} && nixos-install --no-root-passwd --no-channel-copy --root /mnt --option build-dir /nix/var/nix/builds/xeonitte --flake {flake_uri} && mkdir -p /mnt/etc/nixos && cp -rT {flake_dir} /mnt/etc/nixos",
                            key = LUKS_PASSWORD_FILE,
                        )
                    } else {
                        format!(
                            "swapoff -a || true; nix run https://git.oss.uzinfocom.uz/mirrors/disko/archive/latest.tar.gz -- --mode destroy,format,mount {disko_path} --yes-wipe-all-disks && nix flake lock {flake_dir} && nixos-install --no-root-passwd --no-channel-copy --root /mnt --option build-dir /nix/var/nix/builds/xeonitte --flake {flake_uri} && mkdir -p /mnt/etc/nixos && cp -rT {flake_dir} /mnt/etc/nixos",
                        )
                    };
                    INSTALL_BROKER.send(InstallMsg::Install(vec![
                        "/usr/bin/env".to_string(),
                        "pkexec".to_string(),
                        "sh".to_string(),
                        "-c".to_string(),
                        cmd,
                    ]));
                } else {
                    sender.output(AppMsg::error(ErrorPhase::Installation, "No hostname found"));
                }
            }
            InstallAsyncMsg::FinishInstall(timezone, imperative_timezone, mut commands) => {
                // Step 5: Set user passwords
                info!("Step 5: Set user passwords");
                INSTALL_BROKER.send(InstallMsg::ProgressbarTitle(
                    "Step 5: Set user passwords".to_string(),
                ));

                if let Err(e) = setuserpasswd(self.username.clone(), self.password.clone()) {
                    sender.output(AppMsg::error(
                        ErrorPhase::PostInstall,
                        format!("Failed to set user password: {e}"),
                    ));
                    return;
                }

                // Step 6: Set root password
                info!("Step 6: Set root password if specified");
                INSTALL_BROKER.send(InstallMsg::ProgressbarTitle(
                    "Step 6: Set root password if specified".to_string(),
                ));
                if let Some(rootpasswd) = &self.rootpassword
                    && let Err(e) =
                        setuserpasswd(Some("root".to_string()), Some(rootpasswd.clone()))
                {
                    sender.output(AppMsg::error(
                        ErrorPhase::PostInstall,
                        format!("Failed to set root password: {e}"),
                    ));
                    return;
                }

                if imperative_timezone && let Some(timezone) = timezone {
                    commands.insert(
                        0,
                        format!("ln -sf ../etc/zoneinfo/{} /etc/localtime", timezone),
                    );
                }
                let Some(username) = self.username.clone() else {
                    sender.output(AppMsg::error(
                        ErrorPhase::PostInstall,
                        "Username is not set",
                    ));
                    return;
                };

                commands.push(format!(
                    "mkdir -p /home/{}/.config", // avoid not found error
                    &username,
                ));
                commands.push(format!(
                    "chown -R {}:users /home/{}/.config", // path relative to chroot
                    &username, &username,
                ));

                // Step 6.1: Set libreoffice config
                info!("Step 6.1: Set libreoffice config");
                INSTALL_BROKER.send(InstallMsg::ProgressbarTitle(
                    "Step 6.1: Set libreoffice config".to_string(),
                ));
                if let Err(e) = init_libreoffice_config(username.clone()) {
                    sender.output(AppMsg::error(
                        ErrorPhase::PostInstall,
                        format!("Failed to create libreoffice config: {e}"),
                    ));
                    return;
                }

                self.postinstall_commands = commands;
                sender.input(InstallAsyncMsg::RunNextCommand);
            }
            InstallAsyncMsg::RunNextCommand => {
                // Step 7: Run commands
                info!("Step 7: Run commands");
                INSTALL_BROKER.send(InstallMsg::ProgressbarTitle(
                    "Step 7: Almost done!".to_string(),
                ));

                if self.postinstall_commands.is_empty() {
                    let _ = sender.output(AppMsg::Finished);
                    return;
                }
                let mut commands = self.postinstall_commands.clone();
                let active = commands.remove(0);
                self.postinstall_commands = commands;
                INSTALL_BROKER.send(InstallMsg::PostInstall(vec![
                    "/usr/bin/env".to_string(),
                    "pkexec".to_string(),
                    "nixos-enter".to_string(),
                    "--root".to_string(),
                    "/mnt".to_string(),
                    "-c".to_string(),
                    active,
                ]));
            }
        }
    }
}

pub struct MakeConfig {
    pub id: String,
    pub language: Option<String>,
    pub timezone: Option<String>,
    pub keyboard: Option<String>,
    pub user: Option<UserConfig>,
    pub list: HashMap<String, HashMap<String, Choice>>,
    pub bootdisk: Option<String>,
    pub imperative_timezone: bool,
    pub disko: String,
}

pub fn makeconfig(makeconfig: MakeConfig) -> Result<()> {
    /* Configuration keys:
        @NVIDIAOFFLOAD@ - Enable NVIDIA offloading
        @BOOTLOADRER@ - Bootloader
        @NETWORK@ - Network configuration
        @TIMEZONE@ - Timezone
        @LOCALE@ - Localization
        @KEYBOARD@ - Keyboard layout
        @DESKTOP@ - Desktop environment
        @AUTOLOGIN@ - Autologin config
        @PACKAGES@ - Packages to install
        @STATEVERSION@ - NixOS State version
        @DISKO@ - Disko configuration
    */

    /* Value keys:
        @HOSTNAME@ - Hostname
        @USERNAME@ - Username
        @FULLNAME@ - Full name
    */

    let efi = distinst_disks::Bootloader::detect() == distinst_disks::Bootloader::Efi;
    let archout = Command::new("uname")
        .arg("-m")
        .output()
        .context("Failed to get architecture")?;
    let arch = String::from_utf8_lossy(&archout.stdout).trim().to_string();

    fn iterwrite(makeconfig: &MakeConfig, path: &str, efi: bool, arch: &str) -> Result<()> {
        // Iterate through files in configs/
        for file in (fs::read_dir(
            format!("{}/xeonitte/{}/{}", SYSCONFDIR, makeconfig.id, path).replace("//", "/"),
        )?)
        .flatten()
        {
            // Check if it is a dir
            if file.metadata()?.is_dir() {
                // Iterate through files in the dir
                debug!("Iterating through {}", file.path().to_string_lossy());
                debug!("Path: {}", path);
                debug!(
                    "!= {}/xeonitte/{}/modules/{{efiboot,biosboot}}",
                    SYSCONFDIR, makeconfig.id
                );
                let _ = iterwrite(
                    makeconfig,
                    &format!(
                        "{}/{}",
                        path.trim_end_matches('/'),
                        file.file_name().to_string_lossy()
                    ),
                    efi,
                    arch,
                );
            } else if file.file_name().to_string_lossy().ends_with(".nix") {
                let mut config = fs::read_to_string(file.path())?;
                config = config.replace("@NVIDIAOFFLOAD@", "");
                config = config.replace("@ARCH@", &format!("{}-linux", arch));
                config = config.replace("@DISKO@", &makeconfig.disko);

                if efi {
                    config = config.replace("@BOOTLOADER@", "");
                    config =
                        config.replace("@BOOTLOADER_MODULE@", "xinux-modules.nixosModules.efiboot")
                } else {
                    config = config.replace(
                        "@BOOTLOADER@",
                        &format!(
                            r#"  boot.loader.grub.device = "{}";"#,
                            makeconfig
                                .bootdisk
                                .as_ref()
                                .context("Failed to get bootloader disk")?
                        ),
                    );
                    config =
                        config.replace("@BOOTLOADER_MODULE@", "xinux-modules.nixosModules.biosboot")
                }

                config = config.replace(
                    "@NETWORK@",
                    &format!(
                        r#"  # Define your hostname.
  networking.hostName = "{}";"#,
                        makeconfig
                            .user
                            .as_ref()
                            .map(|x| x.hostname.as_ref())
                            .unwrap_or("nixos")
                    ),
                );

                if makeconfig.imperative_timezone {
                    config = config.replace("@TIMEZONE@", "");
                } else if let Some(tz) = &makeconfig.timezone {
                    config = config.replace(
                        "@TIMEZONE@",
                        format!(
                            r#"  # Set your time zone.
  time.timeZone = "{}";"#,
                            tz
                        )
                        .as_str(),
                    );
                }

                if let Some(locale) = &makeconfig.language {
                    config = config.replace(
                        "@LOCALE@",
                        &format!(
                            r#"  # Select internationalisation properties.
  modules.xinux.language = "{}";"#,
                            locale
                        ),
                    );
                }

                if let Some(keymap) = &makeconfig.keyboard {
                    if keymap.contains('+') {
                        let mut split = keymap.split('+');
                        if let (Some(layout), Some(variant)) = (split.next(), split.next()) {
                            config = config.replace(
                                "@KEYBOARD@",
                                &format!(
                                    r#"  # Set the keyboard layout.
  services.xserver.xkb = {{
    layout = "{}";
    variant = "{}";
  }};
  console.useXkbConfig = true;"#,
                                    layout, variant
                                ),
                            );
                        }
                    } else {
                        config = config.replace(
                            "@KEYBOARD@",
                            &format!(
                                r#"  # Set the keyboard layout.
  services.xserver.xkb.layout = "{}";
  console.useXkbConfig = true;"#,
                                keymap
                            ),
                        );
                    }
                }

                if let Some(user) = &makeconfig.user {
                    config = config.replace("@USERNAME@", &user.username);
                    config = config.replace("@FULLNAME@", &user.name);
                    config = config.replace("@HOSTNAME@", &user.hostname);

                    let mut autocfg = String::new();
                    if user.autologin {
                        autocfg.push_str(&format!(
                            r#"  # Enable automatic login for the user.
  services.displayManager.autoLogin.enable = true;
  services.displayManager.autoLogin.user = "{}";
"#,
                            user.username
                        ));
                        autocfg.push_str(
                                    r#"  # Workaround for GNOME autologin: https://github.com/NixOS/nixpkgs/issues/103746#issuecomment-945091229
  systemd.services."getty@tty1".enable = false;
  systemd.services."autovt@tty1".enable = false;
"#,
                                );
                    }
                    config = config.replace("@AUTOLOGIN@", &autocfg);
                }

                // List configuration options
                let mut extrapkgs = vec![];
                for (id, choices) in makeconfig.list.iter() {
                    let mut listcfg = String::new();
                    for (_key, choice) in choices.iter() {
                        if let Some(pkgs) = &choice.packages {
                            for pkg in pkgs {
                                extrapkgs.push(pkg.to_string());
                            }
                        }
                        if let Some(cfg) = &choice.config {
                            cfg.lines()
                                .for_each(|x| listcfg.push_str(&format!("  {}\n", x)));
                        }
                    }
                    config = config.replace(&format!("@{}@", id), &listcfg);
                }

                config = config.replace(
                    "@PACKAGES@",
                    &if extrapkgs.is_empty() {
                        r#"  # List packages installed in system profile.
  environment.systemPackages = with pkgs; [
    libreoffice
  ];"#
                        .to_string()
                    } else {
                        format!(
                            r#"  # List packages installed in system profile.
  environment.systemPackages = with pkgs; [
    libreoffice
    {}
  ];"#,
                            extrapkgs.join("\n    ")
                        )
                    },
                );

                config = config.replace(
                    "@STATEVERSION@",
                    &format!(
                        r#"  system.stateVersion = "{}"; # Did you read the comment?"#,
                        String::from_utf8_lossy(
                            &Command::new("nixos-version")
                                .output()
                                .context("Failed to get nixos version")?
                                .stdout
                        )
                        .to_string()
                        .get(0..5)
                        .context("Failed to get nixos version")?
                    ),
                );

                let mut cmd = Command::new("pkexec")
                    .arg(format!("{}/xeonitte-helper", LIBEXECDIR))
                    .arg("write-file")
                    .arg("--path")
                    .arg(if path.is_empty() {
                        format!(
                            "{}/etc/nixos/{}",
                            TMPDIR,
                            file.file_name().to_string_lossy()
                        )
                    } else {
                        format!(
                            "{}/etc/nixos/{}/{}",
                            TMPDIR,
                            path.replace("ARCH", &format!("{}-linux", arch)).replace(
                                "HOSTNAME",
                                makeconfig
                                    .user
                                    .as_ref()
                                    .map(|x| x.hostname.as_ref())
                                    .unwrap_or("nixos")
                            ),
                            file.file_name().to_string_lossy()
                        )
                    })
                    .arg("--contents")
                    .arg(config)
                    .spawn()?;
                cmd.wait()?;
            } else if file.metadata()?.is_file() {
                Command::new("pkexec")
                    .arg("mkdir")
                    .arg("-p")
                    .arg(if path.is_empty() {
                        format!("{}/etc/nixos/", TMPDIR).to_string()
                    } else {
                        format!(
                            "{}/etc/nixos/{}/",
                            TMPDIR,
                            path.replace("ARCH", &format!("{}-linux", arch)).replace(
                                "HOSTNAME",
                                makeconfig
                                    .user
                                    .as_ref()
                                    .map(|x| x.hostname.as_ref())
                                    .unwrap_or("nixos")
                            )
                        )
                    })
                    .spawn()?
                    .wait()?;

                Command::new("pkexec")
                    .arg("cp")
                    .arg(file.path().to_string_lossy().to_string())
                    .arg(if path.is_empty() {
                        format!(
                            "{}/etc/nixos/{}",
                            TMPDIR,
                            file.file_name().to_string_lossy()
                        )
                    } else {
                        format!(
                            "{}/etc/nixos/{}/{}",
                            TMPDIR,
                            path.replace("ARCH", &format!("{}-linux", arch)).replace(
                                "HOSTNAME",
                                makeconfig
                                    .user
                                    .as_ref()
                                    .map(|x| x.hostname.as_ref())
                                    .unwrap_or("nixos")
                            ),
                            file.file_name().to_string_lossy()
                        )
                    })
                    .spawn()?
                    .wait()?;
            }
        }
        Ok(())
    }

    iterwrite(&makeconfig, "", efi, &arch)
}

fn init_libreoffice_config(username: String) -> Result<()> {
    Command::new("pkexec")
        .arg("mkdir")
        .arg("-p")
        .arg(format!(
            "{}/home/{}/.config/libreoffice/4/user/uno_packages/cache",
            "/mnt", username
        ))
        .output()?;

    Command::new("pkexec")
        .arg("mkdir")
        .arg("-p")
        .arg(format!(
            "{}/home/{}/.config/libreoffice/4/user/",
            "/mnt", username
        ))
        .output()?;

    // for icons
    Command::new("pkexec")
        .arg("cp")
        .arg("-a")
        .arg(format!("{}/xeonitte/configcopy/uno_packages", SYSCONFDIR))
        .arg(format!(
            "{}/home/{}/.config/libreoffice/4/user/uno_packages/cache/",
            "/mnt", username
        ))
        .output()?;

    Command::new("pkexec")
        .arg("rm")
        .arg(format!(
            "{}/home/{}/.config/libreoffice/4/user/registrymodifications.xcu",
            "/mnt", username
        ))
        .output()?;

    Command::new("pkexec")
        .arg("cp")
        .arg("-a")
        .arg(format!(
            "{}/xeonitte/configcopy/registrymodifications.xcu",
            SYSCONFDIR
        ))
        .arg(format!(
            "{}/home/{}/.config/libreoffice/4/user/",
            "/mnt", username
        ))
        .output()?;
    Ok(())
}

fn backup_config() -> Result<()> {
    Command::new("pkexec")
        .arg("rm")
        .arg("-rf")
        .arg("/xeonitte")
        .output()?;

    Command::new("pkexec")
        .arg("mkdir")
        .arg("/xeonitte")
        .output()?;

    Command::new("pkexec")
        .arg("cp")
        .arg("-r")
        .arg(TMPDIR)
        .arg("/xeonitte")
        .output()?;

    Command::new("pkexec")
        .arg("chmod")
        .arg("777")
        .arg("/tmp/xeonitte.log")
        .output()?;

    Command::new("pkexec")
        .arg("chmod")
        .arg("777")
        .arg("/tmp/xeonitte-term.log")
        .output()?;
    Ok(())
}

fn setuserpasswd(username: Option<String>, password: Option<String>) -> Result<()> {
    let mut passwdcmd = Command::new("pkexec")
        .arg("nixos-enter")
        .arg("--root")
        .arg("/mnt")
        .arg("-c")
        .arg("chpasswd -c SHA512")
        .stdin(Stdio::piped())
        .spawn()?;
    let passwdstdin = passwdcmd
        .stdin
        .as_mut()
        .context("Failed to get password stdin")?;
    passwdstdin.write_all(
        format!(
            "{}:{}",
            username.context("No username found")?,
            password.context("No password found")?
        )
        .as_bytes(),
    )?;
    match passwdcmd.wait() {
        Err(e) => {
            error!("Failed to set password: {}", e);
        }
        Ok(status) => {
            if !status.success() {
                error!("Failed to set password");
            }
        }
    }
    Ok(())
}
