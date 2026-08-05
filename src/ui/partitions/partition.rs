use crate::ui::partitions::partition_model::{PARTITION_BROKER, PartitionMsg};
use gettextrs::gettext;
use relm4::{adw::prelude::*, factory::*, *};

#[derive(Default, Debug, PartialEq, Eq, Clone)]
pub struct Partition {
    pub name: String,
    pub size: u64,
    pub mountrow: adw::ComboRow,
    pub device: String,
    pub is_swap: bool,
    pub is_boot: bool,
    pub is_full: bool,
    pub donotmount: String,
    pub donotformat: String,
}

#[derive(Default, Debug, PartialEq, Eq, Clone)]
pub struct PartitionInit {
    pub name: String,
    pub size: u64,
    pub mountrow: adw::ComboRow,
    pub device: String,
}

#[derive(Debug)]
pub enum PartitionRowMsg {
    Deselect(String),
    SetSwap(bool),
    SetBoot(bool),
    Delete,
}

#[derive(Debug)]
pub enum PartitionOut {
    Delete(String),
}

#[relm4::factory(pub)]
impl FactoryComponent for Partition {
    type Init = PartitionInit;
    type Input = PartitionRowMsg;
    type Output = PartitionOut;
    type ParentWidget = gtk::ListBox;
    type CommandOutput = ();

    view! {
        adw::ExpanderRow {
            set_title: &self.name,
            set_subtitle: &size::Size::from_bytes(self.size).to_string(),
            add_row = &adw::ComboRow {
                #[watch]
                set_title: &gettext("Format"),
                // TODO: When switching language the "Leave as is" option does not update
                set_model: Some(&gtk::StringList::new(&[&self.donotformat, "btrfs", "ext4", "ext3", "vfat", "ntfs", "xfs", "swap"])),
                connect_selected_notify[sender, name = self.name.to_string(), device = self.device.to_string(), formatstring = self.donotformat.to_string(), size = self.size, is_full = self.is_full] => move |row| {
                    if let Some(item) = row.selected_item() && let Ok(item) = item.downcast::<gtk::StringObject>() {
                        if item.string() == formatstring {
                            PARTITION_BROKER.send(PartitionMsg::RemoveFormatPartition(name.to_string()));
                        } else {
                            PARTITION_BROKER.send(PartitionMsg::AddFormatPartition(name.to_string(), item.string().to_string(), device.to_string(), size, is_full));
                        }
                        sender.input(PartitionRowMsg::SetSwap(item.string().eq("swap")));
                    }
                }
            },
            #[local_ref]
            add_row = mountrow -> adw::ComboRow {
                #[watch]
                set_visible: !self.is_swap,
                #[watch]
                set_title: &gettext("Mount"),
                // TODO: When switching language the "Do not mount" option does not update
                set_model: Some(&gtk::StringList::new(&[&self.donotmount, " /", "/boot", "/home", "/opt", "/var", "/nix"])),
                connect_selected_notify[sender, name = self.name.to_string(), device = self.device.to_string(), mountstring = self.donotmount.to_string(), size = self.size, is_full = self.is_full] => move |row| {
                    if let Some(item) = row.selected_item() && let Ok(item) = item.downcast::<gtk::StringObject>(){
                        let x = item.string();
                        if x == mountstring {
                            PARTITION_BROKER.send(PartitionMsg::RemoveMountPartition(name.to_string()));
                        } else {
                            PARTITION_BROKER.send(PartitionMsg::AddMountPartition(name.to_string(), item.string().trim().to_string(), device.to_string(), size, is_full));
                        }
                        sender.input(PartitionRowMsg::SetBoot(x.eq("/boot")));
                    }
                }
            },
            add_row = &adw::ActionRow {
                #[watch]
                set_visible: self.is_swap,
                #[watch]
                set_title: &gettext("Mount"),
                add_suffix = &gtk::Label {
                    set_text: "swap",
                }
            },
            add_row = &adw::SwitchRow {
                #[watch]
                set_visible: !self.is_swap && !self.is_boot,
                #[watch]
                set_title: &gettext("Encrypt"),
                #[watch]
                set_subtitle: &gettext("Encrypt this partition with LUKS"),
                connect_active_notify[name = self.name.to_string(), device = self.device.to_string(), size = self.size, is_full = self.is_full] => move |row| {
                    PARTITION_BROKER.send(PartitionMsg::SetPartitionEncryption(
                        name.to_string(),
                        device.to_string(),
                        size,
                        row.is_active(),
                        is_full
                    ));
                }
            },
            add_row = &adw::ActionRow {
                set_activatable: false,
                add_suffix = &gtk::Button {
                    #[watch]
                    set_label: &gettext("Delete"),
                    add_css_class: "raised",
                    add_css_class: "destructive-action",
                    set_halign: gtk::Align::End,
                    set_margin_top: 8,
                    set_margin_bottom: 8,

                    connect_activate => PartitionRowMsg::Delete,
                    connect_clicked => PartitionRowMsg::Delete
                },
            },
        }
    }
    fn init_model(parent: Self::Init, _index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        Partition {
            name: parent.name,
            size: parent.size,
            mountrow: parent.mountrow,
            device: parent.device,
            is_swap: false,
            is_boot: false,
            is_full: false,
            donotmount: gettext("Do not mount"),
            donotformat: gettext("Leave as is"),
        }
    }
    fn init_widgets(
        &mut self,
        _index: &DynamicIndex,
        root: Self::Root,
        _returned_widget: &<Self::ParentWidget as FactoryView>::ReturnedWidget,
        sender: FactorySender<Self>,
    ) -> Self::Widgets {
        let mountrow = &self.mountrow.clone();
        let widgets = view_output!();
        widgets
    }
    fn update(&mut self, msg: Self::Input, _sender: FactorySender<Self>) {
        match msg {
            PartitionRowMsg::Deselect(mount) => {
                if let Some(item) = self.mountrow.selected_item()
                    && let Ok(item) = item.downcast::<gtk::StringObject>()
                    && item.string().eq(&mount)
                {
                    self.mountrow.set_selected(0);
                }
            }
            PartitionRowMsg::SetSwap(status) => {
                self.is_swap = status;
            }
            PartitionRowMsg::SetBoot(status) => {
                self.is_boot = status;
            }
            PartitionRowMsg::Delete => _sender
                .output(PartitionOut::Delete(self.name.clone()))
                .unwrap(),
        }
    }
}
