use crate::ui::window::{AppMsg, StackPage};
use crate::utils::flow::Flow;
use gettextrs::gettext;
use log::trace;
use relm4::{
    ComponentParts, ComponentSender, RelmWidgetExt, SimpleComponent,
    adw::{self, prelude::*},
    gtk::{self},
};

pub struct InstallModeModel {
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
                set_description: Some(&gettext("Recommended mode is standard for most users. Advanced mode has multiple disk customization")),
                #[wrap(Some)]
                set_child = &gtk::Box {
                    set_hexpand: true,
                    set_orientation: gtk::Orientation::Vertical,
                    set_align: gtk::Align::Center,
                    set_spacing: 6,
                    #[name(install_mode)]
                    gtk::CheckButton {
                        set_label: Some(&gettext("Recommended")),
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
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = InstallModeModel {
            selected: Some(Flow::Basic),
        };
        sender.input(InstallModeMsg::SetSelected(Some(Flow::Basic)));

        let widgets = view_output!();
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
