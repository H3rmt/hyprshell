{
  self,
  craneLib,
  pkgs,
}:
rec {
  pname = "hyprshell";
  version = (pkgs.lib.trivial.importTOML ../Cargo.toml).workspace.package.version;
  src = pkgs.lib.cleanSourceWith {
    src = ../.;
    filter =
      path: type:
      (builtins.match ".*(css|service|hpp|cpp)$" path != null) || (craneLib.filterCargoSources path type);
    name = "source";
  };
  meta = {
    mainProgram = pname;
    description = "A modern GTK4-based window switcher and application launcher for Hyprland";
    homepage = "https://github.com/h3rmt/hyprshell";
    license = pkgs.lib.licenses.mit;
    platforms = pkgs.hyprland.meta.platforms;
  };
  stdenv = p: p.stdenv;
  postInstall = ''
    wrapProgram $out/bin/hyprshell \
      --prefix PATH : ${pkgs.lib.makeBinPath [ pkgs.gcc ]} \
      --prefix CPATH : ${pkgs.lib.makeIncludePath (pkgs.hyprland.buildInputs ++ [ pkgs.hyprland ])}
  '';
  commonArgs = {
    inherit
      src
      stdenv
      pname
      version
      ;
    strictDeps = true;
    doCheck = false;
    cargoBuildCommand = "cargo build --profile release --locked";
    cargoTestCommand = "";
    cargoCheckCommand = "";
    cargoExtraArgs = "";

    nativeBuildInputs = [
      pkgs.pkg-config
      pkgs.wrapGAppsHook4
      pkgs.makeBinaryWrapper
    ];

    buildInputs = [
      pkgs.gtk4
      pkgs.gtk4-layer-shell
    ];
  };

  cargoArtifacts = craneLib.buildDepsOnly (
    commonArgs
    // {
      mkDummySrc = craneLib.mkDummySrc {
        inherit stdenv;
        src = ../.;
      };
      pname = "hyprshell-release";
    }
  );

  commonArgsCachedRelease = (
    commonArgs
    // {
      inherit postInstall cargoArtifacts meta;
    }
  );
}
