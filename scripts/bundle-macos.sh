#!/usr/bin/env bash
# Build RusciScope.bundle for macOS: a universal (Apple Silicon + Intel)
# FFGL plugin, ad-hoc signed, zipped with install instructions.
#
#   scripts/bundle-macos.sh
#
# Needs Xcode command line tools and both Rust targets:
#   rustup target add aarch64-apple-darwin x86_64-apple-darwin
#
# Output: target/macos/RusciScope-macos/ and target/macos/RusciScope-macos.zip

set -euo pipefail
cd "$(dirname "$0")/.."

# Resolume 7.11+ runs natively on Apple Silicon, which needs macOS 11;
# Intel builds keep the FFGL SDK's minimum.
export MACOSX_DEPLOYMENT_TARGET="${MACOSX_DEPLOYMENT_TARGET:-10.15}"

version=$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -n 1)
targets=(aarch64-apple-darwin x86_64-apple-darwin)

for target in "${targets[@]}"; do
    cargo build --release --lib --target "$target"
done

out=target/macos/RusciScope-macos
bundle="$out/RusciScope.bundle"
rm -rf target/macos
mkdir -p "$bundle/Contents/MacOS"

libs=()
for target in "${targets[@]}"; do
    libs+=("target/$target/release/librusci_scope.dylib")
done
lipo -create -output "$bundle/Contents/MacOS/RusciScope" "${libs[@]}"
sed "s/@VERSION@/$version/g" packaging/macos/Info.plist > "$bundle/Contents/Info.plist"
cp packaging/macos/INSTALL.txt "$out/"

# Apple Silicon refuses unsigned code; an ad-hoc signature is enough for a
# plugin that the user installs themselves.
codesign --force --sign - --timestamp=none "$bundle"

# Check what Resolume will look for.
lipo "$bundle/Contents/MacOS/RusciScope" -verify_arch arm64 x86_64
nm -gU "$bundle/Contents/MacOS/RusciScope" | grep -q ' _plugMain$'
codesign --verify --strict "$bundle"
plutil -lint "$bundle/Contents/Info.plist" >/dev/null

# ditto keeps permissions and the code signature intact.
ditto -c -k --keepParent "$out" target/macos/RusciScope-macos.zip
echo "Built $bundle ($version, $(lipo -archs "$bundle/Contents/MacOS/RusciScope"))"
echo "Zipped to target/macos/RusciScope-macos.zip"
