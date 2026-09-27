{
  disko.devices = {
      disk = {
        ata-Samsung_990_EVO_500GB_MAIN = {
          content = {
            partitions = {
              ESP = {
                content = {
                  format = "vfat";
                  mountOptions = [
                    "umask=0077"
                  ];
                  mountpoint = "/boot";
                  type = "filesystem";
                };
                label = "ESP";
                size = "2G";
                type = "EF00";
              };
              cryptswap = {
                content = {
                  content = {
                    resumeDevice = true;
                    type = "swap";
                  };
                  name = "cryptswap";
                  passwordFile = "/run/xeonitte-luks.key";
                  settings = {
                    allowDiscards = true;
                  };
                  type = "luks";
                };
                label = "cryptswap";
                size = "8G";
                type = "8309";
              };
              luks = {
                content = {
                  content = {
                    format = "ext4";
                    mountpoint = "/";
                    type = "filesystem";
                  };
                  name = "crypted";
                  passwordFile = "/run/xeonitte-luks.key";
                  settings = {
                    allowDiscards = true;
                  };
                  type = "luks";
                };
                label = "luks";
                size = "100%";
                type = "8309";
              };
            };
            type = "gpt";
          };
          device = "/dev/disk/by-id/ata-Samsung_990_EVO_500GB_MAIN";
          type = "disk";
        };
        ata-WDC_WD10EZEX_1TB_DATA = {
          content = {
            partitions = {
              home = {
                content = {
                  format = "xfs";
                  mountpoint = "/home";
                  type = "filesystem";
                };
                label = "home";
                size = "100%";
                type = "8300";
              };
            };
            type = "gpt";
          };
          device = "/dev/disk/by-id/ata-WDC_WD10EZEX_1TB_DATA";
          type = "disk";
        };
      };
    };
  boot.initrd.systemd.enable = true;
}
