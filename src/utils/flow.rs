use crate::utils::parse::StepType::Welcome;

// pub struct Config {
//     name: String,
//     logo: String,
//     id: ConfigId,
//     imperative_timezone: bool,
//     config_type: ConfigType,
//     steps: Vec<Step>,
// }
pub enum Flow {
    Init,
    Basic,
    Advanced,
}
impl Flow {
    fn steps(&self) -> Vec<Step> {
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
                        },
                        Choice {
                            name: "AppImage".into(),
                            description: "Enable AppImage support by installing the \"appimage-run\" package. For AppImages to work, you must run them with the \"appimage-run\" command.".into(),
                            config: "modules.packagemanagers.appimage.enable = true;".into(),
                        },
                        Choice {
                            name: "Minimal".into(),
                            description: "Install minimal Xinux without GNOME core apps".into(),
                            config: "modules.gnome.remove-utils.enable = lib.mkForce true;".into(),
                        },
                    ],
                },
                List { multiple: false, required: true, title: "Kernel".into(), id: ListId::Kernel, choices: vec![
                    Choice {
                        name: "LTS".into(),
                        description: "Install the latest LTS kernel".into(),
                        config: String::new()
                    },
                    Choice {
                        name: "Latest".into(),
                        description: "Install the latest kernel".into(),
                        config: "boot.kernelPackages = pkgs.linuxPackages_latest;".into()
                    },
                    Choice {
                        name: "Libre".into(),
                        description: "Install the libre kernel".into(),
                        config: "boot.kernelPackages = pkgs.linuxPackages_libre;".into()
                    },
                    Choice {
                        name: "Zen".into(),
                        description: "Install the Zen kernel".into(),
                        config: "boot.kernelPackages = pkgs.linuxPackages_zen;".into()
                    },

                ] }
            ],
        };

        // add steps to initial ones and return
        steps.append(&mut next_steps);
        steps
    }
}

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

pub enum ListId {
    PackageManager,
    Kernel,
}

pub struct Choice {
    name: String,
    description: String,
    config: String,
}
