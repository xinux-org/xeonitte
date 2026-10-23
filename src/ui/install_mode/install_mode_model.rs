use crate::ui::window::{AppMsg, StackPage};
use crate::utils::flow::Flow::{self, Basic};
use gettextrs::gettext;
use log::trace;
use relm4::{
    ComponentParts, ComponentSender, RelmWidgetExt, SimpleComponent,
    adw::{self, prelude::*},
    gtk::{self},
    *,
};

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
            adw::StatusPage {
                set_icon_name: Some("archive-symbolic"),
                set_title: &gettext("Type of installation"),
                set_description: Some(&gettext("Lorem Ipsum is simply dummy text of the printing and typesetting industry. Lorem
                Ipsum has been the industry's standard dummy")),

                #[wrap(Some)]
                set_child = &gtk::Box {
                    set_hexpand: true,
                    set_orientation: gtk::Orientation::Vertical,
                    set_align: gtk::Align::Center,
                    set_spacing: 6,
                    #[name(install_mode)]
                    gtk::CheckButton {
                        set_label: Some(&gettext("Basic mode")),
                        #[watch]
                        set_active: model.selected == Some(Flow::Basic),
                        connect_toggled[sender] => move |btn| {
                            if btn.is_active() {
                                sender.input(InstallModeMsg::SetSelected(Some(Flow::Basic)))
                            }
                        },
                    },
                    gtk::CheckButton {
                        set_label: Some(&gettext("Advanced mode")),
                        set_group: Some(&install_mode),
                        #[watch]
                        set_active: model.selected == Some(Flow::Advanced),
                        connect_toggled[sender] => move |btn| {
                            if btn.is_active() {
                                sender.input(InstallModeMsg::SetSelected(Some(Flow::Advanced)))
                            }
                        },
                    },
                },
            },
            // #[local_ref]
            // selectbox -> gtk::FlowBox {
            //     set_orientation: gtk::Orientation::Horizontal,
            //     set_align: gtk::Align::Center,
            //     set_hexpand: true,
            //     set_column_spacing: 20,
            //     set_row_spacing: 20,
            //     set_selection_mode: gtk::SelectionMode::None,
            //     set_homogeneous: true,
            // }
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
            selected: Some(Flow::Basic),
        };
        sender.input(InstallModeMsg::SetSelected(Some(Flow::Basic)));

        let choices = model
            .config
            .iter()
            .filter(|&choise| choise != Flow::Init)
            .collect::<Vec<_>>();

        let selectbox = gtk::FlowBox::builder()
            .max_children_per_line(choices.len() as u32)
            .build();

        let widgets = view_output!();

        choices.iter().for_each(|config| {
            view! {
                button = gtk::Button {
                    set_size_request: (200, 200),
                    set_align: gtk::Align::Center,
                    connect_clicked[sender, config] => move |_| {
                        sender.input(InstallModeMsg::SetSelected(Some(config)))
                    },
                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_align: gtk::Align::Center,
                        set_spacing: 10,
                        set_margin_all: 10,
                        gtk::Image {
                            set_icon_name: Some(config.logo()),
                            set_pixel_size: 80,
                            set_align: gtk::Align::Center,
                        },
                        gtk::Label {
                            set_label: &gettext(format!("{config:?}")),
                            set_align: gtk::Align::Center,
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
                sender
                    .output(AppMsg::SetStackPageConfig(
                        StackPage::Carousel,
                        mode.unwrap_or(Flow::Init),
                    ))
                    .unwrap();
            }
            InstallModeMsg::CheckSelected => {
                trace!("InstallMode::CheckSelected {}", self.selected.is_some());
                sender.output(AppMsg::SetCanGoForward(self.selected.is_some()));
            }
        }
    }
}
