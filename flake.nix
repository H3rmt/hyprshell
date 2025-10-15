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
    hyprland.url = "github:hyprwm/Hyprland";
    crane.url = "github:ipetkov/crane";
  };
  outputs =
    inputs@{
      self,
      nixpkgs,
      home-manager,
      flake-parts,
      hyprland,
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
          buildLib = import ./nix/build.nix { inherit craneLib pkgs; };
        in
        {
          formatter = pkgs.nixfmt-tree;
          packages = rec {
            hyprshell = craneLib.buildPackage (
              buildLib.commonArgs
              // {
                cargoArtifacts = buildLib.cargoReleaseArtifacts;
                preFixup =
                  buildLib.addWrapWithGccArgs
                    inputs.hyprland.packages.${pkgs.stdenv.hostPlatform.system}.default;
              }
            );
            hyprshell-nixpkgs = craneLib.buildPackage (
              buildLib.commonArgs
              // {
                cargoArtifacts = buildLib.cargoReleaseArtifacts;
                preFixup = buildLib.addWrapWithGccArgs pkgs.hyprland;
              }
            );
            hyprshell-slim = craneLib.buildPackage (
              buildLib.commonArgs
              // {
                cargoArtifacts = buildLib.cargoReleaseArtifacts;
                cargoExtraArgs = "--no-default-features --features slim";
                preFixup =
                  buildLib.addWrapWithGccArgs
                    inputs.hyprland.packages.${pkgs.stdenv.hostPlatform.system}.default;
              }
            );
            hyprshell-slim-nixpkgs = craneLib.buildPackage (
              buildLib.commonArgs
              // {
                cargoArtifacts = buildLib.cargoReleaseArtifacts;
                cargoExtraArgs = "--no-default-features --features slim";
                preFixup = buildLib.addWrapWithGccArgs pkgs.hyprland;
              }
            );
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
