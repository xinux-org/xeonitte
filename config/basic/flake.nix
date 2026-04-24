{
  inputs = {
    nixpkgs.url = "git+https://git.oss.uzinfocom.uz/xinux/nixpkgs?ref=nixos-unstable&shallow=1";

    xinux-lib = {
      url = "git+https://git.oss.uzinfocom.uz/xinux/lib?ref=main&shallow=1";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    xinux-modules = {
      url = "git+https://git.oss.uzinfocom.uz/xinux/modules?ref=main&shallow=1";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    nix-data = {
      url = "git+https://git.oss.uzinfocom.uz/xinux/nix-data?ref=main&shallow=1";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    uz-xkb = {
      url = "github:itsbilolbek/uzbek-linux-keyboard";
      flake = false;
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
        xinux-modules.nixosModules.branding
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
