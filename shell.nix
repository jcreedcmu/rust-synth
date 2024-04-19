{ pkgs ? import <nixpkgs> {} }:
  with pkgs; with builtins; let
    asoundShellHook = ''
    export ALSA_PLUGIN_DIR=${alsa-plugins}/lib/alsa-lib
    '';

    patchedAlsa = trace alsa-lib.patches
      alsa-lib.overrideAttrs (finala: preva: {
      patches = preva.patches ++ [ ./patches/0001-Debug-a-little.patch ];
    });
  in
    mkShell {
      nativeBuildInputs = [
        rustc
        rust-analyzer
        # alsa-lib.dev
        patchedAlsa.dev
        alsa-plugins
      ];
      shellHook = asoundShellHook;
}
