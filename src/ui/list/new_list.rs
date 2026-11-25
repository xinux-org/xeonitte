use adw::prelude::*;
use gettextrs::gettext;
use relm4::{factory::*, *};

pub trait WithIdAndTitle: std::fmt::Debug + Clone + 'static {
    fn title(&self) -> String;
    fn subtitle(&self) -> String;
}

#[derive(Default, Debug)]
pub struct ListItem<T: WithIdAndTitle> {
    pub item: T,
    pub group: Option<gtk::CheckButton>,
    pub locale: Option<String>,
    pub selected: bool,
}

#[derive(Debug)]
pub enum ListItemMsg<T> {
    Select(T),
    Deselect(T),
}

#[relm4::factory(pub)]
impl<T: WithIdAndTitle> FactoryComponent for ListItem<T> {
    type Init = ListItem<T>;
    type Input = ();
    type Output = ListItemMsg<T>;
    type ParentWidget = adw::PreferencesGroup;
    type CommandOutput = ();

    view! {
        adw::ActionRow {
            // #[track(self.changed(ListItem::locale()))]
            #[watch]
            set_title: &gettext(self.item.title()),
            // #[track(self.changed(ListItem::locale()))]
            #[watch]
            set_subtitle: &gettext(self.item.subtitle()),
            set_activatable: true,
            connect_activated[checkbtn] => move |_| {
                checkbtn.activate();
            },
            #[name(checkbtn)]
            add_suffix = &gtk::CheckButton {
                set_group: self.group.as_ref(),
                set_active: self.selected,
                connect_toggled[sender, item = self.item.clone()] => move |checkbtn| {
                    let item = item.clone();
                    if checkbtn.is_active() {
                        let _ = sender.output(ListItemMsg::Select(item));
                    } else {
                        let _ = sender.output(ListItemMsg::Deselect(item));
                    }
                },
            }
        }
    }

    fn init_model(parent: Self::Init, _index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        parent
    }

    fn update(&mut self, _message: Self::Input, _sender: FactorySender<Self>) {
        // self.reset();
    }
}
