{
  description = "hyprshell - A Rust-based GUI designed to enhance window management in hyprland";
  inputs.self.submodules = true;
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
    inputs@{
      self,
      nixpkgs,
      home-manager,
      flake-parts,
      crane,
    }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [
        "aarch64-linux"
        "x86_64-linux"
      ];
      perSystem =
        { pkgs, self', ... }:
        let
          craneLib = crane.mkLib pkgs;
          buildLib = import ./nix/build.nix { inherit craneLib pkgs self; };
        in
        {
          formatter = pkgs.nixfmt-tree;
          packages = rec {
            hyprshell = craneLib.buildPackage buildLib.commonArgsCachedRelease // {
              postInstall = buildLib.postInstall;
            };
            default = hyprshell;
          };
          checks = import ./nix/checks.nix {
            inherit
              self
              pkgs
              craneLib
              buildLib
              home-manager
              ;
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
