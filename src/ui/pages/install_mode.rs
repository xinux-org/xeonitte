use crate::ui::window::{AppMsg, StackPage};
use crate::utils::parse::{InstallationConfig, XeonitteConfig};
use gettextrs::gettext;
use gtk::prelude::{BoxExt, ButtonExt, OrientableExt, WidgetExt};
use log::trace;
use relm4::*;
use relm4::{ComponentParts, ComponentSender, RelmWidgetExt, SimpleComponent, gtk};

pub struct InstallModeModel {
    config: XeonitteConfig,
    selected: Option<InstallationConfig>,
}

#[derive(Debug)]
pub enum InstallModeMsg {
    SetSelected(Option<InstallationConfig>),
    CheckSelected,
}

#[derive(Debug)]
enum InstallModeOutput {
    SetStackPageConfig(StackPage, Option<InstallationConfig>),
}

#[relm4::component(pub)]
impl SimpleComponent for InstallModeModel {
    type Init = XeonitteConfig;
    type Input = InstallModeMsg;
    type Output = AppMsg;

    view! {
        gtk::Box {
            set_margin_all: 20,
            #[local_ref]
            selectbox -> gtk::FlowBox {
                set_orientation: gtk::Orientation::Horizontal,
                set_halign: gtk::Align::Center,
                set_valign: gtk::Align::Center,
                set_hexpand: true,
                set_column_spacing: 20,
                set_row_spacing: 20,
                set_selection_mode: gtk::SelectionMode::None,
                #[watch]
                set_max_children_per_line: selectbox.iter_children().count() as u32,
                set_homogeneous: true,
            }
        }
    }

    // Initialize the UI.
    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = InstallModeModel {
            config: init,
            selected: None,
        };
        let selectbox = gtk::FlowBox::new();
        // Insert the macro code generation here
        let widgets = view_output!();

        model
            .config
            .choices
            .iter()
            .cloned()
            .for_each(|configuration| {
                let config = configuration.config;
                view! {
                    button = gtk::Button {
                        set_width_request: 200,
                        set_height_request: 200,
                        set_halign: gtk::Align::Center,
                        set_valign: gtk::Align::Center,
                        connect_clicked[sender, config] => move |_| {
                            sender.input(InstallModeMsg::SetSelected(Some(config.clone())))
                        },
                        gtk::Box {
                            set_orientation: gtk::Orientation::Vertical,
                            set_halign: gtk::Align::Center,
                            set_valign: gtk::Align::Center,
                            set_spacing: 10,
                            set_margin_all: 10,
                            gtk::Image {
                                set_icon_name: Some(&config.config_logo),
                                set_pixel_size: 80,
                                set_halign: gtk::Align::Center,
                                set_valign: gtk::Align::Center,
                            },
                            gtk::Label {
                                set_label: &gettext(&config.config_name),
                                set_halign: gtk::Align::Center,
                                set_valign: gtk::Align::Center,
                                set_wrap: true,
                                set_justify: gtk::Justification::Center,
                            }
                        }
                    }
                }
                selectbox.append(&button);
            });
        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>) {
        match msg {
            InstallModeMsg::SetSelected(mode) => {
                self.selected = mode.clone();
                let init_steps_len = self
                    .config
                    .get_installation_config("init")
                    .unwrap_or_default()
                    .steps
                    .len();
                let page_start_index = mode
                    .as_ref()
                    .and_then(|x| {
                        x.config_id
                            .ne("init")
                            .then_some(init_steps_len.ne(&0).then_some(init_steps_len + 1))
                    })
                    .flatten()
                    .unwrap_or_default();
                dbg!(page_start_index);
                sender
                    .output(AppMsg::SetStackPageConfig(
                        StackPage::Carousel,
                        mode,
                        page_start_index,
                    ))
                    .unwrap();
            }
            InstallModeMsg::CheckSelected => {
                trace!("WelcomeMsg::CheckSelected {}", self.selected.is_some());
                let _ = sender.output(AppMsg::SetCanGoForward(self.selected.is_some()));
            }
        }
    }
}
