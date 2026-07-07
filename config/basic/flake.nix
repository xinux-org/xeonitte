{
  inputs = {
    nixpkgs.url = "git+https://git.oss.uzinfocom.uz/xinux/nixpkgs?ref=nixos-26.05&shallow=1";

    xinux-lib = {
      url = "git+https://git.oss.uzinfocom.uz/xinux/lib?ref=release-26.05&shallow=1";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    xinux-modules = {
      url = "git+https://git.oss.uzinfocom.uz/xinux/modules?ref=release-26.05&shallow=1";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    nix-data = {
      url = "git+https://git.oss.uzinfocom.uz/xinux/nix-data?ref=release-26.05&shallow=1";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    uz-xkb = {
      url = "git+https://git.oss.uzinfocom.uz/mirrors/uzbek-linux-keyboard?shallow=1";
      flake = false;
    };
    disko = {
      url = "git+https://git.oss.uzinfocom.uz/mirrors/disko?ref=master&shallow=1";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    relago.url = "git+https://git.oss.uzinfocom.uz/xinux/relago?ref=release-26.05";
  };

  outputs = inputs:
    inputs.xinux-lib.mkFlake {
      inherit inputs;
      src = ./.;

      channels-config.allowUnfree = true;
      systems.modules.nixos = with inputs; [
        relago.nixosModules.relago
        disko.nixosModules.disko
        nix-data.nixosModules.nix-data
        @BOOTLOADER_MODULE@
        xinux-modules.nixosModules.meta
      ];
    };
}
