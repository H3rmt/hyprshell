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
    extraConfig = ''
      monitor = Virtual-1, 1920x1080@60, 0x0, 1
      monitor = ,prefered,auto,1
      bind = super, return, exec, kitty
      bind = super, q, killactive
      bind = super, f, exec, firefox
    '';
  };

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
