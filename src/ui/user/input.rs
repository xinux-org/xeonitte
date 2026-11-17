use adw::prelude::*;
use relm4::*;

struct Input;

#[derive(Debug)]
pub enum InputOut {
    Change(String),
}

#[derive(Debug)]
pub struct InputInit {
    placeholder: Option<String>,
    is_password: bool,
}

#[relm4::component]
impl SimpleComponent for Input {
    type Init = InputInit;
    type Input = ();
    type Output = InputOut;

    view! {
        #[root]
        gtk::Box{}
    }

    // additional_fields! {
    //     text: gtk::Entry,
    //     password: gtk::PasswordEntry,
    // }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Input;

        view! {
            text = gtk::Entry {
                set_hexpand: true,
                set_placeholder_text: init.placeholder.as_deref(),
                connect_changed[sender] => move |buffer| {
                    sender.output(InputOut::Change(buffer.text().into()));
                },
            },
            password = gtk::PasswordEntry {
                set_hexpand: true,
                set_placeholder_text: init.placeholder.as_deref(),
                connect_changed[sender] => move |buffer| {
                    sender.output(InputOut::Change(buffer.text().into()));
                },
            }
        }

        // based on init, choose which component to add
        if init.is_password {
            root.append(&password);
        } else {
            root.append(&text);
        }

        let widgets = view_output!();
        ComponentParts { model, widgets }
    }
}
