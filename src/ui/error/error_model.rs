use crate::{
    config::LIBEXECDIR,
    ui::window::AppMsg,
    utils::report::{ErrorPhase, send_report},
};
use adw::prelude::*;
use gettextrs::gettext;
use log::error;
use relm4::*;
use std::process::Command;

pub struct ErrorModel {
    messegebuffer: gtk::TextBuffer,
    uploadbutton: UploadButton,
    url: String,
    spinner: gtk::Spinner,
    phase: ErrorPhase,
    message: String,
}

#[derive(Debug)]
pub enum UploadButton {
    Button,
    Loading,
    Url,
    Success,
    Fail,
}

#[derive(Debug)]
pub enum ErrorMsg {
    Show(ErrorPhase, String),
    UploadReport,
    SetUrl(String),
    SetUploadButton(UploadButton),
}

#[relm4::component(pub)]
impl SimpleComponent for ErrorModel {
    type Init = ();
    type Input = ErrorMsg;
    type Output = AppMsg;

    view! {
        gtk::ScrolledWindow {
            adw::Clamp {
                gtk::Box {
                    set_hexpand: true,
                    set_vexpand: true,
                    set_valign: gtk::Align::Center,
                    set_orientation: gtk::Orientation::Vertical,
                    set_spacing: 11,
                    set_margin_all: 11,
                    gtk::Label {
                        add_css_class: "title-2",
                        #[watch]
                        set_label: &gettext("Installation Failed!"),
                    },
                    gtk::Image {
                        add_css_class: "error",
                        set_icon_name: Some("process-stop-symbolic"),
                        set_pixel_size: 48,
                    },
                    gtk::Frame {
                        gtk::ScrolledWindow {
                            set_height_request: 800,
                            set_width_request: 1000,
                            gtk::TextView {
                                set_editable: false,
                                set_hexpand: true,
                                set_vexpand: true,
                                set_buffer: Some(&model.messegebuffer),
                                set_top_margin: 5,
                                set_bottom_margin: 5,
                                set_left_margin: 5,
                                set_right_margin: 5,
                                set_monospace: true,
                            }
                        }
                    },
                    match &model.uploadbutton {
                        UploadButton::Button => {
                            gtk::Button {
                                #[iterate]
                                add_css_class: ["pill", "suggested-action"],
                                set_halign: gtk::Align::Center,
                                set_valign: gtk::Align::Center,
                                #[watch]
                                set_label: &gettext("Upload Report"),
                                connect_clicked[sender] => move |_| {
                                    sender.input(ErrorMsg::UploadReport);
                                }
                            }
                        },
                        UploadButton::Fail => {
                            gtk::Box{
                                set_orientation: gtk::Orientation::Vertical,
                                set_halign: gtk::Align::Center,
                                set_spacing: 4,
                                gtk::Box {
                                    gtk::Label {
                                        set_text:  &gettext("Failed to report"),
                                        #[iterate]
                                        add_css_class: ["title-4", "error"],
                                    },
                                    gtk::Button{
                                        set_icon_name: "error-outline-symbolic",
                                        set_can_target: false,
                                        #[iterate]
                                        add_css_class: ["flat", "error"],
                                    },
                                },
                                gtk::Button {
                                    add_css_class: "pill",
                                    set_halign: gtk::Align::Center,
                                    #[watch]
                                    set_label: &gettext("Retry"),
                                    connect_clicked[sender] => move |_| {
                                        sender.input(ErrorMsg::UploadReport);
                                    }
                                }
                            }
                        },
                        UploadButton::Success => {
                            gtk::Box{
                                set_halign: gtk::Align::Center,
                                gtk::Label {
                                    set_text:  &gettext("Reported"),
                                    #[iterate]
                                    add_css_class: ["title-4", "success"],
                                },
                                gtk::Button{
                                    set_icon_name: "check-round-outline2-symbolic",
                                    set_can_target: false,
                                    #[iterate]
                                    add_css_class: ["flat", "success"],
                                }
                            }
                        },
                        UploadButton::Loading => {
                            gtk::Box{
                                set_orientation: gtk::Orientation::Vertical,
                                set_spacing: 6,
                                gtk::Label{
                                    set_text: &gettext("Processing..."),
                                    add_css_class: "heading",
                                },
                                #[local]
                                spinner -> gtk::Spinner {
                                    set_spinning: true,
                                    set_halign: gtk::Align::Center,
                                    set_size_request: (48, 48),
                                },
                            }
                        },
                        UploadButton::Url => {
                            gtk::LinkButton {
                                set_css_classes: &["pill", "suggested-action"],
                                #[watch]
                                set_label: &gettext("Open Report"),
                                #[watch]
                                set_uri: &model.url,
                                set_halign: gtk::Align::Center,
                            }
                        }
                    }
                }
            }
        }
    }

    fn init(
        _parent_window: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = ErrorModel {
            uploadbutton: UploadButton::Button,
            url: String::new(),
            messegebuffer: gtk::TextBuffer::new(None),
            spinner: gtk::Spinner::new(),
            phase: ErrorPhase::Installation,
            message: String::new(),
        };
        let spinner = model.spinner.clone();
        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>) {
        match msg {
            ErrorMsg::Show(phase, message) => {
                self.phase = phase;
                self.message = message;
                if let Err(e) = Command::new("pkexec")
                    .arg(format!("{}/xeonitte-helper", LIBEXECDIR))
                    .arg("unmount")
                    .output()
                {
                    error!("Failed to unmount partitions: {}", e);
                }

                let mut outlog = "=== Xeonitte Log ===\n".to_string();
                if let Ok(xeonittelog) = std::fs::read_to_string("/tmp/xeonitte.log") {
                    outlog.push_str(xeonittelog.trim());
                } else {
                    outlog.push_str("No log found!");
                }
                outlog.push_str("\n=== End of Xeonitte Log ===\n\n");
                outlog.push_str("=== nixos-install Log ===\n");
                if let Ok(nixoslog) = std::fs::read_to_string("/tmp/xeonitte-term.log") {
                    outlog.push_str(nixoslog.trim());
                } else {
                    outlog.push_str("No log found!");
                }
                outlog.push_str("\n=== End of nixos-install Log ===\n\n");
                self.messegebuffer.set_text(&outlog);
            }
            ErrorMsg::UploadReport => {
                let phase = self.phase;
                let message = self.message.clone();
                self.uploadbutton = UploadButton::Loading;
                self.spinner.set_spinning(false);
                self.spinner.activate();
                self.spinner.set_spinning(true);

                let command_sender = sender.clone();
                sender.command(move |_, receiver| {
                    receiver
                        .register(async move {
                            let result = tokio::task::spawn_blocking(move || {
                                send_report(
                                    phase,
                                    &message,
                                    &["/tmp/xeonitte.log", "/tmp/xeonitte-term.log"],
                                )
                            })
                            .await;
                            let rep_file = match result {
                                Ok(r) => r,
                                Err(e) => {
                                    error!("Failed to generate report: {e}");
                                    command_sender
                                        .input(ErrorMsg::SetUploadButton(UploadButton::Fail));
                                    return;
                                }
                            };
                            match rep_file {
                                Ok(path) => {
                                    command_sender
                                        .input(ErrorMsg::SetUrl(format!("file://{path}")));
                                    command_sender
                                        .input(ErrorMsg::SetUploadButton(UploadButton::Success));
                                }
                                Err(e) => {
                                    error!("Failed to upload report: {e}");
                                    command_sender
                                        .input(ErrorMsg::SetUploadButton(UploadButton::Fail));
                                }
                            }
                        })
                        .drop_on_shutdown()
                });
            }
            ErrorMsg::SetUrl(url) => {
                self.url = url;
                self.uploadbutton = UploadButton::Url;
            }
            ErrorMsg::SetUploadButton(button) => {
                self.uploadbutton = button;
            }
        }
    }
}
