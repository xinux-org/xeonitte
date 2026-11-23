use gettextrs::gettext;
use log::debug;
use relm4::factory::FactoryVecDeque;
use relm4::{adw::prelude::*, *};

use relm4::{Component, ComponentParts, ComponentSender, gtk};

use crate::modules::disk::fs::ContentUI;
use crate::modules::disk::{DiskLayout, GiB, MiB};
use crate::ui::list::list_model::ListItem;
use crate::ui::partitions::partition_page::MOUNT_POINT_LIST;

#[derive(Debug)]
pub struct AddPartitionDialogModel {
    pub disk_layout: DiskLayout,
    pub name: String,
    pub creating_partition: bool,
    pub new_partition_size: size::Size,
    pub size_adjustment_value: f64,
    pub content_choise: ContentUI,
    pub content_list_factory: FactoryVecDeque<ListItem>,
    pub filesystem_list_factory: FactoryVecDeque<ListItem>,
    pub mountpoint_list_factory: FactoryVecDeque<ListItem>,
    pub stack_page: AddPartitionStackPage,
}

#[derive(Debug)]
pub enum AddPartitionDialogInput {
    SetStackPage(AddPartitionStackPage),
    SetPartitionSize(f64),
    FileSystemSelect(String),
    FileSystemDeSelect(String),
    SetContent(String),
}

#[derive(Debug)]
pub enum XAddPartitionDialogOutput {
    Apply,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub enum AddPartitionStackPage {
    #[default]
    Size,
    Content,
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
                                connect_clicked[dialog] => move |_| {
                                    dialog.close();
                                    dialog.set_can_close(true);
                                }
                            },
                            pack_end = &gtk::Button {
                                set_label: &gettext("Next"),
                                add_css_class: "suggested-action",
                                connect_clicked[sender] => move |_| {
                                    sender.input(AddPartitionDialogInput::SetStackPage(AddPartitionStackPage::Content));
                                }
                            }
                        }
                    },
                    AddPartitionStackPage::Content => {
                        adw::HeaderBar {
                            set_show_end_title_buttons: false,
                            #[wrap(Some)]
                            set_title_widget = &adw::WindowTitle {
                                set_title: &gettext("Select Content"),
                            },
                            pack_start = &gtk::Button {
                                set_label: &gettext("Size"),
                                connect_clicked[sender] => move |_| {
                                    sender.input(AddPartitionDialogInput::SetStackPage(AddPartitionStackPage::Size));
                                }
                            },
                            // 1. FilesSystems with single mountpoint named ContentUI::Filesystem (ext, exfat ...)
                            // 2. With multiple mountpoints ContentUI::Btrfs (Btrfs)
                            // 3. Swap options for swap
                            // 4. Final Content struct will be wrapped to LuksContent if set true
                            pack_end = match model.content_choise {
                                ContentUI::Filesystem => &gtk::Button {
                                    set_label: &gettext("Next"),
                                    add_css_class: "suggested-action",
                                    connect_clicked[sender] => move |_| {
                                        sender.input(AddPartitionDialogInput::SetStackPage(AddPartitionStackPage::FilsSystemFormat));
                                    }
                                },
                                ContentUI::Btrfs => &gtk::Button {
                                    set_label: &gettext("Subvolums"),
                                    add_css_class: "suggested-action",
                                    connect_clicked[sender] => move |_| {
                                        sender.input(AddPartitionDialogInput::SetStackPage(AddPartitionStackPage::MountPoint));
                                    }
                                },
                                ContentUI::Swap => &gtk::Button {
                                    set_label: &gettext("Swap"),
                                    add_css_class: "suggested-action",
                                    // connect_clicked[sender] => move |_| {
                                    //     sender.input(AddPartitionDialogInput::SetStackPage(AddPartitionStackPage::Content));
                                    // }
                                },
                                ContentUI::Luks => &gtk::Button {
                                    set_label: &gettext("Lusk"),
                                    add_css_class: "suggested-action",
                                    // connect_clicked[sender] => move |_| {
                                    //     sender.input(AddPartitionDialogInput::SetStackPage(AddPartitionStackPage::Content));
                                    // }
                                },
                            },
                        }
                    }
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
                                    sender.input(AddPartitionDialogInput::SetStackPage(AddPartitionStackPage::Content));
                                }
                            },
                            pack_end = &gtk::Button {
                                set_label: &gettext("Apply"),
                                add_css_class: "suggested-action",
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
                            pack_start = match model.content_choise {
                                ContentUI::Filesystem => &gtk::Button {
                                    set_label: &gettext("Back"),
                                    connect_clicked[sender] => move |_| {
                                        sender.input(AddPartitionDialogInput::SetStackPage(AddPartitionStackPage::FilsSystemFormat));
                                    }
                                },
                                ContentUI::Btrfs | ContentUI::Swap | ContentUI::Luks => &gtk::Button {
                                    set_label: &gettext("Back"),
                                    connect_clicked[sender] => move |_| {
                                        sender.input(AddPartitionDialogInput::SetStackPage(AddPartitionStackPage::Content));
                                    }
                                },
                            },
                            pack_end = &gtk::Button {
                                set_label: &gettext("Apply"),
                                add_css_class: "suggested-action",
                                // #[watch]
                                // set_sensitive: model.rebuild_sensitive,
                                // connect_clicked[sender] => move |_| {
                                //     sender.output(XAddPartitionDialogOutput::Apply);
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
                                      set_upper: model.disk_layout.free_space::<MiB>().to_bytes() as f64,
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
                                    set_text: &format!("{} MB", (model.disk_layout.free_space::<MiB>().to_bytes() as f64 - model.size_adjustment_value)),
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
                        AddPartitionStackPage::Content => {
                            adw::PreferencesPage {
                                #[local_ref]
                                content_list_factory_widget -> adw::PreferencesGroup {},
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
            disk_layout: init.disk_layout,
            name: init.name,
            creating_partition: init.creating_partition,
            new_partition_size: init.new_partition_size,
            size_adjustment_value: init.size_adjustment_value,
            stack_page: AddPartitionStackPage::Size,
            filesystem_list_factory: init.filesystem_list_factory,
            mountpoint_list_factory: init.mountpoint_list_factory,
            content_choise: init.content_choise,
            content_list_factory: init.content_list_factory,
        };

        let content_list_factory_widget = model.content_list_factory.widget();
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
            AddPartitionDialogInput::FileSystemSelect(filesystem) => {
                // self.selected.push(key);
                // sender.input(ListMsg::CheckSelected);
                // let mut selected = self.choices.iter().cloned().collect::<HashMap<_, _>>();
                // selected.retain(|k, _| self.selected.contains(k));
                // sender.output(AppMsg::SetListConfig(self.id.to_string(), selected));

                debug!("selected: {filesystem}");
            }
            AddPartitionDialogInput::FileSystemDeSelect(key) => {
                // self.selected.retain(|k| k != &key);
                // sender.input(ListMsg::CheckSelected);
                // let mut selected = self.choices.iter().cloned().collect::<HashMap<_, _>>();
                // selected.retain(|k, _| self.selected.contains(k));
                // sender.output(AppMsg::SetListConfig(self.id.to_string(), selected));
                debug!("Unselected: {key}");
            }
            AddPartitionDialogInput::SetContent(partition_content) => {
                debug!("---------partition_content--------: {}", partition_content);
                let partition_content = ContentUI::from(&partition_content);
                self.content_choise = partition_content;

                match self.content_choise {
                    ContentUI::Filesystem | ContentUI::Btrfs => {
                        // Create single/multiple mountpoint widget components
                        self.refresh_mountpoint_list();
                    }
                    ContentUI::Swap => {}
                    ContentUI::Luks => {}
                }
            }
        }
    }
}

impl AddPartitionDialogModel {
    fn refresh_mountpoint_list(&mut self) {
        // Create mountpoint widget components
        let mut mountpoint_list_guard = self.mountpoint_list_factory.guard();
        mountpoint_list_guard.clear();
        let group = match self.content_choise {
            ContentUI::Filesystem => Some(gtk::CheckButton::new()),
            ContentUI::Btrfs | ContentUI::Swap | ContentUI::Luks => None,
        };
        for choice in MOUNT_POINT_LIST.read().clone() {
            let item = ListItem {
                title: choice.to_owned(),
                description: "description".into(),
                group: group.clone(),
                selected: false,
                ..Default::default()
            };
            mountpoint_list_guard.push_back(item);
        }
        mountpoint_list_guard.drop();
    }
}
