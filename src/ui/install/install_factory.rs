use adw::prelude::*;
use gettextrs::gettext;
use relm4::{factory::*, *};

#[derive(Debug)]
#[tracker::track]
pub struct InstallSlide {
    pub title: String,
    pub subtitle: String,
    pub image: String,
    pub locale: Option<String>,
}

#[relm4::factory(pub)]
impl FactoryComponent for InstallSlide {
    type Init = InstallSlide;
    type Input = ();
    type Output = ();
    type ParentWidget = adw::Carousel;
    type CommandOutput = ();

    view! {
        gtk::Box {
            set_hexpand: true,
            set_vexpand: true,
            set_orientation: gtk::Orientation::Vertical,
            set_spacing: 10,
            set_margin_all: 15,
            gtk::Label {
                set_halign: gtk::Align::Center,
                add_css_class: "title-3",
                #[track(self.changed(InstallSlide::locale()))]
                set_label: &gettext(&self.title)
            },
            gtk::Label {
                set_halign: gtk::Align::Center,
                #[track(self.changed(InstallSlide::locale()))]
                set_label: &gettext(&self.subtitle)
            },
            gtk::Picture {
                set_margin_all: 50,
                set_halign: gtk::Align::Center,
                set_valign: gtk::Align::Center,
                set_filename: Some(&self.image)
            }
        }
    }
    fn init_model(init: Self::Init, _index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        init
    }
    fn update(&mut self, _message: Self::Input, _sender: FactorySender<Self>) {
        self.reset();
    }
}
