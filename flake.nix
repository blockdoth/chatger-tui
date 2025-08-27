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
          pkgsX86_64 = import inputs.nixpkgs {
            system = "x86_64-linux";
            overlays = [ inputs.rust-overlay.overlays.default ];
          };  

          crossPkgsAarch = import inputs.nixpkgs {
            inherit system;
            crossSystem = pkgs.lib.systems.examples.aarch64-multiplatform;
            overlays = [ inputs.rust-overlay.overlays.default ];
          };     
          crossPkgsX86_64 = import inputs.nixpkgs {
            inherit system;
            crossSystem = pkgs.lib.systems.examples.x86_64-linux;
            overlays = [ inputs.rust-overlay.overlays.default ];
          };        
          crossPkgsWindows = import inputs.nixpkgs {
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
          
          buildPackage = pkgs: pkgs.rustPlatform.buildRustPackage {
            pname = "chatgertui";
            version = "0.1.0";
            src = ./.;
            cargoLock.lockFile = ./Cargo.lock;
            cargoToml = ./Cargo.toml;
            release = true;
            nativeBuildInputs = [ toolchain ];
          };

          currentSystemPkg = buildPackage pkgs;
          x86_64-linux = buildPackage pkgsX86_64;

          x86_64-linux-cross = buildPackage crossPkgsX86_64;
          aarch64-linux-cross = buildPackage crossPkgsAarch;
          windows-cross = buildPackage crossPkgsWindows;

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
            default = currentSystemPkg;
            x86_64-linux = x86_64-linux; # Mainly added to make ci easier
            x86_64-linux-cross = x86_64-linux-cross;
            aarch64-linux-cross = aarch64-linux-cross;
            windows-cross = windows-cross;
          };
        };
    };
}
