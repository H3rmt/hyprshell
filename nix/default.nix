{ self, pkgs, }:
let pname = "hyprshell"; in
pkgs.rustPlatform.buildRustPackage {
  inherit pname;
  version =
    (pkgs.lib.importTOML ../Cargo.toml).workspace.package.version
    + "_"
    + (self.shortRev or "dirty");

  cargoLock.lockFile = ../Cargo.lock;
  src = pkgs.lib.cleanSource ../.;

  nativeBuildInputs = with pkgs; [
    wrapGAppsHook4
    pkg-config
    makeBinaryWrapper

    # hyprshell-core-lib gets the path at build time
    socat
  ];

  buildInputs = with pkgs; [
    gtk4
    gtk4-layer-shell
  ];

  meta = {
    mainProgram = pname;
    description = "hyprshell is a Rust-based GUI designed to enhance window management in hyprland";
    homepage = "https://github.com/h3rmt/hyprshell";
    license = pkgs.lib.licenses.mit;
    platforms = pkgs.lib.platforms.linux;
  };
}
