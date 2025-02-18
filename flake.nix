{
    inputs = {
        nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
        xinux-lib = {
            url = "github:xinux-org/lib";
            inputs.nixpkgs.follows = "nixpkgs";
        };
    };

    outputs = inputs:
        inputs.xinux-lib.mkFlake {
            inherit inputs;
            alias.packages.default = "xeonitte";
            src = ./.;
        };
}
