{
  craneLib,
  lib,
  parted,
  rustPlatform,
  cryptsetup,
  makeWrapper,
}: let
  src = craneLib.cleanCargoSource ./.;
  commonArgs = {
    inherit src;
    strictDeps = true;
    nativeBuildInputs = [
      rustPlatform.bindgenHook
    ];
    buildInputs = [
      parted
      cryptsetup
    ];
  };

  cargoArtifacts = craneLib.buildDepsOnly commonArgs;
in
  craneLib.buildPackage (commonArgs
    // {
      inherit cargoArtifacts;

      nativeBuildInputs =
        commonArgs.nativeBuildInputs
        ++ [
          makeWrapper
        ];

      postInstall = ''
        wrapProgram  $out/bin/xeonitte-helper \
            --prefix PATH : ${lib.makeBinPath [
          cryptsetup
        ]}
      '';
    })
