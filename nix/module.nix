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
    mkMerge
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

    declarative = mkEnableOption' ''
      Declarative configuration of Hyprshell settings
      If enabled, then the configuration will be generated from `programs.hyprshell.settings`
      Otherwise `programs.hyprshell.configFile` will be used
    '';

    configFile = mkOption {
      description = ''
        File containing Hyprshell configuration
        If value is a string, then that will be used as the contents of the file
      '';
      type = nullOr (either path lines);
      default = null;
    };

    settings = {
      layerrules = mkEnableOption' "Enable layer rules";
      launcher = {
        enable = mkEnableOption' "Enable app launcher";
        width = mkOpt "Launcher width" int 650;
        max_items = mkOpt "Max shown items" int 5;
        animate_launch_ms = mkOpt "Launcher close duration" int 450;
        default_terminal = mkOption {
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
          applications = {
            enable = mkEnableOption' "Open applications";
            cache = mkOpt "Run Cache weeks" int 4;
            execs = mkEnableOption' "Show execs";
          };
          websearch = {
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

      window =
        let
          build = key: {
            open = {
              modifier = mkOption {
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
            navigate = {
              forward = mkOption {
                description = "Key to navigate forwards";
                type = str;
                default = "tab";
              };
              backward = mkOption {
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
                    "same_class"
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
          scale = mkOpt "Scale" float 8.5;
          size_factor = mkOpt "Size Factor" float 6.5;
          workspaces_per_row = mkOpt "Workspaces per row" int 5;
          strip_html_from_workspace_title = mkEnableOption' "Strip HTML from workspace title";
          overview = build true;
          switcher = build false;
        };
    };
  };

  config = mkIf cfg.enable (mkMerge [
    {
      assertions = [
        {
          assertion = cfg.declarative || cfg.configFile != null;
          message = "Configuration must be specified either declaratively or by using `configFile`";
        }
      ];

      home.packages = [ cfg.package ];

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

      xdg.configFile =
        let
          source' =
            conf: file:
            if (isPath conf || isStorePath conf) then cfg.style else pkgs.writeText "hyprshell/${file}" conf;
        in
        {
          "hyprshell/styles.css" = mkIf (cfg.style != null) {
            source = source' cfg.style "styles.css";
          };
          "hyprshell/config.ron" = mkIf (!cfg.declarative) {
            source = source' cfg.configFile "config.ron";
          };
        };
    }

    (mkIf cfg.declarative (
      with cfg.settings;
      {
        assertions = [
          {
            assertion = with launcher; !enable || (default_terminal != null);
            message = "Default terminal must be set";
          }
        ];

        xdg.configFile."hyprshell/config.ron".text =
          let
            launcher' =
              with launcher;
              if launcher.enable then
                ''
                  (
                    default_terminal: "${default_terminal}",
                    width: ${toString width},
                    max_items: ${toString max_items},
                    animate_launch_ms: ${toString animate_launch_ms},
                    plugins: [
                      ${optionalString plugins.calc "Calc(),"}
                      ${optionalString plugins.shell "Shell(),"}
                      ${optionalString plugins.terminal "Terminal(),"}
                      ${optionalString plugins.applications.enable ''
                        Applications(
                          run_cache_weeks: ${toString plugins.applications.cache},
                          show_execs: ${boolStr plugins.applications.execs},
                        ),
                      ''}
                      ${optionalString plugins.websearch.enable ''
                        WebSearch([
                          ${concatStringsSep "" (
                            map (engine: ''
                              (
                                ${concatStringsSep "," (mapAttrsToList (name: value: "${name}: \"${value}\"") engine)},
                              ),
                            '') plugins.websearch.engines
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
                  modifier: ${conf.open.modifier},
                  ${optionalString key "key: \"${conf.open.key}\","}
                ),
                navigate: (
                  forward: "${conf.navigate.forward}",
                  backward: ${conf.navigate.backward},
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
                scale: ${toString window.scale},
                size_factor: ${toString window.size_factor},
                workspaces_per_row: ${toString window.workspaces_per_row},
                strip_html_from_workspace_title: ${boolStr window.strip_html_from_workspace_title},
                overview: ${build window.overview true},
                switch: ${build window.switcher false},
              ),
            )
          '';
      }
    ))
  ]);
}
