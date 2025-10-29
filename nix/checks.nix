{
  self,
  pkgs,
  craneLib,
  buildLib,
  home-manager,
  ...
}:
let
  customLib = import ./util.nix { lib = pkgs.lib; };
in
rec {
  hyprshell-build-deps = buildLib.cargoFullArtifacts;
  hyprshell-config-check = craneLib.buildPackage (
    buildLib.commonArgsFullCached
    // {
      cargoBuildCommand = "cargo build --profile dev --locked";
      cargoExtraArgs = "--features ci_config_check";
    }
  );
  hyprshell-test = craneLib.cargoNextest (
    buildLib.commonArgsFullCached
    // {
      doCheck = true;
      CARGO_PROFILE = "dev";
      cargoNextestExtraArgs = "--all-targets --all-features -p hyprshell-config-lib -p hyprshell-core-lib -p hyprshell-exec-lib -p hyprshell-launcher-lib -p hyprshell-windows-lib -p hyprshell-hyprland-plugin -p hyprshell-clipboard-lib -p hyprshell-config-edit-lib";
    }
  );
  hyprshell-clippy = craneLib.cargoClippy (
    buildLib.commonArgsFullCached
    // {
      CARGO_PROFILE = "dev";
      cargoClippyExtraArgs = "--all-targets --all-features  -p hyprshell-config-lib -p hyprshell-core-lib -p hyprshell-exec-lib -p hyprshell-launcher-lib -p hyprshell-windows-lib -p hyprshell-hyprland-plugin  -p hyprshell-clipboard-lib -p hyprshell-config-edit-lib -- --deny warnings";
    }
  );
  hyprshell-fmt = craneLib.cargoFmt buildLib.commonArgs;
  hyprshell-check-nix-configs =
    let
      base-modules = [
        self.homeModules.hyprshell
        {
          home.stateVersion = "24.05";
          home.username = "test";
          home.homeDirectory = "/home/test";
        }
      ];
      empty-config = home-manager.lib.homeManagerConfiguration {
        inherit pkgs;
        modules = base-modules;
      };
      test-config = home-manager.lib.homeManagerConfiguration {
        inherit pkgs;
        modules = base-modules ++ [
          {
            programs.hyprshell.settings = {
              windows.enable = true;
              windows.overview.enable = true;
              windows.switch.enable = true;
            };
          }
        ];
      };
    in
    pkgs.runCommand "hyprshell-check-nix-configs" { } ''
      set -euo pipefail

      TMP=$(mktemp -d)
      trap 'rm -r "$TMP"' EXIT

      touch "$TMP/test.json"
      echo "test json created at $TMP"
      cat <<EOF> "$TMP/test.json"
      ${builtins.toJSON (
        customLib.filterDisabledAndDropEnable empty-config.config.programs.hyprshell.settings
      )}
      EOF
      chmod 444 "$TMP/test.json"
      ${pkgs.jq}/bin/jq < "$TMP/test.json"
      echo "test json written to $TMP/test.json"

      ${hyprshell-config-check}/bin/hyprshell -vv config check-if-default -c "$TMP/test.json"

      touch "$TMP/test-2.json"
      echo "test-2 json created at $TMP"
      cat <<EOF> "$TMP/test-2.json"
      ${builtins.toJSON (
        customLib.filterDisabledAndDropEnable test-config.config.programs.hyprshell.settings
      )}
      EOF
      chmod 444 "$TMP/test-2.json"
      ${pkgs.jq}/bin/jq < "$TMP/test-2.json"
      echo "test-2 json written to $TMP/test-2.json"

      ${hyprshell-config-check}/bin/hyprshell -vv config check-if-full -c "$TMP/test-2.json"

      mkdir "$out"
    '';
  hyprshell-check-all-feature-combinations = craneLib.mkCargoDerivation (
    buildLib.commonArgsFullCached
    // {
      pnameSuffix = "-check-all-feature-combinations";
      nativeBuildInputs = [
        pkgs.bash
        pkgs.clippy
      ]
      ++ buildLib.commonArgs.nativeBuildInputs;
      cargoClippyExtraArgs = "";
      buildPhaseCargoCommand = ''
        bash ${../scripts/nix/check-all-feature-combinations.sh}
      '';
    }
  );
}
