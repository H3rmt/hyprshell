#!/usr/bin/env bash
set -euo pipefail

cargoBuildLog=$(mktemp cargoBuildLogXXXX.json)

# Function to build with a specific combination of features
build_with_features() {
  local feature_combination="$1"
  local iteration="$2"
  local start_time=$(date +%s.%N)

  if [[ -z "$feature_combination" ]]; then
    echo "[$iteration] Running clippy without any features..."
    cargo clippy --profile dev --locked --no-default-features --message-format json-render-diagnostics > "$cargoBuildLog"
  else
    echo "[$iteration] Building with features: $feature_combination"
    cargo clippy --profile dev --locked --no-default-features --features "$feature_combination" --message-format json-render-diagnostics > "$cargoBuildLog"
  fi

  local duration=$(awk "BEGIN {print $(date +%s.%N) - $start_time}")
  printf "  took %.2f seconds\n" "$duration"
}


test_feature_combinations() {
  local -n features_ref=$1
  local num_features=${#features_ref[@]}
  echo "num_features: $num_features, iterations: $((1 << num_features))"
  for ((i = 0; i < (1 << num_features); i++)); do
    combination=()
    for ((j = num_features - 1; j >= 0; j--)); do
      if ((i & (1 << j))); then
        combination+=("${features_ref[j]}")
      fi
    done
    build_with_features "$(IFS=,; printf '%s' "${combination[*]}")" "$i"
  done
  echo "all features tested"
}

build_with_features "default" default
build_with_features "slim" slim

declare -a crypt_features=("clipboard_encrypt_chacha20poly1305" "clipboard_encrypt_aes_gcm" "clipboard_compress_zstd" "clipboard_compress_lz4")
test_feature_combinations crypt_features

declare -a features=("generate_config_command" "ci_config_check" "launcher_calc" "debug_command" "json5_config")
test_feature_combinations features