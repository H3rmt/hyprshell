#!/usr/bin/env bash
set -euxo pipefail

sudo apt update
sudo apt -y install --no-install-recommends \
  meson ninja-build python3 valac \
  libgtk-4-dev libadwaita-1-dev libpango1.0-dev \
  libgtk4-layer-shell-dev gobject-introspection zstd