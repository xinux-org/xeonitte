{
  stdenv,
  appstream-glib,
  cargo,
  desktop-file-utils,
  gdk-pixbuf,
  gettext,
  git,
  glib,
  lib,
  gnome,
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
  rust-analyzer,
  clippy,
  rustPlatform,
  vte-gtk4,
  wrapGAppsHook4,
  makeWrapper,
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
      cargo
      rust-analyzer
      clippy
      convertyml
      desktop-file-utils
      gettext
      git
      meson
      ninja
      pkg-config
      polkit.dev
      rustc
      rustPlatform.cargoSetupHook
      wrapGAppsHook4
      makeWrapper
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
