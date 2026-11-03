use relm4::factory::{DynamicIndex, FactoryComponent, FactorySender, FactoryVecDeque};
use relm4::{ComponentParts, ComponentSender, SimpleComponent};
use relm4::{adw, adw::prelude::*, gtk};
use std::convert::identity;

#[derive(Debug)]
pub struct SelectorRow {
    content: String,
    group: gtk::ToggleButton,
}

#[derive(Debug)]
pub enum SelectorRowOutput {
    Select(DynamicIndex),
}

#[relm4::factory(pub)]
impl FactoryComponent for SelectorRow {
    type Init = SelectorRow;
    type Input = ();
    type Output = SelectorRowOutput;
    type CommandOutput = ();
    type ParentWidget = gtk::Box;

    view! {
        #[root]
        gtk::ToggleButton {
            set_group: Some(&self.group),
            set_has_frame: false,
            add_css_class: "selector-row",
            remove_css_class: "toggle",

            connect_toggled[sender, index] => move |button| {
                if button.is_active() {
                    sender.output(SelectorRowOutput::Select(index.clone()));
                }
            },

            #[name(label)]
            gtk::Label {
                #[watch]
                set_label: &self.content,
                set_width_chars: 3,
                set_halign: gtk::Align::Start,
            },

        }
    }

    fn init_model(init: Self::Init, _index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        init
    }
}

#[derive(Debug)]
pub struct Selector {
    rows: FactoryVecDeque<SelectorRow>,
}

#[relm4::component(pub)]
impl SimpleComponent for Selector {
    type Init = Vec<String>;
    type Input = ();
    type Output = SelectorRowOutput;

    view! {
        adw::PreferencesGroup {
            gtk::ScrolledWindow {
                set_height_request: 300,
                add_css_class: "rounded-scroll",

                // adw::Clamp{
                    // set_maximum_size: 346,

                    #[local_ref]
                    rows_box -> gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_spacing: 1,
                        set_hexpand: true,
                        set_overflow: gtk::Overflow::Hidden,
                        add_css_class: "selector",
                        add_css_class: "selector-bg",
                    },
                // }
            }
        }

    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        // create elements from input
        let mut rows = FactoryVecDeque::builder()
            .launch(gtk::Box::default())
            .forward(sender.output_sender(), identity);

        let group = gtk::ToggleButton::default();
        for row in init {
            rows.guard().push_back(SelectorRow {
                content: row,
                group: group.clone(),
            });
        }

        // initializing model
        let model = Selector { rows };

        // local component
        let rows_box = model.rows.widget();

        let widgets = view_output!();
        ComponentParts { model, widgets }
    }
}
