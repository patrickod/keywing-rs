let
  rust-version = "1.46.0";

  nixpkgs = fetchGit {
    url = "https://github.com/patrickod/nixpkgs.git";
    rev = "8bb6ca5f0a93bb0b9264b50a8c56431c7c7cc591";
    ref = "personal";
  };

  mozilla-overlay =
    import (builtins.fetchTarball https://github.com/mozilla/nixpkgs-mozilla/archive/master.tar.gz);

  pkgs = import nixpkgs {
    overlays = [ mozilla-overlay ];
  };

  targets = [
    "x86_64-unknown-linux-gnu"
    "thumbv7em-none-eabihf"
  ];

in
  pkgs.mkShell {
    name = "rust-dev";
    nativeBuildInputs = with pkgs; [
      (rustChannels.stable.rust.override {
        targets = targets;
        extensions = [
          "rust-src"
          "rust-analysis"
          "rls-preview"
        ];
      })
      gcc-arm-embedded
      cargo-hf2
      rustup
    ];
  }
