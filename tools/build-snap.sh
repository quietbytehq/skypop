#!/bin/sh
set -eu
cd "$(dirname "$0")/.."
mkdir -p build/snap-payload/bin build/snap-payload/usr/share/doc/skypop
# Install matching Godot export templates first. Local licensed audio is optional.
godot --headless --editor --path . --import --quit
godot --headless --path . --export-release "SKYPOP Linux Desktop" build/snap-payload/bin/SkyPop
cp tools/launch-linux.sh build/snap-payload/bin/launch-skypop
cp LICENSE NOTICE build/snap-payload/usr/share/doc/skypop/
cp -R LICENSES build/snap-payload/usr/share/doc/skypop/
snapcraft pack --use-lxd
