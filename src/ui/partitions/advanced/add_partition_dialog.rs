use gettextrs::gettext;
use log::debug;
use relm4::factory::FactoryVecDeque;
use relm4::{adw::prelude::*, *};

use relm4::{Component, ComponentParts, ComponentSender, gtk};

use crate::modules::disk::LinuxFs;
use crate::ui::list::list_model::ListItem;
use crate::utils::SizeType;

#[derive(Debug)]
pub struct AddPartitionDialogModel {
    pub name: String,
    pub creating_partition: bool,
    pub new_partition_size: size::Size,
    pub free_space: f64,
    pub total_size: u64,
    pub size_type: SizeType,
    pub size_adjustment_value: f64,
    pub stack_page: AddPartitionStackPage,
    pub filesystem_list_factory: FactoryVecDeque<ListItem>,
    pub filesystem_list_choices: Vec<LinuxFs>,
    pub mountpoint_list_factory: FactoryVecDeque<ListItem>,
}

#[derive(Debug)]
pub enum AddPartitionDialogInput {
    SetPartitionSize(f64),
    SetStackPage(AddPartitionStackPage),
    FileSystemSelect(String),
    FileSystemDeSelect(String),
}

#[derive(Debug)]
pub enum XAddPartitionDialogOutput {
    Apply,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub enum AddPartitionStackPage {
    #[default]
    Size,
    FilsSystemFormat,
    MountPoint,
}

#[relm4::component(pub)]
impl Component for AddPartitionDialogModel {
    type Init = AddPartitionDialogModel;
    type Input = AddPartitionDialogInput;
    type Output = ();
    type CommandOutput = ();

    view! {
        #[root]
        dialog = adw::Dialog {
            set_content_width: 450,
            set_content_height: 450,
            set_vexpand: true,
            #[wrap(Some)]
            set_child = &adw::ToolbarView {
                add_top_bar = match model.stack_page {
                    AddPartitionStackPage::Size => {
                        adw::HeaderBar {
                            set_show_end_title_buttons: false,
                            #[wrap(Some)]
                            set_title_widget = &adw::WindowTitle {
                                set_title: &gettext("Add partition size"),
                            },
                            pack_start = &gtk::Button {
                                set_label: &gettext("Cancel"),
                                #[watch]
                                set_visible: true,
                                connect_clicked[dialog] => move |_| {
                                    dialog.close();
                                    dialog.set_can_close(true);
                                }
                            },
                            pack_end = &gtk::Button {
                                set_label: &gettext("Next"),
                                add_css_class: "suggested-action",
                                #[watch]
                                set_visible: true,
                                connect_clicked[sender] => move |_| {
                                    sender.input(AddPartitionDialogInput::SetStackPage(AddPartitionStackPage::FilsSystemFormat));
                                }
                            }
                        }
                    },
                    AddPartitionStackPage::FilsSystemFormat => {
                        adw::HeaderBar {
                            set_show_end_title_buttons: false,
                            #[wrap(Some)]
                            set_title_widget = &adw::WindowTitle {
                                set_title: &gettext("Format Volume"),
                            },
                            pack_start = &gtk::Button {
                                set_label: &gettext("Back"),
                                connect_clicked[sender] => move |_| {
                                    sender.input(AddPartitionDialogInput::SetStackPage(AddPartitionStackPage::Size));
                                }
                            },
                            pack_end = &gtk::Button {
                                set_label: &gettext("Apply"),
                                add_css_class: "suggested-action",
                                #[watch]
                                set_visible: true,
                                connect_clicked[sender] => move |_| {
                                    sender.input(AddPartitionDialogInput::SetStackPage(AddPartitionStackPage::MountPoint));
                                }
                            },
                        }
                    },
                    AddPartitionStackPage::MountPoint => {
                        adw::HeaderBar {
                            set_show_end_title_buttons: false,
                            #[wrap(Some)]
                            set_title_widget = &adw::WindowTitle {
                                set_title: &gettext("Mount partition"),
                            },
                            pack_start = &gtk::Button {
                                set_label: &gettext("Back"),
                                connect_clicked[sender] => move |_| {
                                    sender.input(AddPartitionDialogInput::SetStackPage(AddPartitionStackPage::FilsSystemFormat));
                                }
                            },
                            pack_end = &gtk::Button {
                                set_label: &gettext("Apply"),
                                add_css_class: "suggested-action",
                                #[watch]
                                set_visible: true,
                                // #[watch]
                                // set_sensitive: model.rebuild_sensitive,
                                // connect_clicked[sender] => move |_| {
                                //     sender.input(LanguageModelMsg::CheckSelected);
                                // }
                            },
                        }
                    }
                },
                #[wrap(Some)]
                set_content = &gtk::ScrolledWindow {
                    set_hexpand: true,
                    set_vexpand: true,
                    match model.stack_page {
                        AddPartitionStackPage::Size => {
                            adw::PreferencesPage {
                                #[name(size_scale)]
                                adw::PreferencesGroup {
                                    add: &scaler,
                                },
                                adw::PreferencesGroup {
                                  #[name(size_row)]
                                  adw::SpinRow {
                                    set_title: "Partition Size",
                                    #[watch]
                                    set_text: &format!("{} MB", model.size_adjustment_value),
                                    #[wrap(Some)]
                                    #[name(size_adjustment)]
                                    set_adjustment = &gtk::Adjustment {
                                      #[watch]
                                      set_upper: model.free_space,
                                      set_step_increment: 1.0,
                                      set_page_increment: 10.0,
                                      // #[block_signal(upper_handler)]
                                      #[watch]
                                      set_value: model.size_adjustment_value,
                                      connect_value_changed[sender] => move |adjustment| {
                                          sender.input(AddPartitionDialogInput::SetPartitionSize(adjustment.value()));
                                      },
                                    },
                                  },
                                  #[name(free_space_row)]
                                  adw::SpinRow {
                                    set_title: "Free Space",
                                    set_subtitle: "Space following new partition",
                                    #[watch]
                                    set_text: &format!("{} MB", (model.free_space - model.size_adjustment_value)),
                                    #[wrap(Some)]
                                    #[name(free_following_adjustment)]
                                    set_adjustment = &gtk::Adjustment {
                                      #[watch]
                                      set_upper: size_adjustment.upper(),
                                      set_step_increment: 1.0,
                                      set_page_increment: 10.0,
                                      #[watch]
                                      set_value: size_adjustment.upper() - size_adjustment.value(),
                                      // connect_value_changed[sender] => move |adjustment| {
                                      //     sender.input(XAddPartitionDialogInput::SetPartitionSize(adjustment.value()));
                                      // },
                                    },
                                  }
                                },
                                adw::PreferencesGroup {
                                  #[name(size_unit_combo)]
                                  adw::ComboRow  {
                                    set_title: &gettext("Units"),
                                    // notify::selected => $on_size_unit_changed_cb(template);
                                    set_model: Some(&gtk::StringList::new(&[
                                        &gettext("MB"),
                                        &gettext("GB"),
                                        &gettext("TB"),
                                    ])),
                                  }
                                }
                            }
                        },
                        AddPartitionStackPage::FilsSystemFormat => {
                            adw::PreferencesPage {
                                #[local_ref]
                                filesystem_list_factory_widget -> adw::PreferencesGroup {},
                            }
                        },
                        AddPartitionStackPage::MountPoint => {
                            adw::PreferencesPage {
                                #[local_ref]
                                mountpoint_list_factory_widget -> adw::PreferencesGroup {},
                            }
                        },
                    },
                }
            }
        },
        scaler = gtk::Scale::builder()
            .digits(0)
            .round_digits(2)
            .adjustment(&size_adjustment)
            .name("Partition size")
            .build(),
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self {
            name: init.name,
            creating_partition: init.creating_partition,
            new_partition_size: init.new_partition_size,
            free_space: init.free_space,
            total_size: init.total_size,
            size_type: init.size_type,
            size_adjustment_value: init.free_space,
            stack_page: AddPartitionStackPage::Size,
            filesystem_list_factory: init.filesystem_list_factory,
            filesystem_list_choices: init.filesystem_list_choices,
            mountpoint_list_factory: init.mountpoint_list_factory,
        };

        // let mut list_guard = model.file_system_list_factory.guard();
        // let fs_group = gtk::CheckButton::new();
        // for choice in &model.file_system_list_choices {
        //     let item = ListItem {
        //         title: choice.as_str().to_owned(),
        //         // description: choice.description.clone(),
        //         group: Some(fs_group.clone()),
        //         // locale: model.locale.clone(),
        //         // tracker: 0,
        //         selected: choice == &LinuxFs::Exfat,
        //         ..Default::default()
        //     };
        //     list_guard.push_back(item);
        // }
        // list_guard.drop();
        let filesystem_list_factory_widget = model.filesystem_list_factory.widget();
        let mountpoint_list_factory_widget = model.mountpoint_list_factory.widget();
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>, _root: &Self::Root) {
        match message {
            AddPartitionDialogInput::SetPartitionSize(size) => self.size_adjustment_value = size,
            AddPartitionDialogInput::SetStackPage(stack_page) => {
                debug!("Add partition StackPage: {:?}", stack_page);
                if stack_page.ne(&self.stack_page) {
                    self.stack_page = stack_page;
                }
            }
            AddPartitionDialogInput::FileSystemSelect(key) => {
                // self.selected.push(key);
                // sender.input(ListMsg::CheckSelected);
                // let mut selected = self.choices.iter().cloned().collect::<HashMap<_, _>>();
                // selected.retain(|k, _| self.selected.contains(k));
                // sender.output(AppMsg::SetListConfig(self.id.to_string(), selected));
                debug!("selected: {key}");
            }
            AddPartitionDialogInput::FileSystemDeSelect(key) => {
                // self.selected.retain(|k| k != &key);
                // sender.input(ListMsg::CheckSelected);
                // let mut selected = self.choices.iter().cloned().collect::<HashMap<_, _>>();
                // selected.retain(|k, _| self.selected.contains(k));
                // sender.output(AppMsg::SetListConfig(self.id.to_string(), selected));
                debug!("Unselected: {key}");
            }
        }
    }
}
