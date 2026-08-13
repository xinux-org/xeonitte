use crate::ui::partitions::partition_model::{CustomPartition, PartitionSchema};
use crate::ui::window::{AppMsg, UserConfig};
use adw::prelude::*;
use gettextrs::gettext;
use gnome_desktop::{self, XkbInfo, XkbInfoExt};
use log::debug;
use relm4::{factory::*, *};

pub struct Partition {
    name: String,
    mountpoint: Option<String>,
    format: Option<String>,
}

#[relm4::factory(pub)]
impl FactoryComponent for Partition {
    type Init = (String, CustomPartition);
    type Input = ();
    type Output = ();
    type ParentWidget = adw::PreferencesGroup;
    type CommandOutput = ();

    view! {
        adw::ActionRow {
            set_title: &self.name,
            add_suffix = &gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_spacing: 10,

                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 5,
                    set_margin_start: 10,
                    gtk::Label {
                        #[watch]
                        // Translators: Leave the ':' at the end of the string, it will read Mountpoint: {disk mount location}
                        set_label: &gettext("Mountpoint:"),
                    },
                    gtk::Button {
                        set_halign: gtk::Align::Start,
                        set_valign: gtk::Align::Center,
                        gtk::Label {
                            #[watch]
                            set_markup: &format!("<tt>{}</tt>", self.mountpoint.clone().unwrap_or_else(|| gettext("Do not mount"))),
                        },
                        set_can_target: false,
                    }
                },

                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 5,
                    set_margin_start: 10,
                    gtk::Label {
                        #[watch]
                        // Translators: Leave the ':' at the end of the string, it will read Format: {disk format}
                        set_label: &gettext("Format:"),
                    },
                    gtk::Button {
                        set_halign: gtk::Align::Start,
                        set_valign: gtk::Align::Center,
                        gtk::Label {
                            #[watch]
                            set_markup: &format!("<tt>{}</tt>", self.format.clone().unwrap_or_else(|| gettext("Do not format"))),
                        },
                        set_can_target: false,
                    }
                },
            }
        }
    }

    fn init_model(
        (name, partition): Self::Init,
        _index: &DynamicIndex,
        _sender: FactorySender<Self>,
    ) -> Self {
        Partition {
            name,
            mountpoint: partition.mountpoint,
            format: partition.format,
        }
    }
}
