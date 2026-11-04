use gettextrs::gettext;
use relm4::{adw::prelude::*, *};

use relm4::{Component, ComponentParts, ComponentSender, gtk};

use crate::utils::SizeType;

#[derive(Debug, Default)]
pub struct XAddPartitionDialogModel {
    pub name: String,
    pub creating_partition: bool,
    pub new_partition_size: size::Size,
    pub free_space: f64,
    pub total_size: u64,
    pub size_type: SizeType,

    pub size_adjustment_value: f64,
}

#[derive(Debug)]
pub enum XAddPartitionDialogInput {
    SetPartitionSize(f64),
}

#[derive(Debug)]
pub enum XAddPartitionDialogOutput {
    Apply,
}

#[relm4::component(pub)]
impl Component for XAddPartitionDialogModel {
    type Init = XAddPartitionDialogModel;
    type Input = XAddPartitionDialogInput;
    type Output = ();
    type CommandOutput = ();

    view! {
        #[root]
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
                    adw::PreferencesPage {
                        // size_scale preperties given on scaler variable
                        // |----O-----|
                        #[name(size_scale)]
                        adw::PreferencesGroup {
                            add: &scaler,
                        },
                        adw::PreferencesGroup {
                          #[name(size_row)]
                          adw::SpinRow {
                            set_title: "Partition Size",
                            #[watch]
                            set_text: &format!("{} MB", model.size_adjustment_value),
                            #[wrap(Some)]
                            #[name(size_adjustment)]
                            set_adjustment = &gtk::Adjustment {
                              #[watch]
                              set_upper: model.free_space,
                              set_step_increment: 1.0,
                              set_page_increment: 10.0,
                              // #[block_signal(upper_handler)]
                              #[watch]
                              set_value: model.size_adjustment_value,
                              connect_value_changed[sender] => move |adjustment| {
                                  sender.input(XAddPartitionDialogInput::SetPartitionSize(adjustment.value()));
                              },
                            },
                          },
                          #[name(free_space_row)]
                          adw::SpinRow {
                            set_title: "Free Space",
                            set_subtitle: "Space following new partition",
                            #[watch]
                            set_text: &format!("{} MB", (model.free_space - model.size_adjustment_value)),
                            #[wrap(Some)]
                            #[name(free_following_adjustment)]
                            set_adjustment = &gtk::Adjustment {
                              #[watch]
                              set_upper: size_adjustment.upper(),
                              set_step_increment: 1.0,
                              set_page_increment: 10.0,
                              #[watch]
                              set_value: size_adjustment.upper() - size_adjustment.value(),
                              // connect_value_changed[sender] => move |adjustment| {
                              //     sender.input(XAddPartitionDialogInput::SetPartitionSize(adjustment.value()));
                              // },
                            },
                          }
                        }
                    }
                }
            }
        },
        scaler = gtk::Scale::builder()
            .digits(0)
            .round_digits(2)
            .adjustment(&size_adjustment)
            .name("Partition size")
            .build(),
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self {
            name: init.name,
            creating_partition: init.creating_partition,
            new_partition_size: init.new_partition_size,
            free_space: init.free_space,
            total_size: init.total_size,
            size_type: init.size_type,
            size_adjustment_value: init.free_space,
        };
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>, _root: &Self::Root) {
        match message {
            XAddPartitionDialogInput::SetPartitionSize(size) => self.size_adjustment_value = size,
        }
    }
}
