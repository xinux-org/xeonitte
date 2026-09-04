fn main() {
    println!("cargo:rerun-if-changed=src/config.rs.in");
    println!("cargo:rerun-if-changed=src/config.rs");

    if std::path::Path::new("src/config.rs").exists() {
        return;
    }

    let template = std::fs::read_to_string("src/config.rs.in")
        .expect("src/config.rs.in not found");

    let config = template
        .replace("@APP_ID@", "\"org.xinux.Xeonitte\"")
        .replace("@GETTEXT_PACKAGE@", "\"xeonitte\"")
        .replace("@LOCALEDIR@", "\"/usr/share/locale\"")
        .replace("@PKGDATADIR@", "\"/usr/share/xeonitte\"")
        .replace("@SYSCONFDIR@", "\"/etc\"")
        .replace("@LIBEXECDIR@", "\"/usr/libexec\"")
        .replace("@PROFILE@", "\"\"")
        .replace("@VERSION@", concat!("\"", env!("CARGO_PKG_VERSION"), "\""));

    std::fs::write("src/config.rs", config)
        .expect("failed to write src/config.rs");
}
