{
  description = "Rust project flake";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { flake-parts, ... }@inputs:
    flake-parts.lib.mkFlake { inherit inputs; } {
      imports = [
        (_: {
          perSystem = { config, self', inputs', pkgs, system, ... }:
            let
              rustToolchain =
                inputs.fenix.packages.${system}.complete.toolchain;
              overlays = [
                inputs.fenix.overlays.default
                (final: prev: {
                  rustToolchain = rustToolchain;
                  buildRustPackage = (prev.makeRustPlatform {
                    cargo = rustToolchain;
                    rustc = rustToolchain;
                  }).buildRustPackage;
                })
              ];
            in {
              _module.args = {
                pkgs = import inputs.nixpkgs {
                  inherit system overlays;
                  config.allowUnfree = true;
                };
              };
            };
        })
      ];

      systems = [ "x86_64-linux" ];
      perSystem = { config, system, pkgs, ... }: {
        devShells.default = pkgs.mkShell {
          packages = with pkgs; [
            rustToolchain
            cargo-nextest
            cargo-watch
            just
          ];
        };

      };
    };
}

