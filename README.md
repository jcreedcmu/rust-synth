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

Freeverb code
-------------

I'm using [Ian Hobson's implementation](https://github.com/irh/freeverb-rs) of the freeverb algorithm.

```
Copyright (c) 2018 Ian Hobson

Permission is hereby granted, free of charge, to any person obtaining a copy of
this software and associated documentation files (the "Software"), to deal in
the Software without restriction, including without limitation the rights to
use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies
of the Software, and to permit persons to whom the Software is furnished to do
so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```
