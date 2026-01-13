{
  lib,
  stdenv,
  meson,
  pkg-config,
  glib,
  openssl,
  gtk4,
  libadwaita,
  polkit,
  convertyml,
  desktop-file-utils,
  ninja,
}: let
  filter = name: type: let
    baseName = baseNameOf (toString name);
  in
    (lib.cleanSourceFilter name type)
    && !(type == "directory" && baseName == "xeonitte")
    && !(type == "directory" && baseName == "xeonitte-helper");
in
  stdenv.mkDerivation {
    name = "xeonitte-data";

    src = lib.cleanSourceWith {
      inherit filter;
      src = ./.;
    };

    nativeBuildInputs = [
      meson
      pkg-config
      glib
      openssl
      gtk4
      libadwaita
      polkit
      convertyml
      desktop-file-utils
      ninja
    ];
  }
