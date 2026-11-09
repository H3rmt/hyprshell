{
  lib,
  pkgs,
  inputs,
  ...
}:
{
  imports = [
    inputs.hyprshell.homeModules.hyprshell
  ];

  home.stateVersion = "24.05";
  home.packages = with pkgs; [
    firefox
    chromium
    baobab
  ];
  
  wayland.windowManager.hyprland = {
    enable = true;
    package = inputs.hyprland.packages.x86_64-linux.hyprland;
    portalPackage = inputs.hyprland.packages.x86_64-linux.xdg-desktop-portal-hyprland;
  };

  xdg.configFile."hypr/hyprland.lua".text = ''
    hl.monitor({ output = "Virtual-1", mode = "1920x1080@60", position = "0x0", scale = 1 })
    hl.monitor({ output = "", mode = "preferred", position = "auto", scale = 1 })

    hl.bind("SUPER + RETURN", function()
        hl.dispatch(hl.dsp.exec_cmd("kitty"))
    end)
    hl.bind("SUPER + Q", function()
        hl.dispatch(hl.dsp.window.kill())
    end)
    hl.bind("SUPER + F", function()
        hl.dispatch(hl.dsp.exec_cmd("firefox"))
    end)
  '';

  programs.kitty = {
    enable = true;
    font = {
      name = "JetBrainsMono Nerd Font";
      size = 14;
    };
  };
  
  programs.hyprshell = {
    enable = true;
    systemd = {
      args = "-vv";
    };
    settings = {
        windows.enable = true;
        windows.overview.enable = true;
        windows.switch.enable = true;
        windows.switch.modifier = "ctrl";
    };
  };
}
