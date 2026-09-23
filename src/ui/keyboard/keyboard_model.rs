use crate::ui::{templates::base::BaseSeparator, window::AppMsg};
use crate::utils::report::ErrorPhase;
use adw::prelude::*;
use gettextrs::gettext;
use gnome_desktop::{self, XkbInfo, XkbInfoExt};
use log::trace;
use relm4::*;
use std::process::Command;

const GIO_INPUT_SOURCES: &str = "org.gnome.desktop.input-sources";

// type Layout = (String, (String, String, String, String));
#[derive(Debug, PartialEq, Clone)]
struct Layout {
    title: String,
    name: String,
    language: String,
    country: String,
    variant: String,
}

#[tracker::track]
#[derive(Debug)]
pub struct KeyboardModel {
    #[allow(clippy::type_complexity)]
    layouts: Vec<Layout>,
    language: Option<String>,
    country: Option<String>,
    showall: bool,
    selectiongroup: gtk::CheckButton,
    selected: Option<String>,
    expanders: Vec<adw::ExpanderRow>,
    shortkbdbox: gtk::ListBox,
    xkb: XkbInfo,
    keyboard_settings: gtk::gio::Settings,
}

#[derive(Debug)]
pub enum KeyboardMsg {
    ToggleShowall,
    SetSelected(Option<String>),
    SetCountry(String, String),
    CheckSelected,
}

#[relm4::component(pub)]
impl SimpleComponent for KeyboardModel {
    type Input = KeyboardMsg;
    type Output = AppMsg;
    type Init = ();

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
                        set_label: &gettext("Keyboard Layout"),
                        add_css_class: "title-1"
                    },
                    gtk::ListBox {
                        add_css_class: "boxed-list",
                        set_selection_mode: gtk::SelectionMode::None,
                        adw::EntryRow {
                            #[watch]
                            set_title: &gettext("Test the Keyboard Layout"),
                        }
                    },
                    #[name(kbdstack)]
                    if model.showall {
                        #[local_ref]
                        kbdbox -> gtk::ListBox {
                            add_css_class: "boxed-list",
                            set_selection_mode: gtk::SelectionMode::None,
                        }
                    } else {
                        #[local_ref]
                        shortkbdbox -> gtk::ListBox {
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
                                                "Keyboard check button widget not found",
                                            ));
                                        }, |checkbutton| {
                                            checkbutton.set_active(true);
                                        }
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
                            sender.input(KeyboardMsg::ToggleShowall);
                            sender.input(KeyboardMsg::SetSelected(None));
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
        println!("Keyboard init");
        let xkb = XkbInfo::new();
        let layouts = xkb.all_layouts();

        let mut layoutvec: Vec<Layout> = vec![];

        for layout in layouts {
            let layoutinfo = xkb.layout_info(&layout);

            if let Some((Some(name), Some(lang), Some(country), Some(variant))) = layoutinfo {
                layoutvec.push(Layout {
                    title: layout.into(),
                    name: name.into(),
                    language: lang.into(),
                    country: country.into(),
                    variant: variant.into(),
                });
            }
        }
        layoutvec.sort_by(|a, b| a.title.cmp(&b.title));

        let keyboard_settings = gtk::gio::Settings::new(GIO_INPUT_SOURCES);
        let mut model = KeyboardModel {
            xkb,
            language: Some("en".to_string()),
            country: Some("us".to_string()),
            layouts: layoutvec,
            showall: false,
            selected: None,
            selectiongroup: gtk::CheckButton::new(),
            expanders: vec![],
            shortkbdbox: gtk::ListBox::new(),
            keyboard_settings,
            tracker: 0,
        };

        let kbdbox = gtk::ListBox::new();
        let shortkbdbox = gtk::ListBox::new();

        let mut countries = model
            .layouts
            .clone()
            .iter()
            .map(|layout| layout.country.clone())
            .filter(|x| x != &"custom")
            .collect::<Vec<_>>();
        countries.dedup();
        println!("Pre sort");
        countries.sort_by(|a, b| {
            let aname = gnome_desktop::country_from_code(&a.to_uppercase(), None)
                .map(|x| x.to_string())
                .or_else(|| {
                    model
                        .layouts
                        .iter()
                        .find(|layout| &layout.country == a)
                        .and_then(|layout| {
                            layout.title.split('(').nth(0).map(|s| s.trim().to_string())
                        })
                });
            let bname = gnome_desktop::country_from_code(&b.to_uppercase(), None)
                .map(|x| x.to_string())
                .or_else(|| {
                    model
                        .layouts
                        .iter()
                        .find(|layout| &layout.country == b)
                        .and_then(|layout| {
                            layout.title.split('(').nth(0).map(|s| s.trim().to_string())
                        })
                });
            aname.cmp(&bname)
        });
        println!("Post sort");

        for country in countries {
            let possible_country = model
                .layouts
                .iter()
                .find(|layout| layout.country == country)
                .and_then(|layout| layout.title.split('(').nth(0).map(|s| s.trim().to_string()))
                .clone();
            view! {
                expander = adw::ExpanderRow {
                    set_title: &gnome_desktop::country_from_code(&country.to_uppercase(), None)
                        .map(|x| x.to_string())
                        .or(possible_country)
                        .unwrap_or_else(|| {
                            trace!("Country name can't be found for {}", country);
                            String::from("Unknown")}),
                }
            }

            for layout in model
                .layouts
                .clone()
                .into_iter()
                .filter(|l| l.country == country)
            {
                view! {
                    row = adw::PreferencesRow {
                        set_title: &layout.name,
                        // set_subtitle: &layout,
                        set_activatable: true,
                        // set_subtitle: &locale
                        #[wrap(Some)]
                        set_child = &gtk::Box {
                            set_orientation: gtk::Orientation::Horizontal,
                            set_spacing: 6,
                            set_margin_start: 15,
                            set_margin_end: 7,
                            set_margin_top: 15,
                            set_margin_bottom: 15,
                            gtk::Label {
                                set_label: &layout.name,
                            },
                            #[template]
                            BaseSeparator,
                            gtk::CheckButton {
                                set_halign: gtk::Align::End,
                                set_group: Some(&model.selectiongroup),
                                connect_toggled[sender, layout = layout.clone()] => move |x| {
                                    if x.is_active() {
                                        sender.input(KeyboardMsg::SetSelected(Some(layout.title.clone())))
                                    }
                                }
                            }
                        }
                    }
                }
                expander
                    .first_child()
                    .and_then(|w| w.last_child())
                    .and_then(|w| w.first_child())
                    .and_then(|w| w.downcast::<gtk::ListBox>().ok())
                    .map(|lb| {
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
                                            "Keyboard check button widget not found",
                                        ));
                                    },
                                    |checkbutton| checkbutton.set_active(true),
                                );
                        });
                    })
                    .unwrap_or_else(|| {
                        sender.output(AppMsg::error(
                            ErrorPhase::Setup,
                            "Keyboard expander row widget not found",
                        ));
                    });
                expander.add_row(&row);
            }
            kbdbox.append(&expander);
            model.expanders.push(expander);
        }

        let widgets = view_output!();
        widgets.kbdstack.set_vhomogeneous(false);
        model.shortkbdbox = shortkbdbox;

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>) {
        self.reset();
        match msg {
            KeyboardMsg::SetSelected(layout) => {
                self.selectiongroup.set_active(layout.is_none());
                sender.output(AppMsg::SetCanGoForward(layout.is_some()));

                if layout.is_some() {
                    sender.output(AppMsg::SetKeyboardConfig(layout.clone()));
                }

                self.selected = layout;
                if let Some(selected) = &self.selected {
                    let selected_xkb: [(&str, &String); 1] = [("xkb", selected)];
                    self.keyboard_settings
                        .set_value("sources", &selected_xkb.to_variant());

                    if let (Some(layout), Some(variant)) =
                        (selected.split('+').next(), selected.split('+').nth(1))
                    {
                        let _ = Command::new("setxkbmap")
                            .arg("-layout")
                            .arg(layout)
                            .arg("-variant")
                            .arg(variant)
                            .spawn();
                    } else {
                        let _ = Command::new("setxkbmap").arg(selected).spawn();
                    }
                }
            }
            KeyboardMsg::CheckSelected => {
                trace!("KeyboardMsg::CheckSelected {}", self.selected.is_some());
                if self.selected.is_none() {
                    self.selectiongroup.set_active(true);
                }
                sender.output(AppMsg::SetCanGoForward(self.selected.is_some()));
            }
            KeyboardMsg::ToggleShowall => {
                if !self.showall {
                    for expander in &self.expanders {
                        expander.set_expanded(false);
                    }
                }
                self.showall = !self.showall;
            }
            KeyboardMsg::SetCountry(language, country) => {
                let layouts = self
                    .layouts
                    .iter()
                    .filter_map(|layout| {
                        layout
                            .language
                            .eq(&language.to_lowercase())
                            .then_some(layout.title.clone())
                    })
                    .collect::<Vec<_>>();
                let mut shortvec = layouts
                    .iter()
                    .filter(|k| !k.contains('-') && !k.contains('_'))
                    .filter_map(|x| {
                        self.xkb
                            .layout_info(x)
                            .map(|(name, lang, country, variant)| {
                                Some((
                                    x.to_string(),
                                    (
                                        name?.to_string(),
                                        lang?.to_string(),
                                        country?.to_string(),
                                        variant?.to_string(),
                                    ),
                                ))
                            })?
                    })
                    .collect::<Vec<_>>();

                if shortvec.is_empty() {
                    return;
                }

                shortvec.sort_by(|a, b| {
                    if a.0 == country.to_lowercase() {
                        return std::cmp::Ordering::Less;
                    } else if b.0 == country.to_lowercase() {
                        return std::cmp::Ordering::Greater;
                    };
                    if a.0.contains('+') && !b.0.contains('+') {
                        return std::cmp::Ordering::Greater;
                    } else if b.0.contains('+') && !a.0.contains('+') {
                        return std::cmp::Ordering::Less;
                    };
                    a.0.cmp(&b.0)
                });
                self.selected = if let Some(y) = {
                    shortvec
                        .iter()
                        .filter(|(x, _)| x.contains("latin"))
                        .collect::<Vec<_>>()
                        .first()
                        .map(|(x, _)| x.clone())
                } {
                    Some(y)
                } else if shortvec.iter().any(|(k, _)| k == &country.to_lowercase()) {
                    Some(country.to_lowercase())
                } else {
                    shortvec.first().map(|(k, _)| k.to_string())
                };

                self.shortkbdbox.remove_all();
                for (layout, (name, _lang, _country, _variant)) in shortvec.iter().take(8) {
                    view! {
                        row = adw::PreferencesRow {
                            set_title: name,
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
                                    set_label: name,
                                },
                                #[template]
                                BaseSeparator,
                                #[name(rowbtn)]
                                gtk::CheckButton {
                                    set_halign: gtk::Align::End,
                                    set_group: Some(&self.selectiongroup),
                                    connect_toggled[sender, layout] => move |x| {
                                        if x.is_active() {
                                            sender.input(KeyboardMsg::SetSelected(Some(layout.to_string())))
                                        }
                                    }
                                }
                            }
                        }
                    }
                    self.shortkbdbox.append(&row);
                    rowbtn.set_active(Some(&layout.to_string()) == self.selected.as_ref());
                }
            }
        }
    }
}
