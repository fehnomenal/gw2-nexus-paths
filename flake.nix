{
  inputs = {
    flake-parts.url = "github:hercules-ci/flake-parts";

    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = inputs@{ flake-parts, nixpkgs, rust-overlay, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = nixpkgs.lib.systems.flakeExposed;

      perSystem = { lib, pkgs, system, ... }:
        let
          rustup-toolchain = (builtins.fromTOML (builtins.readFile ./rust-toolchain.toml)).toolchain;

          target = builtins.head rustup-toolchain.targets;

          toolchain = pkgs.rust-bin.fromRustupToolchain rustup-toolchain;

        in

        {
          _module.args.pkgs = import nixpkgs {
            inherit system;

            overlays = [ rust-overlay.overlays.default ];
          };

          devShells.default = pkgs.pkgsCross.mingwW64.mkShell {
            buildInputs = [
              pkgs.openssl
            ];

            nativeBuildInputs = [
              pkgs.pkg-config
              pkgs.rustPlatform.bindgenHook
              toolchain
            ];

            CARGO_BUILD_TARGET = target;
            CARGO_TARGET_X86_64_PC_WINDOWS_GNU_RUSTFLAGS = "-L native=${pkgs.pkgsCross.mingwW64.windows.pthreads}/lib";

            LD_LIBRARY_PATH = lib.makeLibraryPath [ pkgs.openssl ];
          };
        };
    };
}
