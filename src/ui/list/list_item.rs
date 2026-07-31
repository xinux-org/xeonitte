use adw::prelude::*;
use gettextrs::gettext;
use relm4::{adw, factory::FactoryComponent};
use relm4::{factory::*, *};

#[tracker::track]
pub struct ListItem {
    pub title: String,
    pub description: String,
    // #[tracker::no_eq]
    pub group: Option<gtk::CheckButton>,
    pub locale: Option<String>,
}

#[derive(Debug)]
pub enum ListItemMsg {
    Select(String),
    Deselect(String),
}

#[relm4::factory(pub)]
impl FactoryComponent for ListItem {
    type Init = ListItem;
    type Input = ();
    type Output = ListItemMsg;
    type ParentWidget = adw::PreferencesGroup;
    type CommandOutput = ();

    view! {
        adw::ActionRow {
            #[track(self.changed(ListItem::locale()))]
            set_title: &gettext(&self.title),
            #[track(self.changed(ListItem::locale()))]
            set_subtitle: &gettext(&self.description),
            set_activatable: true,
            connect_activated[checkbtn] => move |_| {
                checkbtn.activate();
            },
            #[name(checkbtn)]
            add_suffix = &gtk::CheckButton {
                set_group: self.group.as_ref(),
                connect_toggled[sender, title = self.title.to_string()] => move |checkbtn| {
                    if checkbtn.is_active() {
                        let _ = sender.output(ListItemMsg::Select(title.to_string()));
                    } else {
                        let _ = sender.output(ListItemMsg::Deselect(title.to_string()));
                    }
                },
            }
        }
    }
    fn init_model(parent: Self::Init, _index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        parent
    }
    fn update(&mut self, _message: Self::Input, _sender: FactorySender<Self>) {
        self.reset();
    }
}
