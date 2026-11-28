#!/bin/sh
set -eu
cd "$(dirname "$0")/.."
version=$(sed -n 's/^config\/version="\([^"]*\)"/\1/p' project.godot)
out="build/$version/Linux"
mkdir -p "$out"
touch build/.gdignore
"${GODOT:-godot}" --headless --editor --path . --import --quit
"${GODOT:-godot}" --headless --path . --export-release 'SKYPOP Linux Desktop' "$out/SkyPop.x86_64"
cp tools/launch-linux.sh "$out/SkyPop"
chmod +x "$out/SkyPop" "$out/SkyPop.x86_64"
cp LICENSE NOTICE README.md "$out/"
cp -R LICENSES "$out/"
printf '%s\n' 'Run ./SkyPop to start. G or gamepad Y opens Global Top 100 from the title or local score screen.' > "$out/README.txt"
cp docs/SkyPop-How-to-Play.pdf "$out/"
