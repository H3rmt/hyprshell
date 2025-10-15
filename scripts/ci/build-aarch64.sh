#!/usr/bin/env bash
set -euxo pipefail

apt update && apt -y install --no-install-recommends \
  libgraphene-1.0-dev:arm64

export PKG_CONFIG_ALLOW_CROSS=1
export PKG_CONFIG_PATH=/usr/lib/aarch64-linux-gnu/pkgconfig:/usr/aarch64-linux-gnu/lib/pkgconfig

rustup target add aarch64-unknown-linux-gnu
CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc cargo build --release --target aarch64-unknown-linux-gnu
ls -lh target/aarch64-unknown-linux-gnu/release/hyprshell

tar --zstd -cf /tmp/hyprshell-aarch64.tar.zst LICENSE -C target/aarch64-unknown-linux-gnu/release hyprshell
ls -lh /tmp/hyprshell-aarch64.tar.zst

CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc cargo build --release --target aarch64-unknown-linux-gnu
ls -lh target/aarch64-unknown-linux-gnu/release/hyprshell

tar --zstd -cf /tmp/hyprshell-aarch64-slim.tar.zst LICENSE -C target/aarch64-unknown-linux-gnu/release hyprshell
ls -lh /tmp/hyprshell-aarch64-slim.tar.zst