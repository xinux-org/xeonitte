use std::collections::HashMap;

use crate::utils::parse::ConfigType;

pub const DISTRO_NAME: &str = "Xinux";
pub const BRANDING: &str = "xinux";
pub const INTERNET_CHECK_URL: &str = "http://nmcheck.gnome.org/check_network_status.txt";
pub const DEFAULT_HOSTNAME: &str = "xinux";

#[derive(Clone, Debug, PartialEq)]
pub enum Step {
    Welcome,
    Keyboard,
    Location,
    InstallMode,
    User { root: bool, hostname: bool },
    PackageManagers,
    KernelSelection,
    Partitioning,
    Summary,
}

#[derive(Clone, Debug, PartialEq)]
pub enum InstallFlow {
    Basic,
    Advanced,
}

impl InstallFlow {
    pub fn config_id(&self) -> &'static str {
        match self {
            InstallFlow::Basic => "basic",
            InstallFlow::Advanced => "advanced",
        }
    }

    pub fn config_type(&self) -> ConfigType {
        ConfigType::Xinux
    }

    pub fn imperative_timezone(&self) -> bool {
        true
    }

    pub fn steps(&self) -> Vec<Step> {
        match self {
            InstallFlow::Basic => vec![
                Step::User { root: false, hostname: false },
                Step::Partitioning,
                Step::Summary,
            ],
            InstallFlow::Advanced => vec![
                Step::User { root: true, hostname: true },
                Step::PackageManagers,
                Step::KernelSelection,
                Step::Partitioning,
                Step::Summary,
            ],
        }
    }
}

pub fn init_steps() -> Vec<Step> {
    vec![Step::Welcome, Step::Keyboard, Step::Location, Step::InstallMode]
}

#[derive(Clone, Debug, PartialEq)]
pub struct Choice {
    pub description: Option<String>,
    pub packages: Option<Vec<String>>,
    pub default: bool,
    pub config: Option<String>,
}

pub fn package_manager_choices() -> Vec<HashMap<String, Choice>> {
    vec![
        HashMap::from([("Flatpak".to_string(), Choice {
            description: Some("Enable Flatpak support".to_string()),
            packages: None,
            default: false,
            config: Some("services.flatpak.enable = true;".to_string()),
        })]),
        HashMap::from([("AppImage".to_string(), Choice {
            description: Some("Enable AppImage support by installing the \"appimage-run\" package. For AppImages to work, you must run them with the \"appimage-run\" command.".to_string()),
            packages: None,
            default: false,
            config: Some("modules.packagemanagers.appimage.enable = true;".to_string()),
        })]),
        HashMap::from([("Minimal".to_string(), Choice {
            description: Some("Install minimal Xinux without GNOME core apps".to_string()),
            packages: None,
            default: false,
            config: Some("modules.gnome.remove-utils.enable = lib.mkForce true;".to_string()),
        })]),
    ]
}

pub fn kernel_choices() -> Vec<HashMap<String, Choice>> {
    vec![
        HashMap::from([("LTS".to_string(), Choice {
            description: Some("Install the latest LTS kernel".to_string()),
            packages: None,
            default: true,
            config: None,
        })]),
        HashMap::from([("Latest".to_string(), Choice {
            description: Some("Install the latest kernel".to_string()),
            packages: None,
            default: false,
            config: Some("boot.kernelPackages = pkgs.linuxPackages_latest;".to_string()),
        })]),
        HashMap::from([("Libre".to_string(), Choice {
            description: Some("Install the libre kernel".to_string()),
            packages: None,
            default: false,
            config: Some("boot.kernelPackages = pkgs.linuxPackages_libre;".to_string()),
        })]),
        HashMap::from([("Zen".to_string(), Choice {
            description: Some("Install the Zen kernel".to_string()),
            packages: None,
            default: false,
            config: Some("boot.kernelPackages = pkgs.linuxPackages_zen;".to_string()),
        })]),
    ]
}
