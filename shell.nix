let
  moz_overlay = import (builtins.fetchTarball https://github.com/mozilla/nixpkgs-mozilla/archive/master.tar.gz);
  nixpkgs = import <nixpkgs> { overlays = [ moz_overlay ]; };
  nixpkgs-unstable = import <nixpkgs-unstable> { };
  binaryen = (import (builtins.fetchTarball {
    url = "https://github.com/nixos/nixpkgs/archive/7b1e56acf0674cfc777f47386153e6f5ba9b34a8.tar.gz";
  }) {}).binaryen;
  rustChannel = nixpkgs.rustChannelOf { rustToolchain = ./rust-toolchain; };
in
with nixpkgs;
stdenv.mkDerivation {
  name = "forecast-contracts";
  src = ./.;

  buildInputs = [
    (rustChannel.rust.override {
      targets = [ "wasm32-unknown-unknown" ];
    })
    binaryen
    nodejs
    git
    nodePackages.typescript
    nodePackages.yarn
  ];

  RUST_SRC_PATH = "${rustPlatform.rustLibSrc}";
}
