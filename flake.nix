{
  inputs = {
    nixpkgs.url = "github:xinux-org/nixpkgs/nixos-unstable";
    xinux-lib = {
      url = "github:xinux-org/lib";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = inputs:
    inputs.xinux-lib.mkFlake {
      inherit inputs;
      alias.packages.default = "xeonitte";
      alias.shells.default = "xeonitte";
      src = ./.;
      # hydraJobs = inputs.self.packages.x86_64-linux;
    };
}
