#!/bin/sh
set -eu
cd "$(dirname "$0")/.."
version=$(sed -n 's/^config\/version="\([^" ]*\)"/\1/p' project.godot)
out="build/$version/Windows"
mkdir -p "$out"
touch build/.gdignore
"${GODOT:-godot}" --headless --editor --path . --import --quit
"${GODOT:-godot}" --headless --path . --export-release 'SKYPOP Windows Desktop' "$out/SkyPop.exe"
cp LICENSE NOTICE README.md "$out/"
cp -R LICENSES "$out/"
cp tools/README-windows.txt "$out/README.txt"
