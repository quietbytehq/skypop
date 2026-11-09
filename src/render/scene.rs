// SPDX-License-Identifier: Apache-2.0
//! Draws a `World`: themed backdrop, platforms, orbs, wires, drops, players,
//! the boss and the HUD. Positions are interpolated between the previous and
//! current simulation step so motion stays smooth at any refresh rate.

use macroquad::models::{draw_mesh, Mesh, Vertex};
use macroquad::prelude::*;

use super::characters::{draw_cage, draw_demos_pod, draw_leo, draw_nina, LEO_STYLES};
use super::{
    darken, glossy_orb, glossy_panel, gradient_rect, heart, hex, lighten, mix, orb_color,
    rounded_rect, star, text, text_center, text_shadow, text_width, with_alpha,
};
use crate::sim::level::{Decor, Theme};
use crate::sim::*;

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

pub fn draw_background(theme: &Theme, t: f32) {
    let top = hex(&theme.sky_top);
    let bottom = hex(&theme.sky_bottom);
    let accent = hex(&theme.accent);
    gradient_rect(0.0, 0.0, VIEW_W, VIEW_H, top, bottom);
    match theme.decor {
        Decor::Hills => {
            draw_circle(1000.0, 170.0, 70.0, with_alpha(WHITE, 0.25));
            draw_circle(1000.0, 170.0, 52.0, Color::new(1.0, 0.97, 0.8, 0.9));
            clouds(t, 0.5);
            hills(t, 470.0, 50.0, 0.004, mix(bottom, accent, 0.25), 0.0);
            hills(
                t,
                540.0,
                35.0,
                0.007,
                mix(darken(bottom, 0.15), Color::new(0.3, 0.8, 0.4, 1.0), 0.45),
                2.0,
            );
        }
        Decor::Clouds => {
            clouds(t, 1.0);
            hills(
                t,
                580.0,
                18.0,
                0.01,
                with_alpha(darken(bottom, 0.12), 0.9),
                1.0,
            );
        }
        Decor::City => {
            stars(t, 0.5);
            draw_circle(220.0, 140.0, 46.0, Color::new(1.0, 0.95, 0.85, 0.85));
            skyline(t, accent, bottom);
        }
        Decor::Stars => {
            stars(t, 1.0);
            draw_circle(1060.0, 120.0, 60.0, with_alpha(WHITE, 0.15));
            draw_circle(1060.0, 120.0, 44.0, Color::new(0.95, 0.95, 1.0, 1.0));
            draw_circle(1076.0, 110.0, 40.0, top);
            hills(t, 590.0, 25.0, 0.006, darken(top, 0.3), 0.5);
        }
        Decor::Fortress => {
            stars(t, 0.6);
            fortress(t, accent);
        }
    }
    // Soft vignette at the floor.
    gradient_rect(
        0.0,
        ARENA_BOTTOM - 80.0,
        VIEW_W,
        80.0,
        with_alpha(BLACK, 0.0),
        with_alpha(BLACK, 0.18),
    );
}

fn clouds(t: f32, density: f32) {
    let count = (7.0 * density) as usize + 2;
    for i in 0..count {
        let speed = 8.0 + (i % 3) as f32 * 6.0;
        let w = 1600.0;
        let x = ((i as f32 * 263.0 + t * speed) % w) - 160.0;
        let y = 70.0 + ((i * 97) % 260) as f32;
        let s = 0.7 + ((i * 53) % 10) as f32 / 18.0;
        let c = Color::new(1.0, 1.0, 1.0, 0.55);
        for (dx, dy, r) in [
            (0.0, 0.0, 34.0),
            (34.0, -12.0, 40.0),
            (72.0, 0.0, 30.0),
            (36.0, 10.0, 34.0),
        ] {
            draw_circle(x + dx * s, y + dy * s, r * s, c);
        }
    }
}

fn hills(t: f32, base: f32, amp: f32, freq: f32, c: Color, seed: f32) {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let steps = 64u16;
    let top_c = lighten(c, 0.12);
    for i in 0..=steps {
        let x = i as f32 / steps as f32 * VIEW_W;
        let y = base
            + (x * freq + seed + t * 0.02).sin() * amp
            + (x * freq * 2.3 + seed * 3.0).sin() * amp * 0.35;
        vertices.push(Vertex::new(x, y, 0.0, 0.0, 0.0, top_c));
        vertices.push(Vertex::new(x, VIEW_H, 0.0, 0.0, 0.0, c));
        if i < steps {
            let k = i * 2;
            indices.extend_from_slice(&[k, k + 1, k + 2, k + 1, k + 3, k + 2]);
        }
    }
    draw_mesh(&Mesh {
        vertices,
        indices,
        texture: None,
    });
}

fn stars(t: f32, density: f32) {
    let n = (90.0 * density) as u32;
    for i in 0..n {
        let x = ((i * 7919) % 1280) as f32;
        let y = ((i * 104_729) % 420) as f32 + 20.0;
        let tw = ((t * 2.0 + i as f32 * 1.7).sin() * 0.5 + 0.5) * 0.7 + 0.3;
        draw_circle(
            x,
            y,
            1.0 + (i % 3) as f32 * 0.6,
            Color::new(1.0, 1.0, 1.0, tw),
        );
    }
}

fn skyline(t: f32, accent: Color, bottom: Color) {
    let far = darken(bottom, 0.55);
    let near = darken(bottom, 0.75);
    for (layer, color, base_h) in [(0u32, far, 180.0), (1, near, 120.0)] {
        let mut x = -20.0;
        let mut i = layer * 31;
        while x < VIEW_W {
            let w = 50.0 + ((i * 37) % 60) as f32;
            let h = base_h + ((i * 71) % 150) as f32;
            let y = ARENA_BOTTOM - h;
            draw_rectangle(x, y, w, h, color);
            if layer == 1 {
                for wy in (0..(h as i32 / 22)).map(|k| y + 10.0 + k as f32 * 22.0) {
                    for wx in (0..(w as i32 / 16)).map(|k| x + 6.0 + k as f32 * 16.0) {
                        let lit = ((wx as i32 * 13 + wy as i32 * 7 + i as i32) % 5) == 0;
                        if lit {
                            draw_rectangle(wx, wy, 7.0, 9.0, Color::new(1.0, 0.9, 0.5, 0.75));
                        }
                    }
                }
                let glow = ((t * 1.5 + i as f32).sin() * 0.3 + 0.7).max(0.0);
                draw_rectangle(x, y, w, 3.0, with_alpha(accent, glow));
            }
            x += w + 6.0;
            i += 1;
        }
    }
}

fn fortress(t: f32, accent: Color) {
    let dark = Color::new(0.08, 0.02, 0.12, 1.0);
    for (x, w, h) in [
        (60.0, 120.0, 330.0),
        (240.0, 90.0, 260.0),
        (950.0, 100.0, 280.0),
        (1100.0, 130.0, 360.0),
    ] {
        let y = ARENA_BOTTOM - h;
        draw_rectangle(x, y, w, h, dark);
        for k in 0..(w as i32 / 24) {
            draw_rectangle(x + k as f32 * 24.0, y - 14.0, 14.0, 14.0, dark);
        }
        let blink = ((t * 3.0 + x).sin() > 0.0) as i32 as f32;
        draw_circle(
            x + w * 0.5,
            y + 40.0,
            6.0,
            with_alpha(accent, 0.4 + 0.6 * blink),
        );
    }
    let glow = (t * 0.7).sin() * 0.5 + 0.5;
    gradient_rect(
        0.0,
        0.0,
        VIEW_W,
        200.0,
        with_alpha(accent, 0.12 * glow),
        with_alpha(accent, 0.0),
    );
}

fn draw_frame(theme: &Theme) {
    let accent = hex(&theme.accent);
    let frame = darken(mix(hex(&theme.sky_top), accent, 0.3), 0.35);
    draw_rectangle(0.0, 0.0, VIEW_W, ARENA_TOP, frame);
    draw_rectangle(0.0, 0.0, ARENA_LEFT, ARENA_BOTTOM, frame);
    draw_rectangle(ARENA_RIGHT, 0.0, VIEW_W - ARENA_RIGHT, ARENA_BOTTOM, frame);
    draw_rectangle_lines(
        ARENA_LEFT - 2.0,
        ARENA_TOP - 2.0,
        ARENA_RIGHT - ARENA_LEFT + 4.0,
        ARENA_BOTTOM - ARENA_TOP + 4.0,
        3.0,
        with_alpha(accent, 0.8),
    );
    // Floor.
    gradient_rect(
        0.0,
        ARENA_BOTTOM,
        VIEW_W,
        8.0,
        lighten(accent, 0.3),
        darken(accent, 0.2),
    );
}

fn draw_platform(p: &Platform, t: f32) {
    match p.kind {
        PlatformKind::Solid => {
            glossy_panel(
                p.x,
                p.y,
                p.w,
                p.h,
                6.0,
                Color::new(0.62, 0.6, 0.78, 1.0),
                Color::new(0.3, 0.27, 0.45, 1.0),
            );
            let n = (p.w / 30.0) as i32;
            for i in 1..n {
                let x = p.x + i as f32 * p.w / n as f32;
                draw_circle(x, p.y + p.h * 0.5, 2.5, Color::new(0.2, 0.18, 0.3, 0.8));
            }
        }
        PlatformKind::Breakable => {
            let shimmer = ((t * 2.0 + p.x * 0.01).sin() * 0.5 + 0.5) * 0.25;
            glossy_panel(
                p.x,
                p.y,
                p.w,
                p.h,
                5.0,
                Color::new(0.6 + shimmer, 0.95, 1.0, 0.75),
                Color::new(0.2, 0.6, 0.85, 0.75),
            );
            let mid = p.x + p.w * 0.45;
            let c = Color::new(1.0, 1.0, 1.0, 0.6);
            draw_line(mid, p.y + 3.0, mid + 8.0, p.y + p.h * 0.5, 1.5, c);
            draw_line(
                mid + 8.0,
                p.y + p.h * 0.5,
                mid - 4.0,
                p.y + p.h - 3.0,
                1.5,
                c,
            );
        }
    }
}

fn draw_wire(w: &Wire, t: f32, color: Color) {
    let bottom = ARENA_BOTTOM - 4.0;
    if w.anchor {
        let pulse = if w.stuck.is_some() {
            (t * 18.0).sin() * 0.3 + 0.7
        } else {
            1.0
        };
        let c = Color::new(0.2, 1.0, 0.85, pulse);
        draw_line(w.x, w.tip_y, w.x, bottom, 9.0, with_alpha(c, 0.25 * pulse));
        draw_line(w.x, w.tip_y, w.x, bottom, 3.5, c);
        // Anchor head.
        draw_line(
            w.x - 10.0,
            w.tip_y + 6.0,
            w.x + 10.0,
            w.tip_y + 6.0,
            3.0,
            WHITE,
        );
        draw_circle_lines(w.x, w.tip_y + 16.0, 9.0, 2.5, WHITE);
        draw_triangle(
            vec2(w.x, w.tip_y - 6.0),
            vec2(w.x - 6.0, w.tip_y + 4.0),
            vec2(w.x + 6.0, w.tip_y + 4.0),
            WHITE,
        );
        return;
    }
    // Zig-zag harpoon line.
    let seg = 12.0;
    let mut y = w.tip_y + 12.0;
    let mut flip = 1.0;
    draw_line(w.x, w.tip_y, w.x, bottom, 7.0, with_alpha(color, 0.25));
    while y < bottom {
        let ny = (y + seg).min(bottom);
        draw_line(
            w.x - 3.0 * flip,
            y,
            w.x + 3.0 * flip,
            ny,
            2.2,
            Color::new(0.92, 0.95, 1.0, 1.0),
        );
        flip = -flip;
        y = ny;
    }
    draw_triangle(
        vec2(w.x, w.tip_y - 4.0),
        vec2(w.x - 8.0, w.tip_y + 14.0),
        vec2(w.x + 8.0, w.tip_y + 14.0),
        lighten(color, 0.5),
    );
    draw_triangle(
        vec2(w.x, w.tip_y),
        vec2(w.x - 4.0, w.tip_y + 10.0),
        vec2(w.x + 4.0, w.tip_y + 10.0),
        WHITE,
    );
}

pub fn drop_color(kind: DropKind) -> Color {
    match kind {
        DropKind::Shield => Color::new(0.3, 0.8, 1.0, 1.0),
        DropKind::Freeze => Color::new(0.65, 0.95, 1.0, 1.0),
        DropKind::Slow => Color::new(0.6, 0.45, 1.0, 1.0),
        DropKind::Bonus => Color::new(1.0, 0.8, 0.15, 1.0),
        DropKind::Anchor => Color::new(0.15, 0.95, 0.75, 1.0),
        DropKind::Double => Color::new(1.0, 0.45, 0.8, 1.0),
    }
}

pub fn draw_drop_icon(kind: DropKind, x: f32, y: f32, s: f32, t: f32) {
    let c = drop_color(kind);
    let half = s * 0.5;
    glossy_panel(
        x - half,
        y - half,
        s,
        s,
        s * 0.3,
        lighten(c, 0.35),
        darken(c, 0.3),
    );
    let ic = WHITE;
    let k = s / 30.0;
    match kind {
        DropKind::Shield => {
            draw_circle_lines(x, y, 8.0 * k, 2.5 * k, ic);
            draw_circle(x - 2.0 * k, y - 2.0 * k, 2.5 * k, ic);
        }
        DropKind::Freeze => {
            for i in 0..3 {
                let a = i as f32 * std::f32::consts::PI / 3.0;
                let (dx, dy) = (a.cos() * 9.0 * k, a.sin() * 9.0 * k);
                draw_line(x - dx, y - dy, x + dx, y + dy, 2.2 * k, ic);
            }
        }
        DropKind::Slow => {
            draw_triangle(
                vec2(x - 7.0 * k, y - 9.0 * k),
                vec2(x + 7.0 * k, y - 9.0 * k),
                vec2(x, y),
                ic,
            );
            draw_triangle(
                vec2(x - 7.0 * k, y + 9.0 * k),
                vec2(x + 7.0 * k, y + 9.0 * k),
                vec2(x, y),
                ic,
            );
        }
        DropKind::Bonus => star(x, y, 10.0 * k, t * 2.0, ic),
        DropKind::Anchor => {
            draw_line(x, y - 9.0 * k, x, y + 8.0 * k, 2.5 * k, ic);
            draw_line(
                x - 6.0 * k,
                y - 4.0 * k,
                x + 6.0 * k,
                y - 4.0 * k,
                2.5 * k,
                ic,
            );
            draw_circle_lines(x, y + 2.0 * k, 7.0 * k, 2.0 * k, ic);
        }
        DropKind::Double => {
            for dx in [-4.5, 4.5] {
                let xx = x + dx * k;
                draw_line(xx, y - 4.0 * k, xx, y + 9.0 * k, 2.2 * k, ic);
                draw_triangle(
                    vec2(xx, y - 10.0 * k),
                    vec2(xx - 4.0 * k, y - 3.0 * k),
                    vec2(xx + 4.0 * k, y - 3.0 * k),
                    ic,
                );
            }
        }
    }
}

/// Draws the playfield (everything except the HUD).
pub fn draw_world(w: &World, prev: &World, alpha: f32, theme: &Theme, t: f32) {
    draw_background(theme, t);

    for p in &w.platforms {
        draw_platform(p, t);
    }

    for d in &w.drops {
        if d.landed && d.life < 2.0 && (t * 12.0).sin() < 0.0 {
            continue;
        }
        let bob = if d.landed {
            (t * 4.0 + d.x).sin() * 2.0
        } else {
            0.0
        };
        let py = prev
            .drops
            .iter()
            .find(|p| p.id == d.id)
            .map_or(d.y, |p| lerp(p.y, d.y, alpha));
        draw_drop_icon(d.kind, d.x, py + bob, DROP_SIZE, t);
    }

    for wire in &w.wires {
        let tip = prev
            .wires
            .iter()
            .find(|p| p.id == wire.id)
            .map_or(wire.tip_y, |p| lerp(p.tip_y, wire.tip_y, alpha));
        let mut shown = wire.clone();
        shown.tip_y = tip;
        draw_wire(&shown, t, LEO_STYLES[wire.owner as usize % 2].jacket);
    }

    if let Some(b) = &w.boss {
        let (x, y) = prev.boss.as_ref().map_or((b.x, b.y), |p| {
            (lerp(p.x, b.x, alpha), lerp(p.y, b.y, alpha))
        });
        let damaged = 1.0 - b.hp as f32 / b.max_hp as f32;
        draw_demos_pod(x, y, t, b.flash, damaged);
        // Nina's cage hangs in the corner during the fight.
        let open = if let Status::BossDown(k) = w.status {
            (k / 1.5).min(1.0)
        } else {
            0.0
        };
        draw_cage(ARENA_RIGHT - 120.0, 90.0, 80.0, 90.0, open);
        draw_nina(
            ARENA_RIGHT - 80.0,
            175.0,
            -1.0,
            0.0,
            if open > 0.0 { t } else { 0.0 },
        );
    }

    let frozen = w.freeze > 0.0;
    for o in &w.orbs {
        let (x, y) = prev
            .orbs
            .iter()
            .find(|p| p.id == o.id)
            .map_or((o.x, o.y), |p| {
                (lerp(p.x, o.x, alpha), lerp(p.y, o.y, alpha))
            });
        let mut c = orb_color(o.color);
        if frozen {
            c = mix(c, Color::new(0.75, 0.95, 1.0, 1.0), 0.55);
        }
        glossy_orb(x, y, o.tier.radius(), c, 0.0);
        if frozen {
            draw_circle_lines(
                x,
                y,
                o.tier.radius() + 2.0,
                2.0,
                Color::new(1.0, 1.0, 1.0, 0.7),
            );
        }
    }

    for p in &w.players {
        if !p.alive {
            continue;
        }
        if p.invuln > 0.0 && (t * 20.0).sin() < 0.0 {
            continue;
        }
        let x = prev
            .players
            .get(p.index as usize)
            .map_or(p.x, |q| lerp(q.x, p.x, alpha));
        let walk = if p.moving { x * 0.09 } else { 0.0 };
        let throw = (1.0 - p.shot_t / 0.18).clamp(0.0, 1.0);
        draw_leo(
            x,
            ARENA_BOTTOM,
            p.facing as f32,
            walk,
            throw,
            &LEO_STYLES[p.index as usize % 2],
            1.0,
        );
        if p.shield {
            let pulse = (t * 5.0).sin() * 0.1;
            draw_circle(
                x,
                ARENA_BOTTOM - 38.0,
                46.0,
                Color::new(0.4, 0.9, 1.0, 0.15 + pulse * 0.5),
            );
            draw_circle_lines(
                x,
                ARENA_BOTTOM - 38.0,
                46.0,
                2.5,
                Color::new(0.6, 0.95, 1.0, 0.75 + pulse),
            );
            draw_ellipse(
                x - 18.0,
                ARENA_BOTTOM - 64.0,
                12.0,
                6.0,
                -0.6,
                Color::new(1.0, 1.0, 1.0, 0.5),
            );
        }
    }

    draw_frame(theme);
}

/// Bottom HUD bar and in-arena status banners.
pub fn draw_hud(w: &World, theme: &Theme, t: f32) {
    let accent = hex(&theme.accent);
    gradient_rect(
        0.0,
        ARENA_BOTTOM + 8.0,
        VIEW_W,
        VIEW_H - ARENA_BOTTOM - 8.0,
        Color::new(0.1, 0.06, 0.2, 1.0),
        Color::new(0.03, 0.02, 0.08, 1.0),
    );
    draw_line(
        0.0,
        ARENA_BOTTOM + 9.0,
        VIEW_W,
        ARENA_BOTTOM + 9.0,
        2.0,
        with_alpha(accent, 0.6),
    );

    for p in &w.players {
        let style = &LEO_STYLES[p.index as usize % 2];
        let left = p.index == 0;
        let bx = if left { 24.0 } else { VIEW_W - 300.0 };
        let name = if left { "LEO" } else { "PLAYER 2" };
        text_shadow(
            name,
            bx,
            ARENA_BOTTOM + 38.0,
            22.0,
            lighten(style.jacket, 0.3),
        );
        for i in 0..p.lives.min(8) {
            heart(
                bx + 12.0 + i as f32 * 26.0,
                ARENA_BOTTOM + 52.0,
                9.0,
                Color::new(1.0, 0.3, 0.45, 1.0),
            );
        }
        let wx = bx + 130.0;
        rounded_rect(
            wx,
            ARENA_BOTTOM + 20.0,
            130.0,
            26.0,
            8.0,
            with_alpha(style.jacket, 0.35),
        );
        text(
            p.weapon.label(),
            wx + 10.0,
            ARENA_BOTTOM + 39.0,
            18.0,
            WHITE,
        );
        if p.shield {
            draw_drop_icon(DropKind::Shield, wx + 150.0, ARENA_BOTTOM + 34.0, 22.0, t);
        }
        if !p.alive {
            text(
                "K.O.",
                wx + 10.0,
                ARENA_BOTTOM + 66.0,
                18.0,
                Color::new(1.0, 0.4, 0.4, 1.0),
            );
        }
    }

    let score = format!("{:07}", w.score);
    text_center(&score, VIEW_W * 0.5, ARENA_BOTTOM + 44.0, 36.0, WHITE);
    let stage = format!(
        "STAGE {}  -  {}",
        w.level_index + 1,
        w.level_name.to_uppercase()
    );
    text_center(
        &stage,
        VIEW_W * 0.5,
        ARENA_BOTTOM + 68.0,
        16.0,
        with_alpha(lighten(accent, 0.4), 0.9),
    );

    let secs = w.time_left.ceil().max(0.0) as u32;
    let warn = secs <= 15 && w.status == Status::Running;
    let tc = if warn && (t * 6.0).sin() > 0.0 {
        Color::new(1.0, 0.35, 0.35, 1.0)
    } else {
        WHITE
    };
    let time = format!("TIME {secs:03}");
    let tx = if w.players.len() > 1 {
        VIEW_W * 0.5 + 150.0
    } else {
        VIEW_W - 24.0 - text_width(&time, 26.0)
    };
    text_shadow(&time, tx, ARENA_BOTTOM + 44.0, 26.0, tc);

    // Active power-ups.
    let mut y = ARENA_TOP + 34.0;
    for (label, left, total, c) in [
        ("FREEZE", w.freeze, 4.5, Color::new(0.6, 0.95, 1.0, 1.0)),
        ("SLOW", w.slow, 7.0, Color::new(0.7, 0.55, 1.0, 1.0)),
    ] {
        if left <= 0.0 {
            continue;
        }
        let bw = 160.0;
        let x = VIEW_W * 0.5 - bw * 0.5;
        rounded_rect(x, y - 18.0, bw, 24.0, 10.0, Color::new(0.0, 0.0, 0.1, 0.45));
        rounded_rect(
            x + 2.0,
            y - 16.0,
            (bw - 4.0) * (left / total).min(1.0),
            20.0,
            9.0,
            with_alpha(c, 0.6),
        );
        text_center(label, VIEW_W * 0.5, y + 1.0, 18.0, WHITE);
        y += 30.0;
    }

    if let Some(b) = &w.boss {
        let bw = 420.0;
        let x = 40.0;
        let y = ARENA_TOP + 14.0;
        text_shadow(
            "DR. DEMOS",
            x,
            y + 16.0,
            20.0,
            Color::new(1.0, 0.6, 0.75, 1.0),
        );
        let bx = x + 130.0;
        rounded_rect(bx, y, bw, 20.0, 9.0, Color::new(0.0, 0.0, 0.0, 0.5));
        let f = b.hp as f32 / b.max_hp as f32;
        if f > 0.0 {
            let c = mix(
                Color::new(1.0, 0.2, 0.3, 1.0),
                Color::new(1.0, 0.85, 0.2, 1.0),
                f,
            );
            glossy_panel(
                bx + 2.0,
                y + 2.0,
                (bw - 4.0) * f,
                16.0,
                7.0,
                lighten(c, 0.3),
                darken(c, 0.2),
            );
        }
    }
}
