#!/usr/bin/env bash
# Packages a built executable into dist/ as a release archive:
#   package.sh linux <cadrs>       → dist/cadrs-v<version>-x86_64-linux.tar.gz
#   package.sh windows <cadrs.exe> → dist/cadrs-v<version>-x86_64-windows.zip
# Each holds the stripped executable, the built-in libraries/, README.md and the licenses. Used
# by tools/release/build.sh and .github/workflows/release.yml.
set -euo pipefail

root="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$root"
platform="$1"
exe="$2"
version="$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)"
name="cadrs-v$version-x86_64-$platform"
dist="$root/dist"
dir="$dist/$name"

case "$platform" in
    linux) exe_name=cadrs strip=strip ;;
    windows) exe_name=cadrs.exe strip=x86_64-w64-mingw32-strip ;;
    *) echo "unknown platform: $platform" >&2; exit 1 ;;
esac

rm -rf "$dir"
mkdir -p "$dir"
cp "$exe" "$dir/$exe_name"
"$strip" "$dir/$exe_name"
cp README.md LICENSE-APACHE LICENSE-MIT "$dir/"
# The built-in symbols, footprints and 3D models: the app reads them from `libraries/` beside
# the executable.
cp -r libraries "$dir/"

cd "$dist"
case "$platform" in
    linux) rm -f "$name.tar.gz" && tar czf "$name.tar.gz" "$name" ;;
    windows) rm -f "$name.zip" && zip -qr "$name.zip" "$name" ;;
esac
ls -l "$dist/$name".*
