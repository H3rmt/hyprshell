#!/usr/bin/env bash
set -euo pipefail
# Supply the virtual-keyboard protocol XML from wlr-protocols.
protocol=${1:?path to virtual-keyboard-unstable-v1.xml}
out=${2:?output directory}
mkdir -p "$out"
source_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
wayland-scanner client-header "$protocol" "$out/virtual-keyboard.h"
wayland-scanner private-code "$protocol" "$out/virtual-keyboard.c"
cc -I"$out" "$source_dir/keyboard.c" "$out/virtual-keyboard.c" $(pkg-config --cflags --libs wayland-client xkbcommon) -o "$out/keyboard"
cc "$source_dir/windows.c" $(pkg-config --cflags --libs gtk4) -o "$out/windows"
