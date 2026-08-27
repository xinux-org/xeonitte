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
#[derive(Default, Debug, Clone, PartialEq)]
pub struct InstallConfigData {
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
#[derive(Default, Debug, Clone, PartialEq)]
pub struct Carousel {
    pub page: StackPage,
    pub is_transitioning: bool,
    #[tracker::no_eq]
    pub carousel: adw::Carousel,
    #[tracker::no_eq]
    pub carouselpages: Vec<Step>,
    pub current_page: u32,
    #[tracker::no_eq]
    pub install_flow: Option<InstallFlow>,
}

#[tracker::track]
#[derive(Default, Debug, Clone, PartialEq)]
pub struct Pages {
    #[tracker::no_eq]
    welcome: Option<Controller<WelcomeModel>>,
    #[tracker::no_eq]
    keyboard: Option<Controller<KeyboardModel>>,
    #[tracker::no_eq]
    timezone: Option<Controller<TimeZoneModel>>,
    #[tracker::no_eq]
    install_mode: Option<Controller<InstallModeModel>>,
    #[tracker::no_eq]
    partition: Option<Controller<PartitionModel>>,
    #[tracker::no_eq]
    user: Option<Controller<UserModel>>,
    #[tracker::no_eq]
    summary: Option<Controller<SummaryModel>>,
    #[tracker::no_eq]
    install: Option<Controller<InstallModel>>,
    #[tracker::no_eq]
    package_managers: Option<Controller<ListModel>>,
    #[tracker::no_eq]
    kernel_selection: Option<Controller<ListModel>>,
    #[tracker::no_eq]
    error: Option<Controller<ErrorModel>>,
    #[tracker::no_eq]
    quitdialog: Option<Controller<QuitDialogModel>>,
}

#[tracker::track]
#[derive(Default, Debug, Clone, Patch, PartialEq)]
#[patch(attribute(derive(Debug, Default, Clone)))]
pub struct Data {
    install_config_data: InstallConfigData,
    carousel: Carousel,
    pages: Pages,
}

#[tracker::track]
pub struct AppModel {
    #[tracker::no_eq]
    installworker: WorkerController<InstallAsyncModel>,
    data: Data,
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
    RequestPrev,
    RequestNext,
    Install,
    FinishInstall,
    RunNextCommand,
    Error(ErrorPhase, String),
    UpdateData(DataPatch),
    SetStackPage(StackPage),
    SelectFlow(InstallFlow),
    PageChanged(u32),

    SetLanguageConfig(Option<String>),
    SetPartitionConfig(Option<PartitionSchema>),
    SetListConfig(String, HashMap<String, Choice>),
    SetKeyboardConfig(Option<String>),
    SetTimezoneConfig(Option<String>),
    SetUserConfig(Option<UserConfig>),
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
                if model.data.carousel.page == StackPage::Install {
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
                    set_title_widget = match model.data.carousel.page {
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
                match model.data.carousel.page {
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
                                set_reveal_child: model.data.carousel.current_page > 0 && !model.data.carousel.is_transitioning,
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
                                set_reveal_child: model.can_advance() && !model.data.carousel.is_transitioning,
                                set_halign: gtk::Align::End,
                                set_valign: gtk::Align::Center,
                                set_margin_all: 20,
                                gtk::Button {
                                    set_can_focus: false,
                                    set_can_target: true,
                                    set_height_request: 40,
                                    set_width_request: 40,
                                    #[watch]
                                    set_css_classes: if model.data.carousel.current_page.eq(&main_carousel.n_pages().checked_sub(1).unwrap_or_default()) { &["circular", "suggested-action"] } else { &["circular"] },
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
            // page: startpage,
            // install_flow: None,
            // carousel,
            // carouselpages: Vec::new(),
            // current_page: 0,
            installworker,
            tracker: 0,
            data: Data {
                install_config_data: InstallConfigData::default(),
                carousel: Carousel {
                    page: startpage,
                    is_transitioning: false,
                    carousel,
                    ..Default::default()
                },
                pages: Pages {
                    welcome: welcome.into(),
                    keyboard: keyboard.into(),
                    timezone: timezone.into(),
                    install_mode: install_mode.into(),
                    partition: partition.into(),
                    user: user.into(),
                    summary: summary.into(),
                    install: install.into(),
                    package_managers: package_managers.into(),
                    kernel_selection: kernel_selection.into(),
                    error: error.into(),
                    quitdialog: quitdialog.into(),
                    ..Default::default()
                },
                ..Default::default()
            },
        };

        let main_carousel = &model.data.carousel.carousel;
        let installpage = model.data.pages.install.as_ref().unwrap().widget().clone();
        let errorpage = model.data.pages.error.as_ref().unwrap().widget().clone();
        let widgets = view_output!();

        sender.input(AppMsg::InitPages);

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>, _root: &Self::Root) {
        self.reset();
        match msg {
            AppMsg::SetKeyboardConfig(keyboard) => {
                self.data.install_config_data.set_keyboardconfig(keyboard);
            }
            AppMsg::SetTimezoneConfig(timezone) => {
                self.data.install_config_data.set_timezoneconfig(timezone);
            }
            AppMsg::SetUserConfig(user) => {
                self.data.install_config_data.set_userconfig(user);
            }
            AppMsg::QuitDialog => {
                self.data
                    .pages
                    .quitdialog
                    .as_ref()
                    .unwrap()
                    .widget()
                    .present(relm4::main_application().active_window().as_ref());
            }

            AppMsg::InitPages => {
                let new_carousel = adw::Carousel::new();
                let mut new_carousel_pages = Vec::new();
                for step in init_steps() {
                    match &step {
                        Step::Welcome => {
                            new_carousel.append(self.data.pages.welcome.as_ref().unwrap().widget())
                        }
                        Step::Keyboard => {
                            new_carousel.append(self.data.pages.keyboard.as_ref().unwrap().widget())
                        }
                        Step::Location => {
                            new_carousel.append(self.data.pages.timezone.as_ref().unwrap().widget())
                        }
                        Step::InstallMode => new_carousel
                            .append(self.data.pages.install_mode.as_ref().unwrap().widget()),
                        _ => {}
                    }
                    new_carousel_pages.push(step);
                }
                sender.input(AppMsg::UpdateData(DataPatch {
                    carousel: Carousel {
                        carousel: new_carousel,
                        carouselpages: new_carousel_pages,
                        ..Default::default()
                    }
                    .into(),
                    ..Default::default()
                }));

                // self.data.carousel.apply(CarouselPatch {
                //     page: (),
                //     is_transitioning: (),
                //     carousel: (),
                //     carouselpages: (),
                //     current_page: (),
                //     install_flow: (),
                //     tracker: (),
                // });
            }

            AppMsg::UpdateData(patch) => {
                self.data.apply(patch);
            }

            AppMsg::RequestNext => {
                let carousel = self.data.carousel.clone();
                trace!("AppMsg::RequestNext (page {})", carousel.current_page);
                if carousel.is_transitioning {
                    return;
                }
                if !self.can_advance() {
                    return;
                }
                let next = carousel.current_page + 1;
                if next >= carousel.carousel.n_pages() {
                    sender.input(AppMsg::SetStackPage(StackPage::Install));
                    sender.input(AppMsg::Install);
                } else {
                    sender.input(AppMsg::UpdateData(DataPatch {
                        carousel: Carousel {
                            is_transitioning: true,
                            ..Default::default()
                        }
                        .into(),
                        ..Default::default()
                    }));
                    let w = carousel.carousel.nth_page(next);
                    self.data.carousel.carousel.scroll_to(&w, true);
                }
            }

            AppMsg::RequestPrev => {
                let Carousel {
                    carousel,
                    current_page,
                    is_transitioning,
                    ..
                } = self.data.carousel.clone();
                trace!("AppMsg::RequestPrev (page {})", current_page);
                if is_transitioning || current_page == 0 {
                    return;
                }
                self.data.carousel.set_is_transitioning(true);
                let w = carousel.nth_page(current_page - 1);
                carousel.scroll_to(&w, true);
            }

            AppMsg::PageChanged(idx) => {
                trace!("AppMsg::PageChanged: {}", idx);
                sender.input(AppMsg::UpdateData(DataPatch {
                    carousel: Carousel {
                        is_transitioning: false,
                        current_page: idx,
                        ..Default::default()
                    }
                    .into(),
                    ..Default::default()
                }));

                let carousel = self.data.carousel.clone();
                let pages = self.data.pages.clone();
                let install = self.data.install_config_data.clone();
                if let Some(Step::Summary) = carousel.carouselpages.get(idx as usize) {
                    pages.summary.unwrap().emit(SummaryMsg::SetConfig(
                        install.languageconfig.clone(),
                        install.keyboardconfig.clone(),
                        install.timezoneconfig.clone(),
                        install.partitionconfig.clone(),
                        Box::new(install.userconfig.clone()),
                    ));
                }
            }
            AppMsg::SetStackPage(page) => {
                debug!("StackPage: {:?}", page);
                if page.ne(&self.data.carousel.page) {
                    sender.input(AppMsg::UpdateData(DataPatch {
                        carousel: Carousel {
                            page,
                            ..Default::default()
                        }
                        .into(),
                        ..Default::default()
                    }));
                }
            }

            AppMsg::SelectFlow(flow) => {
                debug!("SelectFlow: {:?}", flow);

                // Set state to zero
                // self.userconfig = None;
                // self.partitionconfig = None;
                // self.listconfig = HashMap::new();

                let new_carousel = adw::Carousel::new();
                let mut new_carousel_pages = Vec::new();
                let mut new_config = InstallConfigData::default();
                let pages = self.data.pages.clone();
                for step in flow.steps() {
                    use Step::*;
                    match &step {
                        Welcome => {
                            new_carousel.append(pages.welcome.clone().unwrap().widget());
                        }
                        Keyboard => {
                            new_carousel.append(pages.keyboard.clone().unwrap().widget());
                        }
                        Location => {
                            new_carousel.append(pages.timezone.clone().unwrap().widget());
                        }
                        InstallMode => {
                            new_carousel.append(pages.install_mode.clone().unwrap().widget());
                        }
                        User { root, hostname } => {
                            new_carousel.append(pages.user.clone().unwrap().widget());
                            pages
                                .user
                                .clone()
                                .unwrap()
                                .emit(UserMsg::SetConfig(*root, *hostname));
                            pages
                                .summary
                                .clone()
                                .unwrap()
                                .emit(SummaryMsg::ShowHostname(*hostname));
                        }
                        PackageManagers => {
                            new_carousel.append(pages.package_managers.clone().unwrap().widget());
                            new_config
                                .listconfig
                                .insert("PACKAGEMANAGERS".to_string(), HashMap::new());
                        }
                        KernelSelection => {
                            new_carousel.append(pages.kernel_selection.clone().unwrap().widget());
                            new_config
                                .listconfig
                                .insert("KERNEL".to_string(), HashMap::new());
                        }
                        Partitioning => {
                            new_carousel.append(pages.partition.clone().unwrap().widget());
                        }
                        Summary => {
                            new_carousel.append(pages.summary.clone().unwrap().widget());
                        }
                        _ => {}
                    }
                    new_carousel_pages.push(step);
                }
                sender.input(AppMsg::UpdateData(DataPatch {
                    install_config_data: new_config.into(),
                    carousel: Carousel {
                        carousel: new_carousel,
                        carouselpages: new_carousel_pages,
                        ..Default::default()
                    }
                    .into(),
                    ..Default::default()
                }));
                // (new_carousel, new_config)

                // self.install_flow = Some(flow);
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
                let pages = self.data.pages.clone();
                let install = self.data.install_config_data.clone();
                self.data.install_config_data.set_languageconfig(language);

                pages
                    .package_managers
                    .unwrap()
                    .emit(ListMsg::SetLocale(install.languageconfig.clone()));
                pages
                    .kernel_selection
                    .unwrap()
                    .emit(ListMsg::SetLocale(install.languageconfig.clone()));
                pages
                    .install
                    .unwrap()
                    .emit(InstallMsg::SetLocale(install.languageconfig.clone()));

                if let Some(language) = &install.languageconfig
                    && let (Ok(lang), Ok(country)) = (
                        get_lang(language.to_string()),
                        get_country(language.to_string()),
                    )
                {
                    pages
                        .keyboard
                        .unwrap()
                        .emit(KeyboardMsg::SetCountry(lang, country));
                }
            }

            AppMsg::SetPartitionConfig(partition) => {
                let devices = Devices { disk: Attrs::new() };
                self.set_partition_config(&partition, devices);
                sender.input(AppMsg::UpdateData(DataPatch {
                    install_config_data: InstallConfigData {
                        partitionconfig: partition,
                        ..Default::default()
                    }
                    .into(),
                    ..Default::default()
                }));
            }

            AppMsg::SetListConfig(id, list) => {
                info!("SetListConfig: {} {:?}", id, list);
                self.data.install_config_data.listconfig.insert(id, list);
                // sender.input(AppMsg::UpdateData(DataPatch {
                //     install_config_data: InstallConfigData {
                //         listconfig: self.data.install_config_data.listconfig.insert(id, list)
                //             ..Default::default(),
                //     }
                //     .into(),
                //     ..Default::default()
                // }));
            }

            AppMsg::Install => {
                debug!("Installing!");
                let install = self.data.install_config_data.clone();
                if let Some(flow) = &self.data.carousel.install_flow {
                    self.installworker.emit(InstallAsyncMsg::Install(
                        flow.config_id().to_string(),
                        install.languageconfig.clone(),
                        install.timezoneconfig.clone(),
                        install.keyboardconfig.clone(),
                        Box::new(install.partitionconfig.clone()),
                        Box::new(install.userconfig.clone()),
                        install.listconfig.clone(),
                        flow.config_type(),
                        flow.imperative_timezone(),
                        install.diskoconfig.clone(),
                    ));
                }
            }

            AppMsg::FinishInstall => {
                debug!("Finishing install!");
                if let Some(flow) = &self.data.carousel.install_flow {
                    self.installworker.emit(InstallAsyncMsg::FinishInstall(
                        self.data.install_config_data.timezoneconfig.clone(),
                        flow.imperative_timezone(),
                        vec![],
                    ));
                }
            }

            AppMsg::RunNextCommand => {
                debug!("Running next postinstall command");
                self.installworker.emit(InstallAsyncMsg::RunNextCommand);
            }

            // AppMsg::Finished => {
            //     debug!("Finished!");
            //     self.page = StackPage::Finished;
            // }
            AppMsg::Error(phase, message) => {
                error!("Error in {phase} phase: {message}");
                sender.input(AppMsg::UpdateData(DataPatch {
                    carousel: Carousel {
                        page: StackPage::Error,
                        ..Default::default()
                    }
                    .into(),
                    ..Default::default()
                }));
                self.data
                    .pages
                    .error
                    .clone()
                    .unwrap()
                    .emit(ErrorMsg::Show(phase, message));
            }
        }
    }

    fn shutdown(&mut self, _widgets: &mut Self::Widgets, _output: Sender<Self::Output>) {}

    fn update_cmd(
        &mut self,
        msg: Self::CommandOutput,
        sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match msg {
            AppAsyncMsg::SetPage(page) => sender.input(AppMsg::UpdateData(DataPatch {
                carousel: Carousel {
                    page,
                    ..Default::default()
                }
                .into(),
                ..Default::default()
            })),
        }
    }
}

impl AppModel {
    pub fn can_advance(&self) -> bool {
        let Carousel {
            carouselpages,
            current_page,
            install_flow,
            ..
        } = self.data.carousel.clone();
        let InstallConfigData {
            languageconfig,
            keyboardconfig,
            timezoneconfig,
            partitionconfig,
            userconfig,
            ..
        } = self.data.install_config_data.clone();
        match carouselpages.get(current_page as usize) {
            Some(Step::Welcome) => languageconfig.is_some(),
            Some(Step::Keyboard) => keyboardconfig.is_some(),
            Some(Step::Location) => timezoneconfig.is_some(),
            Some(Step::InstallMode) => install_flow.is_some(),
            Some(Step::User { .. }) => userconfig.is_some(),
            Some(Step::PackageManagers) => true,
            Some(Step::KernelSelection) => true,
            Some(Step::Partitioning) => partitionconfig.is_some(),
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
                    self.data
                        .install_config_data
                        .set_diskoconfig(devices.clone());
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
                    self.data.install_config_data.set_diskoconfig(devices);
                }
            };
        }
    }
}
