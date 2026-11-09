// SPDX-License-Identifier: Apache-2.0
//! Procedurally drawn characters: Leo, Nina, and Dr. Demos's hover-pod.

use macroquad::prelude::*;

use super::{darken, lighten, radial_disc, rounded_rect, with_alpha};

const SKIN: Color = Color::new(1.0, 0.82, 0.66, 1.0);
const OUTLINE: Color = Color::new(0.1, 0.06, 0.2, 1.0);

pub struct LeoStyle {
    pub jacket: Color,
    pub cap: Color,
}

pub const LEO_STYLES: [LeoStyle; 2] = [
    LeoStyle {
        jacket: Color::new(0.18, 0.5, 1.0, 1.0),
        cap: Color::new(1.0, 0.2, 0.33, 1.0),
    },
    LeoStyle {
        jacket: Color::new(1.0, 0.48, 0.18, 1.0),
        cap: Color::new(0.2, 0.85, 0.5, 1.0),
    },
];

/// Draws Leo with his feet at (`x`, `floor`). `walk` is a running phase,
/// `throw` in 0..1 raises the launcher arm.
pub fn draw_leo(
    x: f32,
    floor: f32,
    facing: f32,
    walk: f32,
    throw: f32,
    style: &LeoStyle,
    alpha: f32,
) {
    let a = |c: Color| with_alpha(c, c.a * alpha);
    let stride = walk.sin() * 7.0;
    let bounce = (walk * 2.0).sin().abs() * 2.5;
    let top = floor - 72.0 - bounce;

    draw_ellipse(
        x,
        floor - 1.0,
        22.0,
        5.0,
        0.0,
        a(Color::new(0.0, 0.0, 0.0, 0.25)),
    );
    // Legs and shoes.
    for (dx, s) in [(-7.0, stride), (7.0, -stride)] {
        draw_rectangle(
            x + dx - 4.0 + s * 0.4,
            floor - 22.0,
            8.0,
            18.0,
            a(Color::new(0.16, 0.18, 0.35, 1.0)),
        );
        rounded_rect(
            x + dx - 6.0 + s * 0.6 + facing * 2.0,
            floor - 7.0,
            14.0,
            7.0,
            3.0,
            a(OUTLINE),
        );
    }
    // Body.
    rounded_rect(
        x - 15.0,
        top + 28.0,
        30.0,
        28.0,
        8.0,
        a(darken(style.jacket, 0.2)),
    );
    rounded_rect(x - 14.0, top + 28.0, 28.0, 22.0, 8.0, a(style.jacket));
    draw_rectangle(
        x - 10.0,
        top + 31.0,
        6.0,
        12.0,
        a(lighten(style.jacket, 0.35)),
    );
    // Launcher arm.
    let arm_up = throw.clamp(0.0, 1.0);
    let hand = vec2(
        x + facing * (12.0 - arm_up * 6.0),
        top + 40.0 - arm_up * 24.0,
    );
    draw_line(
        x + facing * 10.0,
        top + 34.0,
        hand.x,
        hand.y,
        7.0,
        a(darken(style.jacket, 0.1)),
    );
    draw_rectangle(
        hand.x - 4.0,
        hand.y - 14.0,
        8.0,
        16.0,
        a(Color::new(0.75, 0.78, 0.86, 1.0)),
    );
    draw_circle(hand.x, hand.y + 2.0, 4.0, a(SKIN));
    // Head.
    let hx = x + facing * 1.5;
    draw_circle(hx, top + 17.0, 14.0, a(SKIN));
    draw_circle(hx + facing * 5.0, top + 16.0, 2.4, a(OUTLINE));
    draw_circle(
        hx + facing * 11.0,
        top + 21.0,
        2.0,
        a(Color::new(1.0, 0.55, 0.55, 0.8)),
    );
    draw_line(
        hx + facing * 3.0,
        top + 24.0,
        hx + facing * 8.0,
        top + 23.5,
        1.6,
        a(OUTLINE),
    );
    // Cap with brim.
    draw_rectangle(hx - 14.0, top + 4.0, 28.0, 8.0, a(style.cap));
    draw_circle(hx, top + 8.0, 13.5, a(style.cap));
    draw_rectangle(hx - 15.0, top + 9.0, 30.0, 5.0, a(style.cap));
    rounded_rect(
        hx + facing * 6.0 - 9.0,
        top + 10.0,
        18.0,
        5.0,
        2.5,
        a(darken(style.cap, 0.25)),
    );
    draw_circle(
        hx - facing * 4.0,
        top + 2.0,
        3.0,
        a(lighten(style.cap, 0.5)),
    );
}

/// Nina with her feet at (`x`, `floor`). `wave` animates a raised arm.
pub fn draw_nina(x: f32, floor: f32, facing: f32, walk: f32, wave: f32) {
    let stride = walk.sin() * 6.0;
    let top = floor - 68.0 - (walk * 2.0).sin().abs() * 2.0;
    let dress = Color::new(1.0, 0.36, 0.7, 1.0);
    let hair = Color::new(0.45, 0.2, 0.55, 1.0);

    draw_ellipse(
        x,
        floor - 1.0,
        20.0,
        5.0,
        0.0,
        Color::new(0.0, 0.0, 0.0, 0.25),
    );
    for (dx, s) in [(-6.0, stride), (6.0, -stride)] {
        draw_rectangle(x + dx - 3.0 + s * 0.4, floor - 18.0, 6.0, 15.0, SKIN);
        rounded_rect(
            x + dx - 5.0 + s * 0.6,
            floor - 6.0,
            11.0,
            6.0,
            3.0,
            Color::new(0.9, 0.2, 0.4, 1.0),
        );
    }
    // Long hair behind.
    rounded_rect(x - 15.0 - facing * 2.0, top + 8.0, 30.0, 32.0, 12.0, hair);
    // Dress.
    draw_triangle(
        vec2(x, top + 26.0),
        vec2(x - 18.0, floor - 16.0),
        vec2(x + 18.0, floor - 16.0),
        dress,
    );
    rounded_rect(x - 10.0, top + 26.0, 20.0, 16.0, 6.0, dress);
    draw_line(
        x - 14.0,
        floor - 20.0,
        x + 14.0,
        floor - 20.0,
        3.0,
        lighten(dress, 0.5),
    );
    // Arms (one waving).
    let wa = (wave * 10.0).sin() * 0.4;
    draw_line(x - 10.0, top + 32.0, x - 16.0, top + 48.0, 5.0, SKIN);
    if wave > 0.0 {
        draw_line(
            x + 10.0,
            top + 32.0,
            x + 20.0 + wa * 8.0,
            top + 12.0,
            5.0,
            SKIN,
        );
    } else {
        draw_line(x + 10.0, top + 32.0, x + 16.0, top + 48.0, 5.0, SKIN);
    }
    // Head and bow.
    draw_circle(x, top + 16.0, 13.0, SKIN);
    draw_circle(x, top + 9.0, 12.0, hair);
    draw_rectangle(x - 12.0, top + 6.0, 24.0, 5.0, hair);
    draw_circle(x + facing * 5.0, top + 17.0, 2.3, OUTLINE);
    draw_circle(x - facing * 3.0, top + 17.0, 2.3, OUTLINE);
    draw_line(
        x - 2.0,
        top + 23.0,
        x + 3.0,
        top + 23.0,
        1.6,
        Color::new(0.8, 0.2, 0.3, 1.0),
    );
    let bow = Color::new(1.0, 0.85, 0.2, 1.0);
    draw_triangle(
        vec2(x + 8.0, top + 2.0),
        vec2(x + 18.0, top - 4.0),
        vec2(x + 18.0, top + 8.0),
        bow,
    );
    draw_triangle(
        vec2(x + 8.0, top + 2.0),
        vec2(x - 2.0, top - 4.0),
        vec2(x - 2.0, top + 8.0),
        bow,
    );
    draw_circle(x + 8.0, top + 2.0, 3.0, darken(bow, 0.2));
}

/// Dr. Demos in his hover-pod, centred at (`x`, `y`). `flash` > 0 tints it
/// white after a hit; `t` drives lights and thrusters.
pub fn draw_demos_pod(x: f32, y: f32, t: f32, flash: f32, damaged: f32) {
    let hull = Color::new(0.42, 0.36, 0.62, 1.0);
    let tint = |c: Color| super::mix(c, WHITE, (flash * 5.0).min(0.8));

    // Thruster flames.
    for dx in [-55.0, 55.0] {
        let len = 26.0 + (t * 30.0 + dx).sin() * 6.0;
        draw_triangle(
            vec2(x + dx - 12.0, y + 34.0),
            vec2(x + dx + 12.0, y + 34.0),
            vec2(x + dx, y + 34.0 + len),
            Color::new(1.0, 0.55, 0.1, 0.85),
        );
        draw_triangle(
            vec2(x + dx - 6.0, y + 34.0),
            vec2(x + dx + 6.0, y + 34.0),
            vec2(x + dx, y + 34.0 + len * 0.6),
            Color::new(1.0, 0.95, 0.5, 0.95),
        );
    }
    // Claw arms.
    for s in [-1.0, 1.0] {
        let sway = (t * 2.0).sin() * 4.0 * s;
        draw_line(
            x + s * 30.0,
            y + 30.0,
            x + s * 38.0 + sway,
            y + 52.0,
            5.0,
            tint(darken(hull, 0.3)),
        );
        draw_line(
            x + s * 38.0 + sway,
            y + 52.0,
            x + s * 30.0 + sway,
            y + 60.0,
            4.0,
            tint(darken(hull, 0.3)),
        );
    }
    // Dome with the doctor inside.
    radial_disc(
        x,
        y - 18.0,
        40.0,
        x - 14.0,
        y - 34.0,
        Color::new(0.75, 0.95, 1.0, 0.55),
        Color::new(0.2, 0.5, 0.7, 0.55),
    );
    let head = Color::new(0.85, 0.92, 0.75, 1.0);
    draw_circle(x, y - 14.0, 19.0, tint(head));
    draw_rectangle(
        x - 18.0,
        y - 22.0,
        36.0,
        8.0,
        tint(Color::new(0.2, 0.2, 0.25, 1.0)),
    );
    for gx in [-8.0, 8.0] {
        draw_circle(x + gx, y - 18.0, 6.0, tint(Color::new(1.0, 0.25, 0.4, 1.0)));
        draw_circle(x + gx - 1.5, y - 19.5, 2.0, WHITE);
    }
    // Mustache.
    draw_triangle(
        vec2(x, y - 7.0),
        vec2(x - 16.0, y - 1.0),
        vec2(x - 4.0, y - 4.0),
        tint(OUTLINE),
    );
    draw_triangle(
        vec2(x, y - 7.0),
        vec2(x + 16.0, y - 1.0),
        vec2(x + 4.0, y - 4.0),
        tint(OUTLINE),
    );
    draw_ellipse(
        x - 12.0,
        y - 30.0,
        7.0,
        4.0,
        0.3,
        Color::new(1.0, 1.0, 1.0, 0.6),
    );
    // Saucer hull.
    draw_ellipse(x, y + 14.0, 102.0, 30.0, 0.0, tint(darken(hull, 0.35)));
    draw_ellipse(x, y + 8.0, 100.0, 26.0, 0.0, tint(hull));
    draw_ellipse(
        x - 20.0,
        y + 0.0,
        60.0,
        8.0,
        0.0,
        Color::new(1.0, 1.0, 1.0, 0.25),
    );
    // Rim lights.
    for i in 0..8 {
        let fx = x - 84.0 + i as f32 * 24.0;
        let on = ((t * 6.0) as i32 + i) % 3 == 0;
        let c = if on {
            Color::new(1.0, 0.95, 0.3, 1.0)
        } else {
            Color::new(1.0, 0.25, 0.45, 1.0)
        };
        draw_circle(fx, y + 16.0, 4.5, tint(c));
    }
    // Smoke when badly damaged.
    if damaged > 0.5 {
        for i in 0..3 {
            let p = (t * 0.8 + i as f32 * 0.33).fract();
            draw_circle(
                x + 60.0 - i as f32 * 50.0 + p * 10.0,
                y - p * 60.0,
                6.0 + p * 14.0,
                Color::new(0.2, 0.2, 0.25, 0.5 * (1.0 - p)),
            );
        }
    }
}

/// A hanging cage; `open` in 0..1 lifts the door.
pub fn draw_cage(x: f32, y: f32, w: f32, h: f32, open: f32) {
    let bar = Color::new(0.85, 0.75, 0.35, 1.0);
    draw_line(
        x + w * 0.5,
        y - 60.0,
        x + w * 0.5,
        y,
        3.0,
        Color::new(0.5, 0.5, 0.55, 1.0),
    );
    rounded_rect(x - 4.0, y - 6.0, w + 8.0, 10.0, 4.0, darken(bar, 0.2));
    rounded_rect(x - 4.0, y + h - 4.0, w + 8.0, 10.0, 4.0, darken(bar, 0.2));
    let bars = 7;
    for i in 0..=bars {
        let bx = x + w * i as f32 / bars as f32;
        let lift = if (2..=5).contains(&i) { open * h } else { 0.0 };
        draw_line(bx, y, bx, y + h - lift, 3.0, bar);
    }
}
