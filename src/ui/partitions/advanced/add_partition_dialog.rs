use gettextrs::gettext;
use log::debug;
use relm4::factory::FactoryVecDeque;
use relm4::{Component, ComponentParts, ComponentSender, gtk};
use relm4::{adw::prelude::*, *};

use crate::modules::disk::fs::ContentUI;
use crate::modules::disk::{
    BtrfsContent, Content, DiskLayout, Fs, LinuxFs, MiB, PartitionDef, PartitionSize, Size,
    SwapContent,
};
use crate::ui::list::list_model::{ListItem, ListItemGeneric};
use crate::ui::partitions::partition_page::MOUNT_POINT_LIST;

#[derive(Debug)]
pub struct AddPartitionDialogModel {
    pub disk_layout: DiskLayout,
    pub name: String,
    pub creating_partition: bool,
    pub new_partition_size: size::Size,
    pub size_adjustment_value: f64,
    pub content_choise: ContentUI,
    pub linux_fs: Option<LinuxFs>,
    pub content_list_factory: FactoryVecDeque<ListItemGeneric<ContentUI>>,
    pub filesystem_list_factory: FactoryVecDeque<ListItemGeneric<LinuxFs>>,
    pub mountpoint_list_factory: FactoryVecDeque<ListItem>,
    pub stack_page: AddPartitionStackPage,
    pub partition_disko: Option<PartitionDef>,
    pub mountpoint_list: Vec<String>,
}

#[derive(Debug)]
pub enum AddPartitionDialogInput {
    SetStackPage(AddPartitionStackPage),
    SetPartitionSize(f64),
    SelectLinuxFs(LinuxFs),
    DeSelectLinuxFs,
    SetContent(ContentUI),
    MountpointSelect(String),
    MountpointDeSelect(String),
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
                add_top_bar = match &model.stack_page {
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
                            pack_end = match &model.content_choise {
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
                                    sender.input(AddPartitionDialogInput::DeSelectLinuxFs);
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
                                ContentUI::Btrfs | ContentUI::Swap => &gtk::Button {
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
            partition_disko: init.partition_disko,
            linux_fs: init.linux_fs,
            mountpoint_list: init.mountpoint_list,
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
            AddPartitionDialogInput::SelectLinuxFs(filesystem) => {
                debug!("SelectLinuxFs: {filesystem:?}");
                self.linux_fs = Some(filesystem);
            }
            AddPartitionDialogInput::DeSelectLinuxFs => self.linux_fs = None,
            AddPartitionDialogInput::SetContent(partition_content) => {
                self.content_choise = partition_content;

                match self.content_choise {
                    ContentUI::Filesystem => {
                        self.refresh_mountpoint_list();

                        self.partition_disko = PartitionDef {
                            label: None,
                            size: PartitionSize::Gib(Size::new(2)),
                            type_code: None,
                            content: Some(Content::Filesystem(Fs::default())),
                        }
                        .into();
                    }
                    ContentUI::Btrfs => {
                        self.refresh_mountpoint_list();

                        self.partition_disko = PartitionDef {
                            label: None,
                            size: PartitionSize::Gib(Size::new(2)),
                            type_code: None,
                            content: Some(Content::Btrfs(BtrfsContent::default())),
                        }
                        .into();
                    }
                    ContentUI::Swap => {
                        self.partition_disko = PartitionDef {
                            label: None,
                            size: PartitionSize::Gib(Size::new(2)),
                            type_code: None,
                            content: Some(Content::Swap(SwapContent::default())),
                        }
                        .into();
                    }
                }
                debug!("self.partition_disko: {:?}", self.partition_disko);
            }
            AddPartitionDialogInput::MountpointSelect(id) => {
                debug!("added mountpoint: {:?}", id);
                self.mountpoint_list.push(id);
            }
            AddPartitionDialogInput::MountpointDeSelect(id) => {
                debug!("removed mountpoint: {:?}", id);
                self.mountpoint_list.retain(|mountpoint| mountpoint != &id);
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
            ContentUI::Btrfs | ContentUI::Swap => None,
        };
        for choice in MOUNT_POINT_LIST.read().clone() {
            let item: ListItem = ListItem {
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
