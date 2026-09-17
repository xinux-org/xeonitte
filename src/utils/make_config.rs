use anyhow::{Context, Result};
use log::debug;
use std::{collections::HashMap, fs, process::Command};

use crate::{
    config::{LIBEXECDIR, SYSCONFDIR, TMPDIR},
    ui::window::UserConfig,
    utils::flow::Choice,
};

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

    iterwrite(&makeconfig, "", efi, &arch)
}

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
                    config.replace("@BOOTLOADER_MODULE@", "xinux-modules.nixosModules.efiboot");

                // To avoid rewriting bootloader config
                if cfg!(debug_assertions) {
                    config = config.replace(
                        "@DEBUG_MODE_BOOTLOADER@",
                        r#"  # System installed with development environment via usb ssd.
    boot.loader.efi.canTouchEfiVariables = lib.mkForce false;
    boot.loader.grub.efiInstallAsRemovable = lib.mkForce true;"#,
                    );
                }
                // Installer can rewrite bootloader on release profile by default
                if !cfg!(debug_assertions) {
                    config = config.replace("@DEBUG_MODE_BOOTLOADER@", "");
                }
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
                    choice.packages.iter().for_each(|pkg| {
                        extrapkgs.push(pkg.to_string());
                    });
                    choice
                        .config
                        .lines()
                        .for_each(|x| listcfg.push_str(&format!("  {}\n", x)));
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
