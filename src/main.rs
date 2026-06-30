use adw::gio;
use anyhow::{Context, Result};
use gettextrs::{LocaleCategory, gettext};
use gtk::{glib, prelude::ApplicationExt};
use log::{error, info};
use relm4::*;
use simplelog::*;
use std::fs::File;
use xeonitte::{
    config::{GETTEXT_PACKAGE, LOCALEDIR, RESOURCES_FILE},
    ui::window::AppModel,
};

fn main() -> Result<()> {
    CombinedLogger::init(vec![
        TermLogger::new(
            LevelFilter::Warn,
            Config::default(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ),
        WriteLogger::new(
            LevelFilter::Debug,
            Config::default(),
            File::create("/tmp/xeonitte.log").context("Can't open log file: /tmp/xeonitte.log")?,
        ),
    ])
    .context("Failed to initialize loggers")?;
    gtk::init().context("Failed to initialize GTK")?;
    setup_gettext().context("Failed to setup gettext")?;
    glib::set_application_name(&gettext("Xeonitte Installer"));
    if let Ok(res) = gio::Resource::load(RESOURCES_FILE) {
        info!("Resource loaded: {}", RESOURCES_FILE);
        gio::resources_register(&res);
    } else {
        error!("Failed to load resources");
    }
    gtk::Window::set_default_icon_name(xeonitte::config::APP_ID);
    let app = adw::Application::new(
        Some(xeonitte::config::APP_ID),
        gio::ApplicationFlags::empty(),
    );
    app.set_resource_base_path(Some("/org/xinux/Xeonitte"));
    let app = RelmApp::from_app(app);
    app.run::<AppModel>(());

    Ok(())
}

fn setup_gettext() -> Result<()> {
    // Prepare i18n
    gettextrs::setlocale(LocaleCategory::LcAll, "");
    gettextrs::bindtextdomain(GETTEXT_PACKAGE, LOCALEDIR)
        .context("Unable to bind the text domain")?;
    gettextrs::bind_textdomain_codeset(GETTEXT_PACKAGE, "UTF-8")
        .context("Unable to bind the text domain codeset to UTF-8")?;
    gettextrs::textdomain(GETTEXT_PACKAGE).context("Unable to switch to the text domain")?;
    Ok(())
}
