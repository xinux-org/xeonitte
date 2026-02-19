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
  cryptsetup,
  util-linux,
  dosfstools,
  e2fsprogs,
}: let
  convertyml = internal.convertyml;
in
  stdenv.mkDerivation rec {
    pname = "xeonitte";
    version = "0.0.3";

    src = [../..];

    cargoDeps = rustPlatform.importCargoLock {
      lockFile = ../../Cargo.lock;
      outputHashes = {
        "disk-types-0.1.5" = "sha256-QV5VoXnDJ6nU3co/hg5+luZvIuFEip6PoiSkbwSke8w=";
      };
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
    ];

    postFixup = ''
      wrapProgram $out/libexec/xeonitte-helper \
        --prefix PATH : ${lib.makeBinPath [
        cryptsetup
        dosfstools
        e2fsprogs
        parted
        util-linux
      ]}
    '';
  }
