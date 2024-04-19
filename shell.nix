{ pkgs ? import <nixpkgs> {} }:
  with pkgs; with builtins; let
    asoundShellHook = ''
    export ALSA_PLUGIN_DIR=${alsa-plugins}/lib/alsa-lib
    '';
  in
    mkShell {
      nativeBuildInputs = [
        rustc
        cargo
        rust-analyzer
        alsa-lib.dev
        alsa-plugins
      ];
      shellHook = asoundShellHook;
}
