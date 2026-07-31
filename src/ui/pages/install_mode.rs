use crate::ui::window::StackPage;
use crate::utils::i18n::i18n_f;
use crate::utils::parse::{ChoiceEnum, InstallationConfig, XeonitteConfig};
use adw::prelude::*;
use gettextrs::gettext;
use gtk::prelude::{BoxExt, ButtonExt, OrientableExt, WidgetExt};
use relm4::*;
use relm4::{ComponentParts, ComponentSender, RelmWidgetExt, SimpleComponent, gtk};

struct InstallModeModel {
    config: XeonitteConfig,
}

#[derive(Debug)]
enum InstallModeOutput {
    SetStackPageConfig(StackPage, Option<InstallationConfig>),
}

#[relm4::component]
impl SimpleComponent for InstallModeModel {
    type Init = XeonitteConfig;

    type Input = ();
    type Output = InstallModeOutput;

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
        config: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = InstallModeModel { config };
        let selectbox = gtk::FlowBox::new();
        // Insert the macro code generation here
        let widgets = view_output!();

        for item in &model.config.choices {
            match item {
                ChoiceEnum::Configuration { file: _, config } => {
                    view! {
                        button = gtk::Button {
                            set_width_request: 200,
                            set_height_request: 200,
                            set_halign: gtk::Align::Center,
                            set_valign: gtk::Align::Center,
                            connect_clicked[sender, config] => move |_| {
                                sender.output(InstallModeOutput::SetStackPageConfig(StackPage::Carousel, Some(config.clone()))).unwrap();
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
                }
                ChoiceEnum::Live => {
                    view! {
                        button = gtk::Button {
                            set_width_request: 200,
                            set_height_request: 200,
                            set_halign: gtk::Align::Center,
                            set_valign: gtk::Align::Center,
                            connect_clicked => move |_| {
                                relm4::main_application().quit();
                            },
                            gtk::Box {
                                set_orientation: gtk::Orientation::Vertical,
                                set_halign: gtk::Align::Center,
                                set_valign: gtk::Align::Center,
                                set_spacing: 10,
                                set_margin_all: 10,
                                gtk::Image {
                                    set_icon_name: Some("preferences-desktop-display-symbolic"),
                                    set_pixel_size: 80,
                                    set_halign: gtk::Align::Center,
                                    set_valign: gtk::Align::Center,
                                },
                                gtk::Label {
                                    // Translators: Do NOT translate the '{}'
                                    // The string reads "Try {distribution name} live"
                                    set_label: i18n_f("Try {} live", &[&model.config.distribution_name]).as_str(),
                                    set_halign: gtk::Align::Center,
                                    set_valign: gtk::Align::Center,
                                    set_wrap: true,
                                    set_justify: gtk::Justification::Center,
                                }
                            }

                        }
                    }
                    selectbox.append(&button);
                }
            }
        }

        ComponentParts { model, widgets }
    }

    // fn update(&mut self, _msg: Self::Input, _sender: ComponentSender<Self>) {}
}
