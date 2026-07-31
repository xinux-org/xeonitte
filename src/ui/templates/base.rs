use adw::prelude::*;
use relm4::{
    gtk::{self},
    *,
};

#[relm4::widget_template(pub)]
impl WidgetTemplate for BaseComponent {
    view! {
        #[root]
        adw::Clamp {
            #[name(root_box)]
            gtk::Box {
                set_hexpand: true,
                set_vexpand: true,
                set_valign: gtk::Align::Center,
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 20,
                set_margin_all: 20,
            }
        }
    }
}

#[relm4::widget_template(pub)]
impl WidgetTemplate for BaseSeparator {
    view! {
        gtk::Separator {
            set_hexpand: true,
            set_opacity: 0.0,
        },
    }
}
