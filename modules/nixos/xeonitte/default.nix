{
  options,
  config,
  lib,
  pkgs,
  ...
}:

with lib;
let
  cfg = config.xeonitte;
  manifest = (pkgs.lib.importTOML ./Cargo.toml).workspace.package;

  cfg_name = config.services.${manifest.name};
  xeonitte-autostart = pkgs.makeAutostartItem {
    name = "org.xinux.Xeonitte";
    package = pkgs.internal.xeonitte;
  };
in
{
  options.xeonitte = with types; {
    enable = mkEnableOption "Enable Xeonitte Installer";
    config = mkOption {
      type = path;
      default = "${pkgs.internal.xeonitte}/etc/xeonitte";
      description = "Xeonitte configuration location";
    };
    tmp-dir = mkOption {
      type = types.str;
      default = "/tmp";
      example = "/tmp";
      description = "/tmp folder for xeonitte";
    };

    user = mkOption {
      type = types.str;
      default = "xeonitte";
      example = "xeonitte";
      description = "User for running commands with access";
    };

    group = mkOption {
      type = types.str;
      default = "xeonitte";
      example = "xeonitte";
      description = "Group for running xeonitte command";
    };
  };

  config = mkIf cfg.enable {
    environment.systemPackages = with pkgs; [
      internal.xeonitte
      xeonitte-autostart
    ];
    users.users.${cfg_name.user} = {
      description = "xeonitte user";
      isSystemUser = true;
      group = cfg.group;
      useDefaultShell = true;
    };

    users.groups = {
      ${cfg_name.group} = { };
    };

    systemd.tmpfiles.rules = [
      "d ${cfg_name.config-dir} 0770 ${cfg_name.user} ${cfg_name.group} -"
      "d ${cfg_name.tmp-dir}  0770 ${cfg_name.user} ${cfg_name.group} -"
    ];

    systemd.targets."${manifest.name}" = { };

    environment.etc."xeonitte".source = cfg.config;
  };
}
