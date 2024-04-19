{ pkgs ? import <nixpkgs> {} }:
  pkgs.mkShell {
    nativeBuildInputs = with pkgs.buildPackages; [
      rustc
      rust-analyzer
      alsa-lib.dev
    ];
    shellHook = ''
    export ALSA_CONFIG_PATH=./asoundrc
    '';
}
