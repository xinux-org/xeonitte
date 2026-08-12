use crate::{
    ui::partitions::partition_model::{PARTITION_BROKER, PartitionMsg},
    utils::i18n::i18n_f,
};
use relm4::{adw::prelude::*, factory::*, *};

#[derive(Default, Debug, PartialEq, Eq, Clone)]
pub struct WholeDisk {
    pub name: String,
    pub size: u64,
    pub group: gtk::CheckButton,
}

#[relm4::factory(pub)]
impl FactoryComponent for WholeDisk {
    type Init = WholeDisk;
    type Input = ();
    type Output = ();
    type ParentWidget = gtk::ListBox;
    type CommandOutput = ();

    view! {
        adw::ActionRow {
            set_visible: !self.name.contains("zram"),
            set_title: &self.name,
            #[watch]
            // Translators: Do NOT translate the '{}'
            // The string reads "{/dev/sdX} (20 GB minimum needed)" indicating that the given disk is not large enough
            set_subtitle: &if self.size >= 21_474_836_480  { size::Size::from_bytes(self.size).to_string() } else { i18n_f("{} (20 GB minimum needed)", &[&size::Size::from_bytes(self.size).to_string()]) },
            set_activatable: true,
            set_sensitive: self.size >= 21_474_836_480, // 20GB
            #[name(checkbtn)]
            add_suffix = &gtk::CheckButton {
                set_group: Some(&self.group),
                connect_toggled[name = self.name.to_string(), size = self.size] => move |btn| {
                    if btn.is_active() {
                        PARTITION_BROKER.send(PartitionMsg::SetFullDisk(name.to_string(), size));
                    }
                }
            },
            connect_activated[checkbtn, name = self.name.to_string(), size = self.size] => move |_| {
                checkbtn.set_active(true);
                PARTITION_BROKER.send(PartitionMsg::SetFullDisk(name.to_string(), size));
            }
        }
    }

    fn init_model(parent: Self::Init, _index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        parent
    }
}
