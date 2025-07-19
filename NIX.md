# NixOS

**[Cachix Cache](https://app.cachix.org/cache/hyprshell#pull) can be added with `cachix use hyprshell`**

## With Flakes

`flake.nix`:

```nix
{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    hyprshell.url = "github:H3rmt/hyprshell?ref=hyprshell-release";
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

## With Flakes + Home-manager [recommend]

`flake.nix`:

```nix
{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    hyprshell.url = "github:H3rmt/hyprshell?ref=hyprshell-release";
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

All the settings are optional and can be found in the [module.nix](./nix/module.nix)

```nix
{ inputs, ... } : {
  imports = [
    inputs.hyprshell.homeModules.hyprshell
  ];
  programs.hyprshell = {
    enable = true;
    systemd.args = "-v";
    settings = {
      windows = {
        overview = {
          enable = true;
          key = "super_l";
          modifier = "super";
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
        };
        switch.enable = false;
      };
    };
  };
}
```

## Without Flakes (nixpkgs-unstable)

`configuration.nix`:

```nix
{pkgs, ...}: {
  environment.systemPackages = [pkgs.hyprshell];
}
```

## Without Flakes (flake-compat)

`configuration.nix`:

```nix
{pkgs, ...}: let
  flake-compat = builtins.fetchTarball "https://github.com/edolstra/flake-compat/archive/master.tar.gz";
  hyprshell = (import flake-compat {
    src = builtins.fetchTarball "https://github.com/H3rmt/hyprshell/archive/hyprshell-release.tar.gz";
  }).defaultNix;
in {
   environment.systemPackages = [hyprshell.packages.${pkgs.system}.hyprshell];
}
```

## Without Flakes with Home-manager

https://github.com/nix-community/home-manager/pull/7409