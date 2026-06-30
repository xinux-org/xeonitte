use crate::ui::window::AppMsg;
use adw::prelude::*;
use gettextrs::gettext;
use gnome_desktop::{self, XkbInfo, XkbInfoExt};
use log::trace;
use relm4::*;
use std::process::Command;

#[tracker::track]
#[derive(Debug)]
pub struct KeyboardModel {
    #[allow(clippy::type_complexity)]
    layouts: Vec<(String, (String, String, String, String))>,
    language: Option<String>,
    country: Option<String>,
    showall: bool,
    selectiongroup: gtk::CheckButton,
    selected: Option<String>,
    expanders: Vec<adw::ExpanderRow>,
    shortkbdbox: gtk::ListBox,
    xkb: XkbInfo,
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
                                            trace!("CheckButton can't be found");
                                            let _ = sender.output(AppMsg::Error);
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

        let mut layoutvec = vec![];

        for layout in layouts {
            let layoutinfo = xkb.layout_info(&layout);
            if let Some((Some(name), Some(lang), Some(country), Some(variant))) = layoutinfo {
                layoutvec.push((
                    layout.to_string(),
                    (
                        name.to_string(),
                        lang.to_string(),
                        country.to_string(),
                        variant.to_string(),
                    ),
                ));
            }
        }
        layoutvec.sort_by(|a, b| a.0.cmp(&b.0));

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
            tracker: 0,
        };

        let kbdbox = gtk::ListBox::new();
        let shortkbdbox = gtk::ListBox::new();

        let mut countries = model
            .layouts
            .iter()
            .map(|(_, v)| v.2.as_str())
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
                        .find(|(_, v)| &v.2 == a)
                        .and_then(|(_, v)| v.0.split('(').nth(0).map(|s| s.trim().to_string()))
                });
            let bname = gnome_desktop::country_from_code(&b.to_uppercase(), None)
                .map(|x| x.to_string())
                .or_else(|| {
                    model
                        .layouts
                        .iter()
                        .find(|(_, v)| &v.2 == b)
                        .and_then(|(_, v)| v.0.split('(').nth(0).map(|s| s.trim().to_string()))
                });
            aname.cmp(&bname)
        });
        println!("Post sort");

        for country in &countries {
            let possible_country = model
                .layouts
                .iter()
                .find(|(_, v)| &v.2 == country)
                .and_then(|(_, v)| v.0.split('(').nth(0).map(|s| s.trim().to_string()));
            view! {
                expander = adw::ExpanderRow {
                    set_title: &gnome_desktop::country_from_code(&country.to_uppercase(), None)
                        .map(|x| x.to_string())
                        .or_else(|| possible_country)
                        .unwrap_or_else(|| {
                            trace!("Country name can't be found for {}", country);
                            String::from("Unknown")}),
                }
            }

            for (layout, (name, _lang, _country, _variant)) in
                model.layouts.iter().filter(|(_, v)| &v.2 == country)
            {
                view! {
                    row = adw::PreferencesRow {
                        set_title: name,
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
                                set_label: name,
                            },
                            gtk::Separator {
                                set_hexpand: true,
                                set_opacity: 0.0,
                            },
                            gtk::CheckButton {
                                set_halign: gtk::Align::End,
                                set_group: Some(&model.selectiongroup),
                                connect_toggled[sender, layout] => move |x| {
                                    if x.is_active() {
                                        sender.input(KeyboardMsg::SetSelected(Some(layout.to_string())))
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
                                        trace!("CheckButton can't be found");
                                        let _ = sender.output(AppMsg::Error);
                                    },
                                    |checkbutton| checkbutton.set_active(true),
                                );
                        });
                    })
                    .unwrap_or_else(|| {
                        trace!("ExpanderRow or its children can't be found");
                        let _ = sender.output(AppMsg::Error);
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
                if layout.is_none() {
                    self.selectiongroup.set_active(true);
                    let _ = sender.output(AppMsg::SetCanGoForward(false));
                } else {
                    let _ = sender.output(AppMsg::SetCanGoForward(true));
                    let _ = sender.output(AppMsg::SetKeyboardConfig(layout.clone()));
                }
                self.selected = layout;
                if let Some(selected) = &self.selected {
                    let _ = Command::new("gsettings")
                        .arg("set")
                        .arg("org.gnome.desktop.input-sources")
                        .arg("sources")
                        .arg(&format!("[('xkb','{}')]", selected))
                        .spawn();
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
                    let _ = sender.output(AppMsg::SetCanGoForward(false));
                } else {
                    let _ = sender.output(AppMsg::SetCanGoForward(true));
                }
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
                    .filter_map(|(layout, (_name, lang, _country, _variant))| {
                        if lang == &language.to_lowercase() {
                            Some(layout.to_string())
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>();
                let mut shortvec = layouts
                    .iter()
                    .filter(|k| !k.contains('-') && !k.contains('_'))
                    .filter_map(|x| {
                        let layoutinfo = self.xkb.layout_info(x);
                        if let Some((Some(name), Some(lang), Some(country), Some(variant))) =
                            layoutinfo
                        {
                            let y = Some((
                                x.to_string(),
                                (
                                    name.to_string(),
                                    lang.to_string(),
                                    country.to_string(),
                                    variant.to_string(),
                                ),
                            ));
                            y
                        } else {
                            None
                        }
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
                                gtk::Separator {
                                    set_hexpand: true,
                                    set_opacity: 0.0,
                                },
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
