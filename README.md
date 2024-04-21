Rust Synth
==========

Playing around with audio synthesis in rust.

Nix Notes
---------

The presence of the file `shell.nix` in this directory means that I
can run `nix-shell` and I get the appropriate rust tooling and alsa
libraries. There is a subtlety around alsa plugins, --- I'm using the
workaround discussed in [this nixpkgs
issue](https://github.com/NixOS/nixpkgs/issues/187308), which is to
set the environment variable `ALSA_PLUGIN_DIR`. Another possibility
that I haven't explored yet is to use
[alsa-lib-with-plugins](https://github.com/NixOS/nixpkgs/pull/277180).

Similarly, the directory `ardour-shell` contains a `shell.nix`
suitable for running the Ardour DAW. It also achieves quite low
synthesis latency, but I experience a lot of crashes trying to use VST
plugins.
