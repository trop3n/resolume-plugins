#!/usr/bin/env bash
# Build a plugin's macOS bundle: a universal (Apple Silicon + Intel) FFGL
# plugin, ad-hoc signed, zipped with install instructions.
#
#   scripts/bundle-macos.sh <plugin>      e.g. scripts/bundle-macos.sh rusci-scope
#
# <plugin> is a folder under plugins/. Its Cargo.toml must set
# [package.metadata.resolume] file-name, and the folder must have
# packaging/macos/Info.plist (with @VERSION@) and packaging/macos/INSTALL.txt.
#
# Needs Xcode command line tools and both Rust targets:
#   rustup target add aarch64-apple-darwin x86_64-apple-darwin
#
# Output: target/macos/<FileName>-macos/ and target/macos/<FileName>-macos.zip

set -euo pipefail
cd "$(dirname "$0")/.."

plugin="${1:?usage: scripts/bundle-macos.sh <plugin folder under plugins/>}"
dir="plugins/$plugin"
manifest="$dir/Cargo.toml"
[[ -f "$manifest" ]] || { echo "No $manifest" >&2; exit 1; }

# Resolume 7.11+ runs natively on Apple Silicon, which needs macOS 11;
# Intel builds keep the FFGL SDK's minimum.
export MACOSX_DEPLOYMENT_TARGET="${MACOSX_DEPLOYMENT_TARGET:-10.15}"

package=$(sed -n '/^\[package\]/,/^\[/ s/^name = "\(.*\)"/\1/p' "$manifest" | head -n 1)
version=$(sed -n '/^\[package\]/,/^\[/ s/^version = "\(.*\)"/\1/p' "$manifest" | head -n 1)
lib=$(sed -n '/^\[lib\]/,/^\[/ s/^name = "\(.*\)"/\1/p' "$manifest" | head -n 1)
lib="${lib:-${package//-/_}}"
name=$(sed -n 's/^file-name = "\(.*\)"/\1/p' "$manifest" | head -n 1)
[[ -n "$name" ]] || { echo "$manifest has no [package.metadata.resolume] file-name" >&2; exit 1; }

targets=(aarch64-apple-darwin x86_64-apple-darwin)
for target in "${targets[@]}"; do
    cargo build --release --lib -p "$package" --target "$target"
done

out="target/macos/$name-macos"
bundle="$out/$name.bundle"
rm -rf "$out" "$out.zip"
mkdir -p "$bundle/Contents/MacOS"

libs=()
for target in "${targets[@]}"; do
    libs+=("target/$target/release/lib$lib.dylib")
done
lipo -create -output "$bundle/Contents/MacOS/$name" "${libs[@]}"
sed "s/@VERSION@/$version/g" "$dir/packaging/macos/Info.plist" > "$bundle/Contents/Info.plist"
cp "$dir/packaging/macos/INSTALL.txt" "$out/"

# Apple Silicon refuses unsigned code; an ad-hoc signature is enough for a
# plugin that the user installs themselves.
codesign --force --sign - --timestamp=none "$bundle"

# Check what Resolume will look for.
lipo "$bundle/Contents/MacOS/$name" -verify_arch arm64 x86_64
nm -gU "$bundle/Contents/MacOS/$name" | grep -q ' _plugMain$'
codesign --verify --strict "$bundle"
plutil -lint "$bundle/Contents/Info.plist" >/dev/null

# ditto keeps permissions and the code signature intact.
ditto -c -k --keepParent "$out" "$out.zip"
echo "Built $bundle ($version, $(lipo -archs "$bundle/Contents/MacOS/$name"))"
echo "Zipped to $out.zip"
