{ lib
, rustPlatform
, ...
}:
let
  manifest = lib.importTOML ./Cargo.toml;
  program = (lib.head manifest.bin).name;
in
rustPlatform.buildRustPackage {
  pname = manifest.package.name;
  version = manifest.package.version;
  src = lib.cleanSource ./.;

  cargoLock.lockFile = ./Cargo.lock;

  meta = with lib; {
    description = manifest.package.description;
    homepage = manifest.package.repository;
    license = licenses.mit;
    platforms = platforms.all;
    mainProgram = program;
  };
}
