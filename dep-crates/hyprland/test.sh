#!/usr/bin/env bash
set -euo pipefail

export RUSTFLAGS=-Awarnings

# Define the features as an array
declare -a features=("listener" "dispatch" "data" "keyword" "config" "ctl")
declare -a async=("tokio" "async-lite" "")
# Get the total number of features
num_features=${#features[@]}
num_async=${#async[@]}

# print total number of features and async options (num_features * num_async)
echo "Total number of features: $num_features, combination: $((1 << num_features))"
echo "Total number of async options: $num_async, combination: $((num_async * (1 << num_features)))"

# Function to build with a specific combination of features
build_with_features() {
  local feature_combination="$1"
  local async_feature="$2"
  local iteration="$3"

  if [[ -z "$feature_combination" ]]; then
    echo "[$iteration/$async_feature] Building without any features..."
    cargo build --no-default-features --quiet
  else
    echo "[$iteration/$async_feature] Building with features: $feature_combination"
    cargo build --no-default-features --quiet --features="$feature_combination" --features="$async_feature"
  fi
}

# Generate all combinations of features
for async_feature in "${async[@]}"; do
  echo ""
  echo "async: $async_feature, num_features: $num_features, iterations: $((1 << num_features))"
  for ((i = 0; i < (1 << num_features); i++)); do
    combination=()
    for ((j = num_features - 1; j >= 0; j--)); do
      if ((i & (1 << j))); then
        combination+=("${features[j]}")
      fi
    done
    build_with_features "$(IFS=,; printf '%s' "${combination[*]}")" "$async_feature" "$i"
  done
done

# some special cases
build_with_features "ahash" "" "special-case-1"
build_with_features "ahash, listener, dispatch, data, keyword, config, ctl" "" "special-case-1"
build_with_features "parking_lot" "" "special-case-2"
build_with_features "parking_lot" "tokio" "special-case-3"
build_with_features "unsafe-impl" "" "special-case-4"

echo "all features tested with all async tested"