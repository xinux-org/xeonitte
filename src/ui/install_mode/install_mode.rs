use crate::flow::InstallFlow;
use crate::ui::window::AppMsg;
use gettextrs::gettext;
use gtk::prelude::{BoxExt, ButtonExt, OrientableExt, WidgetExt};
use relm4::*;
use relm4::{ComponentParts, ComponentSender, RelmWidgetExt, SimpleComponent, gtk};

pub struct InstallModeModel {
    selected: Option<InstallFlow>,
}

#[derive(Debug)]
pub enum InstallModeMsg {
    SetSelected(InstallFlow),
}

#[relm4::component(pub)]
impl SimpleComponent for InstallModeModel {
    type Init = ();
    type Input = InstallModeMsg;
    type Output = AppMsg;

    view! {
        gtk::Box {
            set_hexpand: true,
            set_vexpand: true,
            set_halign: gtk::Align::Center,
            set_valign: gtk::Align::Center,
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
                set_homogeneous: true,
            }
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = InstallModeModel { selected: None };
        let selectbox = gtk::FlowBox::new();
        let widgets = view_output!();

        let flows: &[(InstallFlow, &str, &str)] = &[
            (
                InstallFlow::Basic,
                "emoji-symbols-symbolic",
                "Basic Installation",
            ),
            (
                InstallFlow::Advanced,
                "preferences-system-symbolic",
                "Advanced Installation",
            ),
        ];

        for (flow, icon, label) in flows.iter() {
            let flow = flow.clone();
            view! {
                button = gtk::Button {
                    set_width_request: 200,
                    set_height_request: 200,
                    set_halign: gtk::Align::Center,
                    set_valign: gtk::Align::Center,
                    connect_clicked[sender, flow] => move |_| {
                        sender.input(InstallModeMsg::SetSelected(flow.clone()));
                    },
                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_halign: gtk::Align::Center,
                        set_valign: gtk::Align::Center,
                        set_spacing: 10,
                        set_margin_all: 10,
                        gtk::Image {
                            set_icon_name: Some(icon),
                            set_pixel_size: 80,
                            set_halign: gtk::Align::Center,
                            set_valign: gtk::Align::Center,
                        },
                        gtk::Label {
                            set_label: &gettext(label.to_owned()),
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
        selectbox.set_max_children_per_line(flows.len() as u32);
        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>) {
        match msg {
            InstallModeMsg::SetSelected(flow) => {
                self.selected = Some(flow.clone());
                sender.output(AppMsg::SelectFlow(flow));
            }
        }
    }
}
