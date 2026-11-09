// SPDX-License-Identifier: Apache-2.0
//! SkyPop: a bright, glossy balloon-popping arcade game.

mod app;
mod audio;
mod input;
mod render;
mod save;
mod sim;

use macroquad::prelude::*;

use sim::{VIEW_H, VIEW_W};

fn window_conf() -> Conf {
    Conf {
        window_title: "SkyPop".to_owned(),
        window_width: VIEW_W as i32,
        window_height: VIEW_H as i32,
        window_resizable: true,
        high_dpi: true,
        sample_count: 4,
        ..Default::default()
    }
}

/// Maps the 1280×720 logical view onto the window, letterboxed.
fn set_view_camera(shake: Vec2) {
    let (sw, sh) = (screen_width(), screen_height());
    let scale = (sw / VIEW_W).min(sh / VIEW_H);
    let (vw, vh) = (VIEW_W * scale, VIEW_H * scale);
    let dpi = screen_dpi_scale();
    let mut cam = Camera2D::from_display_rect(Rect::new(shake.x, shake.y, VIEW_W, VIEW_H));
    cam.viewport = Some((
        (((sw - vw) * 0.5) * dpi) as i32,
        (((sh - vh) * 0.5) * dpi) as i32,
        (vw * dpi) as i32,
        (vh * dpi) as i32,
    ));
    // from_display_rect assumes a render target; flip for the screen.
    cam.zoom.y = -cam.zoom.y;
    set_camera(&cam);
    render::set_ui_scale(scale * dpi);
}

#[macroquad::main(window_conf)]
async fn main() {
    let levels = match sim::level::load_all() {
        Ok(l) => l,
        Err(e) => {
            eprintln!("skypop: failed to load level data: {e}");
            std::process::exit(1);
        }
    };
    let save = save::SaveData::load();
    if save.fullscreen {
        set_fullscreen(true);
    }
    let audio = audio::Audio::new(save.volume).await;
    let mut app = app::App::new(levels, save, audio);

    loop {
        // Clamp long frames (window drags, breakpoints) so the sim never spirals.
        let dt = get_frame_time().min(0.1);
        app.controls.update();
        if is_key_pressed(KeyCode::F11) {
            app.toggle_fullscreen();
        }
        app.update(dt);

        set_default_camera();
        clear_background(BLACK);
        set_view_camera(app.shake());
        app.draw();

        if app.quit {
            break;
        }
        next_frame().await;
    }
}
