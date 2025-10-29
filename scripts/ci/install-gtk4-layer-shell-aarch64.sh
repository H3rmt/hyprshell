#!/usr/bin/env bash
set -euxo pipefail

export PKG_CONFIG_ALLOW_CROSS=1
export PKG_CONFIG_PATH=/usr/lib/aarch64-linux-gnu/pkgconfig

if ! grep -q '^Architectures:' /etc/apt/sources.list.d/ubuntu.sources; then
  sed -i '/^Components:/a Architectures: amd64' /etc/apt/sources.list.d/ubuntu.sources
fi
tee /etc/apt/sources.list.d/ubuntu-arm64.list >/dev/null <<'EOF'
deb [arch=arm64] http://ports.ubuntu.com/ubuntu-ports noble main universe multiverse restricted
deb [arch=arm64] http://ports.ubuntu.com/ubuntu-ports noble-updates main universe multiverse restricted
deb [arch=arm64] http://ports.ubuntu.com/ubuntu-ports noble-security main universe multiverse restricted
deb [arch=arm64] http://ports.ubuntu.com/ubuntu-ports noble-backports main universe multiverse restricted
EOF

dpkg --add-architecture arm64 && apt update
apt -y install --no-install-recommends \
  crossbuild-essential-arm64 \
  libgtk-4-dev:arm64 libadwaita-1-dev:arm64 \
  gobject-introspection \
  libgirepository1.0-dev:arm64

cat <<'EOF' > /tmp/aarch64.txt
[binaries]
c = 'aarch64-linux-gnu-gcc'
cpp = 'aarch64-linux-gnu-g++'
ar = 'aarch64-linux-gnu-ar'
strip = 'aarch64-linux-gnu-strip'
pkgconfig = 'pkg-config'

[host_machine]
system = 'linux'
cpu_family = 'aarch64'
cpu = 'armv8'
endian = 'little'
EOF

cd /tmp/gtk4-layer-shell
meson setup build-arm64 --cross-file /tmp/aarch64.txt --prefix=/usr/aarch64-linux-gnu
ninja -C build-arm64
ninja -C build-arm64 install
ldconfig