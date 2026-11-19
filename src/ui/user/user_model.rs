use crate::ui::user::input::{Input, InputInit, InputOut};
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

    #[garde(if(cond = !self.same_password, length(min = 1)))]
    pub root_password: Option<String>,
    #[garde(if(cond = !self.same_password, matches(root_password), length(min = 1)))]
    pub confirm_root_password: Option<String>,

    #[garde(skip)]
    pub autologin: bool,
    #[garde(skip)]
    pub same_password: bool,
}

#[tracker::track]
pub struct UserModel {
    #[tracker::no_eq]
    fullname_row: Controller<Input>,
    #[tracker::no_eq]
    accountname_row: Controller<Input>,
    #[tracker::no_eq]
    password_row: Controller<Input>,
    #[tracker::no_eq]
    password_verify_row: Controller<Input>,

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

pub fn field_css(dirty: bool, validation: Option<Report>, field: &str) -> Vec<String> {
    if !dirty {
        return vec![];
    }

    let Some(report) = validation.as_ref() else {
        return vec![];
    };

    let field = Path::new(field);
    if report.iter().any(|(path, _)| path.eq(&field)) {
        vec!["error".into()]
    } else {
        vec![]
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
    CheckSelected,
    SetConfig(bool, bool),
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
                        gtk::Box {
                            set_orientation: gtk::Orientation::Vertical,
                            set_spacing: 7,

                            // Inputs
                            model.fullname_row.widget(),
                            model.accountname_row.widget(),
                            gtk::Box {
                                set_spacing: 6,
                                model.password_row.widget(),
                                model.password_verify_row.widget(),
                            },

    //                         adw::EntryRow {
    //                             #[watch]
    //                             set_title: &gettext("Name"),
    //                             // #[watch]
    //                             // set_css_classes: model.field_css("name"),
    //                             connect_changed[sender] => move |entry| {
    //                                 sender.input(UserMsg::Update(UserDataPatch { name: Some(entry.text().to_string()), ..Default::default()}));
    //                             }
    //                         },
    //                         #[local_ref]
    //                         username_row -> adw::EntryRow {
    //                             #[watch]
    //                             set_title: &gettext("Username"),
    //                             // #[watch]
    //                             // set_css_classes: model.field_css("username"),
    //                             connect_text_notify => move |entry| {
    //                                 let mut corrected = String::new();
    //                                 for c in entry.text().chars() {
    //                                     if c.is_ascii_lowercase() || c.is_ascii_digit() {
    //                                         corrected.push(c);
    //                                     }
    //                                 }
    //
    //                                 while let Some(first) = corrected.chars().next() {
    //                                     if first.is_ascii_digit() {
    //                                         if let Some(c) = corrected.get(1..) {
    //                                             corrected = c.to_string();
    //                                         } else {
    //                                             corrected = String::new();
    //                                             break;
    //                                         }
    //                                     } else {
    //                                         break;
    //                                     }
    //                                 }
    //
    //                                 if entry.text() != corrected {
    //                                     entry.set_text(&corrected);
    //                                 }
    //                             },
    //                             connect_changed[sender] => move |entry| {
    //                                 sender.input(UserMsg::Update(UserDataPatch { username: Some(entry.text().to_string()), ..Default::default()}));
    //                             },
    //                         },
    //                         adw::PasswordEntryRow {
    //                             #[watch]
    //                             set_title: &gettext("Password"),
    //                             // #[watch]
    //                             // set_css_classes: model.field_css("password"),
    //                             connect_changed[sender] => move |entry| {
    //                                 sender.input(UserMsg::Update(UserDataPatch {password: Some(entry.text().to_string()), ..Default::default()}));
    //                             }
    //                         },
    //                         #[local_ref]
    //                         confirm_password_row -> adw::PasswordEntryRow {
    //                             #[watch]
    //                             set_title: &gettext("Confirm Password"),
    //                             // #[watch]
    //                             // set_css_classes: model.field_css("confirm_password"),
    //                             connect_changed[sender] => move |entry| {
    //                                 sender.input(UserMsg::Update(UserDataPatch {confirm_password: Some(entry.text().to_string()), ..Default::default()}));
    //                             }
    //                         },
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
    //                     gtk::ListBox {
    //                         #[watch]
    //                         set_visible: model.showhostname,
    //                         add_css_class: "boxed-list",
    //                         set_selection_mode: gtk::SelectionMode::None,
    //                         #[local_ref]
    //                         hostnamerow -> adw::EntryRow {
    //                             #[watch]
    //                             set_title: &gettext("Hostname"),
    //                             // #[watch]
    //                             // set_css_classes: model.field_css("hostname"),
    //                             set_text: "xinux",
    //                             connect_changed[sender] => move |entry| {
    //                                 sender.input(UserMsg::Update(UserDataPatch{ hostname: Some(entry.text().to_string()), ..Default::default()}));
    //                             },
    //                             connect_text_notify => move |entry| {
    //                                 let mut corrected = String::new();
    //                                 for c in entry.text().chars() {
    //                                     if c.is_ascii_alphanumeric() || c.is_ascii_digit() {
    //                                         corrected.push(c);
    //                                     }
    //                                 }
    //
    //                                 if entry.text() != corrected {
    //                                     entry.set_text(&corrected);
    //                                 }
    //                             }
    //                         }
    //                     },
    //                     gtk::ListBox {
    //                         #[watch]
    //                         set_visible: model.showrootpassword,
    //                         add_css_class: "boxed-list",
    //                         set_selection_mode: gtk::SelectionMode::None,
    //                         adw::PasswordEntryRow {
    //                             #[watch]
    //                             set_title: &gettext("Root password"),
    //                             // #[watch]
    //                             // set_css_classes: model.field_css("root_password"),
    //                             connect_changed[sender] => move |entry| {
    //                                 sender.input(UserMsg::Update(UserDataPatch { root_password: Some(Some(entry.text().to_string())), ..Default::default() }));
    //                             }
    //                         },
    //                         #[local_ref]
    //                         confirm_root_password_row -> adw::PasswordEntryRow {
    //                             #[watch]
    //                             set_title: &gettext("Confirm root password"),
    //                             // #[watch]
    //                             // set_css_classes: model.field_css("confirm_root_password"),
    //                             connect_changed[sender] => move |entry| {
    //                                 sender.input(UserMsg::Update(UserDataPatch { confirm_root_password: Some(Some(entry.text().to_string())), ..Default::default()}));
    //                             }
    //                         }
    //                     }
                    }
                }
            }
        }

    fn init(
        _parent_window: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let fullname_row = Input::builder()
            .launch(InputInit {
                placeholder: "Full name".into(),
                css_classes: field_css(false, None, "name"),
                is_password: false,
            })
            .forward(sender.input_sender(), |msg| match msg {
                InputOut::Change(text) => UserMsg::Update(UserDataPatch {
                    name: Some(text),
                    ..Default::default()
                }),
            });
        let accountname_row = Input::builder()
            .launch(InputInit {
                placeholder: "Account name".into(),
                css_classes: field_css(false, None, "username"),
                is_password: false,
            })
            .forward(sender.input_sender(), |msg| match msg {
                InputOut::Change(text) => UserMsg::Update(UserDataPatch {
                    username: Some(text),
                    ..Default::default()
                }),
            });
        let password_row = Input::builder()
            .launch(InputInit {
                placeholder: "Password".into(),
                css_classes: field_css(false, None, "password"),
                is_password: true,
            })
            .forward(sender.input_sender(), |msg| match msg {
                InputOut::Change(text) => UserMsg::Update(UserDataPatch {
                    password: Some(text),
                    ..Default::default()
                }),
            });
        let password_verify_row = Input::builder()
            .launch(InputInit {
                placeholder: "Verify".into(),
                css_classes: field_css(false, None, "confirm_password"),
                is_password: true,
            })
            .forward(sender.input_sender(), |msg| match msg {
                InputOut::Change(text) => UserMsg::Update(UserDataPatch {
                    confirm_password: Some(text),
                    ..Default::default()
                }),
            });
        let model = UserModel {
            fullname_row,
            accountname_row,
            password_row,
            password_verify_row,
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
            tracker: 0,
            dirty: true,
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
            UserMsg::CheckSelected => {
                let valid = self.data.validate().is_ok();
                let _ = sender.output(AppMsg::SetCanGoForward(valid));
            }
            UserMsg::SetConfig(root, showhostname) => {
                self.showrootpassword = root;
                self.showhostname = showhostname;
            }
            UserMsg::Update(patch) => {
                self.data.apply(patch.clone());

                if let Some(name) = patch.name {
                    self.data.username = to_ascii_alphanumeric(&name);
                    self.username_row.set_text(&self.data.username);
                }

                // Validate the inputs, needed to define if the page can be moved to next one or not
                self.validation = self.data.validate().map_or_else(Some, |_| None);
                self.dirty = self.validation.is_some();

                if self.validation.is_none() {
                    let UserData {
                        name,
                        username,
                        password,
                        hostname,
                        root_password,
                        autologin,
                        ..
                    } = self.data.clone();

                    let rootpassword = if self.data.same_password {
                        Some(password.clone())
                    } else {
                        root_password
                    };

                    let _ = sender.output(AppMsg::SetUserConfig(Some(UserConfig {
                        name,
                        username,
                        password,
                        hostname,
                        rootpassword,
                        autologin,
                    })));
                    sender.output(AppMsg::SetCanGoForward(true));
                } else {
                    sender.output(AppMsg::SetUserConfig(None));
                    let _ = sender.output(AppMsg::SetCanGoForward(false));
                }
            }
        }
    }
}
