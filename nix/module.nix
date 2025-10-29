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
      default = null;
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
          key = mkOpt "Key to open overview" str "super_l";
          modifier = mkOpt "Modifier key" (enum [
            "alt"
            "ctrl"
            "super"
          ]) "super";

          filter_by = mkOpt "Filter by" (listOf (enum [
            "same_class"
            "current_monitor"
            "current_workspace"
          ])) [ ];
          hide_filtered = mkOpt "Hide filtered windows" bool false;
          launcher = {
            width = mkOpt "Launcher width" int 650;
            launch_modifier = mkOpt "Launch modifier" (enum [
              "alt"
              "ctrl"
              "super"
            ]) "ctrl";
            max_items = mkOpt "Max shown items" int 5;
            default_terminal = mkOpt "Default terminal" (nullOr (str)) null;
            show_when_empty = mkOpt "Show entries when no text is entered" bool true;

            plugins = {
              applications = {
                enable = mkOpt "Open applications" bool true;
                run_cache_weeks = mkOpt "Run Cache weeks" int 8;
                show_execs = mkOpt "Show execs" bool true;
                show_actions_submenu = mkOpt "Show actions submenu" bool true;
              };
              calc = {
                enable = mkOpt "Enable calculator" bool true;
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
                            custom = lib.mkOption {
                              description = "Custom action object";
                              type = submodule {
                                options = {
                                  names = lib.mkOption {
                                    description = "Names for the action";
                                    type = listOf str;
                                    default = [ ];
                                  };
                                  details = mkOpt "Details about the action" str null;
                                  command = mkOpt "Command to run" str null;
                                  icon = mkOpt "Icon name" str null;
                                };
                              };
                              default = { };
                            };
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
                        custom = {
                          names = [
                            "Kill"
                            "Stop"
                          ];
                          details = "Kill or stop a process by name";
                          command = "pkill \"{}\" && notify-send hyprshell \"stopped {}\"";
                          icon = "remove";
                        };
                      }
                      {
                        custom = {
                          names = [
                            "Reload Hyprshell"
                          ];
                          details = "Reload Hyprshell";
                          command = "sleep 1; hyprshell socat '\"Restart\"'";
                          icon = "system-restart";
                        };
                      }
                    ];
              };
            };
          };
        };
        switch = {
          enable = mkOpt "Enable recent window switcher" bool true;
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
        };
      };
    };
  };

  config = lib.mkIf cfg.enable ({
    home.packages = [ cfg.package ];

    systemd.user.services.hyprshell = lib.mkIf cfg.systemd.enable {
      Unit = {
        Description = "Starts Hyprshell daemon";
        After = [ cfg.systemd.target ];
      };
      Service = {
        Type = "simple";
        ExecStart = "${lib.getExe cfg.package} run ${cfg.systemd.args}";
        Restart = "on-failure";
      };
      Install.WantedBy = [ cfg.systemd.target ];
    };

    xdg.configFile."hyprshell/config.json" =
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
          text = builtins.toJSON (customLib.filterDisabledAndDropEnable cfg.settings);
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
  });
}
