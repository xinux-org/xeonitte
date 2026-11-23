use crate::modules::disk::fs::ContentUI;
use crate::modules::disk::{self, GiB};
use crate::ui::list::list_model::ListItem;
use crate::{
    modules::disk::{DiskLayout, LinuxFs, MiB, PartitionDef},
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

pub const FILESYSTEM_LIST_CHOICES: &[LinuxFs] = &[LinuxFs::Exfat, LinuxFs::Btrfs, LinuxFs::Ext4];

#[derive(Debug)]
pub struct AdvancedDiskGroup {
    pub disk_layout: DiskLayout,
    pub name: String,
    pub brand_name: Option<String>,
    pub partitions: FactoryVecDeque<Partition>,
    pub creating_partition: bool,
    pub new_partition_size: size::Size,
    pub size_type: SizeType,
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
                                    set_text: &self.disk_layout.total_space::<disk::GiB>().to_disko_str_ui(),
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
                                    set_text: &self.disk_layout.free_space::<disk::GiB>().to_disko_str_ui(),
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
                            set_subtitle: &self.disk_layout.free_space::<GiB>().to_disko_str_ui(),
                            set_selectable: false,
                            set_activatable: false,
                        },
                    },
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
            disk_layout: disk.disk_layout.clone(),
            name: disk.name.clone(),
            creating_partition: disk.creating_partition,
            new_partition_size: disk.new_partition_size,
            size_adjustment_value: disk.disk_layout.free_space::<MiB>().to_bytes() as f64,
            stack_page: AddPartitionStackPage::default(),
            content_choise: ContentUI::Filesystem,
            content_list_factory: FactoryVecDeque::builder().launch_default().forward(
                sender.input_sender(),
                |msg| match msg {
                    ListItemMsg::Select(id, _title) => AdvancedDiskGroupMsg::PartitionContent(id),
                    ListItemMsg::Deselect(_key) => AdvancedDiskGroupMsg::NoOperation,
                },
            ),
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
        };

        // Partiiton Content (ext4, swap, luks) widget components. Single selection
        let mut content_list_factory_guard = dialog_model.content_list_factory.guard();
        let content_list_group = gtk::CheckButton::new();
        for choice in &ContentUI::content_list() {
            let mut item = choice.to_list_item();
            item.group = content_list_group.clone().into();
            content_list_factory_guard.push_back(item);
        }
        content_list_factory_guard.drop();

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

        let add_partition_dialog = AddPartitionDialogModel::builder()
            .launch(dialog_model)
            .detach();
        add_partition_dialog.emit(AddPartitionDialogInput::SetContent("fs".into()));

        let mut partitions = FactoryVecDeque::builder()
            .launch(gtk::ListBox::default())
            .forward(sender.input_sender(), |output| match output {
                PartitionOut::Delete(name) => AdvancedDiskGroupMsg::Delete(name),
            });

        let _ = disk
            .partitions
            .iter()
            .map(|partition| {
                let p = Partition {
                    mountrow: adw::ComboRow::new(),
                    device: partition.device.clone(),
                    is_swap: false,
                    is_boot: false,
                    is_full: false,
                    // donotmount: gettext("Do not mount"),
                    donotformat: gettext("Leave as is"),
                    partition_disko: PartitionDef {
                        label: partition.partition_disko.label.clone(),
                        size: partition.partition_disko.size.clone(),
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
                            // self.new_partition_size = Size::default();
                        }

                        // widgets.size_entry.add_css_class("error");
                        // widgets.apply_button.set_can_target(false);
                        // widgets.apply_button.add_css_class("dimmed");
                    }
                }
            }
            AdvancedDiskGroupMsg::Delete(name) => {
                let (index, partition) = self
                    .partitions
                    .iter()
                    .enumerate()
                    .find(|(_, partition)| {
                        partition
                            .partition_disko
                            .label
                            .as_deref()
                            .map(|label| label == name)
                            .unwrap_or_default()
                    })
                    .unwrap();

                // self.free_space.add_assign(partition.partition_disko.size.to_bytes(disk_total));
                self.partitions.guard().remove(index);
            }
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
                if let Some(dialog) = &self.add_partition_dialog {
                    dialog.emit(AddPartitionDialogInput::SetContent(partition_content))
                }
            }
        }
        self.update_view(widgets, sender);
    }
}
