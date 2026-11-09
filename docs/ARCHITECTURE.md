<!-- SPDX-License-Identifier: Apache-2.0 -->
# SkyPop architecture

SkyPop is built so that LAN co-op (milestone 2) can be added without
restructuring the game. The rules below hold from day one.

## Simulation is separate from presentation

`src/sim/` holds the whole game state (`World`) and its rules. It:

- advances only in fixed steps (`STEP = 1/60 s`) through `World::step`;
- reads nothing but `InputFrame`s, one per player per step;
- uses its own seeded PRNG, so the same inputs always produce the same state;
- does no drawing, audio or file I/O. Everything noteworthy comes out as a
  `SimEvent` (`Pop`, `PlayerHit`, `BossHit`, …) that the app turns into sound,
  particles and screen shake;
- is plain data (`Clone + Serialize`), so a `World` can be snapshotted.

A unit test (`simulation_is_deterministic`) runs the same two-player input
script twice and compares the serialised worlds.

## Game loop

`app.rs` owns a `Session` per stage. Each frame it:

1. samples every player's `InputSource`, OR-ing the result into a latch so a
   tap during a frame that runs no step is not lost;
2. runs as many fixed steps as the elapsed time covers (capped at 8);
3. keeps the previous `World` and the leftover fraction `alpha`.

The renderer draws `lerp(prev, current, alpha)` per entity, matched by stable
entity ids, so motion is smooth at any refresh rate. The same interpolation
will smooth network snapshots on the client.

## Input abstraction

```rust
pub trait InputSource {
    fn sample(&mut self, controls: &Controls) -> InputFrame;
}
```

`LocalInput` reads the keyboard and gamepad through the player's bindings.
The simulation never knows where a frame came from.

## LAN co-op plan (milestone 2)

- **Topology**: two PCs. The host runs the only authoritative `World`.
- **Client → host**: the client sends its `InputFrame` each tick, tagged with a
  sequence number, and resends the last few frames to cover packet loss. On
  the host these arrive through a `NetworkInput: InputSource` for player 2.
- **Host → client**: the host sends `World` snapshots (serialised with serde)
  at a fixed rate together with that step's `SimEvent`s. The client renders
  by interpolating between the two most recent snapshots (the same code path
  as local `prev`/`current` interpolation) and plays the events for audio and
  effects.
- **Discovery**: the host broadcasts a small UDP announcement on the LAN. The
  lobby lists hosts it hears, and offers manual IP entry as a fallback.
- **Transport**: renet, or plain UDP if that is simpler. LAN only, no relay
  or NAT traversal.
- **States**: Host and Join lobby screens sit alongside the existing Title,
  Story Intro, Playing, Level Clear, Boss, Pause, Game Over and Victory
  screens. The title menu's *LAN Co-op* entry already leads to the lobby
  placeholder.

The simulation already supports two players (`Carry::players`, per-player
lives, weapons and shields, a shared score, a P2 HUD block), so milestone 2
only has to add the transport, lobby and snapshot handling.

## Content and assets

- Stages live in `data/levels/*.ron`, are embedded with `include_str!`, and
  are validated on load. `SKYPOP_DATA_DIR` overrides them at runtime.
- All art is drawn from primitives (`render/`), including the headline font
  (`render/glyphs.rs`). All sound effects are synthesised into in-memory WAVs
  at start-up (`audio.rs`). There are no external asset files.
