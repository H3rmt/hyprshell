{
  lib,
  pkgs,
  inputs,
  ...
}:
# Configures using hyprshell home manager module
{
  home.stateVersion = "24.05";
  home.packages = with pkgs; [
    firefox
    chromium
    baobab
  ];

  wayland.windowManager.hyprland = {
    enable = true;
    configType = "lua";
    settings = {
      mod = {
        _var = "SUPER";
      };
      config = {
        general = {
          gaps_in = 5;
          gaps_out = 20;
          border_size = 6;
        };
        decoration = {
          rounding = 20;
        };
      };
      monitor = [
        {
          output = "Virtual-1";
          mode = "1920x1080@60";
          position = "0x0";
          scale = "1";
        }
      ];
      bind = [
        {
          _args = [
            (lib.generators.mkLuaInline "mod .. \" + Q\"")
            (lib.generators.mkLuaInline "hl.dsp.window.close()")
            { locked = true; }
          ];
        }
        {
          _args = [
            (lib.generators.mkLuaInline "mod .. \" + RETURN\"")
            (lib.generators.mkLuaInline "hl.dsp.exec_cmd(\"kitty\")")
          ];
        }
        {
          _args = [
            (lib.generators.mkLuaInline "mod .. \" + F\"")
            (lib.generators.mkLuaInline "hl.dsp.exec_cmd(\"firefox\")")
          ];
        }
      ];
    };
    systemd.enable = true;
    systemd.enableXdgAutostart = true;
  };

  programs.kitty = {
    enable = true;
    font = {
      name = "JetBrainsMono Nerd Font";
      size = 14;
    };
  };

  services.hyprshell = {
    enable = true;
    systemd = {
      args = "-v";
    };
    settings = {
      windows.enable = true;
      windows.overview.enable = true;
      windows.switch.enable = true;
      windows.switch.modifier = "ctrl";
    };
  };
}
