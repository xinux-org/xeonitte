use super::pages::{
    error::ErrorModel,
    install::{InstallModel, InstallMsg},
    keyboard::{KeyboardModel, KeyboardMsg},
    list::ListModel,
    partitions::{PartitionMsg, PartitionSchema},
    summary::{SummaryModel, SummaryMsg},
    timezone::TimeZoneMsg,
    user::UserModel,
    welcome::WelcomeMsg,
};
use crate::{
    get_storage_size_for_disko,
    ui::{
        pages::{
            error::ErrorMsg,
            install::INSTALL_BROKER,
            install_mode::{InstallModeModel, InstallModeMsg},
            list::{ListInit, ListMsg},
            partitions::{CustomOptions, FullDiskOptions, PARTITION_BROKER, PartitionModel},
            timezone::TimeZoneModel,
            user::UserMsg,
            welcome::WelcomeModel,
        },
        quitdialog::QuitDialogModel,
    },
    utils::{
        disko::{
            Attrs, DeviceContent, Devices, Disk, Filesystem, Gpt, LUKS_PASSWORD_FILE, Luks,
            NixValue, Partition, PartitionContent, Swap, canonical, luks_encrypted,
        },
        i18n::i18n_f,
        install::{InstallAsyncModel, InstallAsyncMsg},
        language::{get_country, get_lang},
        parse::{Choice, InstallationConfig, StepType, XeonitteConfig, parse_config},
        report::ErrorPhase,
    },
};
use adw::prelude::*;
use gettextrs::gettext;
use log::{debug, error, info, trace, warn};
use relm4::*;
use size::Size;
use std::{
    collections::{BTreeMap, HashMap},
    convert::identity,
    ops::Not,
    process::Command,
    thread, time,
};

#[tracker::track]
pub struct AppModel {
    page: StackPage,
    #[tracker::no_eq]
    config: XeonitteConfig,
    #[tracker::no_eq]
    installconfig: Option<InstallationConfig>,
    #[tracker::no_eq]
    welcome: Controller<WelcomeModel>,
    #[tracker::no_eq]
    keyboard: Controller<KeyboardModel>,
    #[tracker::no_eq]
    timezone: Controller<TimeZoneModel>,
    #[tracker::no_eq]
    install_mode: Controller<InstallModeModel>,
    #[tracker::no_eq]
    partition: Controller<PartitionModel>,
    #[tracker::no_eq]
    user: Controller<UserModel>,
    #[tracker::no_eq]
    summary: Controller<SummaryModel>,
    #[tracker::no_eq]
    install: Controller<InstallModel>,
    #[tracker::no_eq]
    list: HashMap<String, Controller<ListModel>>,
    #[tracker::no_eq]
    listconfig: HashMap<String, HashMap<String, Choice>>,
    #[tracker::no_eq]
    error: Controller<ErrorModel>,
    #[tracker::no_eq]
    quitdialog: Controller<QuitDialogModel>,

    can_go_back: bool,
    can_go_forward: bool,
    carousel: adw::Carousel,
    #[tracker::no_eq]
    carouselpages: Vec<StepType>,
    current_page: u32,

    languageconfig: Option<String>,
    keyboardconfig: Option<String>,
    timezoneconfig: Option<String>,
    #[tracker::no_eq]
    partitionconfig: Option<PartitionSchema>,
    #[tracker::no_eq]
    diskoconfig: Devices,
    userconfig: Option<UserConfig>,

    #[tracker::no_eq]
    installworker: WorkerController<InstallAsyncModel>,
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
    ChangePage(u32),
    SetCanGoBack(bool),
    SetCanGoForward(bool),
    SetStackPage(StackPage),
    SetStackPageConfig(StackPage, Option<InstallationConfig>, usize),
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StackPage {
    Carousel,
    Install,
    Finished,
    Error,
    NoInternet,
}

#[relm4::component(pub)]
#[allow(unused_parens)] // For relm4 match stack macro
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
                // if model.page == StackPage::FrontPage || model.page == StackPage::Install {
                if model.page == StackPage::Carousel || model.page == StackPage::Install {
                    let _ = sender.input(AppMsg::QuitDialog);
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
                                // Translators: Do NOT translate the '{}'
                                // The string reads "{distribution name} Installer"
                                set_label: &i18n_f("{} Installer", &[&model.config.distribution_name])
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
                    // StackPage::FrontPage => {
                    //     gtk::Box {
                    //         append: model.welcome.widget()
                    //     }
                    // },
                    StackPage::Carousel => {
                        gtk::Overlay {
                            #[local_ref]
                            main_carousel -> adw::Carousel {
                                set_interactive: false,
                                set_halign: gtk::Align::Fill,
                                set_valign: gtk::Align::Fill,
                                set_hexpand: true,
                                set_vexpand: true,
                            },
                            add_overlay = &gtk::Revealer {
                                set_transition_type: gtk::RevealerTransitionType::Crossfade,
                                #[watch]
                                set_reveal_child: model.can_go_back && model.current_page > 0,
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
                                    connect_clicked[main_carousel, sender] => move |_| {
                                        let i = adw::Carousel::position(&main_carousel) as u32;
                                        if i > 0 {
                                            let w = main_carousel.nth_page(i-1);
                                            main_carousel.scroll_to(&w, true);
                                        }
                                        sender.input(AppMsg::ChangePage(i - 1));
                                    }
                                }
                            },
                            add_overlay = &gtk::Revealer {
                                set_transition_type: gtk::RevealerTransitionType::Crossfade,
                                #[watch]
                                set_reveal_child: model.can_go_forward,
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
                                    connect_clicked[main_carousel, sender] => move |_| {
                                        let i = adw::Carousel::position(&main_carousel) as u32;
                                        if i < main_carousel.n_pages().checked_sub(1).unwrap_or_default() {
                                            let w = main_carousel.nth_page(i+1);
                                            main_carousel.scroll_to(&w, true);
                                            sender.input(AppMsg::ChangePage(i + 1));
                                        } else {
                                            sender.input(AppMsg::SetStackPage(StackPage::Install));
                                            sender.input(AppMsg::Install);
                                        }
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
        let config = parse_config().expect("Failed to parse config");
        let welcomepage = WelcomeModel::builder()
            .launch(())
            .forward(sender.input_sender(), identity);
        println!("Welcome page launched");
        let keyboardpage = KeyboardModel::builder()
            .launch(())
            .forward(sender.input_sender(), identity);
        println!("Keyboard page launched");
        let timezonepage = TimeZoneModel::builder()
            .launch(())
            .forward(sender.input_sender(), identity);
        println!("Timezone page launched");
        let instal_mode_page = InstallModeModel::builder()
            .launch(config.clone())
            .forward(sender.input_sender(), identity);
        println!("Timezone page launched");
        let partitionpage = PartitionModel::builder()
            .launch_with_broker((), &PARTITION_BROKER)
            .forward(sender.input_sender(), identity);
        println!("Partition page launched");
        let userpage = UserModel::builder()
            .launch(())
            .forward(sender.input_sender(), identity);
        println!("User page launched");
        let summarypage = SummaryModel::builder()
            .launch(())
            .forward(sender.input_sender(), identity);
        println!("Summary page launched");
        let installpage = InstallModel::builder()
            .launch_with_broker(config.branding.to_string(), &INSTALL_BROKER)
            .forward(sender.input_sender(), identity);
        println!("Install page launched");
        let installworker = InstallAsyncModel::builder()
            .detach_worker(())
            .forward(sender.input_sender(), identity);
        println!("Install worker launched");
        let errorpage = ErrorModel::builder()
            .launch(())
            .forward(sender.input_sender(), identity);
        println!("Error page launched");
        let quitdialog = QuitDialogModel::builder()
            .launch(root.clone().upcast())
            .forward(sender.input_sender(), identity);
        println!("Quit dialog launched");

        let res = reqwest::blocking::get(&config.internet_check_url);
        let startpage = if let Ok(res) = res {
            if res.status().is_success() {
                StackPage::Carousel
            } else {
                StackPage::NoInternet
            }
        } else {
            StackPage::NoInternet
        };

        if startpage == StackPage::NoInternet {
            debug!("Waiting for internet connection…");
            let configclone = config.clone();
            sender.oneshot_command(async move {
                loop {
                    let client = reqwest::Client::new();
                    let res = client.get(&configclone.internet_check_url).send().await;
                    if let Ok(res) = res {
                        if res.status().is_success() {
                            debug!("Internet connection found!");
                            break;
                        }
                    }
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                }
                AppAsyncMsg::SetPage(StackPage::Carousel)
            });
        }

        let model = AppModel {
            page: startpage,
            config,
            installconfig: None,
            welcome: welcomepage,
            keyboard: keyboardpage,
            timezone: timezonepage,
            install_mode: instal_mode_page,
            partition: partitionpage,
            user: userpage,
            summary: summarypage,
            install: installpage,
            list: HashMap::new(),
            listconfig: HashMap::new(),
            error: errorpage,
            quitdialog,
            can_go_back: true,
            can_go_forward: true,
            carousel: adw::Carousel::new(),
            carouselpages: Vec::new(),
            current_page: 0,
            languageconfig: None,
            keyboardconfig: None,
            timezoneconfig: None,
            partitionconfig: None,
            userconfig: None,
            installworker,
            tracker: 0,
            diskoconfig: canonical("/dev/sda".into()),
        };

        sender.input(AppMsg::SetStackPageConfig(
            StackPage::Carousel,
            model
                .config
                .choices
                .iter()
                .cloned()
                .find(|x| x.config.config_id == "init")
                .and_then(move |x| x.config.into()),
            0,
        ));
        let main_carousel = &model.carousel;

        // model.carousel.append(model.welcome.widget());
        // model.carouselpages.insert(0, StepType::Welcome);

        let installpage = model.install.widget().clone();
        let errorpage = model.error.widget().clone();
        let widgets = view_output!();

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
            AppMsg::ChangePage(page) => {
                trace!("AppMsg::ChangePage: {}", page);
                if self.current_page > page {
                    self.can_go_forward = true;
                } else {
                    self.can_go_forward = false;
                }

                if let Some(data) = self.carouselpages.get(page as usize) {
                    match data {
                        StepType::Welcome => {
                            self.welcome.emit(WelcomeMsg::CheckSelected);
                        }
                        StepType::Keyboard => {
                            self.keyboard.emit(KeyboardMsg::CheckSelected);
                        }
                        StepType::Location => {
                            self.timezone.emit(TimeZoneMsg::CheckSelected);
                        }
                        StepType::InstallMode => {
                            self.install_mode.emit(InstallModeMsg::CheckSelected);
                        }
                        StepType::Partitioning => {
                            self.partition.emit(PartitionMsg::CheckSelected);
                        }
                        StepType::User {
                            root: _,
                            hostname: _,
                        } => {
                            self.user.emit(UserMsg::CheckSelected);
                        }
                        StepType::Summary => {
                            self.summary.emit(SummaryMsg::SetConfig(
                                self.languageconfig.clone(),
                                self.keyboardconfig.clone(),
                                self.timezoneconfig.clone(),
                                self.partitionconfig.clone(),
                                Box::new(self.userconfig.clone()),
                            ));
                            self.can_go_forward = true;
                        }
                        StepType::List {
                            id: _,
                            multiple: _,
                            required,
                            title,
                            choices: _,
                        } => {
                            if *required {
                                if let Some(listpage) = self.list.get(title) {
                                    listpage.emit(ListMsg::CheckSelected)
                                } else {
                                    error!("List page not found: {}", title);
                                }
                            } else {
                                self.can_go_forward = true;
                            }
                        }
                        _ => {}
                    }
                }

                self.current_page = page;
            }
            AppMsg::SetCanGoBack(can_go_back) => {
                trace!("Carousel can go back: {}", can_go_back);
                self.can_go_back = can_go_back;
            }
            AppMsg::SetCanGoForward(can_go_forward) => {
                trace!("Carousel can go forward: {}", can_go_forward);
                self.can_go_forward = can_go_forward;
            }
            AppMsg::SetStackPage(page) => {
                debug!("StackPage: {:?}", page);
                if page.ne(&self.page) {
                    self.page = page;
                }
            }
            AppMsg::SetStackPageConfig(page, installconfig, index) => {
                trace!("StackPage: {:?}", page);
                trace!("Config: {:?}", installconfig);
                println!(
                    "+++++++++++++++++++++++++++++++++++{}+++++++++++++++++++++++++++++++++++++++++++++++++++++++++",
                    installconfig.clone().unwrap().config_id
                );
                dbg!(index);

                self.page = page;
                self.installconfig = installconfig;

                // remove already existing pages to make switch: advanced -> basic available
                // let mut keys: Vec<usize> = self.carouselpages.keys().cloned().collect();
                // keys.sort_by(|x, y| x.cmp(y));
                // if !&self.carouselpages.is_empty() {
                //     keys.iter().for_each(|k| {
                //         k.gt(&(index)).then(|| self.carouselpages.remove(&k));
                //     });
                // }
                // let keys_after: Vec<usize> = self.carouselpages.keys().cloned().collect();
                self.carouselpages = self.carouselpages.iter().cloned().take(index).collect();
                let len = self.carouselpages.len();
                for i in index..len {
                    let page = self.carousel.nth_page(i as u32);
                    self.carousel.remove(&page);
                }

                // self.carouselpages
                //     .clone()
                //     .iter()
                //     .enumerate()
                //     .rev()
                //     .for_each(|(i, _)| {
                //         println!(
                //             "============================INDEX: {}, LEN: {}",
                //             i,
                //             self.carouselpages.clone().len()
                //         );
                //         index.gt(&i).then(|| self.carouselpages.remove(i));
                //     });

                let carouselpages_before = self.carouselpages.clone();
                dbg!(carouselpages_before);

                if let Some(cfg) = &self.installconfig {
                    let steps: Vec<&StepType> = cfg
                        .steps
                        .iter()
                        .filter(|step| !self.carouselpages.contains(step))
                        .collect();
                    for step in &steps {
                        match step {
                            StepType::Welcome => {
                                if self
                                    .carouselpages
                                    .iter()
                                    .find(|x| **x == StepType::Welcome)
                                    .is_none()
                                {}
                                trace!("Welcome append");
                                self.carousel.append(self.welcome.widget());
                                self.carouselpages.push(StepType::Welcome);
                            }
                            StepType::Keyboard => {
                                trace!("Keyboard append");
                                self.carousel.append(self.keyboard.widget());
                                self.carouselpages.push(StepType::Keyboard);
                            }
                            StepType::Location => {
                                trace!("Timezone append");
                                self.carousel.append(self.timezone.widget());
                                self.carouselpages.push(StepType::Location);
                            }
                            StepType::InstallMode => {
                                trace!("Install Mode append");
                                self.carousel.append(self.install_mode.widget());
                                self.carouselpages.push(StepType::InstallMode);
                            }
                            StepType::Partitioning => {
                                trace!("Partitioning append");
                                self.carousel.append(self.partition.widget());
                                self.carouselpages.push(StepType::Partitioning);
                            }
                            StepType::User { root, hostname } => {
                                trace!("User append");
                                self.carousel.append(self.user.widget());
                                self.carouselpages.push(StepType::User {
                                    root: *root,
                                    hostname: *hostname,
                                });
                                self.user.emit(UserMsg::SetConfig(
                                    if let Some(root) = root { *root } else { false },
                                    if let Some(hostname) = hostname {
                                        *hostname
                                    } else {
                                        false
                                    },
                                    self.config.default_hostname.to_string(),
                                ));
                                self.summary
                                    .emit(SummaryMsg::ShowHostname(hostname.unwrap_or(false)));
                            }
                            StepType::Summary => {
                                trace!("Summary append");
                                self.carousel.append(self.summary.widget());
                                self.carouselpages.push(StepType::Summary);
                            }
                            StepType::List {
                                id,
                                multiple,
                                required,
                                title,
                                choices,
                            } => {
                                trace!("List append: {}", title);
                                let listpage = ListModel::builder()
                                    .launch(ListInit {
                                        id: id.to_string(),
                                        multiple: *multiple,
                                        required: *required,
                                        title: title.to_string(),
                                        choices: choices.clone(),
                                    })
                                    .forward(sender.input_sender(), identity);
                                self.carousel.append(listpage.widget());
                                self.list.insert(title.to_string(), listpage);
                                self.carouselpages.push(StepType::List {
                                    id: id.to_string(),
                                    multiple: *multiple,
                                    required: *required,
                                    title: title.to_string(),
                                    choices: choices.clone(),
                                });
                                self.listconfig.insert(id.to_string(), HashMap::new());
                            }
                            _ => {
                                warn!("Unimplemented step: {:?}", step);
                            }
                        }
                    }
                }

                let carouselpages_after = self.carouselpages.clone();
                dbg!(carouselpages_after);
                sender.input(AppMsg::ChangePage(0));
            }
            AppMsg::SetLanguageConfig(language) => {
                self.languageconfig = language;
                for listpage in self.list.values() {
                    listpage.emit(ListMsg::SetLocale(self.languageconfig.clone()));
                }
                self.install
                    .emit(InstallMsg::SetLocale(self.languageconfig.clone()));
                if let Some(language) = &self.languageconfig {
                    if let (Ok(lang), Ok(country)) = (
                        get_lang(language.to_string()),
                        get_country(language.to_string()),
                    ) {
                        self.keyboard.emit(KeyboardMsg::SetCountry(lang, country));
                    }
                }
            }
            AppMsg::SetKeyboardConfig(keyboard) => {
                self.keyboardconfig = keyboard;
            }
            AppMsg::SetTimezoneConfig(timezone) => {
                self.timezoneconfig = timezone;
            }
            AppMsg::SetPartitionConfig(partition) => {
                let mut devices = Devices { disk: Attrs::new() };
                partition.clone().map(|x| {
                    let _ = match x {
                        PartitionSchema::FullDisk(FullDiskOptions {
                            device,
                            encryption,
                            passphrase,
                            disk_size,
                            hibernation,
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

                            let make_fs_content =
                                |fs: Filesystem, encrypt: bool, luks_name: String| {
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

                            // TODO: improve pattern match with custom names and write it simpler
                            for (name, part) in partitions.iter() {
                                let part_key =
                                    name.rsplit('/').next().unwrap_or(name.as_str()).to_string();
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
                                            // if luks on swap also take it
                                            let content = if any_encrypted {
                                                PartitionContent::Luks(Luks {
                                                    name: format!("crypted-{}", part_key),
                                                    password_file: Some(LUKS_PASSWORD_FILE.into()),
                                                    settings: luks_settings.clone(),
                                                    content: Some(Box::new(DeviceContent::Swap(
                                                        swap,
                                                    ))),
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
                                        Some(get_storage_size_for_disko(Size::from_bytes(
                                            part.size,
                                        )))
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

                            devices = Devices {
                                disk: disk_disko,
                                ..Default::default()
                            }
                        }
                    };
                });

                self.diskoconfig = devices;
                self.partitionconfig = partition;
            }
            AppMsg::SetUserConfig(user) => {
                self.userconfig = user;
            }
            AppMsg::SetListConfig(title, list) => {
                info!("SetListConfig: {} {:?}", title, list);
                self.listconfig.insert(title, list);
                info!("ListConfig: {:?}", self.listconfig);
            }
            AppMsg::Install => {
                debug!("Installing!");
                if let Some(config) = &self.installconfig {
                    self.installworker.emit(InstallAsyncMsg::Install(
                        config.config_id.to_string(),
                        self.languageconfig.clone(),
                        self.timezoneconfig.clone(),
                        self.keyboardconfig.clone(),
                        Box::new(self.partitionconfig.clone()),
                        Box::new(self.userconfig.clone()),
                        self.listconfig.clone(),
                        config.config_type.clone(),
                        config.imperative_timezone.clone(),
                        self.diskoconfig.clone(),
                    ));
                }
            }
            AppMsg::FinishInstall => {
                debug!("Finishing install!");
                if let Some(config) = &self.installconfig {
                    self.installworker.emit(InstallAsyncMsg::FinishInstall(
                        self.timezoneconfig.clone(),
                        config.imperative_timezone.clone(),
                        config.commands.clone(),
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
    fn shutdown(&mut self, widgets: &mut Self::Widgets, output: Sender<Self::Output>) {}

    fn update_cmd(
        &mut self,
        msg: Self::CommandOutput,
        _sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match msg {
            AppAsyncMsg::SetPage(page) => {
                self.page = page;
            }
        }
    }
}
