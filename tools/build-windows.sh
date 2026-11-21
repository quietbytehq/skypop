#!/bin/sh
set -eu
cd "$(dirname "$0")/.."
mkdir -p build/1.1.0/Windows
touch build/.gdignore
"${GODOT:-godot}" --headless --editor --path . --import --quit
"${GODOT:-godot}" --headless --path . --export-release 'SKYPOP Windows Desktop' build/1.1.0/Windows/SkyPop.exe
cp LICENSE NOTICE README.md build/1.1.0/Windows/
cp -R LICENSES build/1.1.0/Windows/
cp tools/README-windows.txt build/1.1.0/Windows/README.txt
