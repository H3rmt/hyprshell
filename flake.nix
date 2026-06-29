{
  description = "hyprshell - A Rust-based GUI designed to enhance window management in hyprland";
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    home-manager = {
      url = "github:nix-community/home-manager";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-parts = {
      url = "github:hercules-ci/flake-parts";
      inputs.nixpkgs-lib.follows = "nixpkgs";
    };
    crane.url = "github:ipetkov/crane";
  };
  outputs =
    inputs@{ self, flake-parts, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [
        "aarch64-linux"
        "x86_64-linux"
      ];
      perSystem =
        { pkgs, self', ... }:
        let
          craneLib = inputs.crane.mkLib pkgs;
          buildLib = import ./nix/build.nix { inherit craneLib pkgs; };
        in
        {
          formatter = pkgs.nixfmt-tree;
          packages = rec {
            hyprshell = craneLib.buildPackage (
              buildLib.commonArgsFull
            );
            hyprshell-nixpkgs = hyprshell;
            hyprshell-slim = craneLib.buildPackage (
              buildLib.commonArgsFull
              // {
                cargoExtraArgs = "--no-default-features --features slim";
              }
            );
            hyprshell-slim-nixpkgs = hyprshell-slim;
            default = hyprshell;
          };
          devShells.default = craneLib.devShell {
            checks = self'.checks;
            stdenv = buildLib.stdenv;
            packages = [
              pkgs.rust-analyzer
            ];
          };
        };
      flake = {
        homeModules = rec {
          hyprshell = import ./nix/module.nix self;
          default = hyprshell;
        };
      };
    };
}
