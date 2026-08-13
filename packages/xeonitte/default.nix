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
}: let
  convertyml = internal.convertyml;
in
  stdenv.mkDerivation rec {
    pname = "xeonitte";
    version = "0.1.0";

    src = [../..];

    cargoDeps = rustPlatform.fetchCargoVendor {
      src = ../..;
	    hash = "sha256-ifGbfFqFMxRdrvGRkzu8LdXK+V7TKdTv5P4LOlKzW/c=";
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
      systemd.dev
      dbus.dev
      zlib
    ];

    postFixup = ''
      wrapProgram $out/libexec/xeonitte-helper \
        --prefix PATH : ${lib.makeBinPath [
        dosfstools
        e2fsprogs
        parted
        util-linux
      ]}
    '';
  }
