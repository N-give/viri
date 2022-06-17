{
  description = "Algorithms flake";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    cargo2nix.url = "github:cargo2nix/cargo2nix";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, cargo2nix }:
  flake-utils.lib.eachDefaultSystem (system:
    let
      packageName = "viri";
      overlays = [ (import rust-overlay) cargo2nix.overlays.default ];
      pkgs = import nixpkgs {
        inherit system overlays;
      };

      rustPkgs = pkgs.rustBuilder.makePackageSet {
        rustVersion = "1.61.0";
        packageFun = import ./Cargo.nix;
      };
    in rec {
      packages = {
        ${packageName} = (rustPkgs.workspace.${packageName} {}).bin;

        default = packages.${packageName};
      };

      devShell = pkgs.mkShell {
        buildInputs = with pkgs; [
          (rust-bin.selectLatestNightlyWith (toolchain: toolchain.default))
          rust-analyzer
        ];
      };
    });

}
