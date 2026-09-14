{
  pkgs,
  mkShell,
  appstream-glib,
  inputs,
  cargo,
  clippy,
  desktop-file-utils,
  gdk-pixbuf,
  gettext,
  gnome-desktop,
  gobject-introspection,
  gtk4,
  internal,
  libadwaita,
  libgweather,
  meson,
  ninja,
  openssl,
  pango,
  parted,
  pkg-config,
  rust-analyzer,
  rustc,
  rustfmt,
  rustPlatform,
  vte-gtk4,
  wrapGAppsHook4,
  util-linux,
  dosfstools,
  e2fsprogs,
  systemd,
  dbus,
  zlib,
  nixd,
  statix,
  deadnix,
  nixfmt,
  polkit,
  just,
  bacon,
  disko,
  ...

}:
let
  system = pkgs.stdenv.hostPlatform.system;
  treefmtEval = inputs.treefmt-nix.lib.evalModule pkgs {
    projectRootFile = "flake.nix";
    programs.nixfmt.enable = true;
    programs.rustfmt.enable = true;
    settings.global.excludes = [ "config/**" ];
  };
  preCommitCheck = inputs.git-hooks.lib."${system}".run {
    src = ./.;
    hooks.treefmt.enable = true;
    hooks.treefmt.package = treefmtEval.config.build.wrapper;
  };
in
mkShell {
  nativeBuildInputs = [
    treefmtEval.config.build.wrapper
    appstream-glib
    cargo
    clippy
    desktop-file-utils
    gdk-pixbuf
    gettext
    gnome-desktop
    gobject-introspection
    gtk4
    internal.convertyml
    libadwaita
    libgweather
    meson
    ninja
    openssl
    pango
    parted
    pkg-config
    rust-analyzer
    rustc
    rustfmt
    rustPlatform.bindgenHook
    vte-gtk4
    wrapGAppsHook4
    util-linux
    dosfstools
    e2fsprogs
    systemd.dev
    dbus.dev
    zlib
    nixd
    statix
    deadnix
    nixfmt
    just
    bacon
    disko
  ];

  # Set Environment Variables
  RUST_BACKTRACE = "full";
  RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
  PKG_CONFIG_PATH = "${polkit.dev}/lib/pkgconfig";
  shellHook = preCommitCheck.shellHook;

}
