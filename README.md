# resolume-plugins

FFGL plugins for Resolume Arena and Avenue, written in Rust. Each plugin is
built for Windows x64 and macOS (one universal bundle for Apple Silicon and
Intel).

| Plugin | Type | What it does |
|--------|------|--------------|
| [Rusci Scope](plugins/rusci-scope/) | Source | Oscilloscope-style vector graphics (shapes, SVGs) using [rusci-render](https://github.com/trop3n/rusci-render) |
| [Risograph](plugins/risograph/) | Effect | Two-ink riso print: separation, grain or halftone screen, misregistration, paper |
| [Pixel Sort](plugins/pixel-sort/) | Effect | GPU pixel sorting along rows or columns inside a brightness mask |
| [Threshold](plugins/threshold/) | Effect | Hard two-colour cut at a brightness level, with photocopy grit |
| [Posterize](plugins/posterize/) | Effect | Few tone levels, on a three-colour gradient or per channel |
| [Solarize](plugins/solarize/) | Effect | Tones past a threshold fold back, in rainbow, colour or mono |
| [Halftone](plugins/halftone/) | Effect | Rotated halftone screen of dots, lines or squares, mono or CMYK |
| [Dither](plugins/dither/) | Effect | Bayer or noise dithering to a few levels at a chosen pixel size |
| [Dot Matrix](plugins/dot-matrix/) | Effect | Grid of dots sized by brightness, printer or LED style |
| [X-Ray](plugins/x-ray/) | Effect | Inverted, high-contrast x-ray with glowing edges, grain and scan lines |
| [Night Vision](plugins/night-vision/) | Effect | Amplified green phosphor image with noise, bloom and vignette |
| [Thermal](plugins/thermal/) | Effect | Thermal-camera false colour: iron, rainbow, white hot or black hot |
| [Retro LCD](plugins/retro-lcd/) | Effect | Low-res two-tone phone LCD with dither and a pixel grid |

## Install

Ready-built plugins come from CI: open the latest run on the
[Actions page](https://github.com/trop3n/resolume-plugins/actions) and
download `<Plugin>-windows` or `<Plugin>-macos` (needs a GitHub login).

- **Windows:** put the `.dll` in `Documents\Resolume Arena\Extra Effects\`.
- **macOS:** unzip and follow the `INSTALL.txt` inside.

Restart Resolume afterwards; it only scans plugins at startup.

### From source

Windows (builds every plugin and installs it for Arena; add `-Avenue` for
Avenue, or name plugins to build only those):

```powershell
powershell -ExecutionPolicy Bypass -File install.ps1
powershell -ExecutionPolicy Bypass -File install.ps1 rusci-scope
```

macOS (needs Xcode command line tools):

```sh
rustup target add aarch64-apple-darwin x86_64-apple-darwin
scripts/bundle-macos.sh rusci-scope
```

## Layout

```
Cargo.toml                 workspace: lists every plugin
plugins/<name>/            one Rust crate per plugin
  Cargo.toml               sets [package.metadata.resolume] file-name
  packaging/macos/         Info.plist (with @VERSION@) and INSTALL.txt
  examples/ffgl_host.rs    optional test host; CI runs it on macOS if present
crates/ffgl-fx/             shared FFGL plumbing for the GLSL effect plugins
tools/fx-host/             test host: renders a PNG through an effect plugin
scripts/bundle-macos.sh    builds a plugin's universal macOS bundle
install.ps1                builds and installs plugins on Windows
.github/workflows/ci.yml   clippy, tests and builds for each plugin
```

## Adding a plugin

1. Create `plugins/<name>/` as a `cdylib` crate exporting `plugMain`.
2. In its `Cargo.toml`, add:
   ```toml
   [package.metadata.resolume]
   file-name = "MyPlugin"   # installs as MyPlugin.dll / MyPlugin.bundle
   ```
3. Add `packaging/macos/Info.plist` and `INSTALL.txt` (copy Rusci Scope's
   and change the names and bundle ID).
4. Add `"plugins/<name>"` to `members` in the root `Cargo.toml`, and `<name>`
   to both `plugin:` lists in `.github/workflows/ci.yml`.

### Shader effects

Effects that are GLSL shaders (everything except Rusci Scope) don't write any FFGL
code. They depend on `crates/ffgl-fx`, declare a `static EffectDef` with
their parameters and shader passes, and call `ffgl_fx::export_effect!`.
Each parameter becomes a GLSL uniform; passes can run several times a frame
and read the previous pass through `passIn`. See the docs at the top of
`crates/ffgl-fx/src/lib.rs`, and `plugins/risograph/` for a one-pass example.

To check one on a GPU without Resolume (Windows or macOS):

```sh
cargo build --release -p risograph -p fx-host
target/release/fx-host target/release/risograph.dll input.png out.png Screen=Halftone "Offset X=0.7"
```

It loads the plugin, checks the GL state after every call the way the FFGL
SDK does, prints the frame time and saves the output. Slider values are
what Resolume sends (0..1); options can be given by name.

## License

GPL-3.0. See [LICENSE](LICENSE).
