{
  description = "forth-lsp — Language Server for the Forth programming language";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    { self
    , nixpkgs
    , flake-utils
    , rust-overlay
    ,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };

        rustToolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
      in
      {
        devShells.default = pkgs.mkShell {
          packages = with pkgs; [
            rustToolchain

            # Cargo helpers
            cargo-edit
            cargo-watch
            cargo-nextest
            cargo-outdated
            cargo-audit
            cargo-deny

            # Native tooling
            pkg-config
          ];

          env = {
            RUST_BACKTRACE = "1";
            # Let rust-analyzer find the stdlib sources
            RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";
          };
        };

        formatter = pkgs.nixpkgs-fmt;
      }
    );
}
