# started off from https://github.com/NixOS/templates/blob/master/rust/flake.nix
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.05";
    naersk = {
      url = "github:nix-community/naersk/master";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, utils, naersk }:
    utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
        naersk-lib = pkgs.callPackage naersk { };
      in
      {
        defaultPackage = naersk-lib.buildPackage ./.;
        devShell = with pkgs; mkShell {
          buildInputs = [
            cargo rustc rustfmt pre-commit rustPackages.clippy
            rust-analyzer
            clang
            mpv
          ];
          RUST_SRC_PATH = rustPlatform.rustLibSrc;
          LIBCLANG_PATH="${pkgs.llvmPackages_latest.libclang.lib}/lib";
        };
      }
    );
}

