{
  self,
  lib,
  rustPlatform,
  pkg-config,
  wrapGAppsHook4,
  hyprland,
  gtk4,
  gtk4-layer-shell,
  ...
}:
let
  inherit (builtins) baseNameOf toString;
  inherit (lib.sources) cleanSource cleanSourceWith;
  inherit (lib.strings) hasSuffix;
  inherit (lib.trivial) importTOML;

  pname = "hyprshell";
in
rustPlatform.buildRustPackage {
  inherit pname;
  version = (importTOML ../Cargo.toml).workspace.package.version + "_" + (self.shortRev or "dirty");

  cargoLock.lockFile = ../Cargo.lock;
  src = cleanSourceWith {
    src = cleanSource ../.;
    filter =
      name: _:
      let
        baseName = baseNameOf (toString name);
      in
      !(hasSuffix ".nix" baseName);
  };

  nativeBuildInputs = [
    pkg-config
    wrapGAppsHook4
  ];

  buildInputs = [
    gtk4
    gtk4-layer-shell
  ];

  meta = {
    mainProgram = pname;
    description = "A modern GTK4-based window switcher and application launcher for Hyprland";
    homepage = "https://github.com/h3rmt/hyprshell";
    license = lib.licenses.mit;
    platforms = hyprland.meta.platforms;
  };
}
