use crate::utils::parse::ConfigType;

pub const DISTRO_NAME: &str = "Xinux";
pub const BRANDING: &str = "xinux";
pub const INTERNET_CHECK_URL: &str = "http://nmcheck.gnome.org/check_network_status.txt";
pub const DEFAULT_HOSTNAME: &str = "xinux";

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Flow {
    Init,
    Basic,
    Advanced,
}
impl Flow {
    pub fn iter(&self) -> impl Iterator<Item = Flow> {
        use Flow::*;
        [Init, Basic, Advanced].iter().copied()
    }
    pub fn logo(&self) -> &str {
        use Flow::*;

        match self {
            Init => "",
            Basic => "emoji-symbols-symbolic",
            Advanced => "preferences-system-symbolic",
        }
    }
    pub fn config_type(&self) -> ConfigType {
        ConfigType::Xinux
    }

    pub fn imperative_timezone(&self) -> bool {
        true
    }
    pub fn steps(&self) -> Vec<Step> {
        use Step::*;

        // initial steps
        let mut steps = vec![Welcome, Keyboard, Location, InstallMode];

        // new steps based on the mode
        let mut next_steps = match self {
            Self::Init => vec![],
            Self::Basic => vec![
                User {
                    root: false,
                    hostname: false,
                },
                Partitioning,
                Summary,
            ],
            Self::Advanced => vec![
                User {
                    root: true,
                    hostname: false,
                },
                List {
                    multiple: true,
                    required: false,
                    title: "Extra Options".into(),
                    id: ListId::PackageManager,
                    choices: vec![
                        Choice {
                            name: "Flatpak".into(),
                            description: "Enable Flatpak support".into(),
                            config: "services.flatpak.enable = true;".into(),
                            default: false,
                            packages: vec![]
                        },
                        Choice {
                            name: "AppImage".into(),
                            description: "Enable AppImage support by installing the \"appimage-run\" package. For AppImages to work, you must run them with the \"appimage-run\" command.".into(),
                            config: "modules.packagemanagers.appimage.enable = true;".into(),
                            default: false,
                            packages: vec![]
                        },
                        Choice {
                            name: "Minimal".into(),
                            description: "Install minimal Xinux without GNOME core apps".into(),
                            config: "modules.gnome.remove-utils.enable = lib.mkForce true;".into(),
                            default: false,
                            packages: vec![]
                        },
                    ],
                },
                List { multiple: false, required: true, title: "Kernel".into(), id: ListId::Kernel, choices: vec![
                    Choice {
                        name: "LTS".into(),
                        description: "Install the latest LTS kernel".into(),
                        config: String::new(),
                        default: true,
                        packages: vec![]
                    },
                    Choice {
                        name: "Latest".into(),
                        description: "Install the latest kernel".into(),
                        config: "boot.kernelPackages = pkgs.linuxPackages_latest;".into(),
                        default: false,
                        packages: vec![]
                    },
                    Choice {
                        name: "Libre".into(),
                        description: "Install the libre kernel".into(),
                        config: "boot.kernelPackages = pkgs.linuxPackages_libre;".into(),
                        default: false,
                        packages: vec![]
                    },
                    Choice {
                        name: "Zen".into(),
                        description: "Install the Zen kernel".into(),
                        config: "boot.kernelPackages = pkgs.linuxPackages_zen;".into(),
                        default: false,
                        packages: vec![]
                    },
                ] },
                Partitioning,
                Summary
            ],
        };

        // add steps to initial ones and return
        steps.append(&mut next_steps);
        steps
    }
}
impl From<Flow> for String {
    fn from(value: Flow) -> Self {
        format!("{value:?}")
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Step {
    Welcome,
    Keyboard,
    Location,
    InstallMode,
    User {
        root: bool,
        hostname: bool,
    },
    List {
        multiple: bool,
        required: bool,
        title: String,
        id: ListId,
        choices: Vec<Choice>,
    },
    Partitioning,
    Summary,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ListId {
    PackageManager,
    Kernel,
}
impl From<ListId> for String {
    fn from(value: ListId) -> Self {
        format!("{value:?}")
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Choice {
    pub name: String,
    pub description: String,
    pub config: String,
    pub default: bool,
    pub packages: Vec<String>,
}
