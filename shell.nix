with import <nixpkgs> {};
let
  moz_overlay = import (builtins.fetchTarball
    https://github.com/mozilla/nixpkgs-mozilla/archive/master.tar.gz);
  unstable = import <nixos-unstable> { overlays = [ moz_overlay ]; };
  nightly_rust = (unstable.latest.rustChannels.nightly.rust.override {
    extensions = [
      "rust-src"
    ];
  });
in
  unstable.mkShell {
    buildInputs = [
      nightly_rust
      rust-analyzer
      rustfmt
      clippy
      cargo-edit
    ];
  }
