{
  craneLib,
  pkgs,
}:
rec {
  # Keep build-time includes and installed assets alongside Cargo sources.
  src = pkgs.lib.cleanSourceWith {
    src = ../.;
    filter =
      path: type:
      (craneLib.filterCargoSources path type)
      || (
        type == "regular"
        && (
          pkgs.lib.hasSuffix ".xml" path
          || pkgs.lib.hasSuffix ".css" path
          || pkgs.lib.hasSuffix "/dep-crates/hyprland/README.md" path
          || pkgs.lib.hasSuffix "/dep-crates/hyprland/hyprland-macros/README.md" path
          || pkgs.lib.hasInfix "/packaging/" path
        )
      );
  };

  commonArgs = {
    pname = "hyprshell";
    inherit src;
    version = (pkgs.lib.trivial.importTOML ../Cargo.toml).package.version;

    meta = {
      mainProgram = "hyprshell";
      description = "A modern GTK4-based window switcher and application launcher for Hyprland";
      homepage = "https://github.com/h3rmt/hyprshell";
      license = pkgs.lib.licenses.mit;
      platforms = pkgs.hyprland.meta.platforms;
    };

    strictDeps = true;
    doCheck = false;
    cargoBuildCommand = "cargo build --release";

    nativeBuildInputs = [
      pkgs.pkg-config
      pkgs.wrapGAppsHook4
    ];

    buildInputs = [
      pkgs.dbus
      pkgs.wayland
      pkgs.libgbm
      pkgs.libadwaita
      pkgs.gtk4-layer-shell
    ];
  };

  postInstall = ''
    # Desktop entry
    install -Dm644 packaging/hyprshell-settings.desktop $out/share/applications/hyprshell-settings.desktop

    # Icon
    install -Dm644 packaging/hyprshell-settings.png $out/share/pixmaps/hyprshell-settings.png

    # Install the runtime data directly from the checked-in source tree.
    mkdir -p $out/share/hyprshell
    cp -r packaging/share/. $out/share/hyprshell/
  '';

  cargoArtifacts = craneLib.buildDepsOnly (
    commonArgs
    // {
      inherit src;
    }
  );

  commonArgsFull = commonArgs // {
    inherit postInstall cargoArtifacts;
  };
}
