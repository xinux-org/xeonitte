use std::{collections::HashMap, convert::identity};

use gettextrs::gettext;
use log::{debug, info, trace};
use relm4::{MessageBroker, SharedState, adw::prelude::*, factory::*, prelude::*};
use serde::Serialize;
use size::Size;

use crate::{
    get_disk_brand_name,
    modules::disk::{self, Content, Disk, DiskLayout, MiB, PartitionDef, PartitionSize},
    ui::{
        partitions::{
            advanced::{
                disk_group::{self, AdvancedDiskGroup},
                partition_row::Partition,
            },
            basic::whole_disk::WholeDisk,
            hibernation::Hibernation,
            luks_password::{LuksPasswordComponent, LuksPasswordMsg},
        },
        templates::base::BaseComponent,
        window::AppMsg,
    },
    utils::SizeType,
};

// Global value that holds all disks -> partitions -> Content inside
// partition qo'shish.
// label xosnom ekanligini tekshirish
// / va /boot bor yo'qligini tekshirish
pub static DISK_LAYOUT_LIST: SharedState<Vec<DiskLayout>> = SharedState::new();

// mountpoint roʻyxatdan turib tanlanadi
// partitionda tanlangan mountpoint qayta tanlanmaydi.
// Roʻyxatgan olib tashlanadi ishlatilgandi
// Roʻyxatgan maxsus qoʻshilganida bori yo ishlatilib olib
// tashlangani qayta qoʻshilmaydi
pub static MOUNT_POINT_LIST: SharedState<Vec<String>> = SharedState::new();

pub static PARTITION_BROKER: MessageBroker<PartitionMsg> = MessageBroker::new();

pub struct PartitionModel {
    basic_disk_factory: FactoryVecDeque<WholeDisk>,
    advanced_disk_factory: FactoryVecDeque<AdvancedDiskGroup>,
    method: PartitionMethod,
    diskgroupbtn: gtk::CheckButton,
    schema: Option<PartitionSchema>,
    efi: bool,
    encryption_enabled: bool,
    luks_password: Controller<LuksPasswordComponent>,
    hibernation: Controller<Hibernation>,
    disks_list: Vec<Disk>,
}

#[derive(Debug)]
pub enum PartitionMsg {
    SetMethod(PartitionMethod),
    SetFullDisk(String, String, u64),
    SetEncryption,
    SetHibernation(bool),
    SetPassphrase,
    SetPassphraseConfirm,
    CheckSelected,
    Refresh,
    RefreshBasic,
    RefreshAdvanced,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum PartitionMethod {
    Basic,
    Advanced,
}

#[derive(Serialize, Debug, Clone)]
pub struct FullDiskOptions {
    // usb-JMicron_XXXXXX
    pub disk_id: String,
    // nvme0n1
    pub disk_name: String,
    pub encryption: bool,
    pub hibernation: bool,
    pub passphrase: Option<String>,
    pub disk_size: u64,
}

#[derive(Serialize, Debug, Clone)]
pub struct CustomOptions {
    pub partitions: HashMap<String, CustomPartition>,
    pub encryption: bool,
    pub passphrase: Option<String>,
    pub disk_size: u64,
}

#[derive(Debug, Clone)]
pub enum PartitionSchema {
    FullDisk(FullDiskOptions),
    Custom(Vec<DiskLayout>),
}

#[derive(Serialize, Debug, Clone)]
pub struct CustomPartition {
    pub format: Option<String>,
    pub mountpoint: Option<String>,
    pub device: String,
    pub size: u64,
    pub encrypt: bool,
    pub is_full: bool,
}

#[relm4::component(pub)]
impl SimpleComponent for PartitionModel {
    type Input = PartitionMsg;
    type Output = AppMsg;
    type Init = ();

    view! {
        gtk::ScrolledWindow {
            #[template]
            BaseComponent {
                #[template_child]
                root_box {
                    set_margin_start: 30,
                    set_margin_end: 30,
                    set_margin_top: 20,
                    set_margin_bottom: 20,
                    #[name(liststack)]
                    match model.method {
                        PartitionMethod::Basic => gtk::Box {
                            set_orientation: gtk::Orientation::Vertical,
                            set_spacing: 20,
                            gtk::Label {
                                #[watch]
                                set_label: &gettext("Select a disk"),
                                add_css_class: "title-1"
                            },
                            gtk::Label {
                                #[watch]
                                set_label: &gettext("Disk will be formatted and all data will be lost"),
                                add_css_class: "dim-label",
                                add_css_class: "title-3"
                            },
                            #[local_ref]
                            basic_disk_box -> gtk::ListBox {
                                add_css_class: "boxed-list",
                                set_hexpand: true,
                                set_selection_mode: gtk::SelectionMode::None,
                            },
                            gtk::Box {
                                set_orientation: gtk::Orientation::Horizontal,
                                set_spacing: 20,
                                set_halign: gtk::Align::Center,

                                gtk::Button {
                                    add_css_class: "pill",
                                    #[watch]
                                    set_label: &gettext("Advanced"),
                                    set_halign: gtk::Align::Center,
                                    connect_clicked[sender] => move |_| {
                                        sender.input(PartitionMsg::SetMethod(PartitionMethod::Advanced));
                                    },
                                },
                                gtk::Button {
                                    set_valign: gtk::Align::Center,
                                    add_css_class: "pill",
                                    connect_clicked[sender] => move |_| {
                                        sender.input(PartitionMsg::Refresh);
                                    },
                                    adw::ButtonContent {
                                        set_icon_name: "view-refresh-symbolic",
                                        #[watch]
                                        set_label: &gettext("Refresh")
                                    }
                                }
                            }
                        },
                        PartitionMethod::Advanced => gtk::Box {
                            set_orientation: gtk::Orientation::Vertical,
                            set_spacing: 20,
                            gtk::Label {
                                #[watch]
                                set_label: &gettext("Select partitions"),
                                add_css_class: "title-1"
                            },

                            gtk::Button {
                                #[watch]
                                set_css_classes: if let Some(PartitionSchema::Custom(disks)) = &model.schema {
                                    let has_root = disks
                                        .iter()
                                        .flat_map(|l| l.partitions())
                                        .any(|p| p.content.as_ref().is_some_and(Content::has_root));

                                    let has_boot = disks
                                        .iter()
                                        .flat_map(|l| l.partitions())
                                        .any(|p| p.content.as_ref().is_some_and(Content::has_boot));

                                    match (has_root, has_boot) {
                                        (true, true) => &["pill", "success"],
                                        (true, false) => &["pill", "error"],
                                        (false, true) => &["pill", "error"],
                                        (false, false) => &["pill", "error"],
                                    }
                                } else {
                                    &["pill", "error"]
                                },
                                set_can_target: false,
                                gtk::Label {
                                    #[watch]
                                    set_markup: &if let Some(PartitionSchema::Custom(disks)) = &model.schema {
                                        let has_root = disks
                                            .iter()
                                            .flat_map(|l| l.partitions())
                                            .any(|p| p.content.as_ref().is_some_and(Content::has_root));

                                        let has_boot = disks
                                            .iter()
                                            .flat_map(|l| l.partitions())
                                            .any(|p| p.content.as_ref().is_some_and(Content::has_boot));

                                        match (has_root, has_boot) {
                                            (true, true) => gettext("Ready to install!"),
                                            // Translators: Do NOT translate anything between the <tt> tags
                                            (true, false) => gettext("Missing <tt>/boot</tt> partition"),
                                            // Translators: Do NOT translate anything between the <tt> tags
                                            (false, true) => gettext("Missing <tt>/</tt> partition"),
                                            // Translators: Do NOT translate anything between the <tt> tags
                                            (false, false) => gettext("Missing <tt>/</tt> and <tt>/boot</tt> partitions"),
                                        }
                                    } else if model.efi {
                                        // Translators: Do NOT translate anything between the <tt> tags
                                        gettext("Missing <tt>/</tt> and <tt>/boot</tt> partitions")
                                    } else {
                                        gettext("Missing <tt>/</tt> partition")
                                    }
                                }
                            },

                            #[local_ref]
                            partitionbox -> gtk::Box {
                                set_hexpand: true,
                                set_orientation: gtk::Orientation::Vertical,
                                set_spacing: 20,
                            },

                            gtk::Box {
                                set_orientation: gtk::Orientation::Horizontal,
                                set_spacing: 20,
                                set_halign: gtk::Align::Center,
                                set_margin_top: 12,

                                gtk::Button {
                                    add_css_class: "pill",
                                    #[watch]
                                    set_label: &gettext("Basic"),
                                    set_halign: gtk::Align::Center,
                                    connect_clicked[sender] => move |_| {
                                        sender.input(PartitionMsg::SetMethod(PartitionMethod::Basic));
                                    }
                                },
                                gtk::Button {
                                    set_valign: gtk::Align::Center,
                                    add_css_class: "pill",
                                    connect_clicked[sender] => move |_| {
                                        sender.input(PartitionMsg::Refresh);
                                    },
                                    adw::ButtonContent {
                                        set_icon_name: "view-refresh-symbolic",
                                        #[watch]
                                        set_label: &gettext("Refresh")
                                    }
                                }
                            }
                        }
                    },

                    // Encryption settings group
                    #[local_ref]
                    luksbox -> adw::PreferencesGroup {
                        #[watch]
                        set_visible: model.encryption_enabled,
                    },

                    #[local_ref]
                    hibernationbox -> adw::PreferencesGroup {
                        set_visible: false
                    },
                },
            },
        },
    }

    fn init(
        _parent_window: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let luks_model = LuksPasswordComponent::builder()
            .launch(())
            .forward(sender.input_sender(), identity);

        let hibernation_model = Hibernation::builder()
            .launch(())
            .forward(sender.input_sender(), identity);

        let basic_disk_factory: FactoryVecDeque<WholeDisk> =
            FactoryVecDeque::builder().launch_default().detach();

        let advanced_disk_factory = FactoryVecDeque::builder()
            .launch(gtk::Box::new(gtk::Orientation::Vertical, 20))
            .detach();

        let model = PartitionModel {
            disks_list: vec![],
            basic_disk_factory,
            advanced_disk_factory,
            method: PartitionMethod::Basic,
            diskgroupbtn: gtk::CheckButton::new(),
            schema: None,
            efi: distinst_disks::Bootloader::detect() == distinst_disks::Bootloader::Efi,
            luks_password: luks_model,
            hibernation: hibernation_model,
            encryption_enabled: true,
        };

        let default_mointpoints = vec![
            "/".to_owned(),
            "/boot".to_owned(),
            "/home".to_owned(),
            "/opt".to_owned(),
            "/var".to_owned(),
            "/nix".to_owned(),
            "/srv".to_owned(),
            "/media".to_owned(),
        ];
        *MOUNT_POINT_LIST.write() = default_mointpoints;

        sender.input(PartitionMsg::Refresh);

        let basic_disk_box = model.basic_disk_factory.widget();
        let partitionbox = model.advanced_disk_factory.widget();
        let luksbox = model.luks_password.widget();
        let hibernationbox = model.hibernation.widget();

        let widgets = view_output!();
        widgets.liststack.set_vhomogeneous(false);

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>) {
        match msg {
            PartitionMsg::Refresh => {
                let disks: Vec<Disk> = Disk::list();
                debug!("Got disks: {:#?}", disks);

                // This for read & write
                *DISK_LAYOUT_LIST.write() = disks
                    .clone()
                    .iter()
                    .map(|disk| DiskLayout::new(disk.id_path(), disk.size))
                    .collect();

                // This needed for only initial UI layout.
                self.disks_list = disks;

                match self.method {
                    PartitionMethod::Basic => sender.input(PartitionMsg::RefreshBasic),
                    PartitionMethod::Advanced => sender.input(PartitionMsg::RefreshAdvanced),
                }

                self.schema = None;
                self.encryption_enabled = self.method == PartitionMethod::Basic;

                sender.output_sender().emit(AppMsg::SetCanGoForward(false));
            }
            PartitionMsg::RefreshBasic => {
                let mut basic_partition_factory_guard = self.basic_disk_factory.guard();
                basic_partition_factory_guard.clear();

                for disk in &self.disks_list {
                    basic_partition_factory_guard.push_back(WholeDisk {
                        name: disk.name.to_string(),
                        id_path: disk.id_path(),
                        brand_name: get_disk_brand_name(&disk.name),
                        size: disk.size,
                        group: self.diskgroupbtn.clone(),
                    });
                }
                basic_partition_factory_guard.drop();
            }
            PartitionMsg::RefreshAdvanced => {
                let mut advansed_partition_groups_guard = self.advanced_disk_factory.guard();
                advansed_partition_groups_guard.clear();

                for disk in &self.disks_list {
                    let mut part_factoryvec: FactoryVecDeque<Partition> =
                        FactoryVecDeque::builder().launch_default().detach();
                    let mut part_guard = part_factoryvec.guard();

                    for part in &disk.partitions {
                        info!(
                            "Partition: {:?} length {}",
                            part.name,
                            size::Size::from_bytes(part.size)
                        );
                        part_guard.push_back(Partition {
                            mountrow: adw::ComboRow::new(),
                            is_swap: false,
                            is_boot: false,
                            is_full: false,
                            donotformat: gettext("Leave as is"),
                            partition_disko: PartitionDef {
                                label: Some(part.name.clone()),
                                size: PartitionSize::Mib(disk::Size::<MiB>::new(part.size)),
                                type_code: None,
                                content: None,
                            },
                        });
                    }

                    part_guard.drop();

                    advansed_partition_groups_guard.push_back(AdvancedDiskGroup {
                        name: disk.name.to_string(),
                        brand_name: get_disk_brand_name(&disk.name),
                        partitions: part_factoryvec,
                        creating_partition: false,
                        new_partition_size: Size::default(),
                        size_type: SizeType::MB,
                        disk_layout: DiskLayout::new(disk.id_path(), disk.size),
                        add_partition_dialog: None,
                    });
                }
                advansed_partition_groups_guard.drop();
            }
            PartitionMsg::SetMethod(method) => {
                self.method = method;
                self.luks_password.emit(LuksPasswordMsg::SetAdvanced(
                    self.method == PartitionMethod::Advanced,
                ));
                self.schema = None;
                self.encryption_enabled = false;
                self.diskgroupbtn.set_active(true);
                let _ = sender.output(AppMsg::SetCanGoForward(false));
                sender.input(PartitionMsg::Refresh);
            }
            PartitionMsg::SetFullDisk(disk_name, disk_id, size) => {
                trace!("SetFullDisk: {}", disk_name);
                self.schema = Some(PartitionSchema::FullDisk(FullDiskOptions {
                    disk_id,
                    disk_name,
                    disk_size: size,
                    hibernation: self.hibernation.model().enabled,
                    encryption: self.luks_password.model().encryption_enabled,
                    passphrase: (self.luks_password.model().encryption_enabled
                        && !self.luks_password.model().passphrase.is_empty())
                    .then_some(self.luks_password.model().passphrase.clone()),
                }));
                sender.input(PartitionMsg::CheckSelected);
                trace!("Schema: {:?}", self.schema);
            }
            PartitionMsg::SetEncryption => {
                match &mut self.schema {
                    Some(PartitionSchema::FullDisk(opts)) => {
                        opts.encryption = self.luks_password.model().encryption_enabled;
                        opts.passphrase = (self.luks_password.model().encryption_enabled
                            && !self.luks_password.model().passphrase.is_empty())
                        .then_some(self.luks_password.model().passphrase.clone());
                    }
                    Some(PartitionSchema::Custom(_)) | None => {}
                }
                sender.input(PartitionMsg::CheckSelected);
            }
            PartitionMsg::SetPassphrase => {
                trace!("SetPassphrase");
                match &mut self.schema {
                    Some(PartitionSchema::FullDisk(opts)) => {
                        opts.passphrase = (self.luks_password.model().encryption_enabled
                            && !self.luks_password.model().passphrase.is_empty())
                        .then_some(self.luks_password.model().passphrase.clone());
                    }
                    Some(PartitionSchema::Custom(_)) | None => {}
                }
                sender.input(PartitionMsg::CheckSelected);
            }
            PartitionMsg::SetPassphraseConfirm => {
                trace!("SetPassphraseConfirm");
                sender.input(PartitionMsg::CheckSelected);
            }
            PartitionMsg::SetHibernation(x) => {
                println!("HIBERNATION: {x}");
            }
            PartitionMsg::CheckSelected => {
                trace!("PartitionMsg::CheckSelected: {:?}", self.schema);

                // Check password validity if encryption is enabled
                let password_valid = !self.luks_password.model().encryption_enabled
                    || (!self.luks_password.model().passphrase.is_empty()
                        && self.luks_password.model().passphrase
                            == self.luks_password.model().passphrase_confirm);

                match &self.schema {
                    Some(PartitionSchema::FullDisk(_)) => {
                        let _ = sender.output(AppMsg::SetCanGoForward(password_valid));
                        if password_valid {
                            let _ = sender.output(AppMsg::SetPartitionConfig(self.schema.clone()));
                        }
                    }
                    Some(PartitionSchema::Custom(_)) | None => {
                        let _ = sender.output(AppMsg::SetCanGoForward(false));
                    }
                }
            }
        }
    }
}
