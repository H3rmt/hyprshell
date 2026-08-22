{
  craneLib,
  pkgs,
}:
rec {
  # Vendored wayland protocol XML files must be kept in the cargo source
  # tree since they are read at build time by wayland-scanner.
  src = pkgs.lib.cleanSourceWith {
    src = ../.;
    filter = path: type:
      (craneLib.filterCargoSources path type)
      || (type == "regular" && pkgs.lib.hasSuffix ".xml" path);
  };

  commonArgs = {
    pname = "hyprshell";
    inherit src;
    version = (pkgs.lib.trivial.importTOML ../Cargo.toml).workspace.package.version;

    meta = {
      mainProgram = "hyprshell";
      description = "A modern GTK4-based window switcher and application launcher for Hyprland";
      homepage = "https://github.com/h3rmt/hyprshell";
      license = pkgs.lib.licenses.mit;
      platforms = pkgs.hyprland.meta.platforms;
    };

    strictDeps = true;
    doCheck = false;
    cargoBuildCommand = "cargo build --release --locked";

    nativeBuildInputs = [
      pkgs.pkg-config
      pkgs.wrapGAppsHook4
    ];

    buildInputs = [
      pkgs.libadwaita
      pkgs.gtk4-layer-shell
    ];
  };

  postInstall = ''
    # Desktop entry
    install -Dm644 packaging/hyprshell-settings.desktop $out/share/applications/hyprshell-settings.desktop

    # Icon
    install -Dm644 packaging/hyprshell-settings.png $out/share/pixmaps/hyprshell-settings.png

    # Extract runtime data
    mkdir -p $out/share/hyprshell
    tar -xf packaging/usr-share.tar -C $out/share/hyprshell
  '';

  cargoArtifacts = craneLib.buildDepsOnly (
    commonArgs
    // {
      inherit src;
    }
  );

  commonArgsFull = (commonArgs // { inherit postInstall cargoArtifacts; });
}
