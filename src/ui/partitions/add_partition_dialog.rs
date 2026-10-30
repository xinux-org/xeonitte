use crate::ui::partitions::partition_model::{PARTITION_BROKER, PartitionMsg};
use gettextrs::gettext;
use relm4::{adw::prelude::*, *};

use relm4::{Component, ComponentParts, ComponentSender, gtk};

#[derive(Debug)]
pub struct XAddPartitionDialogModel {}

#[derive(Debug)]
pub enum XAddPartitionDialogInput {}

#[derive(Debug)]
pub enum XAddPartitionDialogOutput {
    Apply,
}

#[relm4::component(pub)]
impl Component for XAddPartitionDialogModel {
    type CommandOutput = ();
    type Input = XAddPartitionDialogInput;
    type Output = XAddPartitionDialogOutput;
    type Init = ();

    view! {
        dialog = adw::Dialog {
            set_content_width: 450,
            set_content_height: 450,
            set_vexpand: true,
            #[wrap(Some)]
            set_child = &adw::ToolbarView {
                add_top_bar = &adw::HeaderBar {
                    set_show_end_title_buttons: false,
                    #[wrap(Some)]
                    set_title_widget = &adw::WindowTitle {
                        set_title: &gettext("Add partition size"),
                    },
                    pack_start = &gtk::Button {
                        set_label: &gettext("Cancel"),
                        #[watch]
                        set_visible: true,
                        connect_clicked[dialog] => move |_| {
                            dialog.close();
                            dialog.set_can_close(true);
                        }
                    },
                    pack_end = &gtk::Button {
                        set_label: &gettext("Apply"),
                        add_css_class: "suggested-action",
                        #[watch]
                        set_visible: true,
                        // #[watch]
                        // set_sensitive: model.rebuild_sensitive,
                        // connect_clicked[sender] => move |_| {
                        //     sender.input(LanguageModelMsg::CheckSelected);
                        // }
                    },
                },
                #[wrap(Some)]
                set_content = &gtk::ScrolledWindow {
                    set_hexpand: true,
                    set_vexpand: true,
                    // adw::Clamp {
                    //
                    // }
                }
            }
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self {};
        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>, root: &Self::Root) {
        match message {}
    }
}
