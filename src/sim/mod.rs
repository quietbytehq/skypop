// SPDX-License-Identifier: Apache-2.0
//! Deterministic, render-free game simulation.
//!
//! The simulation advances in fixed `STEP` increments and only ever sees
//! `InputFrame`s, never devices. That keeps it usable both locally and on a
//! LAN host, where the second player's frames arrive over the network.
//! Everything audiovisual is driven by the `SimEvent`s each step emits.

pub mod entities;
pub mod level;

use serde::{Deserialize, Serialize};

pub use entities::*;
use level::LevelDef;

/// Fixed simulation rate.
pub const STEP_HZ: u32 = 60;
pub const STEP: f32 = 1.0 / STEP_HZ as f32;

pub const VIEW_W: f32 = 1280.0;
pub const VIEW_H: f32 = 720.0;
pub const ARENA_LEFT: f32 = 16.0;
pub const ARENA_RIGHT: f32 = VIEW_W - 16.0;
pub const ARENA_TOP: f32 = 16.0;
/// The floor the players stand on; the HUD sits below it.
pub const ARENA_BOTTOM: f32 = 640.0;

pub const MAX_PLAYERS: usize = 2;
/// Rightmost centre position of the boss; Nina's cage hangs beyond it.
pub const BOSS_SWEEP_RIGHT: f32 = ARENA_RIGHT - 180.0 - BOSS_W * 0.5;
const FREEZE_TIME: f32 = 4.5;
const SLOW_TIME: f32 = 7.0;
const SLOW_FACTOR: f32 = 0.4;
const HIT_INVULN: f32 = 2.4;
const BONUS_POINTS: u32 = 1000;
const BOSS_HIT_POINTS: u32 = 60;
const BOSS_DOWN_DELAY: f32 = 2.8;
const ALL_DOWN_DELAY: f32 = 1.6;

/// One player's held buttons for a single simulation step.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct InputFrame(pub u8);

impl InputFrame {
    pub const LEFT: u8 = 1;
    pub const RIGHT: u8 = 2;
    pub const FIRE: u8 = 4;

    pub fn new(left: bool, right: bool, fire: bool) -> Self {
        let mut bits = 0;
        if left {
            bits |= Self::LEFT;
        }
        if right {
            bits |= Self::RIGHT;
        }
        if fire {
            bits |= Self::FIRE;
        }
        Self(bits)
    }

    pub fn left(self) -> bool {
        self.0 & Self::LEFT != 0
    }
    pub fn right(self) -> bool {
        self.0 & Self::RIGHT != 0
    }
    pub fn fire(self) -> bool {
        self.0 & Self::FIRE != 0
    }
}

/// Small xorshift PRNG so runs are reproducible and snapshot-able.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Rng(u32);

impl Rng {
    pub fn new(seed: u32) -> Self {
        Self(seed.max(1))
    }

    pub fn next_u32(&mut self) -> u32 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.0 = x;
        x
    }

    pub fn next_f32(&mut self) -> f32 {
        (self.next_u32() >> 8) as f32 / (1u32 << 24) as f32
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum SimEvent {
    Fire {
        player: u8,
        x: f32,
    },
    Pop {
        x: f32,
        y: f32,
        tier: Tier,
        color: u8,
        points: u32,
    },
    WireBlocked {
        x: f32,
        y: f32,
    },
    AnchorStuck {
        x: f32,
        y: f32,
    },
    PlatformBroken {
        x: f32,
        y: f32,
        w: f32,
        h: f32,
    },
    PlayerHit {
        player: u8,
        x: f32,
        y: f32,
    },
    ShieldBroken {
        player: u8,
        x: f32,
        y: f32,
    },
    Pickup {
        player: u8,
        kind: DropKind,
        x: f32,
        y: f32,
    },
    BossHit {
        x: f32,
        y: f32,
    },
    BossDefeated {
        x: f32,
        y: f32,
    },
    TimeUp,
    LevelCleared,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Status {
    Running,
    /// Every orb is gone (or the boss has fallen and its delay elapsed).
    Cleared,
    /// The boss is down; the world keeps animating briefly before `Cleared`.
    BossDown(f32),
    /// No living players remain; `GameOver` follows after a short delay.
    AllDown(f32),
    GameOver,
}

/// State carried between stages.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Carry {
    pub score: u32,
    pub lives: [u8; MAX_PLAYERS],
    pub players: usize,
}

impl Carry {
    pub fn new(players: usize) -> Self {
        Self {
            score: 0,
            lives: [START_LIVES; MAX_PLAYERS],
            players: players.clamp(1, MAX_PLAYERS),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct World {
    pub tick: u64,
    pub level_index: usize,
    pub level_name: String,
    pub speed: f32,
    pub time_limit: f32,
    pub time_left: f32,
    pub drop_chance: f32,
    pub drop_table: Vec<DropKind>,
    pub score: u32,
    pub freeze: f32,
    pub slow: f32,
    pub status: Status,
    pub players: Vec<Player>,
    pub orbs: Vec<Orb>,
    pub wires: Vec<Wire>,
    pub platforms: Vec<Platform>,
    pub drops: Vec<Drop>,
    pub boss: Option<Boss>,
    rng: Rng,
    next_id: u32,
    #[serde(skip)]
    pub events: Vec<SimEvent>,
}

impl World {
    pub fn new(level_index: usize, def: &LevelDef, carry: &Carry) -> Self {
        let mut w = World {
            tick: 0,
            level_index,
            level_name: def.name.clone(),
            speed: def.speed,
            time_limit: def.time_limit,
            time_left: def.time_limit,
            drop_chance: def.drop_chance,
            drop_table: def.drop_table.clone(),
            score: carry.score,
            freeze: 0.0,
            slow: 0.0,
            status: Status::Running,
            players: Vec::new(),
            orbs: Vec::new(),
            wires: Vec::new(),
            platforms: Vec::new(),
            drops: Vec::new(),
            boss: None,
            rng: Rng::new(0x5EED_0000 ^ (level_index as u32 + 1).wrapping_mul(2_654_435_761)),
            next_id: 1,
            events: Vec::new(),
        };
        let mid = (ARENA_LEFT + ARENA_RIGHT) * 0.5;
        for i in 0..carry.players {
            let x = if carry.players == 1 {
                mid
            } else {
                mid + (i as f32 - 0.5) * 200.0
            };
            let lives = carry.lives[i];
            w.players.push(Player {
                index: i as u8,
                x,
                facing: 1,
                moving: false,
                lives,
                alive: lives > 0,
                invuln: 1.0,
                shield: false,
                weapon: def.weapon,
                prev_fire: true,
                shot_t: 10.0,
            });
        }
        for o in &def.orbs {
            let id = w.alloc_id();
            w.orbs.push(Orb {
                id,
                tier: o.tier,
                x: o.x,
                y: o.y,
                vx: o.tier.h_speed() * o.dir as f32,
                vy: 0.0,
                color: o.color,
            });
        }
        for p in &def.platforms {
            let id = w.alloc_id();
            w.platforms.push(Platform {
                id,
                x: p.x,
                y: p.y,
                w: p.w,
                h: p.h,
                kind: p.kind,
            });
        }
        if let Some(b) = &def.boss {
            w.boss = Some(Boss {
                x: (ARENA_LEFT + BOSS_W * 0.5 + 20.0 + BOSS_SWEEP_RIGHT) * 0.5,
                y: 110.0,
                hp: b.hp,
                max_hp: b.hp,
                t: 0.0,
                spawn_timer: 1.5,
                spawn_index: 0,
                flash: 0.0,
            });
        }
        w
    }

    pub fn carry(&self) -> Carry {
        let mut lives = [0; MAX_PLAYERS];
        for p in &self.players {
            lives[p.index as usize] = p.lives;
        }
        Carry {
            score: self.score,
            lives,
            players: self.players.len(),
        }
    }

    fn alloc_id(&mut self) -> u32 {
        let id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1);
        id
    }

    /// Time multiplier applied to orbs and the boss (level speed + power-ups).
    pub fn orb_time_scale(&self) -> f32 {
        if self.freeze > 0.0 {
            0.0
        } else if self.slow > 0.0 {
            self.speed * SLOW_FACTOR
        } else {
            self.speed
        }
    }

    /// Advances the world by one fixed step. `inputs[i]` drives player `i`.
    pub fn step(&mut self, inputs: &[InputFrame], def: &LevelDef) {
        self.tick += 1;
        let dt = STEP;
        match self.status {
            Status::Cleared | Status::GameOver => return,
            Status::BossDown(t) => {
                let t = t + dt;
                self.status = if t >= BOSS_DOWN_DELAY {
                    Status::Cleared
                } else {
                    Status::BossDown(t)
                };
                if let Some(b) = &mut self.boss {
                    b.t += dt;
                    b.y += 60.0 * dt;
                }
                self.update_drops(dt);
                return;
            }
            Status::AllDown(t) => {
                let t = t + dt;
                self.status = if t >= ALL_DOWN_DELAY {
                    Status::GameOver
                } else {
                    Status::AllDown(t)
                };
                // Orbs keep bouncing behind the "game over" fade.
                let odt = dt * self.orb_time_scale();
                self.update_orbs(odt);
                return;
            }
            Status::Running => {}
        }

        self.freeze = (self.freeze - dt).max(0.0);
        self.slow = (self.slow - dt).max(0.0);
        self.update_timer(dt);
        self.update_players(inputs, dt);
        let odt = dt * self.orb_time_scale();
        self.update_orbs(odt);
        if let Some(bdef) = &def.boss {
            self.update_boss(bdef, odt, dt);
        }
        self.update_wires(dt);
        self.update_drops(dt);
        self.check_player_hits();
        self.check_end();
    }

    fn update_timer(&mut self, dt: f32) {
        self.time_left -= dt;
        if self.time_left > 0.0 {
            return;
        }
        self.time_left = self.time_limit;
        self.events.push(SimEvent::TimeUp);
        for p in &mut self.players {
            if p.alive {
                p.lives = p.lives.saturating_sub(1);
                p.alive = p.lives > 0;
                p.invuln = HIT_INVULN;
                p.shield = false;
            }
        }
    }

    fn update_players(&mut self, inputs: &[InputFrame], dt: f32) {
        let half = PLAYER_W * 0.5;
        for i in 0..self.players.len() {
            let input = inputs.get(i).copied().unwrap_or_default();
            let p = &mut self.players[i];
            p.invuln = (p.invuln - dt).max(0.0);
            p.shot_t += dt;
            if !p.alive {
                p.prev_fire = input.fire();
                continue;
            }
            let dir = input.right() as i8 - input.left() as i8;
            p.moving = dir != 0;
            if dir != 0 {
                p.facing = dir;
                p.x = (p.x + dir as f32 * PLAYER_SPEED * dt)
                    .clamp(ARENA_LEFT + half, ARENA_RIGHT - half);
            }
            let fire_edge = input.fire() && !p.prev_fire;
            p.prev_fire = input.fire();
            if !fire_edge {
                continue;
            }
            let (owner, x, weapon) = (p.index, p.x, p.weapon);
            let active = self.wires.iter().filter(|w| w.owner == owner).count();
            if active < weapon.max_wires() {
                let id = self.alloc_id();
                self.wires.push(Wire {
                    id,
                    owner,
                    anchor: weapon == Weapon::AnchorWire,
                    x,
                    tip_y: ARENA_BOTTOM - PLAYER_H * 0.6,
                    stuck: None,
                });
                self.players[i].shot_t = 0.0;
                self.events.push(SimEvent::Fire { player: owner, x });
            }
        }
    }

    fn update_orbs(&mut self, dt: f32) {
        if dt <= 0.0 {
            return;
        }
        let platforms = &self.platforms;
        for o in &mut self.orbs {
            step_orb(o, dt, platforms);
        }
    }

    fn update_boss(&mut self, def: &level::BossDef, odt: f32, dt: f32) {
        let Some(b) = &mut self.boss else { return };
        b.flash = (b.flash - dt).max(0.0);
        let phase = b.phase();
        let pace = [1.0, 1.35, 1.75][phase as usize];
        b.t += odt * def.speed * pace;
        // Sweep between the left wall and Nina's cage in the right corner.
        let (lo, hi) = (ARENA_LEFT + BOSS_W * 0.5 + 20.0, BOSS_SWEEP_RIGHT);
        b.x = (lo + hi) * 0.5 + (hi - lo) * 0.5 * (b.t * 0.55).sin();
        b.y = 110.0 + 14.0 * (b.t * 1.7).sin();
        b.spawn_timer -= odt;
        if b.spawn_timer > 0.0 {
            return;
        }
        let interval = def.spawn_interval * [1.0, 0.75, 0.55][phase as usize];
        b.spawn_timer = interval;
        if self.orbs.len() >= def.max_orbs {
            return;
        }
        let tier = def.spawn_tiers[b.spawn_index % def.spawn_tiers.len()];
        let dir = if b.spawn_index % 2 == 0 { 1.0 } else { -1.0 };
        b.spawn_index += 1;
        let (x, y) = (b.x, b.bottom() + tier.radius() + 4.0);
        let color = (b.spawn_index % 6) as u8;
        let id = self.alloc_id();
        self.orbs.push(Orb {
            id,
            tier,
            x,
            y,
            vx: tier.h_speed() * dir,
            vy: 0.0,
            color,
        });
    }

    fn update_wires(&mut self, dt: f32) {
        let mut i = 0;
        while i < self.wires.len() {
            if self.advance_wire(i, dt) {
                i += 1;
            } else {
                self.wires.swap_remove(i);
            }
        }
    }

    /// Returns false when the wire should be removed.
    fn advance_wire(&mut self, i: usize, dt: f32) -> bool {
        let w = &mut self.wires[i];
        if let Some(t) = &mut w.stuck {
            *t -= dt;
            if *t <= 0.0 {
                return false;
            }
        } else {
            w.tip_y -= WIRE_SPEED * dt;
        }
        let (x, tip) = (w.x, w.tip_y);

        // Orbs touching any part of the wire pop.
        if let Some(oi) = self
            .orbs
            .iter()
            .position(|o| (o.x - x).abs() < o.tier.radius() && o.y + o.tier.radius() > tip)
        {
            self.pop_orb(oi, true);
            return false;
        }
        if self.wires[i].stuck.is_some() {
            return true;
        }

        // Whatever is directly above blocks the wire: boss, platform, ceiling.
        let mut stop = ARENA_TOP;
        let mut platform = None;
        for (pi, p) in self.platforms.iter().enumerate() {
            if p.spans_x(x) && p.bottom() > stop && p.bottom() < ARENA_BOTTOM - PLAYER_H * 0.6 {
                stop = p.bottom();
                platform = Some(pi);
            }
        }
        if let Some(b) = &mut self.boss {
            if self.status == Status::Running
                && b.spans_x(x)
                && b.bottom() > stop
                && tip <= b.bottom()
            {
                b.hp = b.hp.saturating_sub(1);
                b.flash = 0.18;
                self.score += BOSS_HIT_POINTS;
                self.events.push(SimEvent::BossHit { x, y: b.bottom() });
                return false;
            }
        }
        if tip > stop {
            return true;
        }
        self.wires[i].tip_y = stop;
        if let Some(pi) = platform {
            if self.platforms[pi].kind == PlatformKind::Breakable {
                let p = self.platforms.remove(pi);
                self.events.push(SimEvent::PlatformBroken {
                    x: p.x,
                    y: p.y,
                    w: p.w,
                    h: p.h,
                });
                return false;
            }
        }
        if self.wires[i].anchor {
            self.wires[i].stuck = Some(ANCHOR_STICK_TIME);
            self.events.push(SimEvent::AnchorStuck { x, y: stop });
            true
        } else {
            self.events.push(SimEvent::WireBlocked { x, y: stop });
            false
        }
    }

    /// Pops an orb: splits it in two (or destroys a small one).
    fn pop_orb(&mut self, index: usize, may_drop: bool) {
        let o = self.orbs.swap_remove(index);
        let points = o.tier.score();
        self.score += points;
        self.events.push(SimEvent::Pop {
            x: o.x,
            y: o.y,
            tier: o.tier,
            color: o.color,
            points,
        });
        if let Some(next) = o.tier.next() {
            let hop = -(2.0 * GRAVITY * SPLIT_HOP).sqrt();
            let r = next.radius();
            for dir in [-1.0f32, 1.0] {
                let id = self.alloc_id();
                let x = (o.x + dir * r * 0.5).clamp(ARENA_LEFT + r, ARENA_RIGHT - r);
                let y = o.y.max(ARENA_TOP + r);
                self.orbs.push(Orb {
                    id,
                    tier: next,
                    x,
                    y,
                    vx: next.h_speed() * dir,
                    vy: hop,
                    color: o.color,
                });
            }
        }
        if may_drop && !self.drop_table.is_empty() && self.rng.next_f32() < self.drop_chance {
            let kind = self.drop_table[self.rng.next_u32() as usize % self.drop_table.len()];
            let id = self.alloc_id();
            self.drops.push(Drop {
                id,
                kind,
                x: o.x,
                y: o.y,
                landed: false,
                life: DROP_LIFETIME,
            });
        }
    }

    fn update_drops(&mut self, dt: f32) {
        let half = DROP_SIZE * 0.5;
        for d in &mut self.drops {
            if d.landed {
                d.life -= dt;
                continue;
            }
            d.y += DROP_FALL_SPEED * dt;
            // Drops come to rest on platforms as well as the floor.
            let mut rest = ARENA_BOTTOM;
            for p in &self.platforms {
                if p.spans_x(d.x) && d.y - DROP_FALL_SPEED * dt + half <= p.y && p.y < rest {
                    rest = p.y;
                }
            }
            if d.y + half >= rest {
                d.y = rest - half;
                d.landed = true;
            }
        }
        self.drops.retain(|d| d.life > 0.0);

        if self.status != Status::Running {
            return;
        }
        let mut i = 0;
        while i < self.drops.len() {
            let d = &self.drops[i];
            let picker = self.players.iter().position(|p| {
                p.alive
                    && (p.x - d.x).abs() < (PLAYER_W + DROP_SIZE) * 0.5
                    && d.y + half > ARENA_BOTTOM - PLAYER_H
            });
            match picker {
                Some(pi) => {
                    let d = self.drops.swap_remove(i);
                    self.apply_drop(pi, &d);
                }
                None => i += 1,
            }
        }
    }

    fn apply_drop(&mut self, pi: usize, d: &Drop) {
        let p = &mut self.players[pi];
        match d.kind {
            DropKind::Shield => p.shield = true,
            DropKind::Anchor => p.weapon = Weapon::AnchorWire,
            DropKind::Double => p.weapon = Weapon::DoubleWire,
            DropKind::Freeze => self.freeze = FREEZE_TIME,
            DropKind::Slow => self.slow = SLOW_TIME,
            DropKind::Bonus => self.score += BONUS_POINTS,
        }
        let player = self.players[pi].index;
        self.events.push(SimEvent::Pickup {
            player,
            kind: d.kind,
            x: d.x,
            y: d.y,
        });
    }

    fn check_player_hits(&mut self) {
        for p in &mut self.players {
            if !p.alive || p.invuln > 0.0 {
                continue;
            }
            // A slightly forgiving hitbox.
            let (hw, top) = (PLAYER_W * 0.38, ARENA_BOTTOM - PLAYER_H * 0.85);
            let hit = self.orbs.iter().any(|o| {
                let cx = o.x.clamp(p.x - hw, p.x + hw);
                let cy = o.y.clamp(top, ARENA_BOTTOM);
                let (dx, dy) = (o.x - cx, o.y - cy);
                dx * dx + dy * dy < o.tier.radius() * o.tier.radius()
            });
            if !hit {
                continue;
            }
            let (x, y) = (p.x, ARENA_BOTTOM - PLAYER_H * 0.5);
            if p.shield {
                p.shield = false;
                p.invuln = 1.2;
                self.events.push(SimEvent::ShieldBroken {
                    player: p.index,
                    x,
                    y,
                });
            } else {
                p.lives = p.lives.saturating_sub(1);
                p.alive = p.lives > 0;
                p.invuln = HIT_INVULN;
                self.events.push(SimEvent::PlayerHit {
                    player: p.index,
                    x,
                    y,
                });
            }
        }
    }

    fn check_end(&mut self) {
        if !self.players.iter().any(|p| p.alive) {
            self.wires.clear();
            self.status = Status::AllDown(0.0);
            return;
        }
        if let Some(b) = &self.boss {
            if b.hp == 0 {
                let (x, y) = (b.x, b.y);
                while !self.orbs.is_empty() {
                    let o = self.orbs.swap_remove(0);
                    self.score += o.tier.score();
                    self.events.push(SimEvent::Pop {
                        x: o.x,
                        y: o.y,
                        tier: o.tier,
                        color: o.color,
                        points: o.tier.score(),
                    });
                }
                self.wires.clear();
                self.status = Status::BossDown(0.0);
                self.events.push(SimEvent::BossDefeated { x, y });
            }
            return;
        }
        if self.orbs.is_empty() {
            self.wires.clear();
            self.status = Status::Cleared;
            self.events.push(SimEvent::LevelCleared);
        }
    }
}

/// Integrates one orb and resolves collisions with the arena and platforms.
/// Uses exact constant-acceleration integration, so bounce apexes are stable.
fn step_orb(o: &mut Orb, dt: f32, platforms: &[Platform]) {
    let r = o.tier.radius();
    o.x += o.vx * dt;
    o.y += o.vy * dt + 0.5 * GRAVITY * dt * dt;
    o.vy += GRAVITY * dt;

    if o.x - r < ARENA_LEFT {
        o.x = ARENA_LEFT + r;
        o.vx = o.vx.abs();
    } else if o.x + r > ARENA_RIGHT {
        o.x = ARENA_RIGHT - r;
        o.vx = -o.vx.abs();
    }
    if o.y - r < ARENA_TOP {
        o.y = ARENA_TOP + r;
        o.vy = o.vy.abs();
    }
    if o.y + r > ARENA_BOTTOM {
        o.y = ARENA_BOTTOM - r;
        o.vy = -o.tier.bounce_speed();
    }

    for p in platforms {
        let cx = o.x.clamp(p.x, p.x + p.w);
        let cy = o.y.clamp(p.y, p.y + p.h);
        let (dx, dy) = (o.x - cx, o.y - cy);
        let d2 = dx * dx + dy * dy;
        if d2 >= r * r {
            continue;
        }
        let (nx, ny, d) = if d2 > 1e-6 {
            let d = d2.sqrt();
            (dx / d, dy / d, d)
        } else {
            // Centre inside the platform: push out along the shallowest axis.
            let up = o.y - p.y;
            let down = p.y + p.h - o.y;
            if up < down {
                (0.0, -1.0, -up)
            } else {
                (0.0, 1.0, -down)
            }
        };
        o.x += nx * (r - d);
        o.y += ny * (r - d);
        if ny.abs() >= nx.abs() {
            if ny < 0.0 {
                o.vy = -o.tier.bounce_speed();
            } else {
                o.vy = o.vy.abs();
            }
        } else {
            o.vx = o.vx.abs() * nx.signum();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn levels() -> Vec<LevelDef> {
        level::load_all().expect("levels parse")
    }

    fn lone_orb(tier: Tier) -> Orb {
        Orb {
            id: 1,
            tier,
            x: 640.0,
            y: 300.0,
            vx: tier.h_speed(),
            vy: 0.0,
            color: 0,
        }
    }

    #[test]
    fn all_levels_load_and_last_is_boss() {
        let ls = levels();
        assert_eq!(ls.len(), level::LEVEL_COUNT);
        assert!(ls[..9].iter().all(|l| l.boss.is_none()));
        assert!(ls[9].boss.is_some());
    }

    #[test]
    fn bounce_apex_is_fixed_per_tier() {
        for tier in [Tier::XL, Tier::L, Tier::M, Tier::S] {
            let mut o = lone_orb(tier);
            let mut apexes = Vec::new();
            let mut prev_vy = o.vy;
            for _ in 0..(STEP_HZ * 20) {
                step_orb(&mut o, STEP, &[]);
                if prev_vy < 0.0 && o.vy >= 0.0 {
                    apexes.push(o.y);
                }
                prev_vy = o.vy;
            }
            assert!(apexes.len() >= 3, "{tier:?} bounced");
            let want = ARENA_BOTTOM - tier.radius() - tier.bounce_height();
            // Skip the first apex (the drop from spawn height).
            for a in &apexes[1..] {
                assert!((a - want).abs() < 6.0, "{tier:?}: apex {a} vs {want}");
            }
        }
    }

    #[test]
    fn hit_splits_into_two_of_next_tier_moving_apart() {
        let ls = levels();
        let mut w = World::new(0, &ls[0], &Carry::new(1));
        w.drop_chance = 0.0;
        w.orbs = vec![lone_orb(Tier::XL)];
        w.pop_orb(0, false);
        assert_eq!(w.orbs.len(), 2);
        assert!(w.orbs.iter().all(|o| o.tier == Tier::L));
        assert!(w.orbs[0].vx.signum() != w.orbs[1].vx.signum());

        w.orbs = vec![lone_orb(Tier::S)];
        w.pop_orb(0, false);
        assert!(w.orbs.is_empty());
    }

    #[test]
    fn simulation_is_deterministic() {
        let ls = levels();
        let run = || {
            let mut w = World::new(5, &ls[5], &Carry::new(2));
            for t in 0..(STEP_HZ * 30) {
                let a = InputFrame::new(t % 90 < 40, t % 90 >= 45, t % 23 == 0);
                let b = InputFrame::new(t % 70 >= 35, t % 70 < 30, t % 17 == 0);
                w.step(&[a, b], &ls[5]);
                w.events.clear();
            }
            ron::to_string(&w).unwrap()
        };
        assert_eq!(run(), run());
    }

    #[test]
    fn harpoon_reaches_orb_and_scores() {
        let ls = levels();
        let mut w = World::new(0, &ls[0], &Carry::new(1));
        w.drop_chance = 0.0;
        let mut o = lone_orb(Tier::M);
        o.x = w.players[0].x;
        o.vx = 0.0;
        o.y = 200.0;
        w.orbs = vec![o];
        w.players[0].prev_fire = false;
        w.step(&[InputFrame::new(false, false, true)], &ls[0]);
        assert_eq!(w.wires.len(), 1);
        for _ in 0..60 {
            w.step(&[InputFrame::default()], &ls[0]);
        }
        assert!(w.orbs.iter().all(|o| o.tier == Tier::S));
        assert!(w.score >= Tier::M.score());
    }

    #[test]
    fn breakable_platform_breaks_and_solid_blocks() {
        let ls = levels();
        let mut w = World::new(0, &ls[0], &Carry::new(1));
        w.orbs.clear();
        w.orbs.push(Orb {
            id: 99,
            tier: Tier::S,
            x: 100.0,
            y: 100.0,
            vx: 0.0,
            vy: 0.0,
            color: 0,
        });
        let px = w.players[0].x;
        w.platforms = vec![Platform {
            id: 50,
            x: px - 50.0,
            y: 300.0,
            w: 100.0,
            h: 24.0,
            kind: PlatformKind::Breakable,
        }];
        w.players[0].prev_fire = false;
        w.step(&[InputFrame::new(false, false, true)], &ls[0]);
        for _ in 0..40 {
            w.step(&[InputFrame::default()], &ls[0]);
        }
        assert!(w.platforms.is_empty());
        assert!(w.wires.is_empty());
    }

    #[test]
    fn freeze_stops_orbs() {
        let ls = levels();
        let mut w = World::new(0, &ls[0], &Carry::new(1));
        w.freeze = 1.0;
        let before: Vec<(f32, f32)> = w.orbs.iter().map(|o| (o.x, o.y)).collect();
        w.step(&[InputFrame::default()], &ls[0]);
        let after: Vec<(f32, f32)> = w.orbs.iter().map(|o| (o.x, o.y)).collect();
        assert_eq!(before, after);
    }

    #[test]
    fn boss_takes_damage_from_wires() {
        let ls = levels();
        let mut w = World::new(9, &ls[9], &Carry::new(1));
        let hp = w.boss.as_ref().unwrap().hp;
        // Park the player under the boss and keep firing.
        for t in 0..(STEP_HZ * 4) {
            let bx = w.boss.as_ref().unwrap().x;
            w.players[0].x = bx;
            w.players[0].invuln = 10.0;
            w.step(&[InputFrame::new(false, false, t % 2 == 0)], &ls[9]);
        }
        assert!(w.boss.as_ref().unwrap().hp < hp);
    }
}
