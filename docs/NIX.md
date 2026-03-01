# NixOS

## No flakes

This is the easy way to use/configure hyprshell as this way they cant be out of sync.

### nixpkgs

`configuration.nix`:

```nix
{pkgs, ...}: {
  environment.systemPackages = [pkgs.hyprland, pkgs.hyprshell];
}
```

### nixpkgs + Home manager

`./user.nix`:

All the settings are optional and can be found in the [config](CONFIGURE.md)

This config enables overview and switch, but is not type-save like the flake home-manager config.

```nix
{ inputs, ... } : {
  services.hyprshell = {
    enable = true;
    settings = {
      windows = {
        scale = 8.0;
        overview = {
          launcher = {
            max_items = 6;
          };
        };
        switch = {
          modifier = "alt";
        };
      };
    };
  };
}
```

## Flakes

Warning: hyprshell builds a hyprland plugin at runtime which **requires the exact C headers** from the running hyprland instance.

This is trivial for other platforms, but not for NixOS and can cause problems (please report them on github if you encounter any).
The default hyprshell program from this flake has access to the hyprland C headers from the latest hyprland flake (updated every ~2 weeks).
If you use hypland as a flake you MUST use hyprshell as a flake.
To synchronize the hyprland version with the hyprshell version, you have to override the hyprland input in the flake.

A full example nixos config can be found in `test-files/nixos`

### With Home-manager [recommend]

**[Cachix Cache](https://app.cachix.org/cache/hyprshell#pull) can be added with `cachix use hyprshell` but wont be able to cache the full program**

`flake.nix`:

```nix
{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    hyprland.url = "github:hyprwm/Hyprland";
    hyprshell = {
      url = "github:H3rmt/hyprshell";
      # overriding is necessary
      inputs.hyprland.follows = "hyprland";
    };
  };

  outputs = { nixpkgs, hyprshell }@inputs: {
    nixosConfigurations.hostname = nixpkgs.lib.nixosSystem {
      # passes inputs info ./home.nix file
      specialArgs = { inherit inputs; };
      system = "x86_64-linux";
      modules = [
        home-manager.nixosModules.home-manager
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
    # passes inputs into user.nix file
    extraSpecialArgs = { inherit inputs; };
    user.test = import ./user.nix; 
  };
}
```

`./user.nix`:

All the settings are optional and can be found in the [module.nix](../nix/module.nix)

Everything is disabled by default, so you need to enable it (even settings.windows if settings.windows.overview is enabled).

```nix
{ inputs, ... } : {
  imports = [
    # includes the custom programs.hyprshell config 
    inputs.hyprshell.homeModules.default
  ];
  
  programs.hyprshell = {
    enable = true;
    package = inputs.hyprshell.packages.${inputs.nixpkgs.stdenv.hostPlatform.system}.hyprshell;
    # use this if you want the more minimal hyprshell (see Readme.md > Features)
    package = inputs.hyprshell.packages.${inputs.nixpkgs.stdenv.hostPlatform.system}.hyprshell-slim;
    # use this if you dont use hyprland via a flake and override hyprshells hyprland input
    package = inputs.hyprshell.packages.${inputs.nixpkgs.stdenv.hostPlatform.system}.hyprshell-nixpkgs;
    settings = {
      windows = {
        enable = true; # please dont forget to enable windows if you want to use overview or switch
        overview = {
          enable = true;
          key = "super_l";
          modifier = "super";
          launcher = {
            max_items = 6;
          };
        };
        switch.enable = true;
      };
    };
  };

  wayland.windowManager.hyprland = {
    enable = true;
    package = inputs.hyprland.packages.x86_64-linux.hyprland;
    portalPackage = inputs.hyprland.packages.x86_64-linux.xdg-desktop-portal-hyprland;
  };
}
```


### No Home-manager with hyprland from nixpkgs

**[Cachix Cache](https://app.cachix.org/cache/hyprshell#pull) can be added with `cachix use hyprshell`**

`flake.nix`:

```nix
{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    hyprshell = {
      url = "github:H3rmt/hyprshell";
      # no overriding of input necessary
    };
  };

  outputs = { nixpkgs, hyprshell }: {
    nixosConfigurations.hostname = nixpkgs.lib.nixosSystem {
      system = "x86_64-linux";
      modules = [{ 
        environment.systemPackages = [ 
            nixpkgs.hyprland
            hyprshell.packages.${nixpkgs.stdenv.hostPlatform.system}.hyprshell-nixpkgs 
        ]; 
      }];
    };
  };
}
```

### No Home-manager with hyprland from flake

**[Cachix Cache](https://app.cachix.org/cache/hyprshell#pull) can be added with `cachix use hyprshell` but wont be able to cache the full program**

`flake.nix`:

```nix
{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    hyprland.url = "github:hyprwm/Hyprland";
    hyprshell = {
      url = "github:H3rmt/hyprshell";
      # overriding is necessary
      inputs.hyprland.follows = "hyprland";
    };
  };

  outputs = { nixpkgs, hyprland, hyprshell }@inputs: {
    nixosConfigurations.hostname = nixpkgs.lib.nixosSystem {
      system = "x86_64-linux";
      modules = [{ 
        environment.systemPackages = [ 
          hyprland.packages.${nixpkgs.stdenv.hostPlatform.system}.hyprland
          hyprshell.packages.${nixpkgs.stdenv.hostPlatform.system}.hyprshell 
          # Use this if you want the more minimal hyprshell (see Readme.md > Features)
          # hyprshell.packages.${nixpkgs.stdenv.hostPlatform.system}.hyprshell-slim
        ];
      }];
    };
  };
}
```
