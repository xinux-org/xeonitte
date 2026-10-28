use gtk::prelude::*;
use relm4::factory::{DynamicIndex, FactoryComponent, FactorySender, FactoryVecDeque};
use relm4::{ComponentParts, ComponentSender, SimpleComponent, gtk};

use crate::ui::welcome::welcome_model::{WelcomeModel, WelcomeMsg};

#[derive(Debug)]
pub struct SelectorRow {
    value: String,
}

// #[derive(Debug)]
// enum SelectorRowMsg {
//     Select(DynamicIndex),
// }

#[derive(Debug)]
pub enum SelectorRowOutput {
    Select(String),
}

#[relm4::factory(pub)]
impl FactoryComponent for SelectorRow {
    type Init = String;
    type Input = ();
    type Output = SelectorRowOutput;
    type CommandOutput = ();
    type ParentWidget = gtk::ListBox;

    view! {
        #[root]
        gtk::ToggleButton {
            // set_orientation: gtk::Orientation::Horizontal,
            // set_accessible_role: gtk::AccessibleRole::ToggleButton,
            #[name(label)]
            gtk::Label {
                #[watch]
                set_label: &self.value,
                set_width_chars: 3,
            },
            connect_clicked[sender, value = self.value.clone()] => move |_| {
                sender.output(SelectorRowOutput::Select(value.clone()));
            }
        }
    }

    fn init_model(value: Self::Init, _index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        Self { value }
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
    type Output = WelcomeMsg;

    view! {
        gtk::Box{
            #[local_ref]
            rows_box -> gtk::ListBox,
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        // create elements from input
        let mut rows = FactoryVecDeque::builder()
            .launch(gtk::ListBox::default())
            .forward(sender.output_sender(), |output| match output {
                SelectorRowOutput::Select(row) => WelcomeMsg::SetSelected(row.into()),
            });
        for item in init {
            rows.guard().push_back(item);
        }

        // initializing model
        let model = Selector { rows };

        // local component
        let rows_box = model.rows.widget();

        let widgets = view_output!();
        ComponentParts { model, widgets }
    }
}
