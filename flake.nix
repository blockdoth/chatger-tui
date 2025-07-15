{
  description = "The Sensei dev flake";

  inputs = {
    flake-parts.url = "github:hercules-ci/flake-parts";
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs =
    inputs@{ self, flake-parts, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];
      perSystem =
        {
          inputs',
          pkgs,
          system,
          ...
        }:
        let
          pkgs = import inputs.nixpkgs {
            inherit system;
            overlays = [ inputs.rust-overlay.overlays.default ];
          };

          toolchain = pkgs.rust-bin.fromRustupToolchainFile ./toolchain.toml;
        in
        {
          devShells.default = pkgs.mkShell {
            packages = with pkgs; [
              toolchain
              rust-analyzer-unwrapped
              mprocs
            ];

            RUST_SRC_PATH = "${toolchain}/lib/rustlib/src/rust/library";
          };

          packages.default = pkgs.rustPlatform.buildRustPackage {
            pname = "chatgertui";
            version = "0.1.0";
            src = ./.;
            cargoLock.lockFile = ./Cargo.lock;
            cargoToml = ./Cargo.toml;
            nativeBuildInputs = with pkgs; [
              toolchain
            ];
          };    
        };
    };
}
