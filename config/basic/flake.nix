{
  inputs = {
    nixpkgs.url = "github:xinux-org/nixpkgs/nixos-25.11";
    nix-data = {
      url = "github:xinux-org/nix-data/release-25.11";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    xinux-lib = {
      url = "github:xinux-org/lib/release-25.11";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    xinux-modules = {
      url = "github:xinux-org/modules/release-25.11";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = inputs:
    inputs.xinux-lib.mkFlake {
      inherit inputs;
      src = ./.;

      channels-config.allowUnfree = true;
      systems.modules.nixos = with inputs; [
        nix-data.nixosModules.nix-data
        @BOOTLOADER_MODULE@
        xinux-modules.nixosModules.gnome
        xinux-modules.nixosModules.kernel
        xinux-modules.nixosModules.networking
        xinux-modules.nixosModules.packagemanagers
        xinux-modules.nixosModules.pipewire
        xinux-modules.nixosModules.printing
        xinux-modules.nixosModules.xinux
        xinux-modules.nixosModules.metadata
      ];
    };
}
