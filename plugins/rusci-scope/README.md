# Rusci Scope

An FFGL 2.1 **source** plugin for Resolume. It shows up in
Resolume's Sources list and generates oscilloscope-style vector graphics
(Gaussian beam, phosphor persistence, glow) using the engine from
[rusci-render](https://github.com/trop3n/rusci-render), a Rust port of
[osci-render](https://github.com/jameshball/osci-render).

Runs on Windows x64 and macOS (universal: Apple Silicon and Intel). Needs
Resolume 7.0.3 or later (FFGL 2.1).

## Install

Ready-built plugins come from CI: open the latest run on the
[Actions page](https://github.com/trop3n/resolume-plugins/actions) and download
**RusciScope-windows** or **RusciScope-macos** (needs a GitHub login).

### Windows

Put `RusciScope.dll` in `Documents\Resolume Arena\Extra Effects\`. Or
build and install from source, from the repo root:

```powershell
powershell -ExecutionPolicy Bypass -File install.ps1 rusci-scope
```

This builds `target\release\rusci_scope.dll` and copies it to
`Documents\Resolume Arena\Extra Effects\RusciScope.dll`.

### macOS

Unzip the download and follow `INSTALL.txt`. In short, copy
`RusciScope.bundle` to `~/Documents/Resolume Arena/Extra Effects/`, then
clear the download quarantine (the plugin is ad-hoc signed, not signed with
an Apple developer certificate):

```sh
xattr -dr com.apple.quarantine ~/Documents/"Resolume Arena"/"Extra Effects"/RusciScope.bundle
```

To build the bundle yourself on a Mac, from the repo root:

```sh
rustup target add aarch64-apple-darwin x86_64-apple-darwin
scripts/bundle-macos.sh rusci-scope
```

### Then

Restart Resolume (it only scans plugins at startup), then drag
**Rusci Scope** from Sources onto a clip slot.

## Parameters

| Group  | Parameter   | Notes                                        |
|--------|-------------|----------------------------------------------|
| Shape  | Shape       | Circle, Square, Triangle, Star, Lissajous, SVG File |
| Shape  | SVG File    | SVG drawn when Shape is "SVG File"           |
| Shape  | Size        | 0.05 – 1.0 of the frame height               |
| Shape  | Spin        | -1 – 1 turns per second (centre = still)     |
| Beam   | Focus       | Beam width (lower = sharper)                 |
| Beam   | Intensity   | Beam brightness                              |
| Beam   | Persistence | Phosphor trail length                        |
| Beam   | Glow        | Tight bloom                                  |
| Beam   | Scatter     | Wide bloom                                   |
| Beam   | Exposure    | Tone-mapping exposure                        |
| Colour | Colour      | Beam colour (Resolume shows a colour picker) |

All sliders can be mapped to MIDI, BPM or audio in Resolume as usual. The
scope is drawn in a centred square; the sides are transparent.

### SVG files

Set **Shape** to "SVG File" and pick a file with **SVG File**. The drawing
is centred and scaled to fit, keeping its proportions; fills are ignored,
and every path is traced as an outline. Files load in the background, so a
large SVG appears a moment later rather than stalling the output. Load
errors (missing file, invalid SVG, no paths) go to Resolume's log, and
nothing is drawn until a file loads.

Each frame traces the whole drawing with 2048 points, so very detailed SVGs
will look sparse or flicker; simpler line art works best.

## Test without Resolume

```sh
cargo test -p rusci-scope
cargo build --release -p rusci-scope
cargo run --release -p rusci-scope --example ffgl_host
```

`ffgl_host` loads the plugin, creates an OpenGL 4.1 core context (what
Resolume provides), makes the same calls Resolume does, checks after every
call that the plugin left GL in the state the FFGL SDK requires, and writes
one PNG per shape to `target/ffgl-host/`, including `examples/arrow.svg`
loaded through the SVG File parameter. It exits non-zero on any failure.
It runs on Windows (WGL) and macOS (CGL); on macOS it also accepts a
bundle: `cargo run --release -p rusci-scope --example ffgl_host -- path/to/RusciScope.bundle`.

CI runs clippy and the tests on both platforms, builds the plugins, and runs
`ffgl_host` against the macOS bundle on both architectures (Intel through
Rosetta). Windows runners can't run it: they only offer OpenGL 1.1.

Unit tests cover the ABI struct layouts, parameter mapping, `plugMain`
dispatch, shape sampling and SVG loading. They also run on Linux.

## rusci-render dependency

`osci-core`, `osci-synth`, `osci-parsers` and `osci-visualizer` come from
rusci-render, pinned to a commit in this folder's `Cargo.toml`. To update, change the
`rev` on all four lines.

To develop against a local rusci-render checkout instead, create
`.cargo/config.toml` at the repo root (git-ignored):

```toml
[patch."https://github.com/trop3n/rusci-render"]
osci-core = { path = "C:/path/to/rusci-render/crates/osci-core" }
osci-synth = { path = "C:/path/to/rusci-render/crates/osci-synth" }
osci-parsers = { path = "C:/path/to/rusci-render/crates/osci-parsers" }
osci-visualizer = { path = "C:/path/to/rusci-render/crates/osci-visualizer" }
```

The pinned `rev` must still exist on GitHub; Cargo fetches it before
applying the patch.

## Layout

| File                    | What it does                                           |
|-------------------------|--------------------------------------------------------|
| `src/ffi.rs`            | FFGL constants and structs, from Resolume's `FFGL.h`    |
| `src/lib.rs`            | `plugMain` entry point and dispatch                    |
| `src/params.rs`         | Parameter table and slider → real-value mapping        |
| `src/scene.rs`          | Shapes → X/Y samples via `osci_synth::ShapeRenderer`    |
| `src/instance.rs`       | Per-clip state; drives `osci_visualizer::OsciRenderer`  |
| `src/svg_loader.rs`     | Background SVG loading via `osci_parsers::svg`          |
| `src/gl_loader.rs`      | Loads GL functions from the host's context             |
| `examples/ffgl_host.rs` | Stand-in host for testing (Windows and macOS)          |
| `packaging/macos/`      | Bundle `Info.plist` and macOS install notes            |

The FFGL interface is written directly in Rust (no C++ SDK, no bindgen).

## Next steps

- More file types through `osci-parsers` (OBJ, text, GPLA).
- Effects chain from `osci-effects`.
- Lua scripts with sliders mapped to FFGL parameters.

## License

GPL-3.0, like rusci-render and osci-render. See [LICENSE](../../LICENSE).
