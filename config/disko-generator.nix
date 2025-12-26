{lib}: let
  passwordFile = "/tmp/xinux-luks.key";

  generateFullDisk = schema: {
    disko.devices = {
      disk = {
        main = {
          type = "disk";
          device = schema.device;
          content = {
            type = "gpt";
            partitions = {
              ESP = {
                size = "500M";
                type = "EF00";
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
                  name = "crypted";
                  settings.allowDiscards = true;
                  inherit passwordFile;
                  content = {
                    type = "filesystem";
                    format = schema.filesystem;
                    mountpoint = "/";
                  };
                };
              };
            };
          };
        };
      };
    };
  };
in
  schema:
    if schema.mode == "full-disk"
    then generateFullDisk schema
    else throw "Unknown mode: ${schema.mode}"
