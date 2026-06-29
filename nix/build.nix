{
  craneLib,
  pkgs,
}:
rec {
  commonArgs = {
    pname = "hyprshell";
    src = ../.;
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
      src = craneLib.cleanCargoSource ../.;
    }
  );

  commonArgsFull = (commonArgs // { inherit postInstall cargoArtifacts; });
}
