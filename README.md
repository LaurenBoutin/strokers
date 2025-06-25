# strokers

Rust crates with the central theme of controlling adult strokers.

There is also an MPV plugin for controlling strokers using funscripts, please click here
for more information: [strokers_for_mpv](./strokers_for_mpv).

## Crates

- `strokers` (library): top-level library that you can use to load stroker configurations and connect to them.
- `strokers_core` (library): core types of strokers
- `strokers_device_tcode` (library): implementation for T-Code strokers.
  You don't need to use this directly if you use the top-level `strokers` crate.
- `strokers_device_debug` (library): a debug stroker implementation, that just emits log lines. Useful for testing.
  You don't need to use this directly if you use the top-level `strokers` crate.
- `strokers_funscript` (library): basic Funscript loading library, with support for:
  - discovering Funscripts based on a video's path (for all known types of axes,
    both main funscripts and 'alternatives' that have some sort of suffix, e.g. 'hard mode').
  - applying fixups to the Funscript once loaded and normalising the actions to a range of 0.0 to 1.0.
- [`strokers_for_mpv` (MPV plugin)](./strokers_for_mpv): a MPV plugin that uses `strokers` and `strokers_funscript` to synchronise a stroker to a video

## Licence

Licensed under either of [Apache License, Version 2.0](./LICENSE-APACHE) or [MIT license](./LICENSE-MIT) at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this crate by you,
as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions. 

## Home Repository

The current home of this repository is <https://codeberg.org/LaurenBoutin/strokers>.

There is a GitHub mirror at <https://github.com/LaurenBoutin/strokers> but please send PRs and issues on the first.
