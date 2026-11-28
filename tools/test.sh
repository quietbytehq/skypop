#!/bin/sh
set -eu
cd "$(dirname "$0")/.."
test_dir=$(mktemp -d)
trap 'rm -rf "$test_dir"' EXIT HUP INT TERM
export XDG_CONFIG_HOME="$test_dir/config"
for name in game story_online pause_menu browser global_browser; do
  status=0
  XDG_DATA_HOME="$test_dir/$name" "${GODOT:-godot}" --headless --path . --script "tests/test_$name.gd" > "$test_dir/$name.log" 2>&1 || status=$?
  cat "$test_dir/$name.log"
  if [ "$status" -ne 0 ] || grep -Eq 'SCRIPT ERROR|Parse Error|Compile Error|[1-9][0-9]* failures' "$test_dir/$name.log"; then exit 1; fi
done
