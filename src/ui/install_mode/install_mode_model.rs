use crate::ui::window::AppMsg;
use crate::utils::flow::Flow;
use gettextrs::gettext;
use gtk::prelude::{BoxExt, ButtonExt, OrientableExt, WidgetExt};
use log::trace;
use relm4::*;
use relm4::{ComponentParts, ComponentSender, RelmWidgetExt, SimpleComponent, gtk};

pub struct InstallModeModel {
    config: Flow,
    selected: Option<Flow>,
}

#[derive(Debug)]
pub enum InstallModeMsg {
    SetSelected(Option<Flow>),
    CheckSelected,
}

#[relm4::component(pub)]
impl SimpleComponent for InstallModeModel {
    type Init = Flow;
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
        let widgets = view_output!();

        model
            .config
            .iter()
            .filter(|config| !config.eq(&Flow::Init))
            .for_each(|config| {
                view! {
                    button = gtk::Button {
                        set_width_request: 200,
                        set_height_request: 200,
                        set_halign: gtk::Align::Center,
                        set_valign: gtk::Align::Center,
                        connect_clicked[sender, config] => move |_| {
                            sender.input(InstallModeMsg::SetSelected(Some(config)))
                        },
                        gtk::Box {
                            set_orientation: gtk::Orientation::Vertical,
                            set_halign: gtk::Align::Center,
                            set_valign: gtk::Align::Center,
                            set_spacing: 10,
                            set_margin_all: 10,
                            gtk::Image {
                                set_icon_name: Some(config.logo()),
                                set_pixel_size: 80,
                                set_halign: gtk::Align::Center,
                                set_valign: gtk::Align::Center,
                            },
                            gtk::Label {
                                set_label: &gettext(format!("{config:?}")),
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
                self.selected = mode;
                // let init_steps_len = Flow::Init.steps().len();
                // let page_start_index = self
                //     .selected
                //     .as_ref()
                //     .and_then(|x| {
                //         x.ne(&Flow::Init)
                //             .then_some(init_steps_len.ne(&0).then_some(init_steps_len + 1))
                //     })
                //     .flatten()
                //     .unwrap_or_default();
                // sender
                //     .output(AppMsg::SetStackPageConfig(
                //         StackPage::Carousel,
                //         mode,
                //         page_start_index,
                //     ))
                //     .unwrap();
            }
            InstallModeMsg::CheckSelected => {
                trace!("InstallMode::CheckSelected {}", self.selected.is_some());
                sender.output(AppMsg::SetCanGoForward(self.selected.is_some()));
            }
        }
    }
}
