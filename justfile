project_dir := justfile_directory()

default:
    @just --list --justfile {{ justfile() }}

[group('security')]
audit:
    #!/usr/bin/env bash
    if ! command -v cargo-audit >/dev/null 2>&1; then
        echo "cargo-audit not found, installing..."
        if ! command -v cargo binstall >/dev/null 2>&1; then
          cargo install --locked cargo-audit
        else
          echo "installing with cargo binstall"
          cargo binstall cargo-outdated
        fi
    fi
    cargo audit

[group('security')]
outdated:
    #!/usr/bin/env bash
    if ! command -v cargo-outdated >/dev/null 2>&1; then
        echo "cargo-outdated not found, installing..."
        if ! command -v cargo binstall >/dev/null 2>&1; then
          cargo install --locked cargo-outdated
        else
          echo "installing with cargo binstall"
          cargo binstall cargo-outdated
        fi
    fi
    cargo outdated

[group('security')]
shear:
    #!/usr/bin/env bash
    if ! command -v cargo-shear >/dev/null 2>&1; then
        echo "cargo-shear not found, installing..."
        if ! command -v cargo binstall >/dev/null 2>&1; then
          cargo install --locked cargo-shear
        else
          echo "installing with cargo binstall"
          cargo binstall cargo-shear
        fi
    fi
    cargo shear

[group('security')]
bloat:
    #!/usr/bin/env bash
    if ! command -v cargo-bloat >/dev/null 2>&1; then
        echo "cargo-bloat not found, installing..."
        if ! command -v cargo binstall >/dev/null 2>&1; then
          cargo install --locked cargo-bloat
        else
          echo "installing with cargo binstall"
          cargo binstall cargo-bloat
        fi
    fi
    cargo bloat --release

[group('security')]
about:
    #!/usr/bin/env bash
    if ! command -v cargo-about >/dev/null 2>&1; then
        echo "cargo-about not found, installing..."
        if ! command -v cargo binstall >/dev/null 2>&1; then
          cargo install --locked cargo-about
        else
          echo "installing with cargo binstall"
          cargo binstall cargo-about
        fi
    fi
    cargo audit

    OUTPUT_FILE="./packaging/THIRD_PARTY_NOTICES.md"
    cargo about generate --locked --all-features --fail ./packaging/license.hbs > "$OUTPUT_FILE"
    # yoinked from zed (https://github.com/zed-industries/zed/blob/main/script/generate-licenses)
    sed -i.bak 's/&quot;/"/g' "$OUTPUT_FILE"
    sed -i.bak 's/&#x27;/'\''/g' "$OUTPUT_FILE" # The ` '\'' ` thing ends the string, appends a single quote, and re-opens the string
    sed -i.bak 's/&#x3D;/=/g' "$OUTPUT_FILE"
    sed -i.bak 's/&#x60;/`/g' "$OUTPUT_FILE"
    sed -i.bak 's/&lt;/</g' "$OUTPUT_FILE"
    sed -i.bak 's/&gt;/>/g' "$OUTPUT_FILE"
    rm -rf "${OUTPUT_FILE}.bak"

[group('develop')]
format:
    cargo +nightly fmt --all

[group('develop')]
fix:
    cargo fix --allow-dirty -p hyprshell -p hyprshell-config-lib -p hyprshell-core-lib -p hyprshell-exec-lib -p hyprshell-launcher-lib -p hyprshell-windows-lib -p hyprshell-clipboard-lib -p hyprshell-config-edit-lib

[group('develop')]
build profile="dev":
    cargo build --profile {{ profile }}

[group('checks')]
lint profile="dev":
    cargo +nightly fmt -p hyprshell -p hyprshell-config-lib -p hyprshell-core-lib -p hyprshell-exec-lib -p hyprshell-launcher-lib -p hyprshell-windows-lib -p hyprshell-clipboard-lib -p hyprshell-config-edit-lib -- --check
    cargo clippy --profile {{ profile }} --all-targets -p hyprshell -p hyprshell-config-lib -p hyprshell-core-lib -p hyprshell-exec-lib -p hyprshell-launcher-lib -p hyprshell-windows-lib -p hyprshell-clipboard-lib -p hyprshell-config-edit-lib -- --deny warnings --no-deps

[group('checks')]
test profile="dev":
    cargo nextest run --cargo-profile {{ profile }} --features default --all-targets -p hyprshell -p hyprshell-config-lib -p hyprshell-core-lib -p hyprshell-exec-lib -p hyprshell-launcher-lib -p hyprshell-windows-lib -p hyprshell-clipboard-lib -p hyprshell-config-edit-lib

[group('checks')]
check-feature-combinations:
    bash {{ project_dir }}/scripts/check-all-feature-combinations.sh

[group('checks')]
check-default-nix-features:
    nix build '.#checks.x86_64-linux.hyprshell-check-nix-configs' -L

[group('checks')]
check profile="dev": (build profile) (lint profile) (test profile)

pre-release: (check "release")

[group('run')]
run profile="dev" *args="":
    cargo run --profile {{ profile }} -- {{ args }}

[group('run')]
run-run profile="dev" *args="-vv": (run profile "run" args)

[group('run')]
run-edit-config profile="dev" *args="-vv": (run profile "config edit" args)

[group('run')]
run-explain-config profile="dev" *args="-vv": (run profile "config explain" args)

[group('run')]
run-debug profile="dev" *args="": (run profile "debug" args)

[group('dist')]
package-usr-lib:
    #!/usr/bin/env bash
    sudo tar -cvf ar.tar -C /usr/share/hyprshell.debug setup_preview themes
    ls -lah ar.tar
    sudo mv ar.tar ./packaging/usr-share.tar
    sudo chown user:user ./packaging/usr-share.tar
