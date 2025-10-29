#!/usr/bin/env bash
set -euxo pipefail

apt update
apt -y install --no-install-recommends \
  meson ninja-build python3 valac \
  libgtk-4-dev libadwaita-1-dev \
  gobject-introspection

git clone https://github.com/wmww/gtk4-layer-shell /tmp/gtk4-layer-shell
cd /tmp/gtk4-layer-shell
meson setup build
ninja -C build
ninja -C build install
ldconfig