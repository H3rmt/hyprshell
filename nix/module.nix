self:
{
  config,
  pkgs,
  lib,
  ...
}:
let
  inherit (builtins)
    concatStringsSep
    isPath
    map
    readFile
    stringLength
    throw
    toString
    ;
  inherit (lib)
    getExe
    isStorePath
    mapAttrsToList
    mkEnableOption
    mkIf
    mkOption
    optionalString
    types
    ;
  inherit (types)
    either
    float
    int
    lines
    listOf
    nullOr
    package
    path
    str
    submodule
    ;

  boolStr = opt: if opt then "true" else "false";
  mkEnableOption' = _: mkEnableOption _ // { default = true; };
  mkOpt =
    description: type: default:
    mkOption { inherit description type default; };
  cfg = config.programs.hyprshell;
in
{
  options.programs.hyprshell = {
    enable = mkEnableOption "Configure Hyprshell";

    package = mkOption {
      description = "The Hyprshell package";
      type = package;
      default = self.packages.${pkgs.stdenv.hostPlatform.system}.hyprshell;
    };

    systemd = {
      enable = mkEnableOption "Hyprshell systemd service";
      target = mkOption {
        description = "The systemd target that will automatically start the Hyprshell service";
        type = str;
        default = config.wayland.systemd.target;
      };
    };

    style = mkOption {
      description = ''
        CSS style of Hyprshell
        If value is a path, then that will be used as the CSS file
      '';
      type = nullOr (either path lines);
      default = readFile ../core-lib/src/config/generate/default.css;
    };

    settings = {
      layerrules = mkEnableOption' "Enable layer rules";
      launcher = {
        enable = mkEnableOption' "Enable app launcher";
        items = mkOpt "Max items" int 5;
        width = mkOpt "Width" int 650;
        terminal = mkOption {
          description = "Default terminal";
          type = types.nullOr (
            types.enum [
              "alacritty"
              "console"
              "foot"
              "kitty"
              "lilyterm"
              "qterminal"
              "tilix"
              "wezterm"
            ]
          );
          default = null;
        };

        plugins = {
          calc = mkEnableOption' "Calculator";
          shell = mkEnableOption' "Run in Shell";
          terminal = mkEnableOption' "Run in Terminal";
          apps = {
            enable = mkEnableOption' "Open applications";
            cache = mkOpt "Run Cache weeks" int 4;
            execs = mkEnableOption' "Show execs";
          };
          web = {
            enable = mkEnableOption' "Web search";
            engines = mkOption {
              description = "Search engines";
              type = listOf (submodule {
                options = {
                  url = mkOption {
                    description = "Search engine URL";
                    type = str;
                  };
                  name = mkOption {
                    description = "Name of search engine";
                    type = str;
                  };
                  key = mkOption {
                    description = "Key to use for search engine";
                    type = str;
                    apply = key: if (stringLength key) != 1 then throw "Key must be single character" else key;
                  };
                };
              });
              default = [ ];
              example = [
                {
                  url = "https://www.google.com/search?q={}";
                  name = "Google";
                  key = "g";
                }
              ];
            };
          };
        };
      };

      windows =
        let
          build = key: {
            open = {
              mod = mkOption {
                description = "Modifier key";
                type = types.nullOr (
                  types.enum [
                    "alt"
                    "ctrl"
                    "super"
                    "shift"
                  ]
                );
                default = null;
                apply = mod: if (mod != null) then mod else throw "Modifier key must be set";
              };
              key =
                if key then
                  mkOption {
                    description = "Key to open overview";
                    type = str;
                    default = "tab";
                  }
                else
                  { };
            };
            nav = {
              forward = mkOption {
                description = "Key to navigate forwards";
                type = str;
                default = "tab";
              };
              reverse = mkOption {
                description = "Key to navigate backwards";
                type = str;
                default = "Mod(shift)";
                example = "Key(grave)";
              };
            };
            filter = {
              hide = mkEnableOption "Hide filtered windows";
              by = mkOption {
                description = "Filter by";
                type = listOf (
                  types.enum [
                    "current_monitor"
                    "current_workspace"
                  ]
                );
                default = [ ];
              };
            };
          };
        in
        {
          size = mkOpt "Size Factor" float 6.5;
          scale = mkOpt "Scale" float 8.5;
          numWorkspaces = mkOpt "Workspaces per row" int 5;
          stripHtml = mkEnableOption' "Strip HTML from workspace title";
          overview = build true;
          switcher = build false;
        };
    };
  };

  config = mkIf cfg.enable {
    assertions = [
      {
        assertion = with cfg.settings.launcher; !enable || (terminal != null);
        message = "Default terminal must be set";
      }
    ];

    home.packages = [ cfg.package ];

    xdg.configFile = {
      "hyprshell/styles.css" = mkIf (cfg.style != null) {
        source =
          if isPath cfg.style || isStorePath cfg.style then
            cfg.style
          else
            pkgs.writeText "hyprshell/styles.css" cfg.style;
      };

      "hyprshell/config.ron".text =
        with cfg.settings;
        let
          launcher' =
            with launcher;
            if launcher.enable then
              ''
                (
                  default_terminal: "${terminal}",
                  width: ${toString width},
                  max_items: ${toString items},
                  plugins: [
                    ${optionalString plugins.calc "Calc(),"}
                    ${optionalString plugins.shell "Shell(),"}
                    ${optionalString plugins.terminal "Terminal(),"}
                    ${optionalString plugins.apps.enable ''
                      Applications(
                        run_cache_weeks: ${toString plugins.apps.cache},
                        show_execs: ${boolStr plugins.apps.execs},
                      ),
                    ''}
                    ${optionalString plugins.web.enable ''
                      WebSearch([
                        ${concatStringsSep "" (
                          map (engine: ''
                            (
                              ${concatStringsSep "," (mapAttrsToList (name: value: "${name}: \"${value}\"") engine)},
                            ),
                          '') plugins.web.engines
                        )}
                      ]),
                    ''}
                  ],
                )
              ''
            else
              "None";

          build = conf: key: ''
            (
              open: (
                modifier: ${conf.open.mod},
                ${optionalString key "key: \"${conf.open.key}\","}
              ),
              navigate: (
                forward: "${conf.nav.forward}",
                reverse: ${conf.nav.reverse},
              ),
              other: (
                hide_filtered: ${boolStr conf.filter.hide},
                filter_by: [${concatStringsSep "," conf.filter.by}],
              ),
            )
          '';
        in
        ''
          (
            layerrules: ${boolStr layerrules},
            launcher: ${launcher'},
            windows: (
              scale: ${toString windows.scale},
              size_factor: ${toString windows.size},
              workspaces_per_row: ${toString windows.numWorkspaces},
              strip_html_from_workspace_title: ${boolStr windows.stripHtml},
              overview: ${build windows.overview true},
              switch: ${build windows.switcher false},
            ),
          )
        '';
    };

    systemd.user.services.hyprshell = mkIf cfg.systemd.enable {
      Install.WantedBy = [ cfg.systemd.target ];
      Unit = {
        Description = "Starts Hyprshell daemon";
        PartOf = [ cfg.systemd.target ];
        After = [ cfg.systemd.target ];
      };
      Service = {
        ExecStart = "${getExe cfg.package} run";
        Type = "simple";
        Restart = "on-failure";
        RestartSec = 1;
      };
    };
  };
}
