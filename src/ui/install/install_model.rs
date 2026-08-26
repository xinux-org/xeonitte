use crate::{
    config::SYSCONFDIR,
    ui::window::AppMsg,
    utils::{parse::parse_branding, report::ErrorPhase},
};
use anyhow::Context;
use gtk::gio;
use log::{debug, error};
use relm4::{factory::*, *};
use std::{fs::File, process::Command};
use vte::{self, TerminalExt, TerminalExtManual};

#[derive(Debug, Default, Clone, PartialEq)]
pub struct InstallModel {
    terminal: vte::Terminal,
    progressbar_title: String,
    progressbar: gtk::ProgressBar,
    showterminal: bool,
    installing: bool,
    slides: Option<FactoryVecDeque<InstallSlide>>,
    locale: Option<String>,
}

#[derive(Debug, Clone)]
pub enum InstallMsg {
    Pulse,
    NextSlide,
    ToggleTerminal,
    Echo(String),
    Install(Vec<String>),
    VTEOutput(i32),
    SetLocale(Option<String>),
    PostInstall(Vec<String>),
    PreInstall(Vec<String>),
    ProgressbarTitle(String),
}

pub static INSTALL_BROKER: MessageBroker<InstallMsg> = MessageBroker::new();

#[relm4::component(pub)]
impl SimpleComponent for InstallModel {
    type Init = String;
    type Input = InstallMsg;
    type Output = AppMsg;

    view! {
        gtk::ScrolledWindow {
            gtk::Box {
                set_hexpand: true,
                set_vexpand: true,
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 20,
                set_margin_all: 20,
                if model.showterminal {
                    gtk::Box {
                        gtk::Frame {
                            #[local_ref]
                            terminal -> vte::Terminal {
                                set_hexpand: true,
                                connect_child_exited[sender] => move |_term, status| {
                                    sender.input(InstallMsg::VTEOutput(status));
                                },
                                set_input_enabled: false,
                            }
                        }
                    }
                } else {
                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_spacing: 5,
                        #[local_ref]
                        carousel -> adw::Carousel {
                            set_vexpand: true,
                            set_halign: gtk::Align::Fill,
                            set_valign: gtk::Align::Fill,
                        },
                        adw::CarouselIndicatorDots {
                            set_halign: gtk::Align::Center,
                            set_hexpand: true,
                            set_carousel: Some(carousel)
                        }
                    }

                },
                gtk::Label {
                    #[watch]
                    set_label: &model.progressbar_title,
                    set_halign: gtk::Align::Start,
                },
                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 20,
                    #[local_ref]
                    progressbar -> gtk::ProgressBar {
                        set_hexpand: true,
                        set_valign: gtk::Align::Center,
                        set_halign: gtk::Align::Fill,
                        set_pulse_step: 0.02,
                    },
                    gtk::Button {
                        set_valign: gtk::Align::Center,
                        add_css_class: "circular",
                        set_icon_name: "utilities-terminal-symbolic",
                        connect_clicked[sender] => move |_| {
                            sender.input(InstallMsg::ToggleTerminal);
                        }
                    }
                }
            }
        }
    }

    fn init(
        branding: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let mut model = InstallModel {
            terminal: vte::Terminal::new(),
            showterminal: false,
            progressbar_title: String::new(),
            progressbar: gtk::ProgressBar::new(),
            installing: false,
            slides: FactoryVecDeque::builder().launch_default().detach().into(),
            locale: None,
        };

        if let Ok(brandingconfig) = parse_branding(&branding) {
            let mut slides_guard = model.slides.as_mut().unwrap().guard();
            for slide in brandingconfig.slides {
                slides_guard.push_back(InstallSlide {
                    title: slide.title,
                    subtitle: slide.subtitle,
                    image: format!(
                        "{}/xeonitte/branding/{}/{}",
                        SYSCONFDIR, branding, slide.image
                    ),
                    locale: model.locale.clone(),
                    tracker: 0,
                });
            }
            slides_guard.drop();
        }

        let terminal = &model.terminal;
        let progressbar = &model.progressbar;
        let carousel = model.slides.as_ref().unwrap().widget();
        let widgets = view_output!();
        let pulsesender = sender.clone();
        relm4::spawn(async move {
            loop {
                pulsesender.input(InstallMsg::Pulse);
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            }
        });
        relm4::spawn(async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(12)).await;
                sender.input(InstallMsg::NextSlide);
            }
        });
        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>) {
        match msg {
            InstallMsg::Pulse => {
                self.progressbar.pulse();
            }
            InstallMsg::NextSlide => {
                let slides_widget = self.slides.as_ref().unwrap().widget();
                let npages = slides_widget.n_pages();
                let currentpage = slides_widget.position();
                if currentpage.fract() == 0.0 {
                    let next = (slides_widget.position() as u32 + 1) % npages;
                    let next = slides_widget.nth_page(next);
                    slides_widget.scroll_to(&next, true);
                }
            }
            InstallMsg::ToggleTerminal => {
                self.showterminal = !self.showterminal;
            }
            InstallMsg::Echo(s) => {
                self.terminal.spawn_async(
                    vte::PtyFlags::DEFAULT,
                    Some("/"),
                    &["/usr/bin/env", "echo", &s],
                    &[],
                    adw::glib::SpawnFlags::DEFAULT,
                    || (),
                    -1,
                    gio::Cancellable::NONE,
                    |_| (),
                );
            }
            InstallMsg::Install(cmds) => {
                debug!("Installing: {:?}", cmds);
                self.installing = true;
                let cmds: Vec<&str> = cmds.iter().map(|x| &**x).collect();
                self.terminal.spawn_async(
                    vte::PtyFlags::DEFAULT,
                    Some("/"),
                    &cmds,
                    &[],
                    adw::glib::SpawnFlags::DEFAULT,
                    || (),
                    -1,
                    gio::Cancellable::NONE,
                    |err| debug!("VTE Install: {:?}", err),
                );
            }

            // not implemented yet!
            InstallMsg::PreInstall(cmds) => {
                debug!("PreInstall command: {:?}", cmds);
                self.installing = false;
                let cmds: Vec<&str> = cmds.iter().map(|x| &**x).collect();
                self.terminal.spawn_async(
                    vte::PtyFlags::DEFAULT,
                    Some("/"),
                    &cmds,
                    &[],
                    adw::glib::SpawnFlags::DEFAULT,
                    || (),
                    -1,
                    gio::Cancellable::NONE,
                    |err| {
                        debug!("VTE preinstall: {:?}", err);
                    },
                );
            }
            InstallMsg::PostInstall(cmds) => {
                debug!("PostInstall command: {:?}", cmds);
                self.installing = false;
                let cmds: Vec<&str> = cmds.iter().map(|x| &**x).collect();
                self.terminal.spawn_async(
                    vte::PtyFlags::DEFAULT,
                    Some("/"),
                    &cmds,
                    &[],
                    adw::glib::SpawnFlags::DEFAULT,
                    || (),
                    -1,
                    gio::Cancellable::NONE,
                    |err| debug!("VTE postinstall: {:?}", err),
                );
            }
            InstallMsg::VTEOutput(status) => {
                debug!("VTE command exited with status: {}", status);

                let Ok(_) = Command::new("touch")
                    .arg("/tmp/xeonitte-term.log")
                    .output()
                    .context("Cannot create /tmp/xeonitte-term.log")
                else {
                    sender.output(AppMsg::error(
                        ErrorPhase::Installation,
                        "Failed to create /tmp/xeonitte-term.log",
                    ));
                    return;
                };

                if let Ok(file) = File::create("/tmp/xeonitte-term.log") {
                    let output = gio::WriteOutputStream::new(file);
                    if let Err(e) = self.terminal.write_contents_sync(
                        &output,
                        vte::WriteFlags::Default,
                        gio::Cancellable::NONE,
                    ) {
                        error!("InstallMsg::VTEOutput_+| {:?}", e);
                    }
                    let _ = output.flush(gio::Cancellable::NONE);
                }
                if self.installing {
                    if status == 0 {
                        debug!("Installation Success!");
                        let _ = sender.output(AppMsg::FinishInstall);
                    } else {
                        debug!("Installation Failed!");
                        sender.output(AppMsg::error(
                            ErrorPhase::Installation,
                            format!("Installation failed (exit status {status})"),
                        ));
                    }
                } else {
                    if status == 0 {
                        debug!("Post install command success!");
                        sender.output(AppMsg::RunNextCommand);
                    } else {
                        debug!("Post install command failed!");
                        sender.output(AppMsg::error(
                            ErrorPhase::PostInstall,
                            format!("Post-install command failed (exit status {status})"),
                        ));
                    }
                }
            }
            InstallMsg::SetLocale(locale) => {
                self.locale = locale;
                let mut slides_guard = self.slides.as_mut().unwrap().guard();
                for item in slides_guard.iter_mut() {
                    item.set_locale(self.locale.clone());
                }
                slides_guard.drop();
            }
            InstallMsg::ProgressbarTitle(title) => self.progressbar_title = title,
        }
    }
}

use adw::prelude::*;
use gettextrs::gettext;

#[tracker::track]
#[derive(Debug, Default)]
pub struct InstallSlide {
    pub title: String,
    pub subtitle: String,
    pub image: String,
    pub locale: Option<String>,
}

impl CloneableFactoryComponent for InstallSlide {
    fn get_init(&self) -> Self::Init {
        Self::default()
    }
}

#[relm4::factory(pub)]
impl FactoryComponent for InstallSlide {
    type Init = InstallSlide;
    type Input = ();
    type Output = ();
    type ParentWidget = adw::Carousel;
    type CommandOutput = ();

    view! {
        gtk::Box {
            set_hexpand: true,
            set_vexpand: true,
            set_orientation: gtk::Orientation::Vertical,
            set_spacing: 10,
            set_margin_all: 15,
            gtk::Label {
                set_halign: gtk::Align::Center,
                add_css_class: "title-3",
                #[track(self.changed(InstallSlide::locale()))]
                set_label: &gettext(&self.title)
            },
            gtk::Label {
                set_halign: gtk::Align::Center,
                #[track(self.changed(InstallSlide::locale()))]
                set_label: &gettext(&self.subtitle)
            },
            gtk::Picture {
                set_margin_all: 50,
                set_halign: gtk::Align::Center,
                set_valign: gtk::Align::Center,
                set_filename: Some(&self.image)
            }
        }
    }
    fn init_model(init: Self::Init, _index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        init
    }
    fn update(&mut self, _message: Self::Input, _sender: FactorySender<Self>) {
        self.reset();
    }
}
