{ lib
, rustPlatform
}:
let
  manifest = lib.importTOML ./Cargo.toml;
in
rustPlatform.buildRustPackage {
  pname = manifest.package.name;
  version = manifest.package.version;
  src = lib.fileset.toSource {
    root = ./.;
    fileset = lib.fileset.unions [
      ./Cargo.toml
      ./Cargo.lock
      ./src
      ./lib
      ./tests
    ];
  };

  cargoLock.lockFile = ./Cargo.lock;
  cargoTestFlags = [ "--workspace" ];

  meta = {
    description = manifest.package.description;
    homepage = manifest.package.repository;
    license = lib.getLicenseFromSpdxId manifest.package.license;
    mainProgram = manifest.package.name;
  };
}
