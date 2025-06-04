{
  self,
}:
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

  cfg = config.programs.hyprshell;
  mkOpt =
    description: type: default:
    lib.mkOption { inherit description type default; };
  filterDisabledAndDropEnable =
    value:
    if lib.isAttrs value then
      if value ? enable && value.enable == false then
        null
      else
        lib.filterAttrs (k: v: v != null && k != "enable") (
          lib.mapAttrs (_: filterDisabledAndDropEnable) value
        )
    else if lib.isList value then
      lib.filter (v: v != null) (map filterDisabledAndDropEnable value)
    else
      value;
in
{
  options.programs.hyprshell = {
    enable = lib.mkEnableOption "Configure Hyprshell";

    package = lib.mkOption {
      description = "The Hyprshell package";
      type = package;
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
      layerrules = mkOpt "Enable layer rules" bool true;
      kill_bind = mkOpt "Key to kill hyprshell if it is stuck" str "ctrl+shift+alt, h";
      launcher = {
        enable = mkOpt "Enable app launcher" bool true;
        width = mkOpt "Launcher width" int 650;
        max_items = mkOpt "Max shown items" int 5;
        animate_launch_ms = mkOpt "Launcher close duration" int 250;
        default_terminal = mkOpt "Default terminal" (nullOr (str)) null;
        show_when_empty = mkOpt "Show entries when no text is entered" bool true;

        plugins = {
          applications = {
            enable = mkOpt "Open applications" bool true;
            run_cache_weeks = mkOpt "Run Cache weeks" int 4;
            show_execs = mkOpt "Show execs" bool true;
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
                ];
          };
        };
      };

      windows = {
        enable = mkOpt "Enable windows (overview, switch)" bool true;
        scale = mkOpt "Scale" float 8.5 // {
          apply = num: if (num >= 0 && num <= 15) then num else throw "Value must be between 0 and 15";
        };
        workspaces_per_row = mkOpt "Workspaces per row" int 5;
        strip_html_from_workspace_title = mkOpt "Strip HTML from workspace title" bool true;
        overview = {
          enable = mkOpt "Enable overview" bool true;
          open = {
            key = mkOpt "Key to open overview" str "super";
            modifier = mkOpt "Modifier key" (enum [
              "alt"
              "ctrl"
              "super"
              "shift"
            ]) "super";
          };
          navigate = {
            forward = mkOpt "Key to navigate forwards" str "tab";
            reverse = {
              key = mkOpt "Key to navigate backwards (mutually exclusive with mod)" (nullOr (str)) null // {
                example = "tab";
              };
              mod = mkOpt "Modifier to navigate backwards (mutually exclusive with key)" (nullOr (enum [
                "alt"
                "ctrl"
                "super"
                "shift"
              ])) "shift";
            };
          };
          other = {
            filter_by = mkOpt "Filter by" (listOf (enum [
              "same_class"
              "current_monitor"
              "current_workspace"
            ])) [ ];
            hide_filtered = mkOpt "Hide filtered windows" bool false;
          };
        };
        switch = {
          enable = mkOpt "Enable recent window switcher" bool true;
          open = {
            modifier = mkOpt "Modifier key" (enum [
              "alt"
              "ctrl"
              "super"
              "shift"
            ]) "alt";
          };
          navigate = {
            forward = mkOpt "Key to navigate forwards" str "tab";
            reverse = {
              key = mkOpt "Key to navigate backwards (mutually exclusive with mod)" (nullOr (str)) null // {
                example = "tab";
              };
              mod = mkOpt "Modifier to navigate backwards (mutually exclusive with key)" (nullOr (enum [
                "alt"
                "ctrl"
                "super"
                "shift"
              ])) "shift";
            };
          };
          other = {
            filter_by = mkOpt "Filter by" (listOf (enum [
              "same_class"
              "current_monitor"
              "current_workspace"
            ])) [ ];
            hide_filtered = mkOpt "Hide filtered windows" bool true;
          };
        };
      };
    };
  };

  config = lib.mkIf cfg.enable ({
    assertions = [
      {
        assertion = with cfg.settings.windows.overview.navigate.reverse; key == null || mod == null;
        message = "'key' and 'mod' are mutually exclusive";
      }
      {
        assertion = with cfg.settings.windows.switch.navigate.reverse; key == null || mod == null;
        message = "'key' and 'mod' are mutually exclusive";
      }
    ];

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
          text = builtins.toJSON (filterDisabledAndDropEnable cfg.settings);
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
