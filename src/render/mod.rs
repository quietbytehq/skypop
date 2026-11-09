// SPDX-License-Identifier: Apache-2.0
//! Drawing helpers. All art is procedural: gradients, glossy shapes and text.
//! Coordinates are logical (1280×720); `main` installs a letterboxing camera.

pub mod characters;
pub mod fx;
pub mod glyphs;
pub mod scene;

pub use glyphs::{fancy_headline, headline};

use std::cell::Cell;

use macroquad::models::{draw_mesh, Mesh, Vertex};
use macroquad::prelude::*;

pub const ORB_COLORS: [Color; 6] = [
    Color::new(1.0, 0.17, 0.84, 1.0),
    Color::new(0.0, 0.9, 1.0, 1.0),
    Color::new(0.49, 1.0, 0.23, 1.0),
    Color::new(1.0, 0.54, 0.0, 1.0),
    Color::new(0.61, 0.36, 1.0, 1.0),
    Color::new(1.0, 0.9, 0.0, 1.0),
];

pub fn orb_color(i: u8) -> Color {
    ORB_COLORS[i as usize % ORB_COLORS.len()]
}

thread_local! {
    static UI_SCALE: Cell<f32> = const { Cell::new(1.0) };
}

/// Logical-to-physical pixel scale, so text is rasterised at native size.
pub fn set_ui_scale(s: f32) {
    UI_SCALE.with(|c| c.set(s.max(0.1)));
}

fn ui_scale() -> f32 {
    UI_SCALE.with(Cell::get)
}

pub fn hex(s: &str) -> Color {
    let s = s.trim_start_matches('#');
    let v = u32::from_str_radix(s, 16).unwrap_or(0xff00ff);
    Color::from_rgba((v >> 16) as u8, (v >> 8) as u8, v as u8, 255)
}

pub fn with_alpha(c: Color, a: f32) -> Color {
    Color::new(c.r, c.g, c.b, a)
}

pub fn mix(a: Color, b: Color, t: f32) -> Color {
    Color::new(
        a.r + (b.r - a.r) * t,
        a.g + (b.g - a.g) * t,
        a.b + (b.b - a.b) * t,
        a.a + (b.a - a.a) * t,
    )
}

pub fn lighten(c: Color, t: f32) -> Color {
    mix(c, Color::new(1.0, 1.0, 1.0, c.a), t)
}

pub fn darken(c: Color, t: f32) -> Color {
    mix(c, Color::new(0.0, 0.0, 0.0, c.a), t)
}

// ---- easing -------------------------------------------------------------

pub fn ease_out_cubic(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0) - 1.0;
    t * t * t + 1.0
}

pub fn ease_out_back(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    let c1 = 1.70158;
    let c3 = c1 + 1.0;
    1.0 + c3 * (t - 1.0).powi(3) + c1 * (t - 1.0).powi(2)
}

pub fn smoothstep(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Frame-rate independent exponential approach.
pub fn approach(current: f32, target: f32, rate: f32, dt: f32) -> f32 {
    target + (current - target) * (-rate * dt).exp()
}

// ---- shapes ---------------------------------------------------------------

fn vtx(x: f32, y: f32, c: Color) -> Vertex {
    Vertex::new(x, y, 0.0, 0.0, 0.0, c)
}

pub fn gradient_rect(x: f32, y: f32, w: f32, h: f32, top: Color, bottom: Color) {
    let mesh = Mesh {
        vertices: vec![
            vtx(x, y, top),
            vtx(x + w, y, top),
            vtx(x + w, y + h, bottom),
            vtx(x, y + h, bottom),
        ],
        indices: vec![0, 1, 2, 0, 2, 3],
        texture: None,
    };
    draw_mesh(&mesh);
}

/// Radial gradient disc whose bright centre sits at (`cx`, `cy`).
pub fn radial_disc(x: f32, y: f32, r: f32, cx: f32, cy: f32, inner: Color, outer: Color) {
    const SEGS: u16 = 40;
    let mut vertices = Vec::with_capacity(SEGS as usize + 1);
    vertices.push(vtx(cx, cy, inner));
    for i in 0..SEGS {
        let a = i as f32 / SEGS as f32 * std::f32::consts::TAU;
        vertices.push(vtx(x + a.cos() * r, y + a.sin() * r, outer));
    }
    let mut indices = Vec::with_capacity(SEGS as usize * 3);
    for i in 0..SEGS {
        indices.extend_from_slice(&[0, 1 + i, 1 + (i + 1) % SEGS]);
    }
    draw_mesh(&Mesh {
        vertices,
        indices,
        texture: None,
    });
}

/// A glowing, glossy neon orb.
pub fn glossy_orb(x: f32, y: f32, r: f32, base: Color, flash: f32) {
    draw_circle(x, y, r * 1.45, with_alpha(base, 0.08));
    draw_circle(x, y, r * 1.2, with_alpha(base, 0.16));
    let rim = darken(base, 0.45);
    let core = lighten(base, 0.35 + flash * 0.5);
    radial_disc(x, y, r, x - r * 0.28, y - r * 0.3, core, rim);
    // Soft inner reflection at the bottom edge.
    draw_ellipse(
        x + r * 0.15,
        y + r * 0.55,
        r * 0.5,
        r * 0.18,
        0.0,
        with_alpha(lighten(base, 0.6), 0.35),
    );
    // Specular highlight.
    draw_ellipse(
        x - r * 0.33,
        y - r * 0.42,
        r * 0.36,
        r * 0.2,
        -0.5,
        Color::new(1.0, 1.0, 1.0, 0.85),
    );
    draw_circle(
        x - r * 0.05,
        y - r * 0.62,
        r * 0.07,
        Color::new(1.0, 1.0, 1.0, 0.7),
    );
}

pub fn rounded_rect(x: f32, y: f32, w: f32, h: f32, r: f32, c: Color) {
    let r = r.min(w * 0.5).min(h * 0.5);
    draw_rectangle(x + r, y, w - 2.0 * r, h, c);
    draw_rectangle(x, y + r, r, h - 2.0 * r, c);
    draw_rectangle(x + w - r, y + r, r, h - 2.0 * r, c);
    for (cx, cy) in [
        (x + r, y + r),
        (x + w - r, y + r),
        (x + r, y + h - r),
        (x + w - r, y + h - r),
    ] {
        draw_circle(cx, cy, r, c);
    }
}

/// Rounded panel with a vertical gradient and a glossy top sheen.
pub fn glossy_panel(x: f32, y: f32, w: f32, h: f32, r: f32, top: Color, bottom: Color) {
    rounded_rect(x, y, w, h, r, bottom);
    let inset = r * 0.3;
    gradient_rect(x + inset, y + r * 0.5, w - inset * 2.0, h - r, top, bottom);
    rounded_rect(
        x + r * 0.6,
        y + 3.0,
        w - r * 1.2,
        (h * 0.35).max(4.0),
        r * 0.6,
        Color::new(1.0, 1.0, 1.0, 0.22),
    );
}

pub fn star(x: f32, y: f32, r: f32, rot: f32, c: Color) {
    let pts: Vec<Vec2> = (0..10)
        .map(|i| {
            let a = rot + i as f32 * std::f32::consts::PI / 5.0 - std::f32::consts::FRAC_PI_2;
            let rr = if i % 2 == 0 { r } else { r * 0.45 };
            vec2(x + a.cos() * rr, y + a.sin() * rr)
        })
        .collect();
    for i in 0..10 {
        draw_triangle(vec2(x, y), pts[i], pts[(i + 1) % 10], c);
    }
}

pub fn heart(x: f32, y: f32, s: f32, c: Color) {
    draw_circle(x - s * 0.5, y - s * 0.2, s * 0.55, c);
    draw_circle(x + s * 0.5, y - s * 0.2, s * 0.55, c);
    draw_triangle(
        vec2(x - s * 1.02, y),
        vec2(x + s * 1.02, y),
        vec2(x, y + s * 1.1),
        c,
    );
}

// ---- text ---------------------------------------------------------------

pub fn text_width(s: &str, size: f32) -> f32 {
    let scale = ui_scale();
    let px = (size * scale).round().max(1.0) as u16;
    measure_text(s, None, px, 1.0 / scale).width
}

pub fn text(s: &str, x: f32, y: f32, size: f32, c: Color) {
    let scale = ui_scale();
    let px = (size * scale).round().max(1.0) as u16;
    draw_text_ex(
        s,
        x,
        y,
        TextParams {
            font_size: px,
            font_scale: 1.0 / scale,
            color: c,
            ..Default::default()
        },
    );
}

pub fn text_shadow(s: &str, x: f32, y: f32, size: f32, c: Color) {
    let off = (size * 0.06).max(2.0);
    text(
        s,
        x + off,
        y + off,
        size,
        Color::new(0.0, 0.0, 0.08, 0.55 * c.a),
    );
    text(s, x, y, size, c);
}

pub fn text_center(s: &str, cx: f32, y: f32, size: f32, c: Color) {
    text_shadow(s, cx - text_width(s, size) * 0.5, y, size, c);
}
