# SkyPop

A Pang-inspired arcade game written in Rust with macroquad.

Dr. Demos has kidnapped Nina! Leo fights through ten stages of bouncing neon
orbs, culminating in a boss battle against Dr. Demos and his hover-pod.

SkyPop runs on Linux and Windows. Its game art and sound effects are generated
in code. Menu text uses the ProggyClean font bundled with macroquad.

## Playing

- **Orbs** come in four tiers: XL, L, M and S. Each tier always bounces to the
  same height. Hit an orb and it splits into two orbs of the next tier, flying
  apart. Hit an S orb and it's gone. Clear every orb to finish the stage.
- **Weapons**
  - *Harpoon*: fires one wire, which vanishes when it reaches the ceiling or
    hits an orb.
  - *Anchor wire*: sticks to the ceiling for a few seconds, popping any orb
    that touches it.
  - *Double wire*: lets you have two wires out at once.
- **Drops** sometimes fall from popped orbs: Shield (absorbs one hit), Freeze,
  Slow, Bonus score, plus pickups that switch your weapon.
- **Platforms**: solid ones stop wires. Breakable glass shatters when a wire
  hits it.
- **Stage 10** is the boss. Hit the hover-pod with wires to wear it down; it
  drops orbs faster as its health falls.
- Clearing a stage unlocks it under *Stage Select*.

### Default controls

| Action     | Keyboard            | Gamepad            |
|------------|---------------------|--------------------|
| Move       | ← → / A D           | D-pad / left stick |
| Fire       | Space / J           | A / X              |
| Confirm    | Enter / Space       | A / Start          |
| Back       | Esc / Backspace     | B                  |
| Pause      | Esc / P             | Start              |
| Fullscreen | F11                 |                    |

You can rebind every action under **Controls** on the title screen. Pick an
action, then press the new key or gamepad button.

## Building

You need a stable Rust toolchain (1.89 or newer).

```sh
cargo run --release
```

On Linux, install the development packages for ALSA and udev first (for
audio and gamepads), e.g. on Debian/Ubuntu:

```sh
sudo apt install pkg-config libasound2-dev libudev-dev libx11-dev libxi-dev libgl1-mesa-dev
```

On Windows, use the Rust MSVC toolchain with the Visual Studio C++ Build Tools.

Run the tests with `cargo test`.

## Saves

High scores, unlocked stages, key bindings and the fullscreen setting are
stored in `save.ron` in the OS config directory:

- Linux: `$XDG_CONFIG_HOME/skypop/` (usually `~/.config/skypop/`)
- Windows: `%APPDATA%\QuietByte\SkyPop\config\`

Delete the file to reset everything.

## Level data

Stages are defined in [`data/levels/`](data/levels) as RON files: orbs,
platforms, weapon, drop table, speed, timer, theme and boss settings. They are
embedded into the binary at build time. To try edits without rebuilding,
point `SKYPOP_DATA_DIR` at a directory that contains `levels/levelNN.ron`
files. Any file found there replaces the built-in stage with the same number.

```sh
SKYPOP_DATA_DIR=./data cargo run
```

## Project layout

```
src/
  main.rs          window setup, letterboxed 1280×720 camera, main loop
  app.rs           screen state machine (menus, cutscenes, play, overlays)
  sim/             deterministic fixed-step simulation (no rendering, no I/O)
  input.rs         keyboard/gamepad, rebinding, InputSource abstraction
  render/          procedural drawing: scene, characters, headline font, effects
  audio.rs         synthesised sound effects
  save.rs          persistent data in the OS config directory
data/levels/       stage definitions (RON)
docs/              design notes
```

See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for how the simulation,
rendering and the upcoming LAN co-op fit together.

## Roadmap

- **Milestone 1** (this release): the single-player game, all ten stages, the
  boss, cutscenes, saves and rebinding.
- **Milestone 2**: two-player co-op over a local network (two PCs). Host and
  join lobby, discovery by UDP broadcast with manual IP entry as a fallback,
  and a host-authoritative simulation with client-side interpolation.

## License

Licensed under the [Apache License, Version 2.0](LICENSE).

See [NOTICE](NOTICE) for attribution.

Contact [hello@quietbytehq.uk](mailto:hello@quietbytehq.uk).
