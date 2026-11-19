use adw::prelude::*;
use relm4::*;

pub struct Input;

#[derive(Debug)]
pub enum InputOut {
    Change(String),
}

#[derive(Debug)]
pub struct InputInit {
    pub placeholder: String,
    pub is_password: bool,
    pub css_classes: Vec<String>,
}

#[relm4::component(pub)]
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

        let css_classes = init
            .css_classes
            .iter()
            .map(std::ops::Deref::deref)
            .collect::<Vec<&str>>()
            .as_array()
            .unwrap_or(&[])
            .to_owned();

        view! {
            text = gtk::Entry {
                set_hexpand: true,
                set_css_classes: &css_classes,
                add_css_class: "input",
                set_placeholder_text: Some(&init.placeholder),
                connect_changed[sender] => move |buffer| {
                    sender.output(InputOut::Change(buffer.text().into()));
                },
            },
            password = gtk::PasswordEntry {
                set_hexpand: true,
                set_show_peek_icon: true,
                set_css_classes: &css_classes,
                add_css_class: "input",
                set_placeholder_text: Some(&init.placeholder),
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
