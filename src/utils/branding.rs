#[derive(Debug, Clone)]
pub struct Slide {
    pub title: String,
    pub subtitle: String,
    pub image: String,
}

pub fn get_slides() -> Vec<Slide> {
    vec![
        Slide {
            title: "Configurable and Customizable".into(),
            subtitle: "Easily modify and manage your system configuration with the power of NixOS".into(),
            image: "configuration.svg".into(),
        },
        Slide {
            title: "Nix Package Manager".into(),
            subtitle: "Keep your system stable with Nix's reliable and reproducible packages".into(),
            image: "nix.svg".into(),
        },
        Slide {
            title: "Over 140,000 packages available".into(),
            subtitle: "Easily find and install all the packages you need on your Xinux system".into(),
            image: "apps.svg".into(),
        },
        Slide {
            title: "E-imzo Manager".into(),
            subtitle: "A graphical e-imzo keys manager for NixOS based distributions.".into(),
            image: "e-imzo.png".into(),
        },
        Slide {
            title: "Software Center".into(),
            subtitle: "A graphical app store for Nix built with libadwaita, GTK4, and Relm4.".into(),
            image: "software-center.png".into(),
        },
        Slide {
            title: "Module Manager".into(),
            subtitle: "The goal of this project is to provide a simple graphical tool for modifying and managing desktop NixOS configurations.".into(),
            image: "module-manager.png".into(),
        },
        Slide {
            title: "Uzbek Keyboard Layout".into(),
            subtitle: "Easily find and install all the packages you need on your Xinux system".into(),
            image: "keyboard.png".into(),
        },
    ]
}
