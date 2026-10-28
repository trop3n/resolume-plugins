# resolume-plugins

FFGL plugins for Resolume Arena and Avenue, written in Rust. Each plugin is
built for Windows x64 and macOS (one universal bundle for Apple Silicon and
Intel).

| Plugin | Type | What it does |
|--------|------|--------------|
| [Rusci Scope](plugins/rusci-scope/) | Source | Oscilloscope-style vector graphics (shapes, SVGs) using [rusci-render](https://github.com/trop3n/rusci-render) |

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

## License

GPL-3.0. See [LICENSE](LICENSE).
