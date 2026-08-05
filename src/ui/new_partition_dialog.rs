use relm4::{ComponentParts, ComponentSender, RelmWidgetExt, css};
use relm4::{adw, adw::prelude::*, gtk};

pub struct NewPartitionDialog {
    #[allow(unused)]
    device: String,
    parent: gtk::Widget,
}

#[derive(Debug)]
pub enum NewPartitionDialogMsg {
    Save,
}

#[relm4::component(pub)]
impl relm4::Component for NewPartitionDialog {
    type Init = (String, gtk::Widget);
    type Input = NewPartitionDialogMsg;
    type Output = String;
    type CommandOutput = ();

    view! {
        #[root]
        adw::Dialog {
            set_content_width: 420,
            set_title: "Create New Partition",

            #[wrap(Some)]
            set_child = &adw::ToolbarView {
                add_top_bar = &adw::HeaderBar {},

                add_bottom_bar = &gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 10,
                    set_margin_horizontal: 10,
                    set_margin_bottom: 10,

                    gtk::Button {
                        set_label: "Cancel",
                        set_hexpand: true,
                        connect_clicked[root = root.downgrade()] => move |_| {
                            if let Some(root) = root.upgrade() {
                                root.close();
                            }
                        },
                    },

                    gtk::Button {
                        set_label: "Create",
                        set_hexpand: true,
                        add_css_class: css::SUGGESTED_ACTION,
                        connect_clicked => NewPartitionDialogMsg::Save,
                    },
                },
            }
        },
    }

    fn init(
        (device, parent): Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self { device, parent };

        let widgets = view_output!();
        root.present(Some(&model.parent));

        ComponentParts { widgets, model }
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        msg: Self::Input,
        sender: ComponentSender<Self>,
        root: &Self::Root,
    ) {
        match msg {
            NewPartitionDialogMsg::Save => {
                root.close();
            }
        }

        self.update_view(widgets, sender);
    }
}
