use anyhow::Context;
use gettextrs::gettext;
use log::{error, info};
use relm4::{
    RelmApp, adw,
    gtk::{self, gio, glib, prelude::ApplicationExt},
};
use xeonitte::{config::RESOURCES_FILE, setup_gettext, setup_logger, ui::window::AppModel};

mod icon_names {
    pub use shipped::*; // Include all shipped icons by default
    include!(concat!(env!("OUT_DIR"), "/icon_names.rs"));
}

fn main() -> anyhow::Result<()> {
    setup_logger()?;

    relm4_icons::initialize_icons(icon_names::GRESOURCE_BYTES, icon_names::RESOURCE_PREFIX);

    gtk::init().context("Failed to initialize GTK")?;
    xeonitte::utils::report::init();
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
    app.set_resource_base_path(Some("/uz/xinux/Xeonitte"));
    let app = RelmApp::from_app(app);
    app.run::<AppModel>(());

    Ok(())
}
