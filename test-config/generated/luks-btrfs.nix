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
              luks-btrfs = {
                content = {
                  content = {
                    extraArgs = [
                      "-f"
                    ];
                    subvolumes = {
                      "@" = {
                        mountOptions = [
                          "noatime"
                          "compress=zstd"
                        ];
                        mountpoint = "/";
                      };
                      "@home" = {
                        mountOptions = [
                          "noatime"
                        ];
                        mountpoint = "/home";
                      };
                      "@nix" = {
                        mountOptions = [
                          "noatime"
                        ];
                        mountpoint = "/nix";
                      };
                      "@snapshots" = {
                        mountpoint = "/.snapshots";
                      };
                      "@var" = {
                        mountOptions = [
                          "noatime"
                        ];
                        mountpoint = "/var";
                      };
                    };
                    type = "btrfs";
                  };
                  name = "cryptbtrfs";
                  passwordFile = "/run/xeonitte-luks.key";
                  settings = {
                    allowDiscards = true;
                  };
                  type = "luks";
                };
                label = "luks-btrfs";
                size = "100%";
                type = "8309";
              };
            };
            type = "gpt";
          };
          device = "/dev/disk/by-id/ata-Samsung_990_EVO_500GB_MAIN";
          type = "disk";
        };
      };
    };
  boot.initrd.systemd.enable = true;
}
