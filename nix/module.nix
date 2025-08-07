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
      type = package;
      default = self.packages.${pkgs.stdenv.hostPlatform.system}.default;
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
            "shift"
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
              "shift"
            ]) "ctrl";
            max_items = mkOpt "Max shown items" int 5;
            default_terminal = mkOpt "Default terminal" (nullOr (str)) null;
            show_when_empty = mkOpt "Show entries when no text is entered" bool true;

            plugins = {
              applications = {
                enable = mkOpt "Open applications" bool true;
                run_cache_weeks = mkOpt "Run Cache weeks" int 4;
                show_execs = mkOpt "Show execs" bool true;
                show_actions_submenu = mkOpt "Show actions submenu" bool false;
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
            };
          };
        };
        switch = {
          enable = mkOpt "Enable recent window switcher" bool true;
          modifier = mkOpt "Modifier key" (enum [
            "alt"
            "ctrl"
            "super"
            "shift"
          ]) "alt";
          filter_by = mkOpt "Filter by" (listOf (enum [
            "same_class"
            "current_monitor"
            "current_workspace"
          ])) [ ];
          show_workspaces = mkOpt "Show workspaces" bool false;
        };
      };
    };
  };

  config = lib.mkIf cfg.enable ({
    assertions = [
      {
        assertion = if (cfg.package == null) then (if cfg.systemd.enable then false else true) else true;
        message = "Can't set programs.hyprshell.systemd.enable with the package set to null.";
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
