#!/bin/sh
set -eu
if [ -n "${SNAP:-}" ]; then
    game="$SNAP/bin/SkyPop"
else
    game_dir=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
    game="$game_dir/SkyPop.x86_64"
fi
# Honour an explicit display-driver selection when troubleshooting.
case " $* " in
    *" --display-driver "*) exec "$game" "$@" ;;
esac
if [ "${XDG_SESSION_TYPE:-}" = wayland ] && [ -n "${WAYLAND_DISPLAY:-}" ]; then
    exec "$game" --display-driver wayland "$@"
fi
exec "$game" "$@"
