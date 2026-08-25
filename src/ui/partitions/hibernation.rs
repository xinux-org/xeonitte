use crate::ui::partitions::partition_model::PartitionMsg;
use gettextrs::gettext;
use relm4::{adw::prelude::*, *};

#[derive(Debug)]
pub struct Hibernation {
    pub enabled: bool,
}

#[derive(Debug)]
pub enum HibernationMsg {
    SetHybernation(bool),
}

#[relm4::component(pub)]
impl Component for Hibernation {
    type Input = HibernationMsg;
    type Output = PartitionMsg;
    type Init = ();
    type CommandOutput = ();

    view! {
        adw::PreferencesGroup {
            #[watch]
            set_title: &gettext("Hibernation"),
            adw::SwitchRow {
                #[watch]
                set_title: &gettext("Enable Hibernation"),
                // #[watch]
                // set_subtitle: &gettext("Encrypt your disk with "),
                #[watch]
                set_active: model.enabled,
                connect_active_notify[sender] => move |switch| {
                    sender.input(HibernationMsg::SetHybernation(switch.is_active()));
                }
            },
        },
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Hibernation { enabled: false };
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }
    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>, _root: &Self::Root) {
        match message {
            HibernationMsg::SetHybernation(switch) => {
                self.enabled = switch;
                let _ = sender.output(PartitionMsg::SetHibernation(switch));
            }
        }
    }
}
