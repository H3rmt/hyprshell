{
  description = "hyprshell - A Rust-based GUI designed to enhance window management in hyprland";
  inputs.nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
  inputs.home-manager.inputs.nixpkgs.follows = "nixpkgs";
  inputs.home-manager.url = "github:nix-community/home-manager";
  outputs =
    {
      self,
      nixpkgs,
      home-manager,
    }:
    let
      supportedSystems = [
        "x86_64-linux"
        "aarch64-linux"
      ];
      forAllSystems = nixpkgs.lib.genAttrs supportedSystems;
      pkgsFor = nixpkgs.legacyPackages;
    in
    {
      formatter = forAllSystems (system: pkgsFor.${system}.nixfmt-tree);
      packages = forAllSystems (system: rec {
        hyprshell = pkgsFor.${system}.callPackage ./nix/default.nix { inherit self; };
        default = hyprshell;
      });
      devShells = forAllSystems (system: rec {
        hyprshell = pkgsFor.${system}.callPackage ./nix/shell.nix { inherit self; };
        default = hyprshell;
      });
      homeModules = rec {
        hyprshell = import ./nix/module.nix { inherit self; };
        default = hyprshell;
      };
      checks = forAllSystems (
        system:
        let
          filterDisabledAndDropEnable =
            (import ./nix/util.nix { lib = pkgsFor.${system}.lib; }).filterDisabledAndDropEnable;
          command = ''
            TMP=$(mktemp -d)
            touch "$TMP/test.json"
            cat <<EOF> "$TMP/test.json"
            ${builtins.toJSON (
              filterDisabledAndDropEnable
                self.homeConfigurations.test.${system}.config.programs.hyprshell.settings
            )}
            EOF
            ${pkgsFor.${system}.jq}/bin/jq < "$TMP/test.json"
            ${self.packages.${system}.hyprshell}/bin/hyprshell config check -c "$TMP/test.json" && (mkdir $out)
            rm -r "$TMP"
          '';
        in
        {
          test = pkgsFor.${system}.runCommand "test" { } command;
        }
      );

      homeConfigurations.test = forAllSystems (
        system:
        home-manager.lib.homeManagerConfiguration rec {
          pkgs = pkgsFor.${system};
          modules = [
            self.homeModules.hyprshell
            {
              home.stateVersion = "24.05";
              home.username = "test";
              home.homeDirectory = "/home/test";
            }
          ];
        }
      );
    };
}
