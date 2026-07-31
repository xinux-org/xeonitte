use crate::{
    config::LIBEXECDIR,
    ui::{
        partitions::{
            hibernation::Hibernation,
            luks_password::{LuksPasswordComponent, LuksPasswordMsg},
            partition::{Partition, PartitionInit, PartitionRowMsg},
            partition_group::PartitionGroup,
            whole_disk::WholeDisk,
        },
        templates::base::BaseComponent,
        window::AppMsg,
    },
    utils::SizeType,
};
use gettextrs::gettext;
use log::{debug, error, info, trace};
use relm4::{adw::prelude::*, factory::*, *};
use serde::{Deserialize, Serialize};
use size::Size;
use std::{collections::HashMap, convert::identity, process::Command};

pub struct PartitionModel {
    disks: FactoryVecDeque<WholeDisk>,
    method: PartitionMethod,
    partition_groups: FactoryVecDeque<PartitionGroup>,
    diskgroupbtn: gtk::CheckButton,
    schema: Option<PartitionSchema>,
    efi: bool,
    encryption_enabled: bool,
    luks_password: Controller<LuksPasswordComponent>,
    hibernation: Controller<Hibernation>,
}

#[derive(Debug)]
pub enum PartitionMsg {
    SetMethod(PartitionMethod),
    SetFullDisk(String, u64),
    AddFormatPartition(String, String, String, u64, bool),
    AddMountPartition(String, String, String, u64, bool),
    RemoveFormatPartition(String),
    RemoveMountPartition(String),
    AddPartition(String, CustomPartition),
    SetEncryption,
    SetPartitionEncryption(String, String, u64, bool, bool),
    SetHibernation(bool),
    SetPassphrase,
    SetPassphraseConfirm,
    CheckSelected,
    Refresh,
}

pub static PARTITION_BROKER: MessageBroker<PartitionMsg> = MessageBroker::new();

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum PartitionMethod {
    Basic,
    Advanced,
}

#[derive(Serialize, Debug, Clone)]
pub struct FullDiskOptions {
    pub device: String,
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
                            diskbox -> gtk::ListBox {
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

        let disks: FactoryVecDeque<WholeDisk> =
            FactoryVecDeque::builder().launch_default().detach();

        let model = PartitionModel {
            method: PartitionMethod::Basic,
            partition_groups: FactoryVecDeque::builder()
                .launch(gtk::Box::new(gtk::Orientation::Vertical, 20))
                .detach(),
            diskgroupbtn: gtk::CheckButton::new(),
            schema: None,
            efi: distinst_disks::Bootloader::detect() == distinst_disks::Bootloader::Efi,
            luks_password: luks_model,
            hibernation: hibernation_model,
            encryption_enabled: true,
            disks,
        };

        sender.input(PartitionMsg::Refresh);

        let diskbox = model.disks.widget();
        let partitionbox = model.partition_groups.widget();
        let luksbox = model.luks_password.widget();
        let hibernationbox = model.hibernation.widget();

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
                                let used = part_factoryvec.iter().map(|x| x.size).sum::<u64>();
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
                self.encryption_enabled = self.method == PartitionMethod::Basic;
                let _ = sender.output(AppMsg::SetCanGoForward(false));
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
            PartitionMsg::SetFullDisk(device, size) => {
                trace!("SetFullDisk: {}", device);
                self.schema = Some(PartitionSchema::FullDisk(FullDiskOptions {
                    device,
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
                // trace!("SetEncryption: {}", enabled);
                // self.luks_password.model().encryption_enabled = enabled;
                match &mut self.schema {
                    Some(PartitionSchema::FullDisk(opts)) => {
                        opts.encryption = self.luks_password.model().encryption_enabled;
                        opts.passphrase = (self.luks_password.model().encryption_enabled
                            && !self.luks_password.model().passphrase.is_empty())
                        .then_some(self.luks_password.model().passphrase.clone());
                    }
                    Some(PartitionSchema::Custom(opts)) => {
                        opts.encryption = self.luks_password.model().encryption_enabled;
                        opts.passphrase = (self.luks_password.model().encryption_enabled
                            && !self.luks_password.model().passphrase.is_empty())
                        .then_some(self.luks_password.model().passphrase.clone())
                    }
                    None => {}
                }
                sender.input(PartitionMsg::CheckSelected);
            }
            PartitionMsg::SetPartitionEncryption(name, device, size, encrypt, is_full) => {
                trace!("SetPartitionEncryption {} {}", name, encrypt);

                if let Some(PartitionSchema::Custom(opts)) = &mut self.schema {
                    opts.partitions
                        .entry(name)
                        .and_modify(|part| part.encrypt = encrypt)
                        .or_insert_with(|| CustomPartition {
                            format: None,
                            mountpoint: None,
                            is_full,
                            size,
                            device,
                            encrypt,
                        });
                } else {
                    let mut partitions = HashMap::new();
                    partitions.insert(
                        name,
                        CustomPartition {
                            format: None,
                            mountpoint: None,
                            is_full,
                            size,
                            device,
                            encrypt,
                        },
                    );
                    self.schema = Some(PartitionSchema::Custom(CustomOptions {
                        partitions,
                        disk_size: size,
                        encryption: false,
                        passphrase: None,
                    }));
                }

                let mut any_encrypted = false;
                if let Some(PartitionSchema::Custom(opts)) = &mut self.schema {
                    any_encrypted = opts.partitions.values().any(|p| p.encrypt);
                    opts.encryption = any_encrypted;
                    let passphrase = self.luks_password.model().passphrase.clone();
                    opts.passphrase =
                        (any_encrypted && !passphrase.is_empty()).then_some(passphrase);
                }
                self.encryption_enabled = any_encrypted;
                sender.input(PartitionMsg::CheckSelected);
            }
            PartitionMsg::SetPassphrase => {
                trace!("SetPassphrase");
                // self.luks_password.model().passphrase = pass;
                match &mut self.schema {
                    Some(PartitionSchema::FullDisk(opts)) => {
                        opts.passphrase = (self.luks_password.model().encryption_enabled
                            && !self.luks_password.model().passphrase.is_empty())
                        .then_some(self.luks_password.model().passphrase.clone());
                    }
                    Some(PartitionSchema::Custom(opts)) => {
                        let any_encrypted = opts.partitions.values().any(|p| p.encrypt);
                        let passphrase = self.luks_password.model().passphrase.clone();
                        opts.passphrase =
                            (any_encrypted && !passphrase.is_empty()).then_some(passphrase)
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
            PartitionMsg::SetHibernation(x) => {
                println!("HIBERNATION: {x}");
            }
            PartitionMsg::AddFormatPartition(name, format, device, size, is_full) => {
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
                                size,
                                device,
                                is_full,
                                encrypt: false,
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
                            size,
                            device,
                            is_full,
                            encrypt: false,
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
            PartitionMsg::AddMountPartition(name, mount, device, size, is_full) => {
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
                                size,
                                device,
                                is_full,
                                encrypt: false,
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
                            size,
                            device,
                            is_full,
                            encrypt: false,
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
                        let any_encrypted = schema.values().any(|p| p.encrypt);
                        let password_valid = !any_encrypted
                            || (!self.luks_password.model().passphrase.is_empty()
                                && self.luks_password.model().passphrase
                                    == self.luks_password.model().passphrase_confirm);
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
