#!/usr/bin/env bash
set -euxo pipefail

rustup target add x86_64-unknown-linux-gnu
cargo build --release --target x86_64-unknown-linux-gnu
ls -lh target/x86_64-unknown-linux-gnu/release/hyprshell

tar --zstd -cf /tmp/hyprshell-x86_64.tar.zst LICENSE -C target/x86_64-unknown-linux-gnu/release hyprshell
ls -lh /tmp/hyprshell-x86_64.tar.zst

cargo build --release --target x86_64-unknown-linux-gnu --no-default-features --features slim
ls -lh target/x86_64-unknown-linux-gnu/release/hyprshell

tar --zstd -cf /tmp/hyprshell-x86_64-slim.tar.zst LICENSE -C target/x86_64-unknown-linux-gnu/release hyprshell
ls -lh /tmp/hyprshell-x86_64-slim.tar.zst