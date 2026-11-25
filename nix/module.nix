self:
{
  pkgs,
  config,
  lib,
  ...
}:
let
  inherit (lib.types)
    either
    bool
    float
    int
    enum
    lines
    listOf
    nullOr
    package
    path
    str
    submodule
    ;
  customLib = import ./util.nix { inherit lib; };
  cfg = config.programs.hyprshell;
  mkOpt =
    description: type: default:
    lib.mkOption { inherit description type default; };
in
{
  options.programs.hyprshell = {
    enable = lib.mkEnableOption "hyprshell";

    package = lib.mkOption {
      description = "The Hyprshell package";
      type = nullOr package;
      default = self.packages.${pkgs.stdenv.hostPlatform.system}.hyprshell;
    };

    systemd = {
      enable = lib.mkEnableOption "Hyprshell systemd service" // {
        default = true;
      };
      target = lib.mkOption {
        description = "The systemd target that will automatically start the Hyprshell service";
        type = str;
        default = config.wayland.systemd.target;
      };
      args = lib.mkOption {
        description = "Arguments to pass to the Hyprshell service";
        type = str;
        default = "";
        example = "-vv";
      };
    };

    styleFile = lib.mkOption {
      description = ''
        File containing Hyprshell CSS overrides (either a file or text).
      '';
      type = nullOr (either path lines);
      default = ''
        :root {
            --border-color: rgba(65, 72, 104, 0.6);
            --border-color-active: rgba(122, 162, 247, 1);

            --box-shadow-color: rgba(122, 162, 247, 0.3);
            --box-shadow-multiplier: 2;
            --box-shadow-launcher-color: rgba(122, 162, 247, 0.3);
            --box-shadow-launcher-multiplier: 1;

            --bg-color: rgba(26, 27, 38, 0.9);
            --bg-color-hover: rgba(41, 46, 66, 1);

            --border-radius: 12px;
            --border-size: 2px;
            --border-style: solid;

            --text-color: rgb(211 218 250);

            --window-padding: 3px;
            --bg-window-color: rgba(30, 32, 48, 0.92);
        }
      '';
    };

    configFile = lib.mkOption {
      description = ''
        File containing Hyprshell configuration !JSON! (either a file or text).
        Can be used instead of generated config via settings.
      '';
      type = nullOr (either path lines);
      default = null;
    };

    settings = {
      windows = {
        enable = lib.mkEnableOption "Enable windows (overview, switch)";
        scale = mkOpt "Scale" float 8.5 // {
          apply = num: if (num >= 0 && num <= 15) then num else throw "Value must be between 0 and 15";
        };
        items_per_row = mkOpt "Workspaces per row" int 5;
        overview = {
          enable = lib.mkEnableOption "Enable overview";
          key = mkOpt "Key to open overview" str "Super_L";
          top_offset = mkOpt "Top Offset" int 410;
          modifier = mkOpt "Modifier key" (enum [
            "alt"
            "ctrl"
            "super"
          ]) "super";
          exclude_workspaces = mkOpt "Exclude workspaces regex" str "special:.*";
          filter_by = mkOpt "Filter by" (listOf (enum [
            "same_class"
            "current_monitor"
            "current_workspace"
          ])) [ ];

          launcher = {
            width = mkOpt "Launcher width" int 650;
            launch_modifier = mkOpt "Launch modifier" (enum [
              "alt"
              "ctrl"
              "super"
            ]) "ctrl";
            max_items = mkOpt "Max shown items" int 5;
            default_terminal = mkOpt "Default terminal" (nullOr str) null;
            show_when_empty = mkOpt "Show entries when no text is entered" bool true;

            plugins = {
              applications = {
                enable = mkOpt "Open applications" bool true;
              };
              calc = {
                enable = mkOpt "Enable calculator" bool true;
                prefix = mkOpt "Input Prefix" (nullOr (str)) null;
              };
              shell = {
                enable = mkOpt "Run in Shell" bool false;
              };
              terminal = {
                enable = mkOpt "Run in Terminal" bool true;
              };
              websearch = {
                enable = mkOpt "Web search" bool true;
                engines =
                  mkOpt "Search engines"
                    (listOf (submodule {
                      options = {
                        url = mkOpt "Search engine URL" str null;
                        name = mkOpt "Search engine name" str null;
                        key = mkOpt "Key to use for search engine" str null // {
                          apply = key: if (builtins.stringLength key) != 1 then throw "Key must be single character" else key;
                        };
                      };
                    }))
                    [
                      {
                        url = "https://www.google.com/search?q={}";
                        name = "Google";
                        key = "g";
                      }
                      {
                        url = "https://en.wikipedia.org/wiki/Special:Search?search={}";
                        name = "Wikipedia";
                        key = "w";
                      }
                    ];
              };
              path = {
                enable = mkOpt "Open in File manager" bool true;
              };
              actions = {
                enable = mkOpt "Run action" bool true;
                actions =
                  mkOpt "Actions"
                    (listOf (
                      either
                        (enum [
                          "lock_screen"
                          "hibernate"
                          "logout"
                          "reboot"
                          "shutdown"
                          "suspend"
                        ])
                        (submodule {
                          options = {
                            name = mkOpt "Name for the action" str null;
                            details = mkOpt "Details about the action" str null;
                            command = mkOpt "Command to run" str null;
                            icon = mkOpt "Icon name" str null;
                          };
                        })
                    ))
                    [
                      "lock_screen"
                      "hibernate"
                      "logout"
                      "reboot"
                      "shutdown"
                      "suspend"
                      {
                        name = "Kill";
                        details = "Kill or stop a process by name";
                        command = "pkill \"{}\" && notify-send hyprshell \"stopped {}\"";
                        icon = "remove";
                      }
                      {
                        name = "Reload Hyprshell";
                        details = "Reload Hyprshell";
                        command = "sleep 1; hyprshell internal socat '\"Restart\"'";
                        icon = "system-restart";
                      }
                    ];
              };
            };
          };
        };
        switch = {
          enable = mkOpt "Enable recent window switcher" bool true;
          key = mkOpt "Key to open switch" str "Tab";
          modifier = mkOpt "Modifier key" (enum [
            "alt"
            "ctrl"
            "super"
          ]) "alt";
          filter_by = mkOpt "Filter by" (listOf (enum [
            "same_class"
            "current_monitor"
            "current_workspace"
          ])) [ "current_monitor" ];
          switch_workspaces = mkOpt "Switch workspaces" bool false;
          kill_key = mkOpt "Key to kill window" str "q";
          exclude_workspaces = mkOpt "Exclude workspaces regex" str "";
        };
        switch_2 = {
          enable = mkOpt "Enable recent window switcher" bool false;
          key = mkOpt "Key to open switch" str "Tab";
          modifier = mkOpt "Modifier key" (enum [
            "alt"
            "ctrl"
            "super"
          ]) "alt";
          filter_by = mkOpt "Filter by" (listOf (enum [
            "same_class"
            "current_monitor"
            "current_workspace"
          ])) [ "current_monitor" ];
          switch_workspaces = mkOpt "Switch workspaces" bool false;
          kill_key = mkOpt "Key to kill window" str "q";
          exclude_workspaces = mkOpt "Exclude workspaces regex" str "";
        };
      };
    };
  };

  config = lib.mkIf cfg.enable {
    home.packages = [ cfg.package ];

    systemd.user.services.hyprshell = lib.mkIf cfg.systemd.enable {
      Unit = {
        Description = "Starts Hyprshell daemon";
        PartOf = [ cfg.systemd.target ];
        After = [ cfg.systemd.target ];
      };
      Service = {
        Type = "simple";
        ExecStartPre = "/bin/sh -c '[ \"$XDG_CURRENT_DESKTOP\" = \"Hyprland\" ] || exit 0'";
        ExecStart = "${lib.getExe cfg.package} run ${cfg.systemd.args}";
        Restart = "on-failure";
        RestartSec = "20";
      };
      Install.WantedBy = [ cfg.systemd.target ];
    };

    # TODO Check
    xdg.configFile."hyprshell/config.toml" =
      let
        tomlFormat = pkgs.formats.toml { };
      in
      if (lib.isPath cfg.configFile || lib.isStorePath cfg.configFile) then
        {
          source = cfg.configFile;
        }
      else if (builtins.isString cfg.configFile) then
        {
          text = cfg.configFile;
        }
      else
        {
          source = tomlFormat.generate "config.toml" (
            (customLib.filterDisabledAndDropEnable cfg.settings) // { version = 5; }
          );
        };

    xdg.configFile."hyprshell/styles.css" =
      if (lib.isPath cfg.styleFile || lib.isStorePath cfg.styleFile) then
        {
          source = cfg.styleFile;
        }
      else if (builtins.isString cfg.styleFile) then
        {
          text = cfg.styleFile;
        }
      else
        {
          text = "";
        };
  };
}
