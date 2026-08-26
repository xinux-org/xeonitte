use crate::{
    flow::{
        BRANDING, Choice, DISTRO_NAME, INTERNET_CHECK_URL, InstallFlow, Step, init_steps,
        kernel_choices, package_manager_choices,
    },
    get_storage_size_for_disko,
    ui::{
        error::error_model::{ErrorModel, ErrorMsg},
        install::install_model::{INSTALL_BROKER, InstallModel, InstallMsg},
        install_mode::install_mode::InstallModeModel,
        keyboard::keyboard_model::{KeyboardModel, KeyboardMsg},
        list::list_model::{ListInit, ListModel, ListMsg},
        partitions::partition_model::{
            CustomOptions, FullDiskOptions, PARTITION_BROKER, PartitionModel, PartitionSchema,
        },
        quitdialog::QuitDialogModel,
        summary::summary_model::{SummaryModel, SummaryMsg},
        timezone::timezone_model::TimeZoneModel,
        user::user_model::{UserModel, UserMsg},
        welcome::welcome_model::WelcomeModel,
    },
    utils::{
        disko::{
            Attrs, DeviceContent, Devices, Disk, Filesystem, Gpt, LUKS_PASSWORD_FILE, Luks,
            NixValue, Partition, PartitionContent, Swap, canonical, luks_encrypted,
        },
        i18n::i18n_f,
        install::{InstallAsyncModel, InstallAsyncMsg},
        language::{get_country, get_lang},
        report::ErrorPhase,
    },
};
use adw::prelude::*;
use gettextrs::gettext;
use log::{debug, error, info, trace};
use relm4::{gtk, *};
use size::Size;
use std::{
    collections::{BTreeMap, HashMap},
    convert::identity,
    process::Command,
    thread, time,
};

use struct_patch::Patch;

#[tracker::track]
#[derive(Default, Debug, Clone, Patch, PartialEq)]
#[patch(attribute(derive(Debug, Default, Clone)))]
pub struct ConfigData {
    languageconfig: Option<String>,
    keyboardconfig: Option<String>,
    timezoneconfig: Option<String>,
    #[tracker::no_eq]
    partitionconfig: Option<PartitionSchema>,
    #[tracker::no_eq]
    diskoconfig: Devices,
    userconfig: Option<UserConfig>,

    #[tracker::no_eq]
    listconfig: HashMap<String, HashMap<String, Choice>>,
}

#[tracker::track]
#[derive(Default, Debug, Clone, Patch, PartialEq)]
#[patch(attribute(derive(Debug, Default, Clone)))]
pub struct CarouselData {
    page: StackPage,
    is_transitioning: bool,
    #[tracker::no_eq]
    carousel: adw::Carousel,
    #[tracker::no_eq]
    carouselpages: Vec<Step>,
    current_page: u32,
}

#[tracker::track]
#[derive(Default, Debug, Clone, Patch, PartialEq)]
#[patch(attribute(derive(Debug, Default, Clone)))]
pub struct PagesData {
    #[tracker::no_eq]
    install_flow: Option<InstallFlow>,
    #[tracker::no_eq]
    welcome: WelcomeModel,
    #[tracker::no_eq]
    keyboard: KeyboardModel,
    #[tracker::no_eq]
    timezone: TimeZoneModel,
    #[tracker::no_eq]
    install_mode: InstallModeModel,
    #[tracker::no_eq]
    partition: PartitionModel,
    #[tracker::no_eq]
    user: UserModel,
    #[tracker::no_eq]
    summary: SummaryModel,
    #[tracker::no_eq]
    install: InstallModel,
    #[tracker::no_eq]
    package_managers: ListModel,
    #[tracker::no_eq]
    kernel_selection: ListModel,
    #[tracker::no_eq]
    error: ErrorModel,
    #[tracker::no_eq]
    quitdialog: QuitDialogModel,
}

#[tracker::track]
pub struct AppModel {
    #[tracker::no_eq]
    installworker: WorkerController<InstallAsyncModel>,

    config_data: ConfigData,
    carousel_data: CarouselData,
    pages_data: PagesData,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct UserConfig {
    pub name: String,
    pub username: String,
    pub password: String,
    pub hostname: String,
    pub rootpassword: Option<String>,
    pub autologin: bool,
}

#[derive(Debug)]
pub enum AppMsg {
    QuitDialog,
    InitPages,
    RequestNext,
    RequestPrev,
    PageChanged(u32),
    SetStackPage(StackPage),
    SelectFlow(InstallFlow),
    SetLanguageConfig(Option<String>),
    SetKeyboardConfig(Option<String>),
    SetTimezoneConfig(Option<String>),
    SetPartitionConfig(Option<PartitionSchema>),
    SetUserConfig(Option<UserConfig>),
    SetListConfig(String, HashMap<String, Choice>),
    Install,
    FinishInstall,
    RunNextCommand,
    Finished,
    Error(ErrorPhase, String),
}

impl AppMsg {
    pub fn error(phase: ErrorPhase, message: impl Into<String>) -> Self {
        let message = message.into();
        error!("{message}");
        AppMsg::Error(phase, message)
    }
}

#[derive(Debug)]
pub enum AppAsyncMsg {
    SetPage(StackPage),
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum StackPage {
    #[default]
    Carousel,
    Install,
    Finished,
    Error,
    NoInternet,
}

#[relm4::component(pub)]
#[allow(unused_parens)]
impl Component for AppModel {
    type Init = ();
    type Input = AppMsg;
    type Output = ();
    type CommandOutput = AppAsyncMsg;

    view! {
        #[name(main_window)]
        adw::ApplicationWindow {
            set_default_width: 1100,
            set_default_height: 900,
            connect_close_request[sender] => move |_| {
                debug!("Caught close request");
                if model.page == StackPage::Install {
                    sender.input(AppMsg::QuitDialog);
                    relm4::gtk::glib::Propagation::Stop
                } else {
                    debug!("Quit dialog not showed");
                    relm4::main_application().quit();
                    relm4::gtk::glib::Propagation::Proceed
                }
            },
            gtk::Box {
                set_hexpand: true,
                set_vexpand: true,
                set_halign: gtk::Align::Fill,
                set_valign: gtk::Align::Fill,
                set_orientation: gtk::Orientation::Vertical,
                adw::HeaderBar {
                    add_css_class: "flat",
                    #[wrap(Some)]
                    #[transition(Crossfade)]
                    set_title_widget = match model.page {
                        (StackPage::NoInternet) => {
                            gtk::Label {
                                #[watch]
                                set_label: &i18n_f("{} Installer", &[DISTRO_NAME])
                            }
                        },
                        StackPage::Carousel => {
                            adw::CarouselIndicatorDots {
                                set_halign: gtk::Align::Center,
                                set_hexpand: true,
                                set_carousel: Some(main_carousel)
                            }
                        },
                        StackPage::Install => {
                            gtk::Label {
                                #[watch]
                                set_label: &gettext("Installing…")
                            }
                        },
                        StackPage::Finished => {
                            gtk::Label {
                                #[watch]
                                set_label: &gettext("Installation Finished")
                            }
                        },
                        StackPage::Error => {
                            gtk::Label {
                                #[watch]
                                set_label: &gettext("Installation Failed")
                            }
                        }
                    }
                },

                #[transition(SlideLeftRight)]
                match model.page {
                    StackPage::Carousel => {
                        gtk::Overlay {
                            #[local_ref]
                            main_carousel -> adw::Carousel {
                                set_interactive: false,
                                set_halign: gtk::Align::Fill,
                                set_valign: gtk::Align::Fill,
                                set_hexpand: true,
                                set_vexpand: true,
                                connect_page_changed[sender] => move |_, idx| {
                                    sender.input(AppMsg::PageChanged(idx));
                                },
                            },
                            add_overlay = &gtk::Revealer {
                                set_transition_type: gtk::RevealerTransitionType::Crossfade,
                                #[watch]
                                set_reveal_child: model.current_page > 0 && !model.is_transitioning,
                                set_halign: gtk::Align::Start,
                                set_valign: gtk::Align::Center,
                                set_margin_all: 20,
                                gtk::Button {
                                    set_can_focus: false,
                                    set_can_target: true,
                                    set_height_request: 40,
                                    set_width_request: 40,
                                    add_css_class: "circular",
                                    set_halign: gtk::Align::Start,
                                    set_valign: gtk::Align::Center,
                                    set_icon_name: "go-previous-symbolic",
                                    connect_clicked[sender] => move |_| {
                                        sender.input(AppMsg::RequestPrev);
                                    }
                                }
                            },
                            add_overlay = &gtk::Revealer {
                                set_transition_type: gtk::RevealerTransitionType::Crossfade,
                                #[watch]
                                set_reveal_child: model.can_advance() && !model.is_transitioning,
                                set_halign: gtk::Align::End,
                                set_valign: gtk::Align::Center,
                                set_margin_all: 20,
                                gtk::Button {
                                    set_can_focus: false,
                                    set_can_target: true,
                                    set_height_request: 40,
                                    set_width_request: 40,
                                    #[watch]
                                    set_css_classes: if model.current_page.eq(&main_carousel.n_pages().checked_sub(1).unwrap_or_default()) { &["circular", "suggested-action"] } else { &["circular"] },
                                    set_halign: gtk::Align::Start,
                                    set_valign: gtk::Align::Center,
                                    set_icon_name: "go-next-symbolic",
                                    connect_clicked[sender] => move |_| {
                                        sender.input(AppMsg::RequestNext);
                                    }
                                }
                            }
                        }
                    }
                    StackPage::Install => {
                        #[local]
                        installpage -> gtk::ScrolledWindow {}
                    }
                    StackPage::Finished => {
                        gtk::ScrolledWindow {
                            adw::Clamp {
                                gtk::Box {
                                    set_hexpand: true,
                                    set_vexpand: true,
                                    set_valign: gtk::Align::Center,
                                    set_orientation: gtk::Orientation::Vertical,
                                    set_spacing: 30,
                                    set_margin_all: 20,
                                    gtk::Label {
                                        add_css_class: "title-1",
                                        #[watch]
                                        set_label: &gettext("Finished!"),
                                    },
                                    gtk::Image {
                                        add_css_class: "success",
                                        set_icon_name: Some("emblem-ok-symbolic"),
                                        set_pixel_size: 256,
                                    },
                                    gtk::Button {
                                        add_css_class: "suggested-action",
                                        add_css_class: "pill",
                                        set_halign: gtk::Align::Center,
                                        set_valign: gtk::Align::Center,
                                        #[watch]
                                        set_label: &gettext("Reboot"),
                                        connect_clicked => move |_| {
                                            let _ = Command::new("systemctl").arg("reboot").arg("-i").spawn();
                                        }
                                    }
                                }
                            }
                        }
                    },
                    StackPage::Error => {
                        #[local]
                        errorpage -> gtk::ScrolledWindow {}
                    },
                    StackPage::NoInternet => {
                        adw::StatusPage {
                            set_icon_name: Some("network-wireless-offline-symbolic"),
                            set_title: &gettext("No Internet"),
                            set_description: Some(&gettext("Please connect to the Internet to continue")),
                        }
                    }
                }
            }
        }
    }

    fn init(
        _application: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let ten_millis = time::Duration::from_secs(1);
        thread::sleep(ten_millis);

        let welcome = WelcomeModel::builder()
            .launch(())
            .forward(sender.input_sender(), identity);
        let keyboard = KeyboardModel::builder()
            .launch(())
            .forward(sender.input_sender(), identity);
        let timezone = TimeZoneModel::builder()
            .launch(())
            .forward(sender.input_sender(), identity);
        let install_mode = InstallModeModel::builder()
            .launch(())
            .forward(sender.input_sender(), identity);
        let partition = PartitionModel::builder()
            .launch_with_broker((), &PARTITION_BROKER)
            .forward(sender.input_sender(), identity);
        let user = UserModel::builder()
            .launch(())
            .forward(sender.input_sender(), identity);
        let summary = SummaryModel::builder()
            .launch(())
            .forward(sender.input_sender(), identity);
        let install = InstallModel::builder()
            .launch_with_broker(BRANDING.to_string(), &INSTALL_BROKER)
            .forward(sender.input_sender(), identity);
        let package_managers = ListModel::builder()
            .launch(ListInit {
                id: "PACKAGEMANAGERS".to_string(),
                multiple: true,
                required: false,
                title: "Extra Options".to_string(),
                choices: package_manager_choices(),
            })
            .forward(sender.input_sender(), identity);
        let kernel_selection = ListModel::builder()
            .launch(ListInit {
                id: "KERNEL".to_string(),
                multiple: false,
                required: true,
                title: "Kernel".to_string(),
                choices: kernel_choices(),
            })
            .forward(sender.input_sender(), identity);
        let installworker = InstallAsyncModel::builder()
            .detach_worker(())
            .forward(sender.input_sender(), identity);
        let error = ErrorModel::builder()
            .launch(())
            .forward(sender.input_sender(), identity);
        let quitdialog = QuitDialogModel::builder()
            .launch(root.clone().upcast())
            .forward(sender.input_sender(), identity);

        let startpage = reqwest::blocking::get(INTERNET_CHECK_URL)
            .map(|res| res.status().is_success())
            .map_or(StackPage::NoInternet, |status| {
                if status {
                    StackPage::Carousel
                } else {
                    StackPage::NoInternet
                }
            });

        if startpage == StackPage::NoInternet {
            debug!("Waiting for internet connection…");
            sender.oneshot_command(async move {
                loop {
                    let client = reqwest::Client::new();
                    let res = client.get(INTERNET_CHECK_URL).send().await;
                    if let Ok(res) = res
                        && res.status().is_success()
                    {
                        debug!("Internet connection found!");
                        break;
                    }
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                }
                AppAsyncMsg::SetPage(StackPage::Carousel)
            });
        }

        let carousel = adw::Carousel::new();

        let model = AppModel {
            page: startpage,
            install_flow: None,
            welcome,
            keyboard,
            timezone,
            install_mode,
            partition,
            user,
            summary,
            install,
            package_managers,
            kernel_selection,
            error,
            quitdialog,
            is_transitioning: false,
            carousel,
            carouselpages: Vec::new(),
            current_page: 0,
            installworker,
            tracker: 0,
        };

        let main_carousel = &model.carousel;
        let installpage = model.install.widget().clone();
        let errorpage = model.error.widget().clone();
        let widgets = view_output!();

        sender.input(AppMsg::InitPages);

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>, _root: &Self::Root) {
        self.reset();
        match msg {
            AppMsg::QuitDialog => {
                self.quitdialog
                    .widget()
                    .present(relm4::main_application().active_window().as_ref());
            }

            AppMsg::InitPages => {
                for step in init_steps() {
                    match &step {
                        Step::Welcome => self.carousel.append(self.welcome.widget()),
                        Step::Keyboard => self.carousel.append(self.keyboard.widget()),
                        Step::Location => self.carousel.append(self.timezone.widget()),
                        Step::InstallMode => self.carousel.append(self.install_mode.widget()),
                        _ => {}
                    }
                    self.carouselpages.push(step);
                }
            }

            AppMsg::RequestNext => {
                trace!("AppMsg::RequestNext (page {})", self.current_page);
                if self.is_transitioning {
                    return;
                }
                if !self.can_advance() {
                    return;
                }
                let next = self.current_page + 1;
                if next >= self.carousel.n_pages() {
                    sender.input(AppMsg::SetStackPage(StackPage::Install));
                    sender.input(AppMsg::Install);
                } else {
                    self.is_transitioning = true;
                    let w = self.carousel.nth_page(next);
                    self.carousel.scroll_to(&w, true);
                }
            }

            AppMsg::RequestPrev => {
                trace!("AppMsg::RequestPrev (page {})", self.current_page);
                if self.is_transitioning || self.current_page == 0 {
                    return;
                }
                self.is_transitioning = true;
                let w = self.carousel.nth_page(self.current_page - 1);
                self.carousel.scroll_to(&w, true);
            }

            // AppMsg::PageChanged(idx) => {
            //     trace!("AppMsg::PageChanged: {}", idx);
            //     self.is_transitioning = false;
            //     self.current_page = idx;

            //     if let Some(Step::Summary) = self.carouselpages.get(idx as usize) {
            //         self.summary.emit(SummaryMsg::SetConfig(
            //             self.languageconfig.clone(),
            //             self.keyboardconfig.clone(),
            //             self.timezoneconfig.clone(),
            //             self.partitionconfig.clone(),
            //             Box::new(self.userconfig.clone()),
            //         ));
            //     }
            // }
            AppMsg::SetStackPage(page) => {
                debug!("StackPage: {:?}", page);
                if page.ne(&self.page) {
                    self.page = page;
                }
            }

            AppMsg::SelectFlow(flow) => {
                debug!("SelectFlow: {:?}", flow);

                // Set state to zero
                // self.userconfig = None;
                // self.partitionconfig = None;
                // self.listconfig = HashMap::new();

                let that = |flow: InstallFlow| {
                    let new_carousel = adw::Carousel::new();
                    let new_config = ConfigData::default();
                    for step in flow.steps() {
                        use Step::*;
                        match &step {
                            Welcome => {
                                new_carousel.append(self.welcome.widget());
                            }
                            Keyboard => {
                                new_carousel.append(self.keyboard.widget());
                            }
                            Location => {
                                new_carousel.append(self.timezone.widget());
                            }
                            InstallMode => {
                                new_carousel.append(self.install_mode.widget());
                            }
                            User { root, hostname } => {
                                new_carousel.append(self.user.widget());
                                self.user.emit(UserMsg::SetConfig(*root, *hostname));
                                self.summary.emit(SummaryMsg::ShowHostname(*hostname));
                            }
                            PackageManagers => {
                                new_carousel.append(self.package_managers.widget());
                                new_config
                                    .listconfig
                                    .insert("PACKAGEMANAGERS".to_string(), HashMap::new());
                            }
                            KernelSelection => {
                                new_carousel.append(self.kernel_selection.widget());
                                new_config
                                    .listconfig
                                    .insert("KERNEL".to_string(), HashMap::new());
                            }
                            Partitioning => {
                                new_carousel.append(self.partition.widget());
                            }
                            Summary => {
                                new_carousel.append(self.summary.widget());
                            }
                            _ => {}
                        }
                        self.carouselpages.push(step);
                    }
                    (new_carousel, new_config)
                };

                self.install_flow = Some(flow);
                // Trim carousel back to init pages only.
                // let len = flow.steps().len() as u32;
                // while self.carousel.n_pages() > len {
                //     let last = self.carousel.n_pages() - 1;
                //     let page = self.carousel.nth_page(last);
                //     self.carousel.remove(&page);
                // }
                // self.carouselpages.truncate(len as usize);

                // Reset flow-specific state from the previous selection.

                // Append the new flow's pages.
            }

            AppMsg::SetLanguageConfig(language) => {
                self.languageconfig = language;

                self.package_managers
                    .emit(ListMsg::SetLocale(self.languageconfig.clone()));
                self.kernel_selection
                    .emit(ListMsg::SetLocale(self.languageconfig.clone()));
                self.install
                    .emit(InstallMsg::SetLocale(self.languageconfig.clone()));

                if let Some(language) = &self.languageconfig
                    && let (Ok(lang), Ok(country)) = (
                        get_lang(language.to_string()),
                        get_country(language.to_string()),
                    )
                {
                    self.keyboard.emit(KeyboardMsg::SetCountry(lang, country));
                }
            }

            AppMsg::SetKeyboardConfig(keyboard) => {
                self.keyboardconfig = keyboard;
            }

            AppMsg::SetTimezoneConfig(timezone) => {
                self.timezoneconfig = timezone;
            }

            AppMsg::SetPartitionConfig(partition) => {
                let devices = Devices { disk: Attrs::new() };
                self.set_partition_config(&partition, devices);
                self.partitionconfig = partition;
            }

            AppMsg::SetUserConfig(user) => {
                self.userconfig = user;
            }

            AppMsg::SetListConfig(id, list) => {
                info!("SetListConfig: {} {:?}", id, list);
                self.listconfig.insert(id, list);
            }

            AppMsg::Install => {
                debug!("Installing!");
                if let Some(flow) = &self.install_flow {
                    self.installworker.emit(InstallAsyncMsg::Install(
                        flow.config_id().to_string(),
                        self.languageconfig.clone(),
                        self.timezoneconfig.clone(),
                        self.keyboardconfig.clone(),
                        Box::new(self.partitionconfig.clone()),
                        Box::new(self.userconfig.clone()),
                        self.listconfig.clone(),
                        flow.config_type(),
                        flow.imperative_timezone(),
                        self.diskoconfig.clone(),
                    ));
                }
            }

            AppMsg::FinishInstall => {
                debug!("Finishing install!");
                if let Some(flow) = &self.install_flow {
                    self.installworker.emit(InstallAsyncMsg::FinishInstall(
                        self.timezoneconfig.clone(),
                        flow.imperative_timezone(),
                        vec![],
                    ));
                }
            }

            AppMsg::RunNextCommand => {
                debug!("Running next postinstall command");
                self.installworker.emit(InstallAsyncMsg::RunNextCommand);
            }

            AppMsg::Finished => {
                debug!("Finished!");
                self.page = StackPage::Finished;
            }

            AppMsg::Error(phase, message) => {
                error!("Error in {phase} phase: {message}");
                self.page = StackPage::Error;
                self.error.emit(ErrorMsg::Show(phase, message));
            }
        }
    }

    fn shutdown(&mut self, _widgets: &mut Self::Widgets, _output: Sender<Self::Output>) {}

    fn update_cmd(
        &mut self,
        msg: Self::CommandOutput,
        _sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match msg {
            AppAsyncMsg::SetPage(page) => self.page = page,
        }
    }
}

impl AppModel {
    pub fn can_advance(&self) -> bool {
        match self.carouselpages.get(self.current_page as usize) {
            Some(Step::Welcome) => self.languageconfig.is_some(),
            Some(Step::Keyboard) => self.keyboardconfig.is_some(),
            Some(Step::Location) => self.timezoneconfig.is_some(),
            Some(Step::InstallMode) => self.install_flow.is_some(),
            Some(Step::User { .. }) => self.userconfig.is_some(),
            Some(Step::PackageManagers) => true,
            Some(Step::KernelSelection) => true,
            Some(Step::Partitioning) => self.partitionconfig.is_some(),
            Some(Step::Summary) => true,
            None => false,
        }
    }

    fn set_partition_config(&mut self, partition: &Option<PartitionSchema>, mut devices: Devices) {
        if let Some(partition_schema) = partition.clone() {
            match partition_schema {
                PartitionSchema::FullDisk(FullDiskOptions {
                    device,
                    encryption,
                    passphrase: _,
                    disk_size: _,
                    hibernation: _,
                }) => {
                    devices = if encryption {
                        luks_encrypted(device, LUKS_PASSWORD_FILE)
                    } else {
                        canonical(device)
                    };
                    self.diskoconfig = devices.clone();
                }
                PartitionSchema::Custom(CustomOptions {
                    partitions,
                    encryption: _,
                    passphrase: _,
                    disk_size: _,
                }) => {
                    let mut luks_settings = Attrs::new();
                    luks_settings.insert("allowDiscards".into(), NixValue::Bool(true));

                    let any_encrypted = partitions.values().any(|p| p.encrypt);

                    let make_fs_content = |fs: Filesystem, encrypt: bool, luks_name: String| {
                        if encrypt {
                            PartitionContent::Luks(Luks {
                                name: luks_name,
                                password_file: Some(LUKS_PASSWORD_FILE.into()),
                                settings: luks_settings.clone(),
                                content: Some(Box::new(DeviceContent::Filesystem(fs))),
                                ..Default::default()
                            })
                        } else {
                            PartitionContent::Filesystem(fs)
                        }
                    };

                    let mut disk_disko: BTreeMap<String, Disk> = Attrs::new();

                    for (name, part) in partitions.iter() {
                        let part_key = name.rsplit('/').next().unwrap_or(name.as_str()).to_string();
                        let encrypt = part.encrypt;

                        let (type_code, content) =
                            match (part.mountpoint.as_deref(), part.format.as_deref()) {
                                (Some("/boot"), fmt) => (
                                    Some("EF00".to_string()),
                                    PartitionContent::Filesystem(Filesystem {
                                        format: fmt.unwrap_or("vfat").into(),
                                        mountpoint: Some("/boot".into()),
                                        mount_options: vec!["umask=0077".into()],
                                        ..Default::default()
                                    }),
                                ),
                                (None, Some("swap")) => {
                                    let swap = Swap {
                                        resume_device: Some(true),
                                        ..Default::default()
                                    };
                                    let content = if any_encrypted {
                                        PartitionContent::Luks(Luks {
                                            name: format!("crypted-{}", part_key),
                                            password_file: Some(LUKS_PASSWORD_FILE.into()),
                                            settings: luks_settings.clone(),
                                            content: Some(Box::new(DeviceContent::Swap(swap))),
                                            ..Default::default()
                                        })
                                    } else {
                                        PartitionContent::Swap(swap)
                                    };
                                    (None, content)
                                }
                                (Some(mount), fmt) => (
                                    None,
                                    make_fs_content(
                                        Filesystem {
                                            format: fmt.unwrap_or("ext4").into(),
                                            mountpoint: Some(mount.to_string()),
                                            ..Default::default()
                                        },
                                        encrypt,
                                        format!("crypted-{}", part_key),
                                    ),
                                ),
                                (None, Some(fmt)) => (
                                    None,
                                    make_fs_content(
                                        Filesystem {
                                            format: fmt.into(),
                                            ..Default::default()
                                        },
                                        encrypt,
                                        format!("crypted-{}", part_key),
                                    ),
                                ),
                                (None, None) => continue,
                            };

                        let disko_partition = Partition {
                            type_code,
                            size: if part.is_full {
                                Some("100%".into())
                            } else {
                                Some(get_storage_size_for_disko(Size::from_bytes(part.size)))
                            },
                            content: Some(content),
                            ..Default::default()
                        };

                        let disk_key = part
                            .device
                            .rsplit('/')
                            .next()
                            .unwrap_or(part.device.as_str())
                            .to_string();
                        let disk = disk_disko.entry(disk_key).or_insert_with(|| Disk {
                            device: part.device.clone(),
                            content: Some(DeviceContent::Gpt(Gpt::default())),
                            ..Default::default()
                        });
                        if let Some(DeviceContent::Gpt(gpt)) = disk.content.as_mut() {
                            gpt.partitions.insert(part_key, disko_partition);
                        }
                    }
                    devices = Devices { disk: disk_disko };
                    self.diskoconfig = devices;
                }
            };
        }
    }
}
