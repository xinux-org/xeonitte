{
  inputs = {
    nixpkgs.url = "git+https://git.oss.uzinfocom.uz/xinux/nixpkgs?ref=nixos-unstable&shallow=1";
    disko = {
      url = "git+https://git.oss.uzinfocom.uz/mirrors/disko?ref=master&shallow=1";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, disko }:
    let
      pkgs = nixpkgs.legacyPackages.x86_64-linux;
      diskoLib = pkgs.callPackage "${disko}/lib" {
        makeTest   = import (pkgs.path + "/nixos/tests/make-test-python.nix");
        eval-config = import (pkgs.path + "/nixos/lib/eval-config.nix");
      };
    in
    {
      # Run all:   nix flake check
      # Run one:   nix build .#checks.x86_64-linux.canonical-no-swap
      checks.x86_64-linux = import ./tests { inherit pkgs diskoLib; };
    };
}