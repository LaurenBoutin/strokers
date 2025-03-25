# `strokers_for_mpv`: an MPV plugin to control strokers (adult toys)

## Features

- Linux support. (MacOS and Windows support may exist but is untested?)
- Support for T-Code devices including Tempest MAx OSR2(+), SR6, SSR1, etc.
- Multi-axis support (Stroke, Surge, Sway, Twist, Roll, Pitch, Vibration, Valve, Suction, Lubricant) 
- Synchronises video playback to your stroker using funscripts
- Speed limits for safety and comfort
- Axis limits (min/max) for safety and comfort
- Keybindings to change axis limits on the fly

## Limitations

- **No alternative funscripts:** If you have 'alternative' funscripts with suffixes like 'hard mode' etc,
  this plugin currently will not let you choose them,
  although the code is there to find them.
- **Use at your own risk:** Especially depending on configuration, this software has potential to do harm as well as good.
  Please make your own trial runs and experiment safely according to your own comfort.
  **The authors don't take responsibility for your usage of this software.**
  Don't underestimate your hardware: it will often try its best to do what it's told,
  even if that's not what you want.
- When the plugin is enabled, a connection is always attempted to your device
  when MPV is opened, even if your video doesn't have a funscript.
  I hope to address this in the future.

## How to use

### How to build

With a Rust compiler available, `cargo build --release` will build the plugin
as a shared object (`.so`)/dynamically linked library (`.dll`) and emit it to your target directory.

If your Rust setup has not been configured differently, this will appear at `target/release/libstrokers_for_mpv.so` or `target/debug/strokers_for_mpv.dll` (I believe — Windows is untested).
(You may have your target directory set to emit somewhere different — check your cargo configuration `~/.cargo/config.toml`.)

### How to use

You can either copy the plugin to your plugins directory `~/.config/mpv/scripts` (I believe — yet to test this)
or point MPV to it at the command line: `mpv --script=target/release/libstrokers_for_mpv.so path/to/fun_video.mp4`.

Don't specify the plugin multiple times or use both methods of loading it — this will initialise the plugin twice
and it doesn't like this (`tracing_subscriber` error).

### How to configure

#### Stroker setup

Edit `~/.config/strokers.toml` and add the following:

```toml
[stroker]
type = "tcode_serial"
serial_port = "/dev/ttyUSB0"
# baud = 115200 by default


[limits.stroke]
speed = 0.5
default_min = 0.45
default_max = 0.55

[limits.surge]
speed = 0.5
default_min = 0.45
default_max = 0.55

[limits.sway]
speed = 0.5
default_min = 0.45
default_max = 0.55

[limits.twist]
speed = 0.5
default_min = 0.45
default_max = 0.55

[limits.roll]
speed = 0.5
default_min = 0.45
default_max = 0.55

[limits.pitch]
speed = 0.5
default_min = 0.45
default_max = 0.55
```

These limits are very restrictive (boring).
You can increase them according to your own comfort; please 

Update the serial port to reflect reality if `/dev/ttyUSB0` is not the right one for you.

#### MPV keybindings

Edit `~/.config/mpv/input.conf` and add the following block,
updating any keybinds you want to change:

```
## strokers_for_mpv keybinds
KP1 script-binding "libstrokers_for_mpv/axis_limit axis=stroke&min_by=-0.05"
KP2 script-binding "libstrokers_for_mpv/axis_limit axis=stroke&min_by=0.05"
KP7 script-binding "libstrokers_for_mpv/axis_limit axis=stroke&max_by=-0.05"
KP8 script-binding "libstrokers_for_mpv/axis_limit axis=stroke&max_by=0.05"
KP4 script-binding "libstrokers_for_mpv/axis_limit axis=stroke&min_new=0.4&max_new=0.6"
```

(If you change the name of the MPV plugin file (`.so`/`.dll`), you need to update `libstrokers_for_mpv` above to match it.)

The above creates a configuration that:
- number pad 1 lowers the minimum axis limit of the stroke (up/down) axis by 0.05 of its full scale
- number pad 2 raises the minimum axis limit of the stroke axis by 0.05
- number pad 7 lowers the maximum axis limit of the stroke axis by 0.05
- number pad 8 raises the maximum axis limit of the stroke axis by 0.05
- number pad 4 sets the axis limits of the stroke axis to 0.4 minimum and 0.6 minimum in one go, no matter what it was before.

The values are all tweakable and you can set both limits in the same binding if desired.

An axis can be disabled by setting the min and max to the same value.

## Licence

This plugin is currently under the GNU AGPL v3 or later.

As this is all my own work and I am indecisive, this is still subject to change,
but I obviously can't take this away from you so this current copy of the software
is always open source (and I intend to keep future versions that way).

## Alternative Software

- [MultiFunPlayer](https://github.com/Yoooi0/MultiFunPlayer) (.NET, Windows only, might run in Wine?)
- [XTPlayer](https://github.com/jcfain/XTPlayer) (cross-platform, is its own media player)
  - I have packaged [a Nix flake for it here](https://github.com/LaurenBoutin/xtplayer_flake)
