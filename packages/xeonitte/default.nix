{ stdenv
, appstream-glib
, cargo
, desktop-file-utils
, gdk-pixbuf
, gettext
, git
, glib
, gnome
, gnome-desktop
, adwaita-icon-theme
, gtk4
, internal
, libadwaita
, libgweather
, meson
, ninja
, openssl
, parted
, pkg-config
, polkit
, rustc
, rust-analyzer
, clippy
, rustPlatform
, vte-gtk4
, wrapGAppsHook4
}:
let
  convertyml = internal.convertyml;
in
stdenv.mkDerivation rec {
  pname = "xeonitte";
  version = "0.0.2";

  src = [ ../.. ];

  cargoDeps = rustPlatform.importCargoLock {
    lockFile = ../../Cargo.lock;
    outputHashes = {
      "disk-types-0.1.5" = "sha256-QV5VoXnDJ6nU3co/hg5+luZvIuFEip6PoiSkbwSke8w=";
      "vte4-0.7.0" = "sha256-LLZGnHypJz6PoiY6Mb1t0qAPKsx6klUBP3QeVqQfc2k=";
      "gnome-desktop-0.4.2" = "sha256-UlYHRHUtoQax7XlGuXNct8Zys5JO5TdhQPCDpXtlCcM=";
      "gsettings-desktop-schemas-0.3.1" = "sha256-km3pO25ZjYfGD8skYTHYO1rRSjHaHtVYFgRaoDZXq2s=";
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
  ];
}
