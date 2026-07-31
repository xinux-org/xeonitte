{
  pkgs,
  mkShell,
  appstream-glib,
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
  cryptsetup,
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
<<<<<<< Updated upstream
  just,
  bacon,
=======
  lldb,
  glibc,
  gcc,
  vscode-extensions,
>>>>>>> Stashed changes
  ...
}:
mkShell {
  nativeBuildInputs = [
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
    cryptsetup
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
    polkit
<<<<<<< Updated upstream
    just
    bacon
=======
    # for debug adapter
    lldb glibc gcc
    vscode-extensions.vadimcn.vscode-lldb
>>>>>>> Stashed changes
  ];

  # Set Environment Variables
  RUST_BACKTRACE = "full";
  RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";

  # Set env variables so we can refer to them from the vim configuration
   CODELLDB_PATH =
     "${pkgs.vscode-extensions.vadimcn.vscode-lldb}/share/vscode/extensions/vadimcn.vscode-lldb/adapter/codelldb";
   LIBLLDB_PATH =
     "${pkgs.vscode-extensions.vadimcn.vscode-lldb}/share/vscode/extensions/vadimcn.vscode-lldb/lldb/lib/liblldb.so";

}
