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
            nativeBuildInputs = [
              pkgs.rust-bindgen
              toolchain
            ];

            shellHook = ''
              bindgen nexus/api/Nexus.h -o src/nexus/bindings-nexus-api.rs
              bindgen nexus/mumble/Mumble.h -o src/nexus/bindings-mumble-api.rs
            '';

            BINDGEN_EXTRA_CLANG_ARGS = lib.concatStringsSep " " [
              "-x c++"
              "--target=${target}"
            ];

            CARGO_BUILD_TARGET = target;
            CARGO_TARGET_X86_64_PC_WINDOWS_GNU_RUSTFLAGS = "-L native=${pkgs.pkgsCross.mingwW64.windows.pthreads}/lib";
          };
        };
    };
}
