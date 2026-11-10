use crate::ui::welcome::selector::{Selector, SelectorRowOutput};
use crate::utils::language::{Locale, LocaleUtf};
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
    #[tracker::no_eq]
    languages_component: Controller<Selector>,
}

#[derive(Debug)]
pub enum WelcomeMsg {
    SetSelected(Option<String>),
    CheckSelected,
}

#[relm4::component(pub)]
impl SimpleComponent for WelcomeModel {
    type Init = ();
    type Input = WelcomeMsg;
    type Output = AppMsg;

    view! {
        #[root]
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
                        set_label: &gettext("Select a language"),
                        add_css_class: "title-1"
                    },

                    model.languages_component.widget(),
                }
            }
        }
    }

    fn init(
        _parent_window: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let mut languages: Vec<Locale> = get_languages().into_iter().collect();
        languages.sort_by(|a, b| a.name.cmp(&b.name));

        let formatted_languages = WelcomeModel::format_languages(languages.clone());

        let languages_component = Selector::builder()
            .launch(formatted_languages.clone())
            .forward(sender.input_sender(), move |msg| match msg {
                SelectorRowOutput::Select(index) => {
                    let selected = formatted_languages
                        .iter()
                        .nth(index.current_index())
                        .and_then(|selected_title| {
                            languages.iter().find_map(|locale| {
                                locale
                                    .variants
                                    .iter()
                                    .find(|utf| utf.title == *selected_title)
                            })
                        });
                    WelcomeMsg::SetSelected(selected.map(|utf| utf.code.clone()))
                }
            });

        let model = WelcomeModel {
            showall: false,
            default_language: "uz_UZ.UTF-8".into(),
            selected: None,
            selectiongroup: gtk::CheckButton::new(),
            expanders: vec![],
            languages_component,
            tracker: 0,
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>) {
        self.reset();
        match msg {
            WelcomeMsg::SetSelected(lang) => {
                dbg!("Selected language: {:?}", &lang);
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
    pub fn format_languages(languages: Vec<Locale>) -> Vec<String> {
        // List of 6 popular languages
        let default_langs_codes = ["uz_UZ.UTF-8", "en_US.UTF-8", "ru_RU.UTF-8"].map(String::from);

        // get matching utf language and create pair from itʻs headless title
        // Example:
        //  "Deutsch",
        //    {
        //      "de_IT.UTF-8": "Deutsch (Italien)",
        //      "de_LI.UTF-8": "Deutsch (Liechtenstein)",
        //      "de_DE.UTF-8": "Deutsch (Deutschland)",
        //    },
        // Result: ("de_DE.UTF-8", "Deutsch")
        let mut mutable_languages = languages.clone();

        // we extract the default languages locales to `mutable_languages`,
        // and vector without them to the `without_default_langs` variables.
        let mut without_default_langs: Vec<Locale> = mutable_languages
            .extract_if(.., |locale| {
                // check if the locale contains the default utf codes defined above
                locale
                    .variants
                    .iter()
                    .filter(|LocaleUtf { code, .. }| default_langs_codes.contains(&code))
                    .collect::<Vec<_>>()
                    .is_empty()
            })
            .collect();

        mutable_languages.append(&mut without_default_langs);

        mutable_languages
            .iter()
            .map(|locale| locale.variants.clone())
            .flatten()
            .map(|utf| utf.title)
            .collect::<Vec<_>>()
    }
}
