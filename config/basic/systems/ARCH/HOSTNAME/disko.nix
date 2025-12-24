let
  btrfsopt = [
    "compress=zstd"
    "noatime"
    "ssd"
    "space_cache=v2"
    "user_subvol_rm_allowed"
  ];
in {
  disko.devices = {
    disk = {
      main = {
        type = "disk";
        device = "/dev/nvme0n1";
        content = {
          type = "gpt";
          partitions = {
            boot = {
              name = "boot";
              size = "1M";
              type = "ef02";
            };
            esp = {
              name = "esp";
              size = "500M";
              type = "ef00";
              content = {
                type = "filesystem";
                format = "vfat";
                mountpoint = "/boot";
                mountOptions = ["umask=0077"];
              };
            };
            luks = {
              size = "100%";
              content = {
                type = "luks";
                name = "nixos";
                passwordFile = "/tmp/pass";
                additionalKeyFiles = ["/nixos-enc.key"];
                extraFormatArgs = [
                  "--type luks2"
                  "--iter-time 1000"
                ];
                settings = {
                  allowDiscards = true;
                };
                content = {
                  type = "btrfs";
                  subvolumes = {
                    "@root" = {
                      mountpoint = "/";
                      mountOptions = btrfsopt;
                    };
                    "@home" = {
                      mountpoint = "/home";
                      mountOptions = btrfsopt;
                    };
                    "@nix" = {
                      mountpoint = "/nix";
                      mountOptions = btrfsopt;
                    };
                    "@data" = {
                      mountpoint = "/data";
                      mountOptions = btrfsopt;
                    };
                  };
                };
              };
            };
          };
        };
      };
    };
  };
}
