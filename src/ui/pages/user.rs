use crate::ui::window::{AppMsg, UserConfig};
use garde::{Path, Report, Validate};
use gettextrs::gettext;
use relm4::adw::{self, prelude::*};
use relm4::*;
use struct_patch::Patch;

#[tracker::track]
#[derive(PartialEq, Eq, PartialOrd, Ord, Validate, Default, Debug, Clone, Patch)]
#[patch(attribute(derive(Debug, Default, Clone)))]
#[garde(allow_unvalidated)]
pub struct UserData {
    #[garde(length(min = 1))]
    pub name: String,
    #[garde(length(min = 1))]
    pub username: String,

    #[garde(length(min = 1))]
    pub password: String,
    #[garde(matches(password))]
    pub confirm_password: String,

    #[garde(length(min = 1))]
    pub hostname: String,

    #[garde(length(min = 1))]
    pub root_password: Option<String>,
    #[garde(matches(root_password), length(min = 1))]
    pub confirm_root_password: Option<String>,

    #[garde(skip)]
    pub autologin: bool,
}

#[tracker::track]
pub struct UserModel {
    username_row: adw::EntryRow,
    confirm_password_row: adw::PasswordEntryRow,
    confirm_root_password_row: adw::PasswordEntryRow,
    hostnamerow: adw::EntryRow,
    showhostname: bool,
    showrootpassword: bool,
    data: UserData,
    #[no_eq]
    validation: Option<Report>,
    dirty: bool,
}

impl UserModel {
    pub fn field_css(&self, field: &str) -> &[&str] {
        if !self.dirty {
            return &[];
        }

        let Some(report) = self.validation.as_ref() else {
            return &[];
        };

        if report.iter().any(|(path, _)| path.eq(&Path::new(field))) {
            return &["error"];
        } else {
            return &[];
        }
    }
}

pub fn to_ascii_alphanumeric(text: &str) -> String {
    text.to_ascii_lowercase()
        .replace(' ', "")
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .collect::<String>()
}

#[derive(Debug)]
pub enum UserMsg {
    SetConfig(bool, bool, String),
    Update(UserDataPatch),
}

#[relm4::component(pub)]
impl SimpleComponent for UserModel {
    type Init = ();
    type Input = UserMsg;
    type Output = AppMsg;

    view! {
        gtk::ScrolledWindow {
            adw::Clamp {
                gtk::Box {
                    set_hexpand: true,
                    set_vexpand: true,
                    set_valign: gtk::Align::Center,
                    set_orientation: gtk::Orientation::Vertical,
                    set_spacing: 20,
                    set_margin_all: 20,
                    adw::Avatar {
                        set_size: 144,
                    },
                    gtk::Label {
                        #[watch]
                        set_label: &gettext("Create a new user"),
                        add_css_class: "title-1"
                    },
                    gtk::ListBox {
                        add_css_class: "boxed-list",
                        set_selection_mode: gtk::SelectionMode::None,
                        adw::EntryRow {
                            #[watch]
                            set_title: &gettext("Name"),
                            #[watch]
                            set_css_classes: model.field_css("name"),
                            connect_changed[sender] => move |entry| {
                                sender.input(UserMsg::Update(UserDataPatch { name: Some(entry.text().to_string()), ..Default::default()}));
                            }
                        },
                        #[local_ref]
                        username_row -> adw::EntryRow {
                            #[watch]
                            set_title: &gettext("Username"),
                            #[watch]
                            set_css_classes: model.field_css("username"),
                            connect_text_notify => move |entry| {
                                let mut corrected = String::new();
                                for c in entry.text().chars() {
                                    if c.is_ascii_lowercase() || c.is_ascii_digit() {
                                        corrected.push(c);
                                    }
                                }

                                while let Some(first) = corrected.chars().next() {
                                    if first.is_ascii_digit() {
                                        if let Some(c) = corrected.get(1..) {
                                            corrected = c.to_string();
                                        } else {
                                            corrected = String::new();
                                            break;
                                        }
                                    } else {
                                        break;
                                    }
                                }

                                if entry.text() != corrected {
                                    entry.set_text(&corrected);
                                }
                            },
                            connect_changed[sender] => move |entry| {
                                sender.input(UserMsg::Update(UserDataPatch { username: Some(entry.text().to_string()), ..Default::default()}));
                            },
                        },
                        adw::PasswordEntryRow {
                            #[watch]
                            set_title: &gettext("Password"),
                            #[watch]
                            set_css_classes: model.field_css("password"),
                            connect_changed[sender] => move |entry| {
                                sender.input(UserMsg::Update(UserDataPatch {password: Some(entry.text().to_string()), ..Default::default()}));
                            }
                        },
                        #[local_ref]
                        confirm_password_row -> adw::PasswordEntryRow {
                            #[watch]
                            set_title: &gettext("Confirm Password"),
                            #[watch]
                            set_css_classes: model.field_css("confirm_password"),
                            connect_changed[sender] => move |entry| {
                                sender.input(UserMsg::Update(UserDataPatch {confirm_password: Some(entry.text().to_string()), ..Default::default()}));
                            }
                        },
                        adw::ActionRow {
                            #[watch]
                            set_title: &gettext("Log in automatically"),
                            set_activatable: true,
                            connect_activated[autoswitch] => move |_| {
                                autoswitch.activate();
                            },
                            #[name(autoswitch)]
                            add_suffix = &gtk::Switch {
                                set_valign: gtk::Align::Center,
                                connect_state_set[sender] => move |_, state| {
                                    sender.input(UserMsg::Update(UserDataPatch { autologin: Some(state), ..Default::default()}));
                                    relm4::gtk::glib::Propagation::Proceed
                                }
                            }
                        }
                    },
                    gtk::ListBox {
                        #[watch]
                        set_visible: model.showhostname,
                        add_css_class: "boxed-list",
                        set_selection_mode: gtk::SelectionMode::None,
                        #[local_ref]
                        hostnamerow -> adw::EntryRow {
                            #[watch]
                            set_title: &gettext("Hostname"),
                            #[watch]
                            set_css_classes: model.field_css("hostname"),
                            connect_changed[sender] => move |entry| {
                                sender.input(UserMsg::Update(UserDataPatch{ hostname: Some(entry.text().to_string()), ..Default::default()}));
                            },
                            connect_text_notify => move |entry| {
                                let mut corrected = String::new();
                                for c in entry.text().chars() {
                                    if c.is_ascii_alphanumeric() || c.is_ascii_digit() {
                                        corrected.push(c);
                                    }
                                }

                                if entry.text() != corrected {
                                    entry.set_text(&corrected);
                                }
                            }
                        }
                    },
                    gtk::ListBox {
                        #[watch]
                        set_visible: model.showrootpassword,
                        add_css_class: "boxed-list",
                        set_selection_mode: gtk::SelectionMode::None,
                        adw::PasswordEntryRow {
                            #[watch]
                            set_title: &gettext("Root password"),
                            #[watch]
                            set_css_classes: model.field_css("root_password"),
                            connect_changed[sender] => move |entry| {
                                sender.input(UserMsg::Update(UserDataPatch { root_password: Some(Some(entry.text().to_string())), ..Default::default() }));
                            }
                        },
                        #[local_ref]
                        confirm_root_password_row -> adw::PasswordEntryRow {
                            #[watch]
                            set_title: &gettext("Confirm root password"),
                            #[watch]
                            set_css_classes: model.field_css("confirm_root_password"),
                            connect_changed[sender] => move |entry| {
                                sender.input(UserMsg::Update(UserDataPatch { confirm_root_password: Some(Some(entry.text().to_string())), ..Default::default()}));
                            }
                        }
                    }
                }
            }
        }
    }

    fn init(
        _parent_window: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = UserModel {
            username_row: adw::EntryRow::new(),
            hostnamerow: adw::EntryRow::new(),
            confirm_password_row: adw::PasswordEntryRow::new(),
            confirm_root_password_row: adw::PasswordEntryRow::new(),
            showhostname: false,
            showrootpassword: false,
            data: UserData {
                hostname: "xinux".to_string(),
                ..Default::default()
            },
            validation: None,
            dirty: false,
            tracker: 0,
        };
        let username_row = &model.username_row;
        let confirm_password_row = &model.confirm_password_row;
        let confirm_root_password_row = &model.confirm_root_password_row;
        let hostnamerow = &model.hostnamerow;
        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>) {
        self.reset();
        self.data.reset();
        match msg {
            UserMsg::SetConfig(root, showhostname, hostname) => {
                self.showrootpassword = root;
                self.showhostname = showhostname;
                self.data.hostname = hostname.to_string();
            }
            UserMsg::Update(patch) => {
                self.data.apply(patch.clone());

                if let Some(name) = patch.name {
                    self.data.username = to_ascii_alphanumeric(&name);
                    self.username_row.set_text(&self.data.username);
                }

                self.validation = self
                    .data
                    .validate()
                    .map_or_else(|report| Some(report), |_| None);

                self.dirty = true;

                if self.validation.is_none() {
                    let UserData {
                        name,
                        username,
                        password,
                        hostname,
                        root_password: rootpassword,
                        autologin,
                        ..
                    } = self.data.clone();

                    let _ = sender.output(AppMsg::SetUserConfig(Some(UserConfig {
                        name,
                        username,
                        password,
                        hostname,
                        rootpassword,
                        autologin,
                    })));
                    let _ = sender.output(AppMsg::SetCanGoForward(true));
                } else {
                    let _ = sender.output(AppMsg::SetUserConfig(None));
                    let _ = sender.output(AppMsg::SetCanGoForward(false));
                }
            }
        }
    }
}
