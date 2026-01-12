{
  lib,
  craneLib,
  #
  appstream-glib,
  cargo,
  rust-analyzer,
  clippy,
  convertyml,
  desktop-file-utils,
  gettext,
  git,
  meson,
  ninja,
  pkg-config,
  polkit,
  rustc,
  wrapGAppsHook4,
  #
  cryptsetup,
  gdk-pixbuf,
  glib,
  gnome-desktop,
  adwaita-icon-theme,
  gtk4,
  libadwaita,
  libgweather,
  openssl,
  parted,
  rustPlatform,
  vte-gtk4,
  #
  xeonitte-helper,
  makeWrapper,
}: let
  src = craneLib.cleanCargoSource ./.;
  commonArgs = {
    inherit src;
    strictDeps = true;
    nativeBuildInputs = [
      appstream-glib
      cargo
      rust-analyzer
      clippy
      convertyml
      gettext
      git
      meson
      ninja
      pkg-config
      polkit
      rustc
      # rustPlatform.cargoSetupHook
      wrapGAppsHook4
    ];

    buildInputs = [
      cryptsetup
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
    ];
  };

  cargoArtifacts = craneLib.buildDepsOnly commonArgs;
in
  craneLib.buildPackage (commonArgs
    // {
      inherit cargoArtifacts;

      nativeBuildInputs =
        commonArgs.nativeBuildInputs
        ++ [
          makeWrapper
        ];

      postInstall = ''
        wrapProgram  $out/bin/xeonitte \
            --prefix PATH : ${lib.makeBinPath [
          xeonitte-helper
        ]}
      '';
    })
