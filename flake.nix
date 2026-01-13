{
  inputs = {
    nixpkgs.url = "github:xinux-org/nixpkgs/nixos-unstable";
    crane.url = "github:ipetkov/crane";
  };

  outputs = inputs: let
    supportedSystems = [
      "x86_64-linux" # 64-bit Intel/AMD Linux
      "aarch64-linux" # 64-bit ARM Linux
    ];

    forEachSupportedSystem = f:
      inputs.nixpkgs.lib.genAttrs supportedSystems (
        system:
          f {
            inherit system;
            pkgs = import inputs.nixpkgs {
              inherit system;
              config.allowUnfree = true;
            };
          }
      );
  in {
    devShells = forEachSupportedSystem ({pkgs, ...}: {
      default =
        pkgs.mkShellNoCC {
        };
    });
    packages = forEachSupportedSystem ({pkgs, ...}: let
      craneLib = inputs.crane.mkLib pkgs;
      convertyml = pkgs.callPackage ./packages/convertyml {};
      xeonitte-helper = pkgs.callPackage ./xeonitte-helper {inherit craneLib;};
      xeonitte = pkgs.callPackage ./xeonitte {inherit craneLib convertyml xeonitte-helper;};
    in {
      default = xeonitte;
      inherit xeonitte xeonitte-helper;

    });
  } // {
     nixosModules = import ./modules/nixos/xeonitte;
  };
}
