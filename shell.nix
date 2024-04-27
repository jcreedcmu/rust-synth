{ pkgs ? import <nixpkgs> {} }:
  with pkgs; with builtins; let
    asoundShellHook = ''
    export ALSA_PLUGIN_DIR=${alsa-plugins}/lib/alsa-lib
    export SOUND_CARD=2
    '';
  in
    mkShell {
      nativeBuildInputs = [
        rustc
        cargo
        rust-analyzer
        alsa-lib.dev
        alsa-plugins
        alsa-utils
        dbus
        sox
        vorbis-tools
      ];
      shellHook = asoundShellHook;
}
