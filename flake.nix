{
  description = "hyprshell - A Rust-based GUI designed to enhance window management in hyprland";
  inputs.nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
  outputs = { self, nixpkgs, }: let
    supportedSystems = ["x86_64-linux" "aarch64-linux"];
    forAllSystems = nixpkgs.lib.genAttrs supportedSystems;
    pkgsFor = nixpkgs.legacyPackages;
  in {
    packages = forAllSystems (system: rec {
      hyprshell = pkgsFor.${system}.callPackage ./nix/default.nix {inherit self;};
      default = hyprshell;
    });
    devShells = forAllSystems (system: {
      default = pkgsFor.${system}.callPackage ./nix/shell.nix {inherit self;};
    });
  };
}
