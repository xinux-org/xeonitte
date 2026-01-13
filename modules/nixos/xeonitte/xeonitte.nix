{ options, config, lib, pkgs, ... }:

with lib;
let
  cfg = config.xeonitte;
  xeonitte-autostart = pkgs.makeAutostartItem { name = "org.xinux.Xeonitte"; package = pkgs.internal.xeonitte; };
in
{
  options.xeonitte = with types; {
    enable =
      mkEnableOption "Enable Xeonitte Installer";
    config = mkOption {
      type = path;
      default = "${pkgs.internal.xeonitte}/etc/xeonitte";
      description = "Xeonitte configuration location";
    };
  };

  config = mkIf cfg.enable {
    environment.systemPackages = with pkgs; [
      internal.xeonitte
      xeonitte-autostart
    ];
    environment.etc."xeonitte".source = cfg.config;
  };
}
