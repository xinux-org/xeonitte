{
  stdenv,
  appstream-glib,
  cargo,
  desktop-file-utils,
  gdk-pixbuf,
  gettext,
  glib,
  lib,
  gnome-desktop,
  adwaita-icon-theme,
  gtk4,
  internal,
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
  util-linux,
  dosfstools,
  e2fsprogs,
  systemd,
  dbus,
  zlib,
  disko,
  cryptsetup,
  ...
}:
let
  convertyml = internal.convertyml;
in
stdenv.mkDerivation {
  pname = "xeonitte";
  version = "0.1.0";

  src = [ ../.. ];

  cargoDeps = rustPlatform.fetchCargoVendor {
    src = ../..;
    hash = "sha256-FhKXHrN9547It9fZXMU63Vi5vjKIREB3MD2j3SgHS1Q=";
  };

  nativeBuildInputs = [
    appstream-glib
    meson
    ninja
    cargo
    pkg-config
    gettext
    convertyml
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

  postFixup = ''
    wrapProgram $out/libexec/xeonitte-helper \
      --prefix PATH : ${
        lib.makeBinPath [
          cryptsetup
          dosfstools
          e2fsprogs
          parted
          util-linux
        ]
      }
  '';
}
