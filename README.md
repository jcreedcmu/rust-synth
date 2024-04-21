Rust Synth
==========

Playing around with audio synthesis in rust.

Hardware Devices
----------------

Software mixers like pulseaudio, jack, pipewire multiplex access to
the sound card, but I have had a tricky time getting so far getting
any of them to perform with low latency. My main goal with this
repository is to optimize for fun while synthesizing audio from live
performance on a midi keyboard. Minimizing latency is therefore
paramount.

For this reason, I'm directly asking ALSA for `hw:2` (because that
happens to be where my soundcard is) and using `pasuspender` to ask
pulseaudio to relinquish control during the execution of my
synthesizer.

There is a dbus endpoint (see [ardour source
code](https://github.com/Ardour/ardour/blob/master/libs/ardouralsautil/reserve.c)
as an example, or the
[specification](http://git.0pointer.net/reserve.git/tree/reserve.txt))
for requesting reservation of audio devices. This is probably the
right way of handling the issue from inside an executing program.

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
