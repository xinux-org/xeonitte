use adw::prelude::*;
use glib::Properties;
use gtk::glib;
use gtk::subclass::prelude::*;
use relm4::*;

mod imp {
    use super::*;

    #[derive(Default, Properties)]
    #[properties(wrapper_type = super::Input)]
    pub struct Input;

    #[glib::object_subclass]
    impl ObjectSubclass for Input {
        const NAME: &'static str = "Input";
        type Type = super::Input;
        type ParentType = gtk::Entry;
    }

    #[glib::derived_properties]
    impl ObjectImpl for Input {}

    impl WidgetImpl for Input {}
    impl EntryImpl for Input {}
}
glib::wrapper! {
    pub struct Input(ObjectSubclass<imp::Input>)
        @extends gtk::Entry, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable, gtk::Editable, gtk::CellEditable;
}

impl Default for Input {
    fn default() -> Self {
        Self::new()
    }
}

impl Input {
    pub fn new() -> Self {
        glib::Object::builder()
            .property("css_name", "input")
            .build()
    }
}

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
        gtk::Box{
            add_css_class: "input",
            inline_css: "
                .input * {
                    padding: 0;
                    margin: 0;
                    min-height: 0;
                }
                .input entry {
                    padding: 15px;
                }
            "
        }
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
        let model = imp::Input;

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
                set_placeholder_text: Some(&init.placeholder),
                connect_changed[sender] => move |buffer| {
                    sender.output(InputOut::Change(buffer.text().into()));
                },
            },
            password = gtk::PasswordEntry {
                set_hexpand: true,
                set_show_peek_icon: true,
                set_css_classes: &css_classes,
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
