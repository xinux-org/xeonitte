use crate::{
    format_size,
    ui::partitions::{
        partition::{Partition, PartitionInit, PartitionOut},
        partition_model::CustomPartition,
    },
    utils::{SizeType, represent},
};
use gettextrs::gettext;
use relm4::{adw::prelude::*, factory::*, *};
use size::Size;
use std::ops::{AddAssign, SubAssign};

#[derive(Debug, PartialEq)]
pub struct PartitionGroup {
    pub name: String,
    pub partitions: FactoryVecDeque<Partition>,
    pub creating_partition: bool,
    pub new_partition_size: Size,
    pub free_space: Size,
    pub total_size: u64,
    pub size_type: SizeType,
}

#[derive(Debug)]
pub enum PartitionGroupMsg {
    ShowSizeEntry,
    CloseEntry,
    Input(Option<String>),
    Apply,
    Delete(String),
    SetSizeType(SizeType),
    Validate,
}

pub enum PartitionGroupOut {
    ApplyOut(String, CustomPartition),
    Delete(String),
}

#[relm4::factory(pub)]
impl FactoryComponent for PartitionGroup {
    type Init = PartitionGroup;
    type Input = PartitionGroupMsg;
    type Output = ();
    type ParentWidget = gtk::Box;
    type CommandOutput = ();

    view! {
        adw::PreferencesGroup {
            gtk::Box {
                set_visible: !self.name.contains("zram"),
                set_hexpand: true,
                set_orientation: gtk::Orientation::Horizontal,
                set_spacing: 8,
                gtk::Box {
                    set_hexpand: true,
                    set_orientation: gtk::Orientation::Vertical,
                    set_spacing: 6,
                    gtk::Box {
                        set_hexpand: true,
                        set_orientation: gtk::Orientation::Horizontal,
                        set_spacing: 6,
                        set_margin_bottom: 6,
                        gtk::Label {
                            set_text: &self.name,
                            set_margin_all: 6,
                            add_css_class: "heading"
                        },
                        gtk::Box {
                            set_hexpand: true,
                            set_halign: gtk::Align::End,
                            set_spacing: 12,
                            gtk::Box {
                                set_orientation: gtk::Orientation::Horizontal,
                                set_spacing: 2,
                                gtk::Label {
                                    #[watch]
                                    set_text: &format!("{}: ", gettext("Total")),
                                    add_css_class: "heading"
                                },
                                gtk::Label {
                                    #[watch]
                                    set_text: &size::Size::from_bytes::<u64>(self.total_size).to_string(),
                                },
                            },
                            gtk::Box {
                                set_orientation: gtk::Orientation::Horizontal,
                                set_spacing: 2,
                                add_css_class: "success",
                                gtk::Label {
                                    #[watch]
                                    set_text: &format!("{}: ", gettext("Free")),
                                    add_css_class: "heading"
                                },
                                #[name = "free_space"]
                                gtk::Label {
                                    #[watch]
                                    set_text: &format_size(self.free_space),
                                },
                            },
                            gtk::Box {
                                set_orientation: gtk::Orientation::Horizontal,
                                set_halign: gtk::Align::End,
                                set_valign: gtk::Align::Center,
                                set_spacing: 12,
                                // gtk::Box {
                                //     set_orientation: gtk::Orientation::Horizontal,
                                //     add_css_class: "linked",
                                //     gtk::Button {
                                //         set_icon_name: "edit-undo",
                                //         add_css_class: "raised",
                                //     },
                                //     gtk::Button {
                                //         set_icon_name: "edit-redo",
                                //         add_css_class: "raised"
                                //     }
                                // },
                                #[name = "add_partition_button"]
                                gtk::Button {
                                    #[watch]
                                    set_icon_name: "value-increase",
                                    add_css_class: "circular",
                                    connect_clicked[sender] => move |_| {
                                        sender.input(PartitionGroupMsg::ShowSizeEntry);
                                    }
                                },
                            },
                        },
                    },
                   // TODO: make the logic better
                    #[local_ref]
                    testbox -> gtk::ListBox {
                        #[watch]
                        set_visible: !self.partitions.is_empty(),
                        set_hexpand: true,
                        set_selection_mode: gtk::SelectionMode::None,
                        add_css_class: "boxed-list",
                    },
                    gtk::ListBox {
                        #[watch]
                        set_visible: self.partitions.is_empty(),
                        add_css_class: "boxed-list",
                        adw::ActionRow {
                            set_title: &gettext("Free space"),
                            #[watch]
                            set_subtitle: &format_size(self.free_space),
                            set_selectable: false,
                            set_activatable: false,
                        },
                    },
                   // TODO: closing
                    #[name = "new_partition_box"]
                    gtk::Box {
                        #[watch]
                        set_visible: self.creating_partition,
                        set_orientation: gtk::Orientation::Horizontal,
                        set_valign: gtk::Align::Center,
                        set_spacing: 12,
                        set_margin_top: 6,
                        #[name = "size_entry"]
                        adw::EntryRow {
                            #[watch]
                            set_title: &gettext("Enter the new partition size"),
                            set_input_purpose: gtk::InputPurpose::Number,
                            set_max_length: 12,
                            set_activates_default: true,
                            set_hexpand: true,
                            set_show_apply_button: false,
                            #[iterate]
                            add_css_class: ["focused", "frame"],
                            inline_css: "padding-top: 6px; padding-bottom: 6px; border-radius: 12px;",
                            connect_changed[sender] => move |x| {
                                sender.input(PartitionGroupMsg::Input({
                                    let text = x.text();
                                    if text.is_empty() {
                                        None
                                    } else {
                                        Some(text.into())
                                    }
                                }));
                            }
                        },
                        #[name = "dropdown"]
                        gtk::DropDown {
                            set_valign: gtk::Align::Center,
                            set_model: Some(&gtk::StringList::new(&["TiB", "GiB", "MiB", "KiB"])),
                            connect_selected_item_notify[sender, x = self.new_partition_size.clone().bytes()] => move |row| {
                                let t = match row.selected() {
                                    0 => SizeType::TB,
                                    1 => SizeType::GB,
                                    3 => SizeType::KB,
                                    _ => SizeType::MB,
                                };
                                sender.input(PartitionGroupMsg::SetSizeType(t));
                            },
                        },
                        gtk::Button {
                           set_icon_name: "value-decrease",
                           #[iterate]
                           add_css_class: ["raised", "circular", "destructive-action"],
                           set_valign: gtk::Align::Center,
                           connect_clicked => PartitionGroupMsg::CloseEntry,
                        },
                        #[name = "apply_button"]
                        gtk::Button {
                            set_icon_name: "adw-entry-apply-symbolic",
                            set_valign: gtk::Align::Center,
                            #[iterate]
                            add_css_class: ["suggested-action", "circular", "apply-button", "image-button", "disabled"],
                            connect_activate => PartitionGroupMsg::Apply,
                            connect_clicked => PartitionGroupMsg::Apply,
                        },
                    },
                },
            },
        }
    }

    fn init_model(parent: Self::Init, _index: &DynamicIndex, sender: FactorySender<Self>) -> Self {
        let mut partitions = FactoryVecDeque::builder()
            .launch(gtk::ListBox::default())
            .forward(sender.input_sender(), |output| match output {
                PartitionOut::Delete(x) => PartitionGroupMsg::Delete(x),
            });

        let _ = parent
            .partitions
            .iter()
            .map(|x| {
                let p = PartitionInit {
                    name: x.name.clone(),
                    device: x.device.clone(),
                    mountrow: adw::ComboRow::new(),
                    size: x.size,
                };
                partitions.guard().push_back(p);
            })
            .collect::<Vec<_>>();

        Self {
            name: parent.name,
            creating_partition: parent.creating_partition,
            new_partition_size: parent.new_partition_size,
            free_space: parent.free_space,
            total_size: parent.total_size,
            size_type: parent.size_type,
            partitions,
        }
    }

    fn init_widgets(
        &mut self,
        _index: &DynamicIndex,
        root: Self::Root,
        _returned_widget: &<Self::ParentWidget as FactoryView>::ReturnedWidget,
        sender: FactorySender<Self>,
    ) -> Self::Widgets {
        let testbox = self.partitions.widget();
        let widgets = view_output!();
        widgets.dropdown.set_selected(2);
        widgets
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::Input,
        sender: FactorySender<Self>,
    ) {
        match message {
            PartitionGroupMsg::ShowSizeEntry => {
                self.creating_partition = true;
                let tip = match self
                    .free_space
                    .to_string()
                    .split(" ")
                    .nth(1)
                    .unwrap_or_default()
                {
                    "TiB" => {
                        widgets.dropdown.set_selected(0);
                        SizeType::TB
                    }
                    "GiB" => {
                        widgets.dropdown.set_selected(1);
                        SizeType::GB
                    }
                    "MiB" => {
                        widgets.dropdown.set_selected(2);
                        SizeType::MB
                    }
                    _ => {
                        widgets.dropdown.set_selected(3);
                        SizeType::KB
                    }
                };
                self.size_type = tip;
                self.new_partition_size = self.free_space;
                widgets.size_entry.set_text(
                    format_size(self.new_partition_size)
                        .split(" ")
                        .nth(0)
                        .unwrap_or_default(),
                );
                widgets.size_entry.add_css_class("focused");
                widgets.apply_button.set_can_target(true);
                widgets.apply_button.remove_css_class("dimmed");
            }

            PartitionGroupMsg::CloseEntry => {
                self.creating_partition = false;
                widgets.size_entry.remove_css_class("focused");
                widgets.apply_button.set_can_target(false);
                widgets.apply_button.add_css_class("dimmed");
            }

            PartitionGroupMsg::Input(x) => {
                let x = x.unwrap_or_default();
                match x.parse::<f64>() {
                    Ok(y) => {
                        self.new_partition_size = represent(self.size_type, y);

                        sender.input(PartitionGroupMsg::Validate);
                    }
                    Err(_) => {
                        if x.is_empty() {
                            self.new_partition_size = Size::default();
                        }

                        widgets.size_entry.add_css_class("error");
                        widgets.apply_button.set_can_target(false);
                        widgets.apply_button.add_css_class("dimmed");
                    }
                }
            }

            PartitionGroupMsg::Apply => {
                if self.new_partition_size.bytes().is_positive() {
                    let index = self.partitions.len() + 1;
                    let mut new_size = represent(
                        self.size_type,
                        widgets.size_entry.text().parse::<f64>().unwrap_or_default(),
                    );
                    let free_size_ui =
                        Size::from_str(widgets.free_space.text().as_str()).unwrap_or_default();

                    let is_full = new_size.eq(&free_size_ui);
                    if is_full {
                        new_size = self.free_space;
                        self.free_space = Size::from_bytes(0);
                    } else {
                        self.free_space.sub_assign(self.new_partition_size);
                    }
                    let device = self
                        .partitions
                        .front()
                        .unwrap_or(&Partition {
                            name: self.name.clone() + "1",
                            size: 0,
                            mountrow: adw::ComboRow::new(),
                            device: self.name.clone(),
                            is_swap: false,
                            is_boot: false,
                            is_full,
                            donotmount: "".to_string(),
                            donotformat: "".to_string(),
                        })
                        .device
                        .clone();

                    // fix partitioning bugs from removing 20MB
                    // if new_size.ge(&Size::from_gb(1)) {
                    //     new_size.sub_assign(Size::from_mb(20));
                    // }
                    self.partitions.guard().push_back({
                        PartitionInit {
                            name: format!("{device}{index}"),
                            size: new_size.bytes() as u64,
                            mountrow: adw::ComboRow::new(),
                            device,
                        }
                    });
                    sender.input(PartitionGroupMsg::CloseEntry);
                }
                sender.input(PartitionGroupMsg::Validate);
            }

            PartitionGroupMsg::Delete(name) => {
                let (index, x) = self
                    .partitions
                    .iter()
                    .cloned()
                    .enumerate()
                    .find_map(|(i, x)| if x.name == name { Some((i, x)) } else { None })
                    .unwrap_or_default();

                self.free_space.add_assign(Size::from_bytes(x.size));
                self.partitions.guard().remove(index);
            }

            PartitionGroupMsg::SetSizeType(x) => {
                let new_size = represent(
                    x,
                    widgets.size_entry.text().parse::<f64>().unwrap_or_default(),
                );
                self.size_type = x;
                self.new_partition_size = new_size;
                if self.free_space.ge(&new_size) {
                    widgets.size_entry.remove_css_class("error");
                    widgets.apply_button.set_can_target(true);
                    widgets.apply_button.remove_css_class("dimmed");
                } else {
                    widgets.size_entry.add_css_class("error");
                    widgets.apply_button.set_can_target(false);
                    widgets.apply_button.add_css_class("dimmed");
                }
                widgets.size_entry.add_css_class("focused");
            }

            PartitionGroupMsg::Validate => {
                if self.free_space.ge(&self.new_partition_size) {
                    widgets.size_entry.remove_css_class("error");
                    widgets.apply_button.set_can_target(true);
                    widgets.apply_button.remove_css_class("dimmed");
                } else {
                    widgets.size_entry.add_css_class("error");
                    widgets.apply_button.set_can_target(false);
                    widgets.apply_button.add_css_class("dimmed");
                }
            }
        }
        self.update_view(widgets, sender);
    }
}
