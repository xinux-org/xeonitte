use crate::ui::partitions::partition_model::PartitionMsg;
use gettextrs::gettext;
use relm4::{adw::prelude::*, *};

#[derive(Debug, Default, Clone, PartialEq)]
pub struct LuksPasswordComponent {
    pub encryption_enabled: bool,
    pub advanced: bool,
    pub passphrase: String,
    pub passphrase_confirm: String,
}

#[derive(Debug, Clone)]
pub enum LuksPasswordMsg {
    SetEncryption(bool),
    SetPassphrase(String),
    SetPassphraseConfirm(String),
    SetAdvanced(bool),
}

#[relm4::component(pub)]
impl SimpleComponent for LuksPasswordComponent {
    type Input = LuksPasswordMsg;
    type Output = PartitionMsg;
    type Init = ();

    view! {
        adw::PreferencesGroup {
            #[watch]
            set_title: &gettext("Encryption"),
            adw::SwitchRow {
                #[watch]
                set_visible: !model.advanced,
                #[watch]
                set_title: &gettext("Enable Disk Encryption"),
                #[watch]
                set_subtitle: &gettext("Encrypt your disk with LUKS"),
                #[watch]
                set_active: model.encryption_enabled,
                connect_active_notify[sender] => move |switch| {
                    sender.input(LuksPasswordMsg::SetEncryption(switch.is_active()));
                }
            },
            adw::PasswordEntryRow {
                #[watch]
                set_title: &gettext("Encryption Password"),
                #[watch]
                set_visible: model.encryption_enabled || model.advanced,
                connect_changed[sender] => move |entry| {
                    sender.input(LuksPasswordMsg::SetPassphrase(entry.text().to_string()));
                }
            },
            adw::PasswordEntryRow {
                #[watch]
                set_title: &gettext("Confirm Password"),
                #[watch]
                set_visible: model.encryption_enabled || model.advanced,
                connect_changed[sender] => move |entry| {
                    sender.input(LuksPasswordMsg::SetPassphraseConfirm(entry.text().to_string()));
                }
            },
            gtk::Label {
                #[watch]
                set_visible: (model.encryption_enabled || model.advanced) && !model.passphrase.is_empty() && model.passphrase != model.passphrase_confirm,
                #[watch]
                set_label: &gettext("Passwords do not match"),
                add_css_class: "error",
            },
            gtk::Label {
                #[watch]
                set_visible: (model.encryption_enabled || model.advanced) && model.passphrase.is_empty(),
                #[watch]
                set_label: &gettext("Password is required"),
                add_css_class: "warning",
            },
        },
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = LuksPasswordComponent {
            encryption_enabled: false,
            advanced: false,
            passphrase: String::new(),
            passphrase_confirm: String::new(),
        };
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }
    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        match message {
            LuksPasswordMsg::SetEncryption(switch) => {
                self.encryption_enabled = switch;
                let _ = sender.output(PartitionMsg::SetEncryption);
            }
            LuksPasswordMsg::SetPassphrase(entry) => {
                self.passphrase = entry;
                let _ = sender.output(PartitionMsg::SetPassphrase);
            }
            LuksPasswordMsg::SetPassphraseConfirm(entry) => {
                self.passphrase_confirm = entry;
                let _ = sender.output(PartitionMsg::SetPassphraseConfirm);
            }
            LuksPasswordMsg::SetAdvanced(advanced) => {
                self.advanced = advanced;
                self.encryption_enabled = false;
            }
        }
    }
}
