use std::collections::HashMap;

use crate::ui::templates::base::BaseSeparator;
use crate::utils::report::ErrorPhase;
use crate::{ui::window::AppMsg, utils::language::get_languages};
use adw::prelude::*;
use gettextrs::gettext;
use log::{info, trace};
use relm4::*;

#[tracker::track]
pub struct WelcomeModel {
    showall: bool,
    default_language: String,
    selected: Option<String>,
    selectiongroup: gtk::CheckButton,
    expanders: Vec<adw::ExpanderRow>,
}

#[derive(Debug)]
pub enum WelcomeMsg {
    ToggleShowall,
    SetSelected(Option<String>),
    CheckSelected,
}

#[relm4::component(pub)]
impl SimpleComponent for WelcomeModel {
    type Init = ();
    type Input = WelcomeMsg;
    type Output = AppMsg;

    view! {
        gtk::ScrolledWindow {
            set_hexpand: true,
            set_vexpand: true,
            adw::Clamp {
                gtk::Box {
                    set_hexpand: true,
                    set_vexpand: true,
                    set_valign: gtk::Align::Center,
                    set_orientation: gtk::Orientation::Vertical,
                    set_spacing: 20,
                    set_margin_all: 20,
                    gtk::Label {
                        #[watch]
                        set_label: &gettext("Welcome!"),
                        add_css_class: "title-1",
                    },
                    gtk::Label {
                        #[watch]
                        set_label: &gettext("Choose a language"),
                        add_css_class: "title-3"
                    },
                    #[name(langstack)]
                    if model.showall {
                        #[local_ref]
                        langbox -> gtk::ListBox {
                            add_css_class: "boxed-list",
                            set_selection_mode: gtk::SelectionMode::None,
                            connect_row_activated[sender] => move |_, row| {
                                row
                                    .child()
                                    .and_then(|w| w.downcast::<gtk::Box>().ok())
                                    .and_then(|b| b.last_child())
                                    .and_then(|w| w.downcast::<gtk::CheckButton>().ok())
                                    .map_or_else(
                                        || {
                                            sender.output(AppMsg::error(
                                                ErrorPhase::Setup,
                                                "Language check button widget not found",
                                            ));
                                        },
                                        |checkbutton| checkbutton.set_active(true)
                                    );
                            },
                        }
                    } else {
                        #[local_ref]
                        shortlangbox -> gtk::ListBox {
                            add_css_class: "boxed-list",
                            set_selection_mode: gtk::SelectionMode::None,
                            connect_row_activated[sender] => move |_, row| {
                                row
                                    .child()
                                    .and_then(|w| w.downcast::<gtk::Box>().ok())
                                    .and_then(|b| b.last_child())
                                    .and_then(|b| b.downcast::<gtk::CheckButton>().ok())
                                    .map_or_else(
                                        || {
                                            sender.output(AppMsg::error(
                                                ErrorPhase::Setup,
                                                "Language check button widget not found",
                                            ));
                                        },
                                        |checkbutton| checkbutton.set_active(true)
                                    );
                            },
                        }
                    },
                    gtk::Button {
                        add_css_class: "pill",
                        set_halign: gtk::Align::Center,
                        #[watch]
                        set_label: &if model.showall { gettext("Show less") } else { gettext("Show all") },
                        connect_clicked[sender] => move |_| {
                            sender.input(WelcomeMsg::ToggleShowall);
                            sender.input(WelcomeMsg::SetSelected(None));
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
        let mut model = WelcomeModel {
            showall: false,
            default_language: "uz_UZ.UTF-8".to_owned(),
            selected: None,
            selectiongroup: gtk::CheckButton::new(),
            expanders: vec![],
            tracker: 0,
        };

        let mut languages = get_languages().into_iter().collect::<Vec<_>>();
        languages.sort_by(|a, b| a.0.cmp(&b.0));

        let langbox = gtk::ListBox::new();
        let shortlangbox = Self::create_shortlang_boxes(&model, &sender, languages.clone());

        for (title, languages) in languages {
            if languages.len() > 1 {
                view! {
                    expander = adw::ExpanderRow {
                        set_title: &title,
                    }
                };
                langbox.append(&expander);

                let mut langvec = languages.into_iter().collect::<Vec<_>>();
                langvec.sort_by(|a, b| a.1.cmp(&b.1));
                for (locale, title) in &langvec {
                    view! {
                        row = adw::PreferencesRow {
                            set_title: locale,
                            set_activatable: true,
                            #[wrap(Some)]
                            set_child = &gtk::Box {
                                set_orientation: gtk::Orientation::Horizontal,
                                set_spacing: 6,
                                set_margin_start: 15,
                                set_margin_end: 7,
                                set_margin_top: 15,
                                set_margin_bottom: 15,
                                gtk::Label {
                                    set_label: title,
                                },
                                #[template]
                                BaseSeparator,
                                gtk::CheckButton {
                                    set_halign: gtk::Align::End,
                                    set_group: Some(&model.selectiongroup),
                                    connect_toggled[sender, locale] => move |x| {
                                        if x.is_active() {
                                            sender.input(WelcomeMsg::SetSelected(Some(locale.to_string())))
                                        }
                                    }
                                }
                            }
                        }
                    };
                    expander
                        .first_child()
                        .and_then(|w| w.last_child())
                        .and_then(|w| w.first_child())
                        .and_then(|w| w.downcast::<gtk::ListBox>().ok())
                        .map_or_else(
                            || {
                                sender.output(AppMsg::error(
                                    ErrorPhase::Setup,
                                    "Language expander row widget not found",
                                ));
                            },
                            |lb| {
                                let sender = sender.clone();
                                lb.connect_row_activated(move |_, x| {
                                    x.child()
                                        .and_then(|w| w.downcast::<gtk::Box>().ok())
                                        .and_then(|b| b.last_child())
                                        .and_then(|w| w.downcast::<gtk::CheckButton>().ok())
                                        .map_or_else(
                                            || {
                                                sender.output(AppMsg::error(
                                                    ErrorPhase::Setup,
                                                    "Language check button widget not found",
                                                ));
                                            },
                                            |checkbutton| checkbutton.set_active(true),
                                        );
                                });
                            },
                        );
                    expander.add_row(&row);
                }
                model.expanders.push(expander);
            } else if let Some((locale, title)) = languages.into_iter().next() {
                view! {
                    row = adw::PreferencesRow {
                        set_title: &locale,
                        set_activatable: true,
                        #[wrap(Some)]
                        set_child = &gtk::Box {
                            set_orientation: gtk::Orientation::Horizontal,
                            set_spacing: 6,
                            set_margin_start: 15,
                            set_margin_end: 7,
                            set_margin_top: 15,
                            set_margin_bottom: 15,
                            gtk::Label {
                                set_label: &title,
                            },
                            #[template]
                            BaseSeparator,
                            gtk::CheckButton {
                                set_halign: gtk::Align::End,
                                set_group: Some(&model.selectiongroup),
                                connect_toggled[sender, locale] => move |x| {
                                    if x.is_active() {
                                        sender.input(WelcomeMsg::SetSelected(Some(locale.to_string())))
                                    }
                                }
                            }
                        }
                    }
                };
                langbox.append(&row);
            }
        }
        let widgets = view_output!();
        widgets.langstack.set_vhomogeneous(false);

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>) {
        self.reset();
        match msg {
            WelcomeMsg::ToggleShowall => {
                if !self.showall {
                    self.expanders
                        .iter()
                        .for_each(|expander| expander.set_expand(false));
                }
                self.set_showall(!self.showall);
            }
            WelcomeMsg::SetSelected(lang) => {
                info!("Selected language: {:?}", lang);
                // TODO: Change output and input signal values with SharedState
                sender.output(AppMsg::SetLanguageConfig(lang.clone()));
                sender.output(AppMsg::SetCanGoForward(lang.is_some()));

                self.selectiongroup.set_active(lang.is_none());
                self.selected = lang;
                gettextrs::setlocale(
                    gettextrs::LocaleCategory::LcAll,
                    self.selected
                        .as_deref()
                        .unwrap_or_default()
                        .split('.')
                        .next()
                        .unwrap_or_default(),
                );
            }
            WelcomeMsg::CheckSelected => {
                trace!("WelcomeMsg::CheckSelected {}", self.selected.is_some());
                let _ = sender.output(AppMsg::SetCanGoForward(self.selected.is_some()));
            }
        }
    }
}

impl WelcomeModel {
    pub fn create_shortlang_boxes(
        model: &WelcomeModel,
        sender: &ComponentSender<WelcomeModel>,
        languages: Vec<(String, HashMap<String, String>)>,
    ) -> gtk::ListBox {
        // List of 6 popular languages
        let default_langs: Vec<String> = vec![
            "uz_UZ.UTF-8".into(),
            "en_US.UTF-8".into(),
            "ru_RU.UTF-8".into(),
        ];

        let short_lang_box = gtk::ListBox::new();

        // get matching utf language and create pair from itʻs headless title
        // Example:
        //  "Deutsch",
        //    {
        //      "de_IT.UTF-8": "Deutsch (Italien)",
        //      "de_LI.UTF-8": "Deutsch (Liechtenstein)",
        //      "de_DE.UTF-8": "Deutsch (Deutschland)",
        //    },
        // Result: ("de_DE.UTF-8", "Deutsch")
        let immutable_languages = languages.clone();
        let shortlangs: Vec<(String, String)> = default_langs
            .iter()
            .flat_map(|default_lang_utf| {
                immutable_languages
                    .iter()
                    .filter_map(|(group_title, langs)| {
                        langs.get(default_lang_utf).map(|_language_varient| {
                            (default_lang_utf.clone(), group_title.clone())
                        })
                    })
            })
            .collect();
        for (utf_locale, locale) in shortlangs {
            view! {
                row = adw::PreferencesRow {
                    set_title: &locale,
                    set_activatable: true,
                    #[wrap(Some)]
                    set_child = &gtk::Box {
                        set_orientation: gtk::Orientation::Horizontal,
                        set_spacing: 6,
                        set_margin_start: 15,
                        set_margin_end: 7,
                        set_margin_top: 15,
                        set_margin_bottom: 15,
                        gtk::Label {
                            set_label: &locale,
                        },
                        #[template]
                        BaseSeparator,
                        #[name(rowbtn)]
                        gtk::CheckButton {
                            set_halign: gtk::Align::End,
                            set_group: Some(&model.selectiongroup),
                            connect_toggled[sender, utf_locale] => move |x| {
                                if x.is_active() {
                                    sender.input(WelcomeMsg::SetSelected(Some(utf_locale.to_owned())))
                                }
                            }
                        }
                    }
                }
            };
            short_lang_box.append(&row);
            // set selected at start for default language
            rowbtn.set_active(utf_locale == model.default_language);
        }

        short_lang_box
    }
}
