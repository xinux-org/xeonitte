{
  inputs = {
    nixpkgs.url = "git+https://git.oss.uzinfocom.uz/xinux/nixpkgs?ref=nixos-25.11&shallow=1";
    xinux-lib = {
      url = "git+https://git.oss.uzinfocom.uz/xinux/lib?ref=release-25.11&shallow=1";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = inputs:
    inputs.xinux-lib.mkFlake {
      inherit inputs;
      alias.packages.default = "xeonitte";
      alias.shells.default = "xeonitte";
      src = ./.;
      hydraJobs = inputs.self.packages.x86_64-linux;
    };
}
