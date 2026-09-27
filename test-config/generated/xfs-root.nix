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
              root = {
                content = {
                  format = "xfs";
                  mountpoint = "/";
                  type = "filesystem";
                };
                label = "root";
                size = "100%";
                type = "8300";
              };
              swap = {
                content = {
                  resumeDevice = true;
                  type = "swap";
                };
                label = "swap";
                size = "8G";
                type = "8200";
              };
            };
            type = "gpt";
          };
          device = "/dev/disk/by-id/ata-Samsung_990_EVO_500GB_MAIN";
          type = "disk";
        };
      };
    };
}
