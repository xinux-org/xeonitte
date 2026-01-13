flake: {
  options,
  config,
  lib,
  pkgs,
  ...
}:
with lib; let
  cfg = config.xeonitte;
  xeonitte = flake.self.packages.${pkgs.stdenv.hostPlatform.system}.xeonitte;
  xeonitte-autostart = pkgs.makeAutostartItem {
    name = "org.xinux.Xeonitte";
    package = xeonitte;
  };
in {
  options.xeonitte = with types; {
    enable =
      mkEnableOption "Enable Xeonitte Installer";
    config = mkOption {
      type = path;
      default = "${xeonitte}/etc/xeonitte";
      description = "Xeonitte configuration location";
    };
  };

  config = mkIf cfg.enable {
    environment.systemPackages = [
      xeonitte
      xeonitte-autostart
    ];
    environment.etc."xeonitte".source = cfg.config;
  };
}
