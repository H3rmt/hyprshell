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
  test-config = home-manager.lib.homeManagerConfiguration {
    inherit pkgs;
    modules = [
      self.homeModules.hyprshell
      {
        home.stateVersion = "24.05";
        home.username = "test";
        home.homeDirectory = "/home/test";
      }
    ];
  };
in
rec {
  hyprshell-default-check = craneLib.buildPackage (
    buildLib.commonArgsCachedRelease
    // {
      cargoExtraArgs = "--features config_check_is_default";
    }
  );
  hyprshell-test = craneLib.cargoNextest (
    buildLib.commonArgsCachedRelease
    // {
      doCheck = true;
      cargoNextestExtraArgs = "--workspace";
    }
  );
  hyprshell-clippy = craneLib.cargoClippy (
    buildLib.commonArgsCachedRelease
    // {
      buildPhaseCargoCommand = "cargo clippyy"; # no release check
      cargoClippyExtraArgs = "--all-targets -- --deny warnings";
    }
  );
  hyprshell-fmt = craneLib.cargoFmt buildLib.commonArgs;
  check-nix-config = pkgs.runCommand "check-nix-config" { } ''
    TMP=$(mktemp -d)
    touch "$TMP/test.json"
    echo "test json created at $TMP"
    cat <<EOF> "$TMP/test.json"
    ${builtins.toJSON (
      customLib.filterDisabledAndDropEnable test-config.config.programs.hyprshell.settings
    )}
    EOF
    ${pkgs.jq}/bin/jq < "$TMP/test.json"
    ${hyprshell-default-check}/bin/hyprshell -vv config check-if-default -c "$TMP/test.json" && (mkdir "$out")
    rm -r "$TMP"
  '';
  check-all-feature-combinations = craneLib.mkCargoDerivation (
    buildLib.commonArgsCachedRelease
    // {
      pnameSuffix = "-check-all-feature-combinations";
      nativeBuildInputs = [
        pkgs.bash
        pkgs.clippy
      ] ++ buildLib.commonArgs.nativeBuildInputs;
      cargoClippyExtraArgs = "";
      buildPhaseCargoCommand = ''
        cargoBuildLog=$(mktemp cargoBuildLogXXXX.json)

        # Define the features as an array
        # features=$(cargo metadata --format-version 1 --no-deps | ${pkgs.jq}/bin/jq -r '.packages[4].features | keys[]')
        declare -a features=("generate_config_command" "config_check_is_default" "launcher_calc" "debug_command")

        # Get the total number of features
        num_features=''${#features[@]}

        # Function to build with a specific combination of features
        build_with_features() {
          local feature_combination="$1"
          local iteration="$2"
          local start_time=$(date +%s.%N)

          if [[ -z "$feature_combination" ]]; then
            echo "[$iteration] Running clippy without any features..."
            cargo clippy --release --locked --no-default-features --message-format json-render-diagnostics > "$cargoBuildLog"
          else
            echo "[$iteration] Building with features: $feature_combination"
            cargo clippy --release --locked --no-default-features --features "$feature_combination" --message-format json-render-diagnostics > "$cargoBuildLog"
          fi

          local duration=$(awk "BEGIN {print $(date +%s.%N) - $start_time}")
          printf "  took %.2f seconds\n" "$duration"
        }

        echo "num_features: $num_features, iterations: $((1 << num_features))"
        for ((i = 0; i < (1 << num_features); i++)); do
          combination=()
          for ((j = num_features - 1; j >= 0; j--)); do
            if ((i & (1 << j))); then
              combination+=("''${features[j]}")
            fi
          done
          build_with_features "$(IFS=,; printf '%s' "''${combination[*]}")" "$i"
        done
        echo "all features tested"
      '';
    }
  );
}
