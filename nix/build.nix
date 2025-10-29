{
  craneLib,
  pkgs,
}:
rec {
  meta = import ./meta.nix { inherit pkgs; };
  pname = "hyprshell";
  version = (pkgs.lib.trivial.importTOML ../Cargo.toml).workspace.package.version;
  # no more filtering, excluded to many files
  src = ../.;
  stdenv = p: p.stdenv;
  # use in preFixup
  addWrapWithGccArgs = hyprland: ''
    gappsWrapperArgs+=(
       --prefix PATH : '${pkgs.lib.makeBinPath [ pkgs.gcc ]}'
       --prefix CPATH : '${
         pkgs.lib.makeIncludePath (
           hyprland.buildInputs
           ++ [
             hyprland
             pkgs.pixman
           ]
         )
       }'
     )
  '';
  commonArgs = {
    inherit
      src
      stdenv
      pname
      version
      meta
      ;
    strictDeps = true;
    doCheck = false;
    cargoBuildCommand = "cargo build --release --locked";
    cargoTestCommand = "";
    cargoCheckCommand = "";
    cargoCheckExtraArgs = "";
    cargoExtraArgs = "";

    nativeBuildInputs = [
      pkgs.pkg-config
      pkgs.wrapGAppsHook4
    ];

    buildInputs = [
      pkgs.libadwaita
      pkgs.gtk4-layer-shell
    ];
  };

  cargoReleaseArtifacts = craneLib.buildDepsOnly (
    commonArgs
    // {
      mkDummySrc = craneLib.mkDummySrc {
        inherit stdenv;
        src = ../.;
      };
      pname = "hyprshell";
    }
  );
  cargoFullArtifacts = craneLib.buildDepsOnly (
    commonArgs
    // {
      src = craneLib.cleanCargoSource ../.;
      pname = "hyprshell-full";
      doCheck = true;
      cargoBuildCommand = "cargo build --profile dev --locked --all-targets --all-features";
      cargoCheckCommand = "cargo check --profile dev --locked --all-targets --all-features";
      cargoTestCommand = "cargo test --profile dev --locked --all-targets --all-features";
    }
  );
  commonArgsFullCached = (
    commonArgs
    // {
      cargoArtifacts = cargoFullArtifacts;
    }
  );
}
