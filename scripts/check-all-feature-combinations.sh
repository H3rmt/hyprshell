#!/usr/bin/env bash
set -euo pipefail

cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.."

# Match the maintained packages checked by xtask, including libraries whose
# feature-gated code would otherwise be treated as an unchecked dependency.
packages=(hyprshell hyprshell-core-lib hyprshell-config-lib hyprshell-exec-lib
  hyprshell-windows-lib hyprshell-launcher-lib hyprshell-config-edit-lib
  hyprshell-clipboard-lib hyprshell-xtask)
package_args=()
for package in "${packages[@]}"; do
  package_args+=(-p "$package")
done

build_with_features() {
  local feature_combination="$1"
  local iteration="$2"
  local start_time=$SECONDS
  local feature_args=()
  if [[ -n "$feature_combination" ]]; then
    feature_args+=(--features "$feature_combination")
  fi
  echo "[$iteration] Checking features: ${feature_combination:-none}"
  cargo clippy --profile dev --locked --all-targets --no-deps \
    "${package_args[@]}" --no-default-features "${feature_args[@]}" -- -D warnings
  printf '  took %s seconds\n' "$((SECONDS - start_time))"
}

test_feature_combinations() {
  local features=("$@")
  local num_features=${#features[@]}
  local i j
  local combination=()
  for ((i = 0; i < (1 << num_features); i++)); do
    combination=()
    for ((j = 0; j < num_features; j++)); do
      if ((i & (1 << j))); then
        combination+=("${features[j]}")
      fi
    done
    build_with_features "$(IFS=,; printf '%s' "${combination[*]}")" "$i"
  done
}

build_with_features default default
build_with_features slim slim
# Independent features from Cargo.toml; default and slim are aliases above.
test_feature_combinations gui_settings_editor ci_config_check launcher_calc json5_config live_windows dev
echo "All feature combinations passed"
