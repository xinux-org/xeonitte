use crate::modules::disk;
use crate::modules::disk::fs::ContentUI;
use crate::ui::list::list_model::ListItem;
use crate::{
    modules::disk::{DiskLayout, LinuxFs, MiB, PartitionDef, PartitionSize},
    ui::{
        list::list_model::ListItemMsg,
        partitions::{
            advanced::{
                add_partition_dialog::{
                    AddPartitionDialogInput, AddPartitionDialogModel, AddPartitionStackPage,
                },
                partition_row::{Partition, PartitionOut},
            },
            partition_page::CustomPartition,
        },
    },
    utils::{SizeType, represent},
};
use gettextrs::gettext;
use log::debug;
use relm4::{adw::prelude::*, factory::*, *};
use size::Size;
use std::ops::AddAssign;

pub const FILESYSTEM_LIST_CHOICES: &[LinuxFs] = &[LinuxFs::Exfat, LinuxFs::Btrfs, LinuxFs::Ext4];

#[derive(Debug)]
pub struct AdvancedDiskGroup {
    pub name: String,
    pub brand_name: Option<String>,
    pub partitions: FactoryVecDeque<Partition>,
    pub creating_partition: bool,
    pub new_partition_size: Size,
    pub free_space: u64,
    pub total_size: u64,
    pub size_type: SizeType,
    pub disk_layout: DiskLayout,
    // We create dialog model for each disk.
    pub add_partition_dialog: Option<Controller<AddPartitionDialogModel>>,
}

#[derive(Debug)]
pub enum AdvancedDiskGroupMsg {
    AddPartitionDialog,
    ShowSizeEntry,
    CloseEntry,
    Input(Option<String>),
    // Apply,
    Delete(String),
    FileSystemSelect(String),
    FileSystemDeSelect(String),
    // SetSizeType(SizeType),
    // Validate,
    NoOperation,
    PartitionContent(String),
}

pub enum AdvancedDiskGroupOut {
    ApplyOut(String, CustomPartition),
    Delete(String),
}

#[relm4::factory(pub)]
impl FactoryComponent for AdvancedDiskGroup {
    type Init = AdvancedDiskGroup;
    type Input = AdvancedDiskGroupMsg;
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
                            set_text: &self.brand_name.as_ref().map_or_else(|| self.name.clone(), |brand_name| { brand_name.to_owned() }) ,
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
                                    set_text: &self.free_space.to_string(),
                                },
                            },
                            gtk::Box {
                                set_orientation: gtk::Orientation::Horizontal,
                                set_halign: gtk::Align::End,
                                set_valign: gtk::Align::Center,
                                set_spacing: 12,
                                #[name = "add_partition_button"]
                                gtk::Button {
                                    #[watch]
                                    set_icon_name: "value-increase",
                                    add_css_class: "circular",
                                    connect_clicked[sender] => move |_| {
                                        sender.input(AdvancedDiskGroupMsg::AddPartitionDialog);
                                    }
                                },
                            },
                        },
                    },
                   // TODO: make the logic better
                    #[local_ref]
                    partition_box -> gtk::ListBox {
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
                            set_subtitle: &self.free_space.to_string(),
                            set_selectable: false,
                            set_activatable: false,
                        },
                    },
                   // TODO: closing
                    // #[name = "new_partition_box"]
                    // gtk::Box {
                    //     #[watch]
                    //     set_visible: self.creating_partition,
                    //     set_orientation: gtk::Orientation::Horizontal,
                    //     set_valign: gtk::Align::Center,
                    //     set_spacing: 12,
                    //     set_margin_top: 6,
                    //     #[name = "size_entry"]
                    //     adw::EntryRow {
                    //         #[watch]
                    //         set_title: &gettext("Enter the new partition size"),
                    //         set_input_purpose: gtk::InputPurpose::Number,
                    //         set_max_length: 12,
                    //         set_activates_default: true,
                    //         set_hexpand: true,
                    //         set_show_apply_button: false,
                    //         #[iterate]
                    //         add_css_class: ["focused", "frame"],
                    //         inline_css: "padding-top: 6px; padding-bottom: 6px; border-radius: 12px;",
                    //         connect_changed[sender] => move |x| {
                    //             sender.input(PartitionGroupMsg::Input({
                    //                 let text = x.text();
                    //                 if text.is_empty() {
                    //                     None
                    //                 } else {
                    //                     Some(text.into())
                    //                 }
                    //             }));
                    //         }
                    //     },
                    //     #[name = "dropdown"]
                    //     gtk::DropDown {
                    //         set_valign: gtk::Align::Center,
                    //         set_model: Some(&gtk::StringList::new(&["TiB", "GiB", "MiB", "KiB"])),
                    //         connect_selected_item_notify[sender] => move |row| {
                    //             let t = match row.selected() {
                    //                 0 => SizeType::TB,
                    //                 1 => SizeType::GB,
                    //                 3 => SizeType::KB,
                    //                 _ => SizeType::MB,
                    //             };
                    //             sender.input(PartitionGroupMsg::SetSizeType(t));
                    //         },
                    //     },
                    //     gtk::Button {
                    //        set_icon_name: "value-decrease",
                    //        #[iterate]
                    //        add_css_class: ["raised", "circular", "destructive-action"],
                    //        set_valign: gtk::Align::Center,
                    //        connect_clicked => PartitionGroupMsg::CloseEntry,
                    //     },
                    //     #[name = "apply_button"]
                    //     gtk::Button {
                    //         set_icon_name: "adw-entry-apply-symbolic",
                    //         set_valign: gtk::Align::Center,
                    //         #[iterate]
                    //         add_css_class: ["suggested-action", "circular", "apply-button", "image-button", "disabled"],
                    //         connect_activate => PartitionGroupMsg::Apply,
                    //         connect_clicked => PartitionGroupMsg::Apply,
                    //     },
                    // },
                },
            },
        }
    }

    fn init_model(disk: Self::Init, _index: &DynamicIndex, sender: FactorySender<Self>) -> Self {
        // ALL_DISKS.subscribe(sender.input_sender(), |state| PartitionMsg::XXX {
        //     system_packages: state.installed_system_packages.clone(),
        //     user_packages: state.installed_user_packages.clone(),
        // });
        let mut dialog_model = AddPartitionDialogModel {
            name: disk.name.clone(),
            creating_partition: disk.creating_partition,
            new_partition_size: disk.new_partition_size,
            free_space: disk.free_space as f64,
            total_size: disk.total_size,
            size_type: disk.size_type,
            size_adjustment_value: disk.free_space as f64,
            stack_page: AddPartitionStackPage::default(),
            filesystem_list_factory: FactoryVecDeque::builder().launch_default().forward(
                sender.input_sender(),
                |msg| match msg {
                    ListItemMsg::Select(id, _title) => AdvancedDiskGroupMsg::FileSystemSelect(id),
                    ListItemMsg::Deselect(_key) => AdvancedDiskGroupMsg::NoOperation,
                },
            ),
            mountpoint_list_factory: FactoryVecDeque::builder().launch_default().forward(
                sender.input_sender(),
                |msg| match msg {
                    ListItemMsg::Select(_id, title) => {
                        AdvancedDiskGroupMsg::FileSystemSelect(title)
                    }
                    ListItemMsg::Deselect(key) => AdvancedDiskGroupMsg::FileSystemDeSelect(key),
                },
            ),
            content_choise: ContentUI::Filesystem,
            content_list_factory: FactoryVecDeque::builder().launch_default().forward(
                sender.input_sender(),
                |msg| match msg {
                    ListItemMsg::Select(id, _title) => AdvancedDiskGroupMsg::PartitionContent(id),
                    ListItemMsg::Deselect(_key) => AdvancedDiskGroupMsg::NoOperation,
                },
            ),
        };

        // filesystem widget components
        let mut filesystem_list_guard = dialog_model.filesystem_list_factory.guard();
        let fs_group = gtk::CheckButton::new();
        for choice in FILESYSTEM_LIST_CHOICES {
            let item = ListItem {
                id: choice.as_str().to_owned().into(),
                title: choice.as_str().to_owned(),
                description: "description".into(),
                group: Some(fs_group.clone()),
                selected: choice == &LinuxFs::Exfat,
                ..Default::default()
            };
            filesystem_list_guard.push_back(item);
        }
        filesystem_list_guard.drop();

        // Partiiton Content (ext4, swap, luks) widget components. Single selection
        let mut content_list_factory_guard = dialog_model.content_list_factory.guard();
        let content_list_group = gtk::CheckButton::new();
        for choice in &ContentUI::content_list() {
            let mut item = choice.to_list_item();
            item.group = content_list_group.clone().into();
            content_list_factory_guard.push_back(item);
        }
        content_list_factory_guard.drop();

        let add_partition_dialog = AddPartitionDialogModel::builder()
            .launch(dialog_model)
            .detach();

        let mut partitions = FactoryVecDeque::builder()
            .launch(gtk::ListBox::default())
            .forward(sender.input_sender(), |output| match output {
                PartitionOut::Delete(name) => AdvancedDiskGroupMsg::Delete(name),
            });

        let _ = disk
            .partitions
            .iter()
            .map(|x| {
                let p = Partition {
                    name: x.name.clone(),
                    size: x.size,
                    mountrow: adw::ComboRow::new(),
                    device: x.device.clone(),
                    is_swap: false,
                    is_boot: false,
                    is_full: false,
                    // donotmount: gettext("Do not mount"),
                    donotformat: gettext("Leave as is"),
                    partition_disko: PartitionDef {
                        label: Some(x.name.clone()),
                        size: PartitionSize::Mib(disk::Size::<MiB>::new(x.size)),
                        type_code: None,
                        content: None,
                    },
                };
                partitions.guard().push_back(p);
            })
            .collect::<Vec<_>>();

        Self {
            name: disk.name,
            brand_name: disk.brand_name,
            creating_partition: disk.creating_partition,
            new_partition_size: disk.new_partition_size,
            free_space: disk.free_space,
            total_size: disk.total_size,
            size_type: disk.size_type,
            partitions,
            disk_layout: disk.disk_layout,
            add_partition_dialog: Some(add_partition_dialog),
        }
    }

    fn init_widgets(
        &mut self,
        _index: &DynamicIndex,
        root: Self::Root,
        _returned_widget: &<Self::ParentWidget as FactoryView>::ReturnedWidget,
        sender: FactorySender<Self>,
    ) -> Self::Widgets {
        let partition_box = self.partitions.widget();
        let widgets = view_output!();
        widgets
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::Input,
        sender: FactorySender<Self>,
    ) {
        match message {
            AdvancedDiskGroupMsg::ShowSizeEntry => {
                self.creating_partition = true;
                // let tip: SizeType = match self
                //     .free_space
                //     .to_string()
                //     .split(" ")
                //     .nth(1)
                //     .unwrap_or_default()
                // {
                //     "TiB" => {
                //         widgets.dropdown.set_selected(0);
                //         SizeType::TB
                //     }
                //     "GiB" => {
                //         widgets.dropdown.set_selected(1);
                //         SizeType::GB
                //     }
                //     "MiB" => {
                //         widgets.dropdown.set_selected(2);
                //         SizeType::MB
                //     }
                //     _ => {
                //         widgets.dropdown.set_selected(3);
                //         SizeType::KB
                //     }
                // };
                // self.size_type = tip;
                // self.new_partition_size = self.free_space;
                // widgets.size_entry.set_text(
                //     format_size(self.new_partition_size)
                //         .split(" ")
                //         .nth(0)
                //         .unwrap_or_default(),
                // );
                // widgets.size_entry.add_css_class("focused");
                // widgets.apply_button.set_can_target(true);
                // widgets.apply_button.remove_css_class("dimmed");
            }

            AdvancedDiskGroupMsg::CloseEntry => {
                self.creating_partition = false;
                // widgets.size_entry.remove_css_class("focused");
                // widgets.apply_button.set_can_target(false);
                // widgets.apply_button.add_css_class("dimmed");
            }

            AdvancedDiskGroupMsg::Input(x) => {
                let x = x.unwrap_or_default();
                match x.parse::<f64>() {
                    Ok(y) => {
                        self.new_partition_size = represent(self.size_type, y);

                        // sender.input(PartitionGroupMsg::Validate);
                    }
                    Err(_) => {
                        if x.is_empty() {
                            self.new_partition_size = Size::default();
                        }

                        // widgets.size_entry.add_css_class("error");
                        // widgets.apply_button.set_can_target(false);
                        // widgets.apply_button.add_css_class("dimmed");
                    }
                }
            }

            //             PartitionGroupMsg::Apply => {
            //                 if self.new_partition_size.bytes().is_positive() {
            //                     let index = self.partitions.len() + 1;
            //                     // let mut new_size = represent(
            //                     //     self.size_type,
            //                     //     widgets.size_entry.text().parse::<f64>().unwrap_or_default(),
            //                     // );
            //                     let free_size_ui =
            //                         Size::from_str(widgets.free_space.text().as_str()).unwrap_or_default();
            //
            //                     let is_full = new_size.eq(&free_size_ui);
            //                     if is_full {
            //                         new_size = self.free_space;
            //                         self.free_space = Size::from_bytes(0);
            //                     } else {
            //                         self.free_space.sub_assign(self.new_partition_size);
            //                     }
            //                     let device = self
            //                         .partitions
            //                         .front()
            //                         .unwrap_or(&Partition {
            //                             name: self.name.clone() + "1",
            //                             size: 0,
            //                             mountrow: adw::ComboRow::new(),
            //                             device: self.name.clone(),
            //                             is_swap: false,
            //                             is_boot: false,
            //                             is_full,
            //                             donotmount: "".to_string(),
            //                             donotformat: "".to_string(),
            //                         })
            //                         .device
            //                         .clone();
            //
            //                     // fix partitioning bugs from removing 20MB
            //                     // if new_size.ge(&Size::from_gb(1)) {
            //                     //     new_size.sub_assign(Size::from_mb(20));
            //                     // }
            //                     self.partitions.guard().push_back({
            //                         PartitionInit {
            //                             name: format!("{device}{index}"),
            //                             size: new_size.bytes() as u64,
            //                             mountrow: adw::ComboRow::new(),
            //                             device,
            //                         }
            //                     });
            //                     sender.input(PartitionGroupMsg::CloseEntry);
            //                 }
            //                 sender.input(PartitionGroupMsg::Validate);
            //             }
            AdvancedDiskGroupMsg::Delete(name) => {
                let (index, partition) = self
                    .partitions
                    .iter()
                    .cloned()
                    .enumerate()
                    .find_map(|(i, x)| if x.name == name { Some((i, x)) } else { None })
                    .unwrap();

                self.free_space.add_assign(partition.size);
                self.partitions.guard().remove(index);
            }

            //             PartitionGroupMsg::SetSizeType(x) => {
            //                 // let new_size: Size = represent(
            //                 //     x,
            //                 //     widgets.size_entry.text().parse::<f64>().unwrap_or_default(),
            //                 // );
            //                 self.size_type = x;
            //                 self.new_partition_size = new_size;
            //                 // if self.free_space.ge(&new_size) {
            //                 //     widgets.size_entry.remove_css_class("error");
            //                 //     widgets.apply_button.set_can_target(true);
            //                 //     widgets.apply_button.remove_css_class("dimmed");
            //                 // } else {
            //                 //     widgets.size_entry.add_css_class("error");
            //                 //     widgets.apply_button.set_can_target(false);
            //                 //     widgets.apply_button.add_css_class("dimmed");
            //                 // }
            //                 // widgets.size_entry.add_css_class("focused");
            //             }
            //
            //             PartitionGroupMsg::Validate => {
            //                 // if self.free_space.ge(&self.new_partition_size) {
            //                 //     widgets.size_entry.remove_css_class("error");
            //                 //     widgets.apply_button.set_can_target(true);
            //                 //     widgets.apply_button.remove_css_class("dimmed");
            //                 // } else {
            //                 //     widgets.size_entry.add_css_class("error");
            //                 //     widgets.apply_button.set_can_target(false);
            //                 //     widgets.apply_button.add_css_class("dimmed");
            //                 // }
            //             }
            AdvancedDiskGroupMsg::AddPartitionDialog => {
                if let Some(dialog) = &self.add_partition_dialog {
                    dialog
                        .widget()
                        .present(relm4::main_application().active_window().as_ref());
                }
            }
            AdvancedDiskGroupMsg::FileSystemSelect(key) => {
                if let Some(dialog) = &self.add_partition_dialog {
                    dialog.emit(AddPartitionDialogInput::FileSystemSelect(key))
                }
            }
            AdvancedDiskGroupMsg::FileSystemDeSelect(key) => {
                if let Some(dialog) = &self.add_partition_dialog {
                    dialog.emit(AddPartitionDialogInput::FileSystemDeSelect(key))
                }
            }
            AdvancedDiskGroupMsg::NoOperation => {}
            AdvancedDiskGroupMsg::PartitionContent(partition_content) => {
                debug!("AdvancedDiskGroupMsg::PartitionContent: {partition_content}");
                if let Some(dialog) = &self.add_partition_dialog {
                    dialog.emit(AddPartitionDialogInput::SetContent(partition_content))
                }
            }
        }
        self.update_view(widgets, sender);
    }
}
