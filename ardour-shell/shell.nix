# The point of this is to have a shell to run ardour in
{ pkgs ? import <nixpkgs> {} }:
  with pkgs; with builtins; let
    asoundShellHook = ''
    export ALSA_PLUGIN_DIR=${alsa-plugins}/lib/alsa-lib
    '';
  in
    mkShell {
      nativeBuildInputs = [
        ardour
        alsa-lib.dev
        alsa-plugins
        alsa-utils
      ];
      shellHook = asoundShellHook;
}
