{
  description = "Hyprland-rs devshell";
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    dream2nix = {
      url = "github:71/dream2nix/rust-ws-inherit-version";
    };
    nci = {
      url = "github:yusdacra/nix-cargo-integration";
      inputs = {
        dream2nix.follows = "dream2nix";
      };
    };
    flake-parts = {
      inputs = {
        nixpkgs-lib.follows = "nixpkgs";
      };
    };
  };
  outputs = inputs @ {
    flake-parts,
    nci,
    ...
  }:
    flake-parts.lib.mkFlake {inherit inputs;} {
      imports = [
        nci.flakeModule
      ];
      systems = ["x86_64-linux" "aarch64-linux"];
      perSystem = {
        pkgs,
        config,
        ...
      }: let
        crateName = "hyprland";
        crateOutputs = config.nci.outputs.${crateName};
      in {
        nci.projects.${crateName}.relPath = "";
        nci.crates.${crateName} = {
          export = true;
        };
        devShells.default = crateOutputs.devShell;
        packages.default = crateOutputs.packages.release;
      };
    };
}
