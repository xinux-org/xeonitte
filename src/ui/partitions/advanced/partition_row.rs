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
    pub is_swap: bool,
    pub is_boot: bool,
    pub is_full: bool,
    pub partition_disko: PartitionDef,
}

#[derive(Debug)]
pub enum PartitionRowMsg {
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
                &gettext("Edit mountpoint") => EditMountpointPartitionAction,
                &gettext("Resize") => ResizePartitionAction,
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
        }
    }
    fn init_model(parent: Self::Init, _index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        Partition {
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
    EditMountpointPartitionAction,
    PartitionActionGroup,
    "edit-mountpoint-partition"
);
relm4::new_stateless_action!(
    ResizePartitionAction,
    PartitionActionGroup,
    "resize-partition"
);
