use crate::modules::disk::PartitionDef;
use gettextrs::gettext;
use relm4::{
    actions::{RelmAction, RelmActionGroup},
    adw::prelude::*,
    factory::*,
    *,
};

#[derive(Debug, Clone)]
pub struct Partition {
    pub mountrow: adw::ComboRow,
    pub is_swap: bool,
    pub is_boot: bool,
    pub is_full: bool,
    pub donotformat: String,
    pub partition_disko: PartitionDef,
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
    type Init = Partition;
    type Input = PartitionRowMsg;
    type Output = PartitionOut;
    type ParentWidget = gtk::ListBox;
    type CommandOutput = ();

    menu! {
        block_menu: {
            section! {
                &gettext("Delete...") => DeletePartitionAction,
                &gettext("Add Btrfs subvolume") => AddBtrfsSubvolumeAction,
            }
        }
    }

    view! {
        #[root]
        &adw::ExpanderRow {
            set_title: &self.partition_disko.size.to_disko_str_ui(),
            set_subtitle: &self.partition_disko.label.clone().unwrap(),
            #[name = "block_menu_id"]
            add_suffix = &gtk::MenuButton {
                set_icon_name: "view-more-symbolic",
                set_tooltip_text: Some(&gettext("Partition Options")),
                set_menu_model: Some(&block_menu),
                add_css_class: "flat",
            },
            add_row = &adw::ActionRow {
                set_title: "Mountpoint",
                #[watch]
                set_subtitle: &self.partition_disko.label.clone().unwrap(),
                set_subtitle_selectable: true,
            },
            // add_row = &adw::ComboRow {
            //     #[watch]
            //     set_title: &gettext("Format"),
            //     // TODO: When switching language the "Leave as is" option does not update
            //     set_model: Some(&gtk::StringList::new(&[&self.donotformat, "btrfs", "ext4", "ext3", "vfat", "ntfs", "xfs", "swap"])),
            //     connect_selected_notify[sender, name = self.name.to_string(), device = self.device.to_string(), formatstring = self.donotformat.to_string(), size = self.size, is_full = self.is_full] => move |row| {
            //         if let Some(item) = row.selected_item() && let Ok(item) = item.downcast::<gtk::StringObject>() {
            //             if item.string() == formatstring {
            //                 PARTITION_BROKER.send(PartitionMsg::RemoveFormatPartition(name.to_string()));
            //             } else {
            //                 PARTITION_BROKER.send(PartitionMsg::AddFormatPartition(name.to_string(), item.string().to_string(), device.to_string(), size, is_full));
            //             }
            //             sender.input(PartitionRowMsg::SetSwap(item.string().eq("swap")));
            //         }
            //     }
            // },
            // #[local_ref]
            // add_row = mountrow -> adw::ComboRow {
            //     #[watch]
            //     set_visible: !self.is_swap,
            //     #[watch]
            //     set_title: &gettext("Mount"),
            //     // TODO: When switching language the "Do not mount" option does not update
            //     set_model: Some(&gtk::StringList::new(&[&self.donotmount, " /", "/boot", "/home", "/opt", "/var", "/nix"])),
            //     connect_selected_notify[sender, name = self.name.to_string(), device = self.device.to_string(), mountstring = self.donotmount.to_string(), size = self.size, is_full = self.is_full] => move |row| {
            //         if let Some(item) = row.selected_item() && let Ok(item) = item.downcast::<gtk::StringObject>(){
            //             let x = item.string();
            //             if x == mountstring {
            //                 PARTITION_BROKER.send(PartitionMsg::RemoveMountPartition(name.to_string()));
            //             } else {
            //                 PARTITION_BROKER.send(PartitionMsg::AddMountPartition(name.to_string(), item.string().trim().to_string(), device.to_string(), size, is_full));
            //             }
            //             sender.input(PartitionRowMsg::SetBoot(x.eq("/boot")));
            //         }
            //     }
            // },
            // add_row = &adw::ActionRow {
            //     #[watch]
            //     set_visible: self.is_swap,
            //     #[watch]
            //     set_title: &gettext("Mount"),
            //     add_suffix = &gtk::Label {
            //         set_text: "swap",
            //     }
            // },
            // add_row = &adw::SwitchRow {
            //     #[watch]
            //     set_visible: !self.is_swap && !self.is_boot,
            //     #[watch]
            //     set_title: &gettext("Encrypt"),
            //     #[watch]
            //     set_subtitle: &gettext("Encrypt this partition with LUKS"),
            //     connect_active_notify[name = self.name.to_string(), device = self.device.to_string(), size = self.size, is_full = self.is_full] => move |row| {
            //         PARTITION_BROKER.send(PartitionMsg::SetPartitionEncryption(
            //             name.to_string(),
            //             device.to_string(),
            //             size,
            //             row.is_active(),
            //             is_full
            //         ));
            //     }
            // },
        }
    }
    fn init_model(parent: Self::Init, _index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        Partition {
            mountrow: parent.mountrow,
            donotformat: parent.donotformat,
            is_swap: false,
            is_boot: false,
            is_full: false,
            partition_disko: parent.partition_disko,
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

        let mut actions = RelmActionGroup::<PartitionActionGroup>::new();

        let delete_sender = sender.clone();
        let delete_partition_action =
            RelmAction::<DeletePartitionAction>::new_stateless(move |_| {
                delete_sender.input(PartitionRowMsg::Delete);
            });

        // let add_btrfs_subvolume_action =
        //     RelmAction::<DeletePartitionAction>::new_stateless(move |_| {
        //         sender.input(PartitionRowMsg::Delete);
        //     });

        actions.add_action(delete_partition_action);
        // actions.add_action(add_btrfs_subvolume_action);
        actions.register_for_widget(&widgets.block_menu_id);

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
                .output(PartitionOut::Delete(
                    self.partition_disko.label.clone().unwrap(),
                ))
                .unwrap(),
        }
    }
}

relm4::new_action_group!(pub PartitionActionGroup, "partition");
relm4::new_stateless_action!(
    DeletePartitionAction,
    PartitionActionGroup,
    "delete-partition"
);
relm4::new_stateless_action!(
    AddBtrfsSubvolumeAction,
    PartitionActionGroup,
    "add-btrfs-subvolume"
);
