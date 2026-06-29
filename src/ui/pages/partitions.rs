use crate::{
    config::LIBEXECDIR,
    ui::{
        util::{SizeType, format_size, represent},
        window::AppMsg,
    },
    utils::{
        disko::{Attrs, Devices, Disk},
        i18n::i18n_f,
    },
};
use gettextrs::gettext;
use log::{debug, error, info, trace};
use relm4::{adw::prelude::*, factory::*, *};
use serde::{Deserialize, Serialize};
use size::Size;
use std::{
    collections::HashMap,
    convert::identity,
    ops::{AddAssign, Sub, SubAssign},
    process::Command,
};

pub struct PartitionModel {
    disks: FactoryVecDeque<WholeDisk>,
    method: PartitionMethod,
    partition_groups: FactoryVecDeque<PartitionGroup>,
    diskgroupbtn: gtk::CheckButton,
    schema: Option<PartitionSchema>,
    efi: bool,
    luks_password: Controller<LuksPasswordComponent>,
}

#[derive(Debug)]
pub enum PartitionMsg {
    SetMethod(PartitionMethod),
    SetFullDisk(String, u64),
    AddFormatPartition(String, String, String, u64),
    AddMountPartition(String, String, String, u64),
    RemoveFormatPartition(String),
    RemoveMountPartition(String),
    AddPartition(String, CustomPartition),
    SetEncryption,
    SetPassphrase,
    SetPassphraseConfirm,
    CheckSelected,
    Refresh,
}

pub static PARTITION_BROKER: MessageBroker<PartitionMsg> = MessageBroker::new();

#[derive(Debug, PartialEq, Eq)]
pub enum PartitionMethod {
    Basic,
    Advanced,
}

#[derive(Serialize, Debug, Clone)]
pub struct FullDiskOptions {
    pub device: String,
    pub encryption: bool,
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

#[derive(Serialize, Debug, Clone)]
pub enum PartitionSchema {
    FullDisk(FullDiskOptions),
    Custom(CustomOptions),
}

#[derive(Serialize, Debug, Clone)]
pub struct CustomPartition {
    pub format: Option<String>,
    pub mountpoint: Option<String>,
    pub device: String,
    pub size: u64,
}

#[relm4::component(pub)]
impl SimpleComponent for PartitionModel {
    type Input = PartitionMsg;
    type Output = AppMsg;
    type Init = ();

    view! {
        gtk::ScrolledWindow {
            set_hexpand: true,
            set_vexpand: true,
            adw::Clamp {
                gtk::Box {
                    set_hexpand: true,
                    set_vexpand: true,
                    set_valign: gtk::Align::Center,
                    set_orientation: gtk::Orientation::Vertical,
                    set_spacing: 20,
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
                            diskbox -> gtk::ListBox {
                                add_css_class: "boxed-list",
                                set_hexpand: true,
                                set_selection_mode: gtk::SelectionMode::None,
                            },

                            // Encryption settings group for Basic mode
                            model.luks_password.widget(),

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
                                set_css_classes: if let Some(PartitionSchema::Custom(opts)) = &model.schema {
                                    let schema = &opts.partitions;
                                    let mut root = false;
                                    let mut bootefi = !model.efi;
                                    for v in schema.values() {
                                        if let Some(file) = &v.mountpoint {
                                            if file == "/" {
                                                root = true;
                                            }
                                            if file == "/boot" {
                                                bootefi = true;
                                            }
                                        }
                                    }
                                    match (root, bootefi) {
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
                                    set_markup: &if let Some(PartitionSchema::Custom(opts)) = &model.schema {
                                        let schema = &opts.partitions;
                                        let mut root = false;
                                        let mut bootefi = !model.efi;
                                        for v in schema.values() {
                                            if let Some(file) = &v.mountpoint {
                                                if file == "/" {
                                                    root = true;
                                                }
                                                if file == "/boot" {
                                                    bootefi = true;
                                                }
                                            }
                                        }
                                        match (root, bootefi) {
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

                            // Encryption settings for Advanced mode
                            model.luks_password.widget(),

                            // gtk::Button {
                            //     add_css_class: "pill",
                            //     adw::ButtonContent {
                            //         set_icon_name: "drive-multidisk-symbolic",
                            //         #[watch]
                            //         set_label: &gettext("Launch GParted"),
                            //     },
                            //     set_halign: gtk::Align::Center,
                            //     connect_clicked => move |_| {
                            //         let cmd = Command::new("gparted").spawn();
                            //         if let Err(e) = cmd {
                            //             error!("Failed to launch GParted: {}", e);
                            //         }
                            //     }
                            // },
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
                }
            }
        }
    }

    fn init(
        _parent_window: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let luks_model = LuksPasswordComponent::builder()
            .launch(())
            .forward(sender.input_sender(), identity);

        let model = PartitionModel {
            disks: FactoryVecDeque::builder().launch_default().detach(),
            method: PartitionMethod::Basic,
            partition_groups: FactoryVecDeque::builder()
                .launch(gtk::Box::new(gtk::Orientation::Vertical, 20))
                .detach(),
            diskgroupbtn: gtk::CheckButton::new(),
            schema: None,
            efi: distinst_disks::Bootloader::detect() == distinst_disks::Bootloader::Efi,
            luks_password: luks_model,
        };

        sender.input(PartitionMsg::Refresh);

        let diskbox = model.disks.widget();
        let partitionbox = model.partition_groups.widget();

        let widgets = view_output!();
        widgets.liststack.set_vhomogeneous(false);

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>) {
        match msg {
            PartitionMsg::Refresh => {
                let mut disks_guard = self.disks.guard();
                let mut partition_groups_guard = self.partition_groups.guard();

                disks_guard.clear();
                partition_groups_guard.clear();

                let out = Command::new("pkexec")
                    .arg(format!("{}/xeonitte-helper", LIBEXECDIR))
                    .arg("get-partitions")
                    .output();

                match out {
                    Ok(out) => {
                        let output = String::from_utf8_lossy(&out.stdout).to_string();
                        let stderr = String::from_utf8_lossy(&out.stderr).to_string();
                        #[derive(Deserialize, Debug)]
                        struct InputDisk {
                            name: String,
                            size: u64,
                            partitions: Vec<InputPartition>,
                        }

                        #[derive(Deserialize, Debug)]
                        struct InputPartition {
                            name: String,
                            #[serde(rename = "format")]
                            _format: String,
                            size: u64,
                        }
                        let disks: serde_json::Result<Vec<InputDisk>> =
                            serde_json::from_str(&output);
                        if let Ok(disks) = disks {
                            debug!("Got disks: {:?}", disks);

                            for disk in disks {
                                disks_guard.push_back(WholeDisk {
                                    name: disk.name.to_string(),
                                    size: disk.size,
                                    group: self.diskgroupbtn.clone(),
                                });

                                let mut part_factoryvec: FactoryVecDeque<Partition> =
                                    FactoryVecDeque::builder().launch_default().detach();
                                let mut part_guard = part_factoryvec.guard();

                                for part in disk.partitions {
                                    info!(
                                        "Partition: {:?} length {}",
                                        part.name,
                                        size::Size::from_bytes(part.size)
                                    );
                                    part_guard.push_back(PartitionInit {
                                        mountrow: adw::ComboRow::new(),
                                        name: part.name.clone(),
                                        size: part.size,
                                        device: disk.name.to_string(),
                                    });
                                }

                                part_guard.drop();

                                let name = disk.name.to_string();
                                let used =
                                    part_factoryvec.iter().map(|x| x.size).fold(0, |x, y| x + y);
                                let total_size = disk.size;
                                let free_space = Size::from_bytes(total_size - used);

                                partition_groups_guard.push_back(PartitionGroup {
                                    partitions: part_factoryvec,
                                    creating_partition: false,
                                    new_partition_size: Size::default(),
                                    name,
                                    free_space,
                                    total_size,
                                    size_type: SizeType::MB,
                                });
                            }
                        } else {
                            error!("Failed to parse partitions: {} : {}", output, stderr);
                        }
                    }
                    Err(e) => {
                        error!("Failed to get partitions: {}", e);
                    }
                }

                disks_guard.drop();
                partition_groups_guard.drop();
                self.schema = None;
            }
            PartitionMsg::SetMethod(method) => {
                self.method = method;
                self.schema = None;
                self.diskgroupbtn.set_active(true);
                let _ = sender.output(AppMsg::SetCanGoForward(false));
                sender.input(PartitionMsg::Refresh);
            }
            PartitionMsg::SetFullDisk(device, size) => {
                trace!("SetFullDisk: {}", device);
                self.schema = Some(PartitionSchema::FullDisk(FullDiskOptions {
                    device,
                    disk_size: size,
                    encryption: self.luks_password.model().encryption_enabled,
                    passphrase: if self.luks_password.model().encryption_enabled
                        && !self.luks_password.model().passphrase.is_empty()
                    {
                        Some(self.luks_password.model().passphrase.clone())
                    } else {
                        None
                    },
                }));
                sender.input(PartitionMsg::CheckSelected);
                trace!("Schema: {:?}", self.schema);
            }
            PartitionMsg::SetEncryption => {
                // trace!("SetEncryption: {}", enabled);
                // self.luks_password.model().encryption_enabled = enabled;
                match &mut self.schema {
                    Some(PartitionSchema::FullDisk(opts)) => {
                        opts.encryption = self.luks_password.model().encryption_enabled;
                        opts.passphrase = if self.luks_password.model().encryption_enabled
                            && !self.luks_password.model().passphrase.is_empty()
                        {
                            Some(self.luks_password.model().passphrase.clone())
                        } else {
                            None
                        };
                    }
                    Some(PartitionSchema::Custom(opts)) => {
                        opts.encryption = self.luks_password.model().encryption_enabled;
                        opts.passphrase = if self.luks_password.model().encryption_enabled
                            && !self.luks_password.model().passphrase.is_empty()
                        {
                            Some(self.luks_password.model().passphrase.clone())
                        } else {
                            None
                        };
                    }
                    None => {}
                }
                sender.input(PartitionMsg::CheckSelected);
            }
            PartitionMsg::SetPassphrase => {
                trace!("SetPassphrase");
                // self.luks_password.model().passphrase = pass;
                match &mut self.schema {
                    Some(PartitionSchema::FullDisk(opts)) => {
                        opts.passphrase = if self.luks_password.model().encryption_enabled
                            && !self.luks_password.model().passphrase.is_empty()
                        {
                            Some(self.luks_password.model().passphrase.clone())
                        } else {
                            None
                        };
                    }
                    Some(PartitionSchema::Custom(opts)) => {
                        opts.passphrase = if self.luks_password.model().encryption_enabled
                            && !self.luks_password.model().passphrase.is_empty()
                        {
                            Some(self.luks_password.model().passphrase.clone())
                        } else {
                            None
                        };
                    }
                    None => {}
                }
                sender.input(PartitionMsg::CheckSelected);
            }
            PartitionMsg::SetPassphraseConfirm => {
                trace!("SetPassphraseConfirm");
                // self.luks_password.model().passphrase_confirm = pass;
                sender.input(PartitionMsg::CheckSelected);
            }
            PartitionMsg::AddFormatPartition(name, format, device, size) => {
                trace!("AddFormatPartition");
                if let Some(PartitionSchema::Custom(opts)) = &mut self.schema {
                    if let Some(part) = opts.partitions.get_mut(&name) {
                        part.format = Some(format);
                    } else {
                        opts.partitions.insert(
                            name,
                            CustomPartition {
                                format: Some(format),
                                mountpoint: None,
                                size: size,
                                device,
                            },
                        );
                    }
                } else {
                    let mut partitions = HashMap::new();
                    partitions.insert(
                        name,
                        CustomPartition {
                            format: Some(format),
                            mountpoint: None,
                            size: size,
                            device,
                        },
                    );
                    self.schema = Some(PartitionSchema::Custom(CustomOptions {
                        partitions,
                        disk_size: size,
                        encryption: self.luks_password.model().encryption_enabled,
                        passphrase: if self.luks_password.model().encryption_enabled
                            && !self.luks_password.model().passphrase.is_empty()
                        {
                            Some(self.luks_password.model().passphrase.clone())
                        } else {
                            None
                        },
                    }));
                }
                sender.input(PartitionMsg::CheckSelected);
                trace!("Schema: {:?}", self.schema);
            }
            PartitionMsg::AddMountPartition(name, mount, device, size) => {
                trace!("AddMountPartition");
                if let Some(PartitionSchema::Custom(opts)) = &mut self.schema {
                    // Check if the mountpoint is already in use
                    for part in opts.partitions.values() {
                        if let Some(partmount) = &part.mountpoint {
                            if partmount == &mount {
                                let mut partition_group_guard = self.partition_groups.guard();
                                for i in 0..partition_group_guard.len() {
                                    let partition_guard =
                                        partition_group_guard[i].partitions.guard();
                                    for j in 0..partition_guard.len() {
                                        if partition_guard[j].name != name {
                                            trace!(
                                                "Deselecting {} {}",
                                                partition_guard[j].name, mount
                                            );
                                            partition_guard.send(
                                                j,
                                                PartitionRowMsg::Deselect(mount.to_string()),
                                            );
                                        }
                                    }
                                }
                            }
                        }
                    }

                    if let Some(part) = opts.partitions.get_mut(&name) {
                        part.mountpoint = Some(mount);
                    } else {
                        opts.partitions.insert(
                            name,
                            CustomPartition {
                                format: None,
                                mountpoint: Some(mount),
                                size: size,
                                device,
                            },
                        );
                    }
                } else {
                    let mut partitions = HashMap::new();
                    partitions.insert(
                        name,
                        CustomPartition {
                            format: None,
                            mountpoint: Some(mount),
                            size: size,
                            device,
                        },
                    );
                    self.schema = Some(PartitionSchema::Custom(CustomOptions {
                        partitions,
                        disk_size: size,
                        encryption: self.luks_password.model().encryption_enabled,
                        passphrase: if self.luks_password.model().encryption_enabled
                            && !self.luks_password.model().passphrase.is_empty()
                        {
                            Some(self.luks_password.model().passphrase.clone())
                        } else {
                            None
                        },
                    }));
                }
                sender.input(PartitionMsg::CheckSelected);
                trace!("Schema: {:?}", self.schema);
            }
            PartitionMsg::RemoveFormatPartition(name) => {
                trace!("RemoveFormatPartition");
                if let Some(PartitionSchema::Custom(opts)) = &mut self.schema {
                    if let Some(part) = opts.partitions.get_mut(&name) {
                        if part.mountpoint.is_none() {
                            opts.partitions.remove(&name);
                        } else {
                            part.format = None;
                        }
                    }
                }
                sender.input(PartitionMsg::CheckSelected);
                trace!("Schema: {:?}", self.schema);
            }
            PartitionMsg::RemoveMountPartition(name) => {
                trace!("RemoveMountPartition");
                if let Some(PartitionSchema::Custom(opts)) = &mut self.schema {
                    if let Some(part) = opts.partitions.get_mut(&name) {
                        if part.format.is_none() {
                            opts.partitions.remove(&name);
                        } else {
                            part.mountpoint = None;
                        }
                    }
                }
                sender.input(PartitionMsg::CheckSelected);
                trace!("Schema: {:?}", self.schema);
            }
            PartitionMsg::AddPartition(name, part) => {
                trace!("AddPartition");
                if let Some(PartitionSchema::Custom(opts)) = &mut self.schema {
                    opts.partitions.insert(name, part);
                } else {
                    let mut partitions = HashMap::new();
                    partitions.insert(name, part.clone());
                    self.schema = Some(PartitionSchema::Custom(CustomOptions {
                        partitions,
                        disk_size: part.size,
                        encryption: self.luks_password.model().encryption_enabled,
                        passphrase: if self.luks_password.model().encryption_enabled
                            && !self.luks_password.model().passphrase.is_empty()
                        {
                            Some(self.luks_password.model().passphrase.clone())
                        } else {
                            None
                        },
                    }));
                }
                sender.input(PartitionMsg::CheckSelected);
                trace!("Schema: {:?}", self.schema);
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
                    Some(PartitionSchema::Custom(opts)) => {
                        let schema = &opts.partitions;
                        let mut root = false;
                        let mut bootefi = false;
                        for part in schema.values() {
                            if part.mountpoint == Some("/".to_string()) {
                                root = true;
                            }
                            if part.mountpoint == Some("/boot".to_string()) || !self.efi {
                                bootefi = true;
                            }
                        }

                        let partitions_valid = root && bootefi;
                        let can_proceed = partitions_valid && password_valid;

                        let _ = sender.output(AppMsg::SetCanGoForward(can_proceed));
                        if can_proceed {
                            let _ = sender.output(AppMsg::SetPartitionConfig(self.schema.clone()));
                        }
                    }
                    None => {
                        let _ = sender.output(AppMsg::SetCanGoForward(false));
                    }
                }
            }
        }
    }
}

#[derive(Default, Debug, PartialEq, Eq, Clone)]
pub struct WholeDisk {
    name: String,
    size: u64,
    group: gtk::CheckButton,
}

#[relm4::factory(pub)]
impl FactoryComponent for WholeDisk {
    type Init = WholeDisk;
    type Input = ();
    type Output = ();
    type ParentWidget = gtk::ListBox;
    type CommandOutput = ();

    view! {
        adw::ActionRow {
            set_title: &self.name,
            #[watch]
            // Translators: Do NOT translate the '{}'
            // The string reads "{/dev/sdX} (20 GB minimum needed)" indicating that the given disk is not large enough
            set_subtitle: &if self.size >= 21_474_836_480  { size::Size::from_bytes(self.size).to_string() } else { i18n_f("{} (20 GB minimum needed)", &[&size::Size::from_bytes(self.size).to_string()]) },
            set_activatable: true,
            set_sensitive: self.size >= 21_474_836_480, // 20GB
            #[name(checkbtn)]
            add_suffix = &gtk::CheckButton {
                set_group: Some(&self.group),
                connect_toggled[name = self.name.to_string(), size = self.size] => move |btn| {
                    if btn.is_active() {
                        PARTITION_BROKER.send(PartitionMsg::SetFullDisk(name.to_string(), size));
                    }
                }
            },
            connect_activated[checkbtn, name = self.name.to_string(), size = self.size] => move |_| {
                checkbtn.set_active(true);
                PARTITION_BROKER.send(PartitionMsg::SetFullDisk(name.to_string(), size));
            }
        }
    }

    fn init_model(parent: Self::Init, _index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        parent
    }
}

#[derive(Default, Debug, PartialEq, Eq, Clone)]
pub struct Partition {
    name: String,
    size: u64,
    mountrow: adw::ComboRow,
    device: String,
    swap: bool,
    donotmount: String,
    donotformat: String,
}

#[derive(Default, Debug, PartialEq, Eq, Clone)]
pub struct PartitionInit {
    name: String,
    size: u64,
    mountrow: adw::ComboRow,
    device: String,
}

#[derive(Debug)]
pub enum PartitionRowMsg {
    Deselect(String),
    SetSwap(bool),
    Delete,
}

#[derive(Debug)]
pub enum PartitionOut {
    Delete(String),
}

#[relm4::factory(pub)]
impl FactoryComponent for Partition {
    type Init = PartitionInit;
    type Input = PartitionRowMsg;
    type Output = PartitionOut;
    type ParentWidget = gtk::ListBox;
    type CommandOutput = ();

    view! {
        adw::ExpanderRow {
            set_title: &self.name,
            set_subtitle: &size::Size::from_bytes(self.size).to_string(),
            add_row = &adw::ComboRow {
                #[watch]
                set_title: &gettext("Format"),
                // TODO: When switching language the "Leave as is" option does not update
                set_model: Some(&gtk::StringList::new(&[&self.donotformat, "btrfs", "ext4", "ext3", "fat32", "ntfs", "xfs", "swap"])),
                connect_selected_notify[sender, name = self.name.to_string(), device = self.device.to_string(), formatstring = self.donotformat.to_string(), size = self.size] => move |row| {
                    if let Some(item) = row.selected_item() {
                        if let Ok(item) = item.downcast::<gtk::StringObject>() {
                            if item.string() == formatstring {
                                PARTITION_BROKER.send(PartitionMsg::RemoveFormatPartition(name.to_string()));
                            } else {
                                PARTITION_BROKER.send(PartitionMsg::AddFormatPartition(name.to_string(), item.string().to_string(), device.to_string(), size));
                            }
                            sender.input(PartitionRowMsg::SetSwap(item.string().eq("swap")));
                        }
                    }
                }
            },
            #[local_ref]
            add_row = mountrow -> adw::ComboRow {
                #[watch]
                set_visible: !self.swap,
                #[watch]
                set_title: &gettext("Mount"),
                // TODO: When switching language the "Do not mount" option does not update
                set_model: Some(&gtk::StringList::new(&[&self.donotmount, " /", "/boot", "/home", "/opt", "/var", "/nix"])),
                connect_selected_notify[name = self.name.to_string(), device = self.device.to_string(), mountstring = self.donotmount.to_string(), size = self.size] => move |row| {
                    if let Some(item) = row.selected_item() {
                        if let Ok(item) = item.downcast::<gtk::StringObject>() {
                            if item.string() == mountstring {
                                PARTITION_BROKER.send(PartitionMsg::RemoveMountPartition(name.to_string()));
                            } else {
                                PARTITION_BROKER.send(PartitionMsg::AddMountPartition(name.to_string(), item.string().trim().to_string(), device.to_string(), size));
                            }
                        }
                    }
                }
            },
            add_row = &adw::ActionRow {
                #[watch]
                set_visible: self.swap,
                #[watch]
                set_title: &gettext("Mount"),
                add_suffix = &gtk::Label {
                    set_text: "swap",
                }
            },

            add_row = &adw::ActionRow {
                set_activatable: false,
                add_suffix = &gtk::Button {
                    set_label: "Delete",
                    add_css_class: "raised",
                    add_css_class: "destructive-action",
                    set_halign: gtk::Align::End,
                    set_margin_top: 8,
                    set_margin_bottom: 8,

                    connect_activate => PartitionRowMsg::Delete,
                    connect_clicked => PartitionRowMsg::Delete
                },
            },
        }
    }

    fn init_model(parent: Self::Init, _index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        Partition {
            name: parent.name,
            size: parent.size,
            mountrow: parent.mountrow,
            device: parent.device,
            swap: false,
            donotmount: gettext("Do not mount"),
            donotformat: gettext("Leave as is"),
        }
    }

    fn init_widgets(
        &mut self,
        _index: &DynamicIndex,
        root: Self::Root,
        _returned_widget: &<Self::ParentWidget as FactoryView>::ReturnedWidget,
        sender: FactorySender<Self>,
    ) -> Self::Widgets {
        let mountrow = &self.mountrow.clone();
        let widgets = view_output!();
        widgets
    }

    fn update(&mut self, msg: Self::Input, _sender: FactorySender<Self>) {
        match msg {
            PartitionRowMsg::Deselect(mount) => {
                if let Some(item) = self.mountrow.selected_item() {
                    if let Ok(item) = item.downcast::<gtk::StringObject>() {
                        if item.string().eq(&mount) {
                            self.mountrow.set_selected(0);
                        }
                    }
                }
            }
            PartitionRowMsg::SetSwap(swap) => {
                self.swap = swap;
            }
            PartitionRowMsg::Delete => _sender
                .output(PartitionOut::Delete(self.name.clone()))
                .unwrap(),
        }
    }
}

#[derive(Debug)]
pub struct PartitionGroup {
    name: String,
    partitions: FactoryVecDeque<Partition>,
    creating_partition: bool,
    new_partition_size: Size,
    free_space: Size,
    total_size: u64,
    size_type: SizeType,
}

#[derive(Debug)]
pub enum PartitionGroupMsg {
    ShowSizeEntry,
    CloseEntry,
    Input(Option<String>),
    Apply,
    Delete(String),
    SetSizeType(SizeType),
    Validate,
}

pub enum PartitionGroupOut {
    ApplyOut(String, CustomPartition),
    Delete(String),
}

#[relm4::factory(pub)]
impl FactoryComponent for PartitionGroup {
    type Init = PartitionGroup;
    type Input = PartitionGroupMsg;
    type Output = ();
    type ParentWidget = gtk::Box;
    type CommandOutput = ();

    view! {
        adw::PreferencesGroup {

            gtk::Box {
                set_hexpand: true,
                set_orientation: gtk::Orientation::Horizontal,
                set_spacing: 8,

                gtk::Box {
                    set_hexpand: true,
                    set_orientation: gtk::Orientation::Vertical,
                    set_spacing: 6,

                    gtk::Box {
                        set_hexpand: true,
                        set_orientation: gtk::Orientation::Horizontal,
                        set_spacing: 6,
                        set_margin_bottom: 6,

                        gtk::Label {
                            set_text: &self.name,
                            set_margin_all: 6,
                            add_css_class: "heading"
                        },

                        gtk::Box {
                            set_hexpand: true,
                            set_halign: gtk::Align::End,
                            set_spacing: 12,

                            gtk::Box {
                                set_orientation: gtk::Orientation::Horizontal,
                                set_spacing: 2,

                                gtk::Label {
                                    set_text: "Total: ",
                                    add_css_class: "heading"
                                },

                                gtk::Label {
                                    set_text: &size::Size::from_bytes::<u64>(self.total_size).to_string(),
                                },
                            },

                            gtk::Box {
                                set_orientation: gtk::Orientation::Horizontal,
                                set_spacing: 2,
                                gtk::Label {
                                    set_text: "Free: ",
                                    add_css_class: "heading"
                                },
                                gtk::Label {
                                    #[watch]
                                    set_text: &format_size(self.free_space),
                                },
                            },

                            gtk::Box {
                                set_orientation: gtk::Orientation::Horizontal,
                                set_halign: gtk::Align::End,
                                set_valign: gtk::Align::Center,
                                set_spacing: 12,

                                gtk::Box {
                                    set_orientation: gtk::Orientation::Horizontal,
                                    add_css_class: "linked",

                                    gtk::Button {
                                        set_icon_name: "edit-undo",
                                        add_css_class: "raised",
                                    },
                                    gtk::Button {
                                        set_icon_name: "edit-redo",
                                        add_css_class: "raised"
                                    }
                                },

                                #[name = "add_partition_button"]
                                gtk::Button {
                                    #[watch]
                                    set_icon_name: "value-increase",
                                    add_css_class: "circular",
                                    connect_clicked[sender] => move |_| {
                                        sender.input(PartitionGroupMsg::ShowSizeEntry);
                                    }
                                },
                            },
                        },
                    },

                   // TODO: make the logic better
                    #[local_ref]
                    testbox -> gtk::ListBox {
                        #[watch]
                        set_visible: self.partitions.len() > 0,
                        set_hexpand: true,
                        set_selection_mode: gtk::SelectionMode::None,
                        add_css_class: "boxed-list",
                    },
                    gtk::ListBox {
                        #[watch]
                        set_visible: self.partitions.len() <= 0,
                        add_css_class: "boxed-list",
                        adw::ActionRow {
                            set_title: "Free space",
                            #[watch]
                            set_subtitle: &format_size(self.free_space),
                            set_activatable: false,
                            // add_css_class: "card",
                        },
                    },
                   // TODO: closing

                    #[name = "new_partition_box"]
                    gtk::Box {
                        set_orientation: gtk::Orientation::Horizontal,
                        set_valign: gtk::Align::Center,
                        set_spacing: 12,
                        set_margin_top: 6,

                        #[watch]
                        set_visible: self.creating_partition,

                        #[name = "size_entry"]
                        adw::EntryRow {
                            set_title: "Enter the new partition size",
                            set_input_purpose: gtk::InputPurpose::Number,
                            set_max_length: 12,
                            set_activates_default: true,
                            set_hexpand: true,
                            // #[watch]
                            set_show_apply_button: false,
                            #[iterate]
                            add_css_class: ["focused", "frame"],
                            inline_css: "padding-top: 6px; padding-bottom: 6px; border-radius: 12px;",
                            // connect_apply[sender] => move |_| {
                            //     sender.input(PartitionGroupMsg::Apply);
                            // },
                            connect_changed[sender] => move |x| {
                                sender.input(PartitionGroupMsg::Input({
                                    let text = x.text();
                                    if text.is_empty() {
                                        None
                                    } else {
                                        Some(text.into())
                                    }
                                }));
                            }
                        },

                        #[name = "dropdown"]
                        gtk::DropDown {
                            set_valign: gtk::Align::Center,
                            set_model: Some(&gtk::StringList::new(&["TiB", "GiB", "MiB", "KiB"])),
                            connect_selected_item_notify[sender, x = self.new_partition_size.clone().bytes()] => move |row| {
                                let t = match row.selected() {
                                    0 => SizeType::TB,
                                    1 => SizeType::GB,
                                    3 => SizeType::KB,
                                    _ => SizeType::MB,
                                };
                                sender.input(PartitionGroupMsg::SetSizeType(t));
                            },
                        },

                        gtk::Button {
                           set_icon_name: "value-decrease",
                           #[iterate]
                           add_css_class: ["raised", "circular", "destructive-action"],
                           set_valign: gtk::Align::Center,

                           connect_clicked => PartitionGroupMsg::CloseEntry,
                        },


                        #[name = "apply_button"]
                        gtk::Button {
                            set_icon_name: "adw-entry-apply-symbolic",
                            set_valign: gtk::Align::Center,
                            #[iterate]
                            add_css_class: ["suggested-action", "circular", "apply-button", "image-button", "disabled"],

                            connect_activate => PartitionGroupMsg::Apply,
                            connect_clicked => PartitionGroupMsg::Apply,
                        },

                    },
                },
            },
        }
    }

    fn init_model(parent: Self::Init, _index: &DynamicIndex, sender: FactorySender<Self>) -> Self {
        let mut partitions = FactoryVecDeque::builder()
            .launch(gtk::ListBox::default())
            .forward(sender.input_sender(), |output| match output {
                PartitionOut::Delete(x) => PartitionGroupMsg::Delete(x),
            });

        let _ = parent
            .partitions
            .iter()
            .map(|x| {
                let p = PartitionInit {
                    name: x.name.clone(),
                    device: x.device.clone(),
                    mountrow: adw::ComboRow::new(),
                    size: x.size,
                };
                partitions.guard().push_back(p);
            })
            .collect::<Vec<_>>();

        Self {
            name: parent.name,
            creating_partition: parent.creating_partition,
            new_partition_size: parent.new_partition_size,
            free_space: parent.free_space,
            total_size: parent.total_size,
            size_type: parent.size_type,
            partitions,
        }
    }

    fn init_widgets(
        &mut self,
        _index: &DynamicIndex,
        root: Self::Root,
        _returned_widget: &<Self::ParentWidget as FactoryView>::ReturnedWidget,
        sender: FactorySender<Self>,
    ) -> Self::Widgets {
        let testbox = self.partitions.widget();
        let widgets = view_output!();
        widgets.dropdown.set_selected(2);
        widgets
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::Input,
        sender: FactorySender<Self>,
    ) {
        match message {
            PartitionGroupMsg::ShowSizeEntry => {
                self.creating_partition = true;
                // let all_zero = |x: &str| x.chars().into_iter().all(|y| y == '0');
                // let size = self
                //     .free_space
                //     .to_string()
                //     .split(" ")
                //     .nth(0)
                //     .unwrap_or_default()
                //     .to_string();
                let tip = match self
                    .free_space
                    .to_string()
                    .split(" ")
                    .nth(1)
                    .unwrap_or_default()
                    .as_ref()
                {
                    "TiB" => {
                        widgets.dropdown.set_selected(0);
                        SizeType::TB
                    }
                    "GiB" => {
                        widgets.dropdown.set_selected(1);
                        SizeType::GB
                    }
                    "MiB" => {
                        widgets.dropdown.set_selected(2);
                        SizeType::MB
                    }
                    _ => {
                        widgets.dropdown.set_selected(3);
                        SizeType::KB
                    }
                };
                self.size_type = tip;
                self.new_partition_size = self.free_space;
                widgets.size_entry.set_text(
                    format_size(self.new_partition_size)
                        .split(" ")
                        .nth(0)
                        .unwrap_or_default(),
                );
                widgets.size_entry.add_css_class("focused");
                widgets.apply_button.set_can_target(true);
                widgets.apply_button.remove_css_class("dimmed");
            }
            PartitionGroupMsg::CloseEntry => {
                self.creating_partition = false;
                widgets.size_entry.remove_css_class("focused");
                widgets.apply_button.set_can_target(false);
                widgets.apply_button.add_css_class("dimmed");
                // widgets.size_entry.set_show_apply_button(false);
                // self.appliable = false;
            }
            PartitionGroupMsg::Input(x) => {
                let x = x.unwrap_or_default();
                match x.parse::<f64>() {
                    Ok(y) => {
                        // let tip = match self.size_type {
                        //     SizeType::TB => "TiB",
                        // };
                        self.new_partition_size = represent(self.size_type, y);

                        sender.input(PartitionGroupMsg::Validate);
                    }
                    Err(_) => {
                        if !x.is_empty() {
                            let y = &x
                                .chars()
                                .into_iter()
                                .filter(|y| !y.is_ascii_digit())
                                .collect::<String>();
                            // let index = x.find(y).unwrap().try_into().unwrap();
                            // widgets.size_entry.delete_text(index, index + 1);
                        } else {
                            self.new_partition_size = Size::default();
                            // widgets.size_entry.set_show_apply_button(false);
                            // widgets.size_entry.remove_css_class("error");
                        }

                        // widgets.size_entry.set_show_apply_button(false);
                        // self.apply_visible = false;
                        widgets.size_entry.add_css_class("error");
                        widgets.apply_button.set_can_target(false);
                        widgets.apply_button.add_css_class("dimmed");
                    }
                }
            }
            PartitionGroupMsg::Apply => {
                // let mut free = Size::from_str(&self.free_space.to_string()).unwrap_or_default();
                // let mut new =
                //     Size::from_str(&self.new_partition_size.to_string()).unwrap_or_default();
                // let mut eq = free.eq(&new);

                // println!(
                //     "BEEEEEFOOOORE: \nfree: {free}\n\nnew: {new}\n\n\neq: {eq}\nfree_space: {:?}\n\nnew_size: {:?}",
                //     self.free_space.bytes(),
                //     self.new_partition_size.bytes()
                // );

                // if eq
                //     || self
                //         .free_space
                //         .sub(self.new_partition_size)
                //         .bytes()
                //         .is_negative()
                // {
                // let mut remainder = self.new_partition_size.sub(self.free_space);
                // remainder.add_assign(Size::from_megabytes(1));
                // println!("\nTHE REMAINDER IS: {remainder}\n");
                //     self.new_partition_size = self.free_space;
                //     self.free_space = Size::from_bytes(0)
                // } else {
                //     self.free_space.sub_assign(self.new_partition_size);
                // }

                // free = Size::from_str(&self.free_space.to_string()).unwrap_or_default();
                // new = Size::from_str(&self.new_partition_size.to_string()).unwrap_or_default();
                // eq = free.eq(&new);

                // println!(
                //     "AAFFTEEERRRRRR: \nfree: {free}\n\nnew: {new}\n\n\neq: {eq}\nfree_space: {:?}\n\nnew_size: {:?}",
                //     self.free_space.bytes(),
                //     self.new_partition_size.bytes()
                // );

                if self.new_partition_size.bytes() > 0 {
                    let index = self.partitions.len() + 1;
                    let device = self
                        .partitions
                        .front()
                        .unwrap_or(&Partition {
                            name: self.name.clone() + "1",
                            size: 0,
                            mountrow: adw::ComboRow::new(),
                            device: self.name.clone(),
                            swap: false,
                            donotmount: "".to_string(),
                            donotformat: "".to_string(),
                        })
                        .device
                        .clone();
                    let size = self.new_partition_size.bytes() as u64;
                    self.partitions.guard().push_back({
                        PartitionInit {
                            name: format!("{device}{index}"),
                            size: size,
                            mountrow: adw::ComboRow::new(),
                            device,
                        }
                    });

                    self.free_space.sub_assign(self.new_partition_size);
                    sender.input(PartitionGroupMsg::CloseEntry);
                }
                sender.input(PartitionGroupMsg::Validate);
            }
            PartitionGroupMsg::Delete(name) => {
                let (index, x) = self
                    .partitions
                    .iter()
                    .cloned()
                    .enumerate()
                    .find_map(|(i, x)| if x.name == name { Some((i, x)) } else { None })
                    .unwrap_or_default();

                self.free_space.add_assign(Size::from_bytes(x.size));
                self.partitions.guard().remove(index.clone());
            }
            PartitionGroupMsg::SetSizeType(x) => {
                let new_size = represent(
                    x,
                    widgets.size_entry.text().parse::<f64>().unwrap_or_default(),
                );
                self.size_type = x;
                self.new_partition_size = new_size;
                if self.free_space.ge(&new_size) {
                    widgets.size_entry.remove_css_class("error");
                    widgets.apply_button.set_can_target(true);
                    widgets.apply_button.remove_css_class("dimmed");
                } else {
                    // widgets.size_entry.set_show_apply_button(true);
                    widgets.size_entry.add_css_class("error");
                    widgets.apply_button.set_can_target(false);
                    widgets.apply_button.add_css_class("dimmed");
                }
                // sender.input(PartitionGroupMsg::Validate);
            }
            PartitionGroupMsg::Validate => {
                // let free = Size::from_str(&self.free_space.to_string()).unwrap_or_default();
                // let new = Size::from_str(&self.new_partition_size.to_string()).unwrap_or_default();
                // let eq = free.eq(&new);

                // println!("I AMMM WOOORRRKIINNGGGG");

                if self.free_space.ge(&self.new_partition_size) {
                    widgets.size_entry.remove_css_class("error");
                    widgets.apply_button.set_can_target(true);
                    widgets.apply_button.remove_css_class("dimmed");
                } else {
                    // widgets.size_entry.set_show_apply_button(true);
                    widgets.size_entry.add_css_class("error");
                    widgets.apply_button.set_can_target(false);
                    widgets.apply_button.add_css_class("dimmed");
                }
            }
        }
        self.update_view(widgets, sender);
    }
}

struct LuksPasswordComponent {
    encryption_enabled: bool,
    passphrase: String,
    passphrase_confirm: String,
}

#[derive(Debug)]
enum LuksPasswordMsg {
    SetEncryption(bool),
    SetPassphrase(String),
    SetPassphraseConfirm(String),
}

#[relm4::component(pub)]
impl SimpleComponent for LuksPasswordComponent {
    type Input = LuksPasswordMsg;
    type Output = PartitionMsg;
    type Init = ();

    view! {
        adw::PreferencesGroup {
            #[watch]
            set_title: &gettext("Encryption"),
            adw::SwitchRow {
                #[watch]
                set_title: &gettext("Enable Disk Encryption"),
                #[watch]
                set_subtitle: &gettext("Encrypt your disk with LUKS"),
                #[watch]
                set_active: model.encryption_enabled,
                connect_active_notify[sender] => move |switch| {
                    sender.input(LuksPasswordMsg::SetEncryption(switch.is_active()));
                }
            },
            adw::PasswordEntryRow {
                #[watch]
                set_title: &gettext("Encryption Password"),
                #[watch]
                set_visible: model.encryption_enabled,
                connect_changed[sender] => move |entry| {
                    sender.input(LuksPasswordMsg::SetPassphrase(entry.text().to_string()));
                }
            },
            adw::PasswordEntryRow {
                #[watch]
                set_title: &gettext("Confirm Password"),
                #[watch]
                set_visible: model.encryption_enabled,
                connect_changed[sender] => move |entry| {
                    sender.input(LuksPasswordMsg::SetPassphraseConfirm(entry.text().to_string()));
                }
            },
            gtk::Label {
                #[watch]
                set_visible: model.encryption_enabled && !model.passphrase.is_empty() && model.passphrase != model.passphrase_confirm,
                #[watch]
                set_label: &gettext("Passwords do not match"),
                add_css_class: "error",
            },
            gtk::Label {
                #[watch]
                set_visible: model.encryption_enabled && model.passphrase.is_empty(),
                #[watch]
                set_label: &gettext("Password is required"),
                add_css_class: "warning",
            },
        },
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = LuksPasswordComponent {
            encryption_enabled: false,
            passphrase: String::new(),
            passphrase_confirm: String::new(),
        };
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }
    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        match message {
            LuksPasswordMsg::SetEncryption(switch) => {
                self.encryption_enabled = switch;
                let _ = sender.output(PartitionMsg::SetEncryption);
            }
            LuksPasswordMsg::SetPassphrase(entry) => {
                self.passphrase = entry;
                let _ = sender.output(PartitionMsg::SetPassphrase);
            }
            LuksPasswordMsg::SetPassphraseConfirm(entry) => {
                self.passphrase_confirm = entry;
                let _ = sender.output(PartitionMsg::SetPassphraseConfirm);
            }
        }
    }
}

pub fn get_storage_size_for_disko(size: u64) -> String {
    let size = Size::from_bytes(size)
        .format()
        .with_style(size::Style::Abbreviated)
        .to_string();

    let mut ssize = size.split_ascii_whitespace().map(|x| {
        if let Some(y) = x.find(".") {
            &x[0..y]
        } else {
            x
        }
    });
    format!(
        "{}{}",
        ssize.next().unwrap(),
        ssize.next().unwrap().chars().nth(0).unwrap_or_default()
    )
}
