{
  description = "A very basic flake";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    home-manager.url = "github:nix-community/home-manager";
    # hyprland.url = "github:hyprwm/Hyprland";
    # hyprland.url = "github:hyprwm/Hyprland?ref=v0.50.1";
    hyprshell.url = "github:H3rmt/hyprshell";
  };

  outputs =
    {
      self,
      nixpkgs,
      home-manager,
      hyprshell,
    }@inputs:
    {
      formatter."x86_64-linux" = nixpkgs.legacyPackages.x86_64-linux.nixfmt-tree;
      nixosConfigurations.nixos = nixpkgs.lib.nixosSystem ({
        specialArgs = { inherit inputs; };
        system = "x86_64-linux";
        modules = [
          home-manager.nixosModules.home-manager
          ./configuration.nix
          ./home.nix
        ];
      });
    };
}
