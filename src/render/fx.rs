// SPDX-License-Identifier: Apache-2.0
//! Render-side juice: particles, floating score text, screen shake and
//! flashes. Driven entirely by `SimEvent`s; never feeds back into the sim.

use macroquad::prelude::*;
use macroquad::rand::gen_range;

use super::{lighten, orb_color, text_center, with_alpha};
use crate::sim::{DropKind, SimEvent, Tier};

#[derive(Clone, Copy, PartialEq)]
enum Shape {
    Dot,
    Spark,
    Ring,
    Shard,
    Heart,
}

struct Particle {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    gravity: f32,
    life: f32,
    max: f32,
    size: f32,
    color: Color,
    shape: Shape,
}

struct FloatText {
    text: String,
    x: f32,
    y: f32,
    life: f32,
    color: Color,
}

#[derive(Default)]
pub struct Fx {
    particles: Vec<Particle>,
    texts: Vec<FloatText>,
    shake: f32,
    flash: f32,
    flash_color: Color,
}

impl Fx {
    pub fn clear(&mut self) {
        *self = Fx::default();
    }

    pub fn shake_offset(&self) -> Vec2 {
        if self.shake <= 0.01 {
            return Vec2::ZERO;
        }
        vec2(gen_range(-1.0, 1.0), gen_range(-1.0, 1.0)) * self.shake
    }

    pub fn add_shake(&mut self, amount: f32) {
        self.shake = (self.shake + amount).min(16.0);
    }

    fn flash(&mut self, c: Color, a: f32) {
        self.flash = a;
        self.flash_color = c;
    }

    #[allow(clippy::too_many_arguments)]
    fn burst(
        &mut self,
        x: f32,
        y: f32,
        n: usize,
        speed: f32,
        color: Color,
        shape: Shape,
        size: f32,
    ) {
        for _ in 0..n {
            let a = gen_range(0.0, std::f32::consts::TAU);
            let s = gen_range(speed * 0.3, speed);
            let life = gen_range(0.35, 0.8);
            self.particles.push(Particle {
                x,
                y,
                vx: a.cos() * s,
                vy: a.sin() * s,
                gravity: if shape == Shape::Spark { 200.0 } else { 500.0 },
                life,
                max: life,
                size: gen_range(size * 0.5, size),
                color,
                shape,
            });
        }
    }

    fn ring(&mut self, x: f32, y: f32, size: f32, color: Color, life: f32) {
        self.particles.push(Particle {
            x,
            y,
            vx: 0.0,
            vy: 0.0,
            gravity: 0.0,
            life,
            max: life,
            size,
            color,
            shape: Shape::Ring,
        });
    }

    pub fn float_text(&mut self, text: String, x: f32, y: f32, color: Color) {
        self.texts.push(FloatText {
            text,
            x,
            y,
            life: 1.0,
            color,
        });
    }

    /// Floating hearts for the rescue cutscene.
    pub fn hearts(&mut self, x: f32, y: f32, n: usize) {
        for _ in 0..n {
            let life = gen_range(1.0, 2.0);
            self.particles.push(Particle {
                x: x + gen_range(-30.0, 30.0),
                y,
                vx: gen_range(-40.0, 40.0),
                vy: gen_range(-160.0, -80.0),
                gravity: -20.0,
                life,
                max: life,
                size: gen_range(6.0, 11.0),
                color: Color::new(1.0, gen_range(0.2, 0.5), 0.6, 1.0),
                shape: Shape::Heart,
            });
        }
    }

    pub fn confetti(&mut self, x: f32, y: f32, n: usize) {
        for i in 0..n {
            self.burst(x, y, 1, 520.0, orb_color(i as u8), Shape::Shard, 9.0);
        }
    }

    pub fn on_event(&mut self, ev: &SimEvent) {
        match *ev {
            SimEvent::Pop {
                x,
                y,
                tier,
                color,
                points,
            } => {
                let c = orb_color(color);
                let r = tier.radius();
                let n = match tier {
                    Tier::XL => 34,
                    Tier::L => 26,
                    Tier::M => 18,
                    Tier::S => 12,
                };
                self.burst(x, y, n, 260.0 + r * 5.0, c, Shape::Dot, r * 0.32 + 3.0);
                self.burst(x, y, n / 2, 420.0, lighten(c, 0.6), Shape::Spark, 5.0);
                self.ring(x, y, r, lighten(c, 0.4), 0.35);
                self.float_text(format!("+{points}"), x, y - r, lighten(c, 0.5));
                self.add_shake(1.0 + r * 0.06);
            }
            SimEvent::WireBlocked { x, y } | SimEvent::AnchorStuck { x, y } => {
                self.burst(x, y, 6, 160.0, WHITE, Shape::Spark, 4.0);
            }
            SimEvent::PlatformBroken { x, y, w, h } => {
                for i in 0..24 {
                    let px = x + w * (i as f32 + 0.5) / 24.0;
                    self.burst(
                        px,
                        y + h * 0.5,
                        1,
                        260.0,
                        Color::new(0.7, 0.95, 1.0, 0.9),
                        Shape::Shard,
                        10.0,
                    );
                }
                self.add_shake(3.0);
            }
            SimEvent::PlayerHit { x, y, .. } => {
                self.burst(
                    x,
                    y,
                    30,
                    380.0,
                    Color::new(1.0, 0.3, 0.35, 1.0),
                    Shape::Dot,
                    7.0,
                );
                self.add_shake(9.0);
                self.flash(Color::new(1.0, 0.2, 0.3, 1.0), 0.45);
            }
            SimEvent::ShieldBroken { x, y, .. } => {
                self.ring(x, y, 50.0, Color::new(0.4, 0.9, 1.0, 1.0), 0.5);
                self.burst(
                    x,
                    y,
                    20,
                    300.0,
                    Color::new(0.5, 0.95, 1.0, 1.0),
                    Shape::Shard,
                    8.0,
                );
                self.add_shake(4.0);
            }
            SimEvent::Pickup { kind, x, y, .. } => {
                let (label, c) = match kind {
                    DropKind::Shield => ("SHIELD", Color::new(0.4, 0.9, 1.0, 1.0)),
                    DropKind::Freeze => ("FREEZE!", Color::new(0.6, 0.95, 1.0, 1.0)),
                    DropKind::Slow => ("SLOW", Color::new(0.7, 0.6, 1.0, 1.0)),
                    DropKind::Bonus => ("+1000", Color::new(1.0, 0.85, 0.2, 1.0)),
                    DropKind::Anchor => ("ANCHOR WIRE", Color::new(0.2, 1.0, 0.85, 1.0)),
                    DropKind::Double => ("DOUBLE WIRE", Color::new(1.0, 0.6, 0.9, 1.0)),
                };
                self.float_text(label.to_string(), x, y - 30.0, c);
                self.burst(x, y, 14, 220.0, c, Shape::Spark, 5.0);
                if kind == DropKind::Freeze {
                    self.flash(Color::new(0.6, 0.9, 1.0, 1.0), 0.35);
                }
            }
            SimEvent::BossHit { x, y } => {
                self.burst(
                    x,
                    y,
                    12,
                    300.0,
                    Color::new(1.0, 0.8, 0.3, 1.0),
                    Shape::Spark,
                    6.0,
                );
                self.add_shake(2.5);
            }
            SimEvent::BossDefeated { x, y } => {
                for i in 0..6 {
                    self.burst(
                        x,
                        y,
                        30,
                        500.0 + i as f32 * 60.0,
                        orb_color(i),
                        Shape::Dot,
                        10.0,
                    );
                }
                self.ring(x, y, 120.0, WHITE, 0.8);
                self.add_shake(16.0);
                self.flash(WHITE, 0.9);
            }
            SimEvent::TimeUp => {
                self.add_shake(6.0);
                self.flash(Color::new(1.0, 0.3, 0.3, 1.0), 0.35);
            }
            SimEvent::LevelCleared => self.flash(WHITE, 0.3),
            SimEvent::Fire { .. } => {}
        }
    }

    pub fn update(&mut self, dt: f32) {
        for p in &mut self.particles {
            p.life -= dt;
            p.vy += p.gravity * dt;
            p.vx *= (-1.5 * dt).exp();
            p.x += p.vx * dt;
            p.y += p.vy * dt;
        }
        self.particles.retain(|p| p.life > 0.0);
        for t in &mut self.texts {
            t.life -= dt * 0.9;
            t.y -= 50.0 * dt;
        }
        self.texts.retain(|t| t.life > 0.0);
        self.shake = super::approach(self.shake, 0.0, 9.0, dt);
        self.flash = (self.flash - dt * 1.8).max(0.0);
    }

    pub fn draw(&self) {
        for p in &self.particles {
            let k = (p.life / p.max).clamp(0.0, 1.0);
            let c = with_alpha(p.color, p.color.a * k);
            match p.shape {
                Shape::Dot => draw_circle(p.x, p.y, p.size * (0.4 + 0.6 * k), c),
                Shape::Spark => {
                    draw_line(
                        p.x,
                        p.y,
                        p.x - p.vx * 0.03,
                        p.y - p.vy * 0.03,
                        p.size * 0.6,
                        c,
                    );
                }
                Shape::Shard => {
                    let a = p.life * 12.0;
                    let (s, co) = a.sin_cos();
                    let d = p.size * 0.6;
                    draw_triangle(
                        vec2(p.x + co * d, p.y + s * d),
                        vec2(p.x - s * d, p.y + co * d * 0.5),
                        vec2(p.x - co * d, p.y - s * d),
                        c,
                    );
                }
                Shape::Ring => {
                    let r = p.size * (1.0 + (1.0 - k) * 1.2);
                    draw_circle_lines(p.x, p.y, r, 3.0 + 4.0 * k, c);
                }
                Shape::Heart => super::heart(p.x, p.y, p.size, c),
            }
        }
        for t in &self.texts {
            let a = t.life.min(1.0);
            text_center(&t.text, t.x, t.y, 22.0, with_alpha(t.color, a));
        }
    }

    /// Full-screen flash; drawn last.
    pub fn draw_flash(&self) {
        if self.flash > 0.0 {
            draw_rectangle(
                0.0,
                0.0,
                1280.0,
                720.0,
                with_alpha(self.flash_color, self.flash * 0.6),
            );
        }
    }
}
