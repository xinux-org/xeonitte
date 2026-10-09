use gettextrs::gettext;

#[derive(Debug, Clone)]
pub struct Slide {
    pub title: String,
    pub subtitle: String,
    pub image: String,
}

pub fn get_slides() -> Vec<Slide> {
    vec![
        Slide {
            title: gettext("Configurable and Customizable"),
            subtitle: gettext(
                "Easily modify and manage your system configuration with the power of NixOS",
            ),
            image: "configuration.svg".into(),
        },
        Slide {
            title: gettext("Nix Package Manager"),
            subtitle: gettext(
                "Keep your system stable with Nix's reliable and reproducible packages",
            ),
            image: "nix.svg".into(),
        },
        Slide {
            title: gettext("Over 140,000 packages available"),
            subtitle: gettext(
                "Easily find and install all the packages you need on your Xinux system",
            ),
            image: "apps.svg".into(),
        },
        Slide {
            title: gettext("E-imzo Manager"),
            subtitle: gettext("A graphical e-imzo keys manager for NixOS based distributions."),
            image: "e-imzo.png".into(),
        },
        Slide {
            title: gettext("Software Center"),
            subtitle: gettext(
                "A graphical app store for Nix built with libadwaita, GTK4, and Relm4.",
            ),
            image: "software-center.png".into(),
        },
        Slide {
            title: gettext("Module Manager"),
            subtitle: gettext(
                "The goal of this project is to provide a simple graphical tool for modifying and managing desktop NixOS configurations.",
            ),
            image: "module-manager.png".into(),
        },
        Slide {
            title: gettext("Uzbek Keyboard Layout"),
            subtitle: gettext(
                "Easily find and install all the packages you need on your Xinux system",
            ),
            image: "keyboard.png".into(),
        },
    ]
}
