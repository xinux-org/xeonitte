use super::window::AppMsg;
use adw::prelude::*;
use gettextrs::gettext;
use relm4::*;

pub struct QuitDialogModel;

#[relm4::component(pub)]
impl SimpleComponent for QuitDialogModel {
    type Init = gtk::Window;
    type Input = ();
    type Output = AppMsg;
    type Widgets = QuitCheckWidgets;

    view! {
        dialog = adw::AlertDialog {
            #[watch]
            set_heading: Some(&gettext("Quit Installation")),
            #[watch]
            set_body: &gettext("Quitting while the installation is in progress may leave your system in an unbootable state!"),
            add_response: ("cancel", &gettext("Cancel")),
            add_response: ("quit", &gettext("Quit")),
            #[watch]
            set_response_label: ("cancel", &gettext("Cancel")),
            #[watch]
            set_response_label: ("quit", &gettext("Quit")),
            set_response_appearance: ("quit", adw::ResponseAppearance::Destructive),

            connect_response: (None, move |dialog, response| {
                if response == "quit" {
                    let _ = dialog.clone();
                    relm4::main_application().quit();
                };
            }),
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = QuitDialogModel;
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }
}
