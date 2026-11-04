{
  stdenv,
  appstream-glib,
  cargo,
  desktop-file-utils,
  gdk-pixbuf,
  gettext,
  glib,
  gnome-desktop,
  adwaita-icon-theme,
  gtk4,
  libadwaita,
  libgweather,
  meson,
  ninja,
  openssl,
  parted,
  pkg-config,
  polkit,
  rustc,
  rustPlatform,
  vte-gtk4,
  wrapGAppsHook4,
  systemd,
  dbus,
  zlib,
  disko,
  cryptsetup,
  ...
}:
stdenv.mkDerivation {
  pname = "xeonitte";
  version = "0.1.0";

  src = [ ../.. ];

  cargoDeps = rustPlatform.fetchCargoVendor {
    src = ../..;
    hash = "sha256-T7OcrvGfhq3sDXjHFLGqOSzigEOWYBk2RZhKzJmKLLw=";
  };

  nativeBuildInputs = [
    appstream-glib
    meson
    ninja
    cargo
    pkg-config
    gettext
    desktop-file-utils
    polkit
    rustc
    rustPlatform.cargoSetupHook
    wrapGAppsHook4
    cryptsetup
    disko
  ];

  buildInputs = [
    desktop-file-utils
    gdk-pixbuf
    glib
    gnome-desktop
    adwaita-icon-theme
    gtk4
    libadwaita
    libgweather
    openssl
    parted
    rustPlatform.bindgenHook
    vte-gtk4
    cryptsetup
    systemd.dev
    dbus.dev
    zlib
  ];
}
