{
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
          crossAarchPkgs = import inputs.nixpkgs {
            system = "x86_64-linux";
            crossSystem = pkgs.lib.systems.examples.aarch64-multiplatform;
            overlays = [ inputs.rust-overlay.overlays.default ];
          };          
          crossWindowsPkgs = import inputs.nixpkgs {
            inherit system;
            crossSystem = pkgs.lib.systems.examples.mingwW64;
            overlays = [
              inputs.rust-overlay.overlays.default
              (final: prev: {
                rhash = prev.rhash.overrideAttrs (old: { dontFixup = true; });     
              })            
            ];
          };
          toolchain = pkgs.rust-bin.fromRustupToolchainFile ./toolchain.toml;
          
          pkgX86_64-linux = pkgs.rustPlatform.buildRustPackage {
            pname = "chatgertui";
            version = "0.1.0";
            src = ./.;
            cargoLock.lockFile = ./Cargo.lock;
            cargoToml = ./Cargo.toml;
            release = true;
            nativeBuildInputs = [ toolchain ];
          };
          pkgAarch64-linux = crossAarchPkgs.rustPlatform.buildRustPackage {
            pname = "chatgertui";
            version = "0.1.0";
            src = ./.;
            cargoLock.lockFile = ./Cargo.lock;
            cargoToml = ./Cargo.toml;
            release = true;
            nativeBuildInputs = [ toolchain ];
          };
          pkgWindows = crossWindowsPkgs.rustPlatform.buildRustPackage {
            pname = "chatgertui";
            version = "0.1.0";
            src = ./.;
            cargoLock.lockFile = ./Cargo.lock;
            cargoToml = ./Cargo.toml;
            release = true;
            nativeBuildInputs = [ toolchain ];
          };          
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

          packages = {
            default = pkgX86_64-linux;
            x86_64-linux = pkgX86_64-linux;
            aarch64-linux = pkgAarch64-linux;
            windows = pkgWindows;
          };
        };
    };
}
