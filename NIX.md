# NixOS

- Supported Architectures: `x86_64-linux`, `aarch64-linux`

## With Flakes

`flake.nix`:

```nix
{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    hyprshell.url = "github:H3rmt/hyprshell?ref=hyprshell";
    hyprshell.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = { nixpkgs, hyprshell }: {
    nixosConfigurations.hostname = nixpkgs.lib.nixosSystem {
      system = "x86_64-linux";
      modules = [{ environment.systemPackages = [ hyprshell.packages.x86_64-linux.hyprshell ]; }];
    };
  };
}
```

## With Home-manager

`flake.nix`:

```nix
{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    hyprshell.url = "github:H3rmt/hyprshell?ref=hyprshell";
    hyprshell.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = { nixpkgs, hyprshell }@inputs: {
    nixosConfigurations.hostname = nixpkgs.lib.nixosSystem {
      specialArgs = { inherit inputs; };
      system = "x86_64-linux";
      modules = [ 
        ./home.nix
      ];
    };
  };
}
```

`./home.nix`:

```nix
{ inputs, ... } : {
  home-manager = {
    extraSpecialArgs = { inherit inputs; };
    user.test = import ./user.nix; 
  };
}
```

`./user.nix`:

```nix
{ inputs, ... } : {
  imports = [
    inputs.hyprshell.homeModules.hyprshell
  ];
  programs.hyprshell = {
    enable = true;
    systemd.args = "-v";
    settings = {
      launcher = {
        max_items = 6;
        plugins.websearch = {
            enable = true;
            engines = [{
                name = "DuckDuckGo";
                url = "https://duckduckgo.com/?q=%s";
                key = "d";
            }];
        };
      };
      window.switcher.enable = false;
    };
  };
}
```

## Without Flakes

`configuration.nix`:

```nix
{pkgs, ...}: let
  flake-compat = builtins.fetchTarball "https://github.com/edolstra/flake-compat/archive/master.tar.gz";
  hyprshell = (import flake-compat {
    src = builtins.fetchTarball "https://github.com/H3rmt/hyprshell/archive/hyprshell.tar.gz";
  }).defaultNix;
in {
   environment.systemPackages = [hyprshell.packages.${pkgs.system}.hyprshell];
}
```
