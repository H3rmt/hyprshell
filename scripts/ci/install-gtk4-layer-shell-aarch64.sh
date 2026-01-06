#!/usr/bin/env bash
set -euxo pipefail

export PKG_CONFIG_ALLOW_CROSS=1
export PKG_CONFIG_PATH=/usr/lib/aarch64-linux-gnu/pkgconfig

sudo tee /etc/apt/sources.list.d/debian-arm64.list >/dev/null <<'EOF'
deb [arch=arm64] http://deb.debian.org/debian testing main contrib non-free
EOF

sudo apt remove -y libpango1.0-dev:amd64 libgtk-4-dev:amd64 libadwaita-1-dev:amd64
sudo apt autoremove -y

sudo dpkg --add-architecture arm64 && sudo apt update
sudo apt -y install --no-install-recommends \
  crossbuild-essential-arm64 \
  libgtk-4-dev:arm64 libadwaita-1-dev:arm64 libpango1.0-dev:arm64 \
  libgirepository1.0-dev:arm64 libgtk4-layer-shell-dev:arm64 \
  gobject-introspection