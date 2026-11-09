// SPDX-License-Identifier: Apache-2.0
//! Screen state machine: menus, cutscenes, gameplay sessions and overlays.

use macroquad::prelude::*;

use crate::audio::{Audio, Sfx};
use crate::input::{Action, Controls, InputSource, LocalInput, ACTIONS};
use crate::render::characters::{draw_cage, draw_demos_pod, draw_leo, draw_nina, LEO_STYLES};
use crate::render::fx::Fx;
use crate::render::scene::{draw_background, draw_hud, draw_world};
use crate::render::{
    approach, darken, ease_out_back, ease_out_cubic, fancy_headline, glossy_orb, glossy_panel,
    headline, lighten, orb_color, rounded_rect, smoothstep, text_center, text_shadow, with_alpha,
};
use crate::save::{SaveData, ScoreEntry};
use crate::sim::level::{Decor, LevelDef, Theme, LEVEL_COUNT};
use crate::sim::*;

const TIME_BONUS_PER_SEC: u32 = 10;

/// One stage being played: the simulation plus its input sources.
struct Session {
    level: usize,
    start: Carry,
    world: World,
    prev: World,
    acc: f32,
    alpha: f32,
    /// "Get ready" banner time remaining; the sim waits for it.
    ready: f32,
    inputs: Vec<Box<dyn InputSource>>,
    /// Inputs seen since the last sim step, so a tap during a frame that runs
    /// no step (high refresh rates) is not lost.
    latched: Vec<InputFrame>,
}

impl Session {
    fn new(level: usize, def: &LevelDef, carry: Carry) -> Self {
        let world = World::new(level, def, &carry);
        // Milestone 2 swaps the second source for frames received over LAN.
        let inputs: Vec<Box<dyn InputSource>> = (0..carry.players)
            .map(|_| Box::new(LocalInput) as _)
            .collect();
        Self {
            level,
            start: carry,
            prev: world.clone(),
            world,
            acc: 0.0,
            alpha: 0.0,
            ready: if def.boss.is_some() { 2.6 } else { 1.6 },
            latched: vec![InputFrame::default(); inputs.len()],
            inputs,
        }
    }

    /// Runs as many fixed steps as `dt` covers and returns the emitted events.
    fn advance(&mut self, dt: f32, def: &LevelDef, controls: &Controls) -> Vec<SimEvent> {
        let mut events = Vec::new();
        if self.ready > 0.0 {
            self.ready -= dt;
            return events;
        }
        let frames: Vec<InputFrame> = self.inputs.iter_mut().map(|s| s.sample(controls)).collect();
        for (l, f) in self.latched.iter_mut().zip(&frames) {
            l.0 |= f.0;
        }
        self.acc += dt;
        let mut steps = 0;
        while self.acc >= STEP && steps < 8 {
            self.prev = self.world.clone();
            self.world.step(&self.latched, def);
            self.latched.clone_from(&frames);
            events.append(&mut self.world.events);
            self.acc -= STEP;
            steps += 1;
        }
        if steps == 8 {
            self.acc = 0.0;
        }
        self.alpha = self.acc / STEP;
        events
    }
}

struct NameEntry {
    letters: [u8; 3],
    pos: usize,
    score: u32,
    stage: u8,
}

impl NameEntry {
    fn new(score: u32, stage: u8) -> Self {
        Self {
            letters: [b'A'; 3],
            pos: 0,
            score,
            stage,
        }
    }

    fn name(&self) -> String {
        self.letters.iter().map(|&b| b as char).collect()
    }
}

enum Screen {
    Title,
    StageSelect,
    HighScores { highlight: Option<usize> },
    ControlsMenu { waiting: bool },
    Lobby,
    StoryIntro { page: usize, t: f32, full: bool },
    Playing,
    Boss,
    Paused { boss: bool },
    LevelClear { t: f32, bonus: u32 },
    GameOver { t: f32, entry: Option<NameEntry> },
    Victory { t: f32, entry: Option<NameEntry> },
}

const TITLE_ITEMS: [&str; 7] = [
    "START",
    "STAGE SELECT",
    "LAN CO-OP",
    "HIGH SCORES",
    "CONTROLS",
    "FULLSCREEN",
    "QUIT",
];
const PAUSE_ITEMS: [&str; 3] = ["RESUME", "RESTART STAGE", "QUIT TO TITLE"];

const STORY: [&str; 4] = [
    "Sky City. A bright morning in the park with Leo and Nina...",
    "Suddenly Dr. Demos swoops down in his hover-pod and snatches Nina!",
    "He fills the skies with bouncing orbs to cover his escape.",
    "Leo grabs his trusty harpoon. Ten stages stand between him and Nina!",
];

struct TitleOrb {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    tier: Tier,
    color: u8,
}

pub struct App {
    levels: Vec<LevelDef>,
    save: SaveData,
    audio: Audio,
    pub controls: Controls,
    fx: Fx,
    screen: Screen,
    session: Option<Session>,
    t: f32,
    fade: f32,
    sel: usize,
    sel_anim: f32,
    title_theme: Theme,
    title_orbs: Vec<TitleOrb>,
    pub quit: bool,
}

impl App {
    pub fn new(levels: Vec<LevelDef>, save: SaveData, audio: Audio) -> Self {
        let controls = Controls::new(save.bindings.clone());
        let title_orbs = (0..6)
            .map(|i| {
                let tier = [Tier::XL, Tier::L, Tier::M, Tier::L, Tier::S, Tier::M][i];
                TitleOrb {
                    x: 150.0 + i as f32 * 190.0,
                    y: 120.0 + (i % 3) as f32 * 60.0,
                    vx: if i % 2 == 0 { 90.0 } else { -90.0 },
                    vy: 0.0,
                    tier,
                    color: i as u8,
                }
            })
            .collect();
        Self {
            levels,
            save,
            audio,
            controls,
            fx: Fx::default(),
            screen: Screen::Title,
            session: None,
            t: 0.0,
            fade: 1.0,
            sel: 0,
            sel_anim: 0.0,
            title_theme: Theme {
                sky_top: "#5b2a86".into(),
                sky_bottom: "#ff8fb1".into(),
                accent: "#ff4fd8".into(),
                decor: Decor::Clouds,
            },
            title_orbs,
            quit: false,
        }
    }

    pub fn toggle_fullscreen(&mut self) {
        self.save.fullscreen = !self.save.fullscreen;
        set_fullscreen(self.save.fullscreen);
        self.save.store();
    }

    fn go(&mut self, screen: Screen) {
        self.screen = screen;
        self.fade = 1.0;
        self.sel = 0;
    }

    fn start_level(&mut self, level: usize, carry: Carry) {
        let def = &self.levels[level];
        let boss = def.boss.is_some();
        self.session = Some(Session::new(level, def, carry));
        self.fx.clear();
        self.go(if boss { Screen::Boss } else { Screen::Playing });
        self.audio
            .play(if boss { Sfx::Warning } else { Sfx::MenuSelect });
    }

    fn menu_nav(&mut self, count: usize) {
        if self.controls.pressed(Action::Up) {
            self.sel = (self.sel + count - 1) % count;
            self.audio.play(Sfx::MenuMove);
        }
        if self.controls.pressed(Action::Down) {
            self.sel = (self.sel + 1) % count;
            self.audio.play(Sfx::MenuMove);
        }
    }

    fn confirm(&self) -> bool {
        self.controls.pressed(Action::Confirm)
    }

    fn back(&self) -> bool {
        self.controls.pressed(Action::Back)
    }

    pub fn update(&mut self, dt: f32) {
        self.t += dt;
        self.fade = (self.fade - dt * 2.5).max(0.0);
        self.sel_anim = approach(self.sel_anim, self.sel as f32, 14.0, dt);
        self.fx.update(dt);
        self.update_title_orbs(dt);

        let screen = std::mem::replace(&mut self.screen, Screen::Title);
        let next = self.update_screen(screen, dt);
        // `go` may already have replaced the screen; only restore if not.
        if let Some(s) = next {
            self.screen = s;
        }
    }

    /// Returns the screen to keep, or `None` if `go` installed a new one.
    fn update_screen(&mut self, screen: Screen, dt: f32) -> Option<Screen> {
        match screen {
            Screen::Title => {
                self.menu_nav(TITLE_ITEMS.len());
                if !self.confirm() {
                    return Some(Screen::Title);
                }
                self.audio.play(Sfx::MenuSelect);
                match self.sel {
                    0 => self.go(Screen::StoryIntro {
                        page: 0,
                        t: 0.0,
                        full: false,
                    }),
                    1 => self.go(Screen::StageSelect),
                    2 => self.go(Screen::Lobby),
                    3 => self.go(Screen::HighScores { highlight: None }),
                    4 => self.go(Screen::ControlsMenu { waiting: false }),
                    5 => {
                        self.toggle_fullscreen();
                        return Some(Screen::Title);
                    }
                    _ => {
                        self.quit = true;
                        return Some(Screen::Title);
                    }
                }
                None
            }
            Screen::StageSelect => {
                let unlocked = (self.save.unlocked as usize).clamp(1, LEVEL_COUNT);
                if self.controls.pressed(Action::Left) || self.controls.pressed(Action::Up) {
                    self.sel = (self.sel + unlocked - 1) % unlocked;
                    self.audio.play(Sfx::MenuMove);
                }
                if self.controls.pressed(Action::Right) || self.controls.pressed(Action::Down) {
                    self.sel = (self.sel + 1) % unlocked;
                    self.audio.play(Sfx::MenuMove);
                }
                if self.confirm() {
                    let level = self.sel;
                    self.start_level(level, Carry::new(1));
                    return None;
                }
                if self.back() {
                    self.go_title(1);
                    return None;
                }
                Some(Screen::StageSelect)
            }
            Screen::HighScores { highlight } => {
                if self.confirm() || self.back() {
                    self.go_title(3);
                    return None;
                }
                Some(Screen::HighScores { highlight })
            }
            Screen::Lobby => {
                if self.confirm() || self.back() {
                    self.go_title(2);
                    return None;
                }
                Some(Screen::Lobby)
            }
            Screen::ControlsMenu { waiting } => self.update_controls_menu(waiting),
            Screen::StoryIntro { page, t, full } => {
                let t = t + dt;
                let full = full || story_chars(t) >= STORY[page].len();
                if self.back() {
                    self.start_level(0, Carry::new(1));
                    return None;
                }
                if self.confirm() {
                    if !full {
                        return Some(Screen::StoryIntro {
                            page,
                            t,
                            full: true,
                        });
                    }
                    self.audio.play(Sfx::MenuMove);
                    if page + 1 >= STORY.len() {
                        self.start_level(0, Carry::new(1));
                        return None;
                    }
                    return Some(Screen::StoryIntro {
                        page: page + 1,
                        t: 0.0,
                        full: false,
                    });
                }
                Some(Screen::StoryIntro { page, t, full })
            }
            Screen::Playing | Screen::Boss => self.update_play(screen, dt),
            Screen::Paused { boss } => {
                self.menu_nav(PAUSE_ITEMS.len());
                if self.controls.pressed(Action::Pause) && self.sel == 0 || self.back() {
                    return Some(if boss { Screen::Boss } else { Screen::Playing });
                }
                if self.confirm() {
                    self.audio.play(Sfx::MenuSelect);
                    match self.sel {
                        0 => return Some(if boss { Screen::Boss } else { Screen::Playing }),
                        1 => {
                            let s = self.session.as_ref().expect("session while paused");
                            let (level, carry) = (s.level, s.start.clone());
                            self.start_level(level, carry);
                        }
                        _ => {
                            self.session = None;
                            self.go_title(0);
                        }
                    }
                    return None;
                }
                Some(Screen::Paused { boss })
            }
            Screen::LevelClear { t, bonus } => {
                let t = t + dt;
                if t > 1.4 && self.confirm() {
                    let s = self.session.as_ref().expect("session on level clear");
                    let mut carry = s.world.carry();
                    carry.score += bonus;
                    let next = s.level + 1;
                    self.start_level(next, carry);
                    return None;
                }
                Some(Screen::LevelClear { t, bonus })
            }
            Screen::GameOver { t, entry } => self.update_end(t + dt, entry, false),
            Screen::Victory { t, entry } => {
                let t = t + dt;
                if (t - dt) < 2.4 && t >= 2.4 {
                    self.fx.hearts(640.0, 560.0, 20);
                }
                if t > 3.0 && (self.t * 2.0).fract() < dt * 2.0 {
                    self.fx.confetti(rand::gen_range(200.0, 1080.0), 200.0, 20);
                }
                self.update_end(t, entry, true)
            }
        }
    }

    fn go_title(&mut self, sel: usize) {
        self.go(Screen::Title);
        self.sel = sel;
        self.sel_anim = sel as f32;
    }

    fn update_title_orbs(&mut self, dt: f32) {
        for o in &mut self.title_orbs {
            let r = o.tier.radius();
            o.vy += GRAVITY * dt * 0.6;
            o.x += o.vx * dt;
            o.y += o.vy * dt;
            if o.x < r || o.x > VIEW_W - r {
                o.vx = -o.vx;
                o.x = o.x.clamp(r, VIEW_W - r);
            }
            if o.y > VIEW_H - r - 20.0 {
                o.y = VIEW_H - r - 20.0;
                o.vy = -o.tier.bounce_speed() * 0.85;
            }
        }
    }

    fn update_controls_menu(&mut self, waiting: bool) -> Option<Screen> {
        let rows = ACTIONS.len() + 1;
        if waiting {
            let action = ACTIONS[self.sel];
            if let Some(b) = self.controls.last_button {
                self.controls.bindings.set_button(action, b);
            } else if let Some(k) = self.controls.last_key {
                if k != KeyCode::Escape {
                    self.controls.bindings.set_key(action, k);
                }
            } else {
                return Some(Screen::ControlsMenu { waiting: true });
            }
            self.save.bindings = self.controls.bindings.clone();
            self.save.store();
            self.audio.play(Sfx::MenuSelect);
            return Some(Screen::ControlsMenu { waiting: false });
        }
        self.menu_nav(rows);
        if self.back() {
            self.go_title(4);
            return None;
        }
        if self.confirm() {
            self.audio.play(Sfx::MenuSelect);
            if self.sel == ACTIONS.len() {
                self.controls.bindings = Default::default();
                self.save.bindings = self.controls.bindings.clone();
                self.save.store();
                return Some(Screen::ControlsMenu { waiting: false });
            }
            return Some(Screen::ControlsMenu { waiting: true });
        }
        Some(Screen::ControlsMenu { waiting: false })
    }

    fn update_play(&mut self, screen: Screen, dt: f32) -> Option<Screen> {
        let boss = matches!(screen, Screen::Boss);
        let session = self.session.as_mut().expect("session while playing");
        if self.controls.pressed(Action::Pause) && session.ready <= 0.0 {
            self.audio.play(Sfx::MenuMove);
            self.sel = 0;
            self.sel_anim = 0.0;
            return Some(Screen::Paused { boss });
        }
        let def = &self.levels[session.level];
        let events = session.advance(dt, def, &self.controls);
        for ev in &events {
            self.fx.on_event(ev);
            play_event_sound(&self.audio, ev);
        }
        let session = self.session.as_ref().expect("session while playing");
        match session.world.status {
            Status::Cleared if boss => {
                let w = &session.world;
                self.save.unlock(LEVEL_COUNT as u8);
                self.save.store();
                self.audio.play(Sfx::Victory);
                let entry = self
                    .save
                    .qualifies(w.score)
                    .then(|| NameEntry::new(w.score, LEVEL_COUNT as u8));
                self.fx.clear();
                self.go(Screen::Victory { t: 0.0, entry });
                None
            }
            Status::Cleared => {
                let w = &session.world;
                let bonus = w.time_left.max(0.0) as u32 * TIME_BONUS_PER_SEC;
                self.save.unlock((session.level + 2).min(LEVEL_COUNT) as u8);
                self.save.store();
                self.audio.play(Sfx::Clear);
                Some(Screen::LevelClear { t: 0.0, bonus })
            }
            Status::GameOver => {
                let w = &session.world;
                let stage = (session.level + 1) as u8;
                let entry = self
                    .save
                    .qualifies(w.score)
                    .then(|| NameEntry::new(w.score, stage));
                self.audio.play(Sfx::GameOver);
                Some(Screen::GameOver { t: 0.0, entry })
            }
            _ => Some(screen),
        }
    }

    /// Shared by Game Over and Victory: optional initials entry, then exit.
    fn update_end(&mut self, t: f32, entry: Option<NameEntry>, victory: bool) -> Option<Screen> {
        let wrap = |t, entry| {
            if victory {
                Screen::Victory { t, entry }
            } else {
                Screen::GameOver { t, entry }
            }
        };
        let ready = t > if victory { 6.0 } else { 1.2 };
        let Some(mut e) = entry else {
            if ready && self.confirm() {
                self.session = None;
                self.go_title(0);
                return None;
            }
            return Some(wrap(t, None));
        };
        if !ready {
            return Some(wrap(t, Some(e)));
        }
        let c = &mut e.letters[e.pos];
        if self.controls.pressed(Action::Up) {
            *c = if *c == b'Z' { b'A' } else { *c + 1 };
            self.audio.play(Sfx::MenuMove);
        }
        if self.controls.pressed(Action::Down) {
            *c = if *c == b'A' { b'Z' } else { *c - 1 };
            self.audio.play(Sfx::MenuMove);
        }
        if self.controls.pressed(Action::Left) && e.pos > 0 {
            e.pos -= 1;
        }
        if self.controls.pressed(Action::Right) && e.pos < 2 {
            e.pos += 1;
        }
        if self.confirm() || self.controls.pressed(Action::Fire) {
            if e.pos < 2 {
                e.pos += 1;
                self.audio.play(Sfx::MenuMove);
            } else {
                let rank = self.save.add_score(ScoreEntry {
                    name: e.name(),
                    score: e.score,
                    stage: e.stage,
                });
                self.save.store();
                self.audio.play(Sfx::MenuSelect);
                self.session = None;
                self.go(Screen::HighScores { highlight: rank });
                return None;
            }
        }
        Some(wrap(t, Some(e)))
    }

    // ---- drawing -----------------------------------------------------------

    pub fn shake(&self) -> Vec2 {
        self.fx.shake_offset()
    }

    pub fn draw(&self) {
        match &self.screen {
            Screen::Title => self.draw_title(),
            Screen::StageSelect => self.draw_stage_select(),
            Screen::HighScores { highlight } => self.draw_high_scores(*highlight),
            Screen::ControlsMenu { waiting } => self.draw_controls(*waiting),
            Screen::Lobby => self.draw_lobby(),
            Screen::StoryIntro { page, t, full } => self.draw_story(*page, *t, *full),
            Screen::Playing | Screen::Boss => self.draw_session(),
            Screen::Paused { .. } => {
                self.draw_session();
                self.draw_pause();
            }
            Screen::LevelClear { t, bonus } => {
                self.draw_session();
                self.draw_level_clear(*t, *bonus);
            }
            Screen::GameOver { t, entry } => {
                self.draw_session();
                self.draw_game_over(*t, entry.as_ref());
            }
            Screen::Victory { t, entry } => self.draw_victory(*t, entry.as_ref()),
        }
        self.fx.draw_flash();
        if self.fade > 0.0 {
            draw_rectangle(
                0.0,
                0.0,
                VIEW_W,
                VIEW_H,
                Color::new(0.02, 0.0, 0.06, smoothstep(self.fade)),
            );
        }
    }

    fn draw_menu_backdrop(&self) {
        draw_background(&self.title_theme, self.t);
        draw_rectangle(0.0, 0.0, VIEW_W, VIEW_H, Color::new(0.05, 0.0, 0.12, 0.2));
        for o in &self.title_orbs {
            glossy_orb(o.x, o.y, o.tier.radius(), orb_color(o.color), 0.0);
        }
    }

    fn draw_menu_list(&self, items: &[&str], cx: f32, y0: f32, gap: f32, dimmed: &[usize]) {
        let hy = y0 + self.sel_anim * gap;
        let w = 380.0;
        glossy_panel(
            cx - w * 0.5,
            hy - 34.0,
            w,
            46.0,
            18.0,
            Color::new(1.0, 0.45, 0.85, 0.9),
            Color::new(0.55, 0.15, 0.85, 0.9),
        );
        for (i, item) in items.iter().enumerate() {
            let y = y0 + i as f32 * gap;
            let c = if dimmed.contains(&i) {
                Color::new(0.85, 0.8, 0.95, 0.6)
            } else {
                WHITE
            };
            let size = if i == self.sel { 32.0 } else { 28.0 };
            text_center(item, cx, y, size, c);
        }
    }

    fn draw_title(&self) {
        self.draw_menu_backdrop();
        let pop = ease_out_back((self.t * 1.2).min(1.0));
        fancy_headline("SKYPOP", VIEW_W * 0.5, 170.0, 120.0 * pop, self.t);
        text_center(
            "a balloon-popping arcade adventure",
            VIEW_W * 0.5,
            215.0,
            24.0,
            Color::new(1.0, 0.95, 1.0, 0.9),
        );
        let mut items = TITLE_ITEMS;
        items[5] = if self.save.fullscreen {
            "WINDOWED"
        } else {
            "FULLSCREEN"
        };
        self.draw_menu_list(&items, VIEW_W * 0.5, 300.0, 52.0, &[]);
        text_center(
            "Quiet Byte contributors  -  Apache-2.0",
            VIEW_W * 0.5,
            VIEW_H - 18.0,
            16.0,
            with_alpha(WHITE, 0.7),
        );
        let hint = format!(
            "{} to select   -   F11 fullscreen",
            self.controls.bindings.describe(Action::Confirm)
        );
        text_center(
            &hint,
            VIEW_W * 0.5,
            VIEW_H - 44.0,
            16.0,
            with_alpha(WHITE, 0.7),
        );
    }

    fn draw_stage_select(&self) {
        self.draw_menu_backdrop();
        headline("STAGE SELECT", VIEW_W * 0.5, 110.0, 56.0, WHITE);
        let unlocked = (self.save.unlocked as usize).clamp(1, LEVEL_COUNT);
        for (i, def) in self.levels.iter().enumerate() {
            let col = i % 5;
            let row = i / 5;
            let (w, h) = (200.0, 150.0);
            let x = 90.0 + col as f32 * 225.0;
            let y = 180.0 + row as f32 * 190.0;
            let open = i < unlocked;
            let selected = open && i == self.sel;
            let lift = if selected {
                8.0 + (self.t * 4.0).sin() * 3.0
            } else {
                0.0
            };
            let top = crate::render::hex(&def.theme.sky_top);
            let bottom = crate::render::hex(&def.theme.sky_bottom);
            let (top, bottom) = if open {
                (top, bottom)
            } else {
                (darken(top, 0.7), darken(bottom, 0.7))
            };
            if selected {
                rounded_rect(x - 6.0, y - lift - 6.0, w + 12.0, h + 12.0, 22.0, WHITE);
            }
            glossy_panel(x, y - lift, w, h, 18.0, top, bottom);
            let label = if i + 1 == LEVEL_COUNT {
                "BOSS".to_string()
            } else {
                format!("{}", i + 1)
            };
            headline(
                &label,
                x + w * 0.5,
                y - lift + 90.0,
                54.0,
                if open {
                    WHITE
                } else {
                    Color::new(1.0, 1.0, 1.0, 0.35)
                },
            );
            let name = if open { def.name.as_str() } else { "LOCKED" };
            text_center(
                name,
                x + w * 0.5,
                y - lift + 126.0,
                18.0,
                if open {
                    WHITE
                } else {
                    Color::new(1.0, 1.0, 1.0, 0.4)
                },
            );
        }
        text_center(
            "Clear a stage to unlock the next.",
            VIEW_W * 0.5,
            640.0,
            20.0,
            with_alpha(WHITE, 0.85),
        );
    }

    fn draw_high_scores(&self, highlight: Option<usize>) {
        self.draw_menu_backdrop();
        headline("HIGH SCORES", VIEW_W * 0.5, 100.0, 56.0, WHITE);
        glossy_panel(
            340.0,
            130.0,
            600.0,
            500.0,
            26.0,
            Color::new(0.3, 0.12, 0.5, 0.85),
            Color::new(0.12, 0.04, 0.25, 0.9),
        );
        if self.save.high_scores.is_empty() {
            text_center(
                "No scores yet. Go pop some orbs!",
                VIEW_W * 0.5,
                380.0,
                26.0,
                WHITE,
            );
        }
        for (i, e) in self.save.high_scores.iter().enumerate() {
            let y = 180.0 + i as f32 * 44.0;
            let hot = highlight == Some(i);
            let c = if hot {
                orb_color((self.t * 6.0) as u8)
            } else {
                WHITE
            };
            text_shadow(&format!("{:>2}.", i + 1), 390.0, y, 28.0, c);
            text_shadow(&e.name, 470.0, y, 28.0, c);
            text_shadow(&format!("{:07}", e.score), 600.0, y, 28.0, c);
            let stage = if e.stage as usize >= LEVEL_COUNT {
                "BOSS".to_string()
            } else {
                format!("ST {}", e.stage)
            };
            text_shadow(&stage, 790.0, y, 24.0, with_alpha(c, 0.8));
        }
        text_center(
            "Press Confirm or Back",
            VIEW_W * 0.5,
            680.0,
            18.0,
            with_alpha(WHITE, 0.8),
        );
    }

    fn draw_controls(&self, waiting: bool) {
        self.draw_menu_backdrop();
        headline("CONTROLS", VIEW_W * 0.5, 90.0, 56.0, WHITE);
        let y0 = 160.0;
        let gap = 50.0;
        let hy = y0 + self.sel_anim * gap;
        glossy_panel(
            170.0,
            hy - 32.0,
            940.0,
            44.0,
            16.0,
            Color::new(1.0, 0.45, 0.85, 0.85),
            Color::new(0.55, 0.15, 0.85, 0.85),
        );
        for (i, a) in ACTIONS.iter().enumerate() {
            let y = y0 + i as f32 * gap;
            text_shadow(a.label(), 200.0, y, 26.0, WHITE);
            let binding = if waiting && i == self.sel {
                "press a key or button...".to_string()
            } else {
                self.controls.bindings.describe(*a)
            };
            text_shadow(&binding, 520.0, y, 24.0, Color::new(0.85, 0.95, 1.0, 1.0));
        }
        text_shadow(
            "Reset to defaults",
            200.0,
            y0 + ACTIONS.len() as f32 * gap,
            26.0,
            WHITE,
        );
        text_center(
            "Confirm: rebind primary key/button   -   Back: return",
            VIEW_W * 0.5,
            690.0,
            18.0,
            with_alpha(WHITE, 0.8),
        );
    }

    fn draw_lobby(&self) {
        self.draw_menu_backdrop();
        headline("LAN CO-OP", VIEW_W * 0.5, 130.0, 56.0, WHITE);
        glossy_panel(
            290.0,
            190.0,
            700.0,
            340.0,
            26.0,
            Color::new(0.3, 0.12, 0.5, 0.85),
            Color::new(0.12, 0.04, 0.25, 0.9),
        );
        for (i, line) in ["HOST GAME", "JOIN GAME"].iter().enumerate() {
            text_center(
                line,
                VIEW_W * 0.5,
                270.0 + i as f32 * 60.0,
                34.0,
                Color::new(1.0, 1.0, 1.0, 0.45),
            );
        }
        text_center(
            "Two players, two PCs, one local network.",
            VIEW_W * 0.5,
            420.0,
            22.0,
            WHITE,
        );
        text_center(
            "Arrives in milestone 2.",
            VIEW_W * 0.5,
            460.0,
            22.0,
            Color::new(1.0, 0.8, 0.95, 1.0),
        );
        draw_leo(560.0, 650.0, 1.0, self.t * 6.0, 0.0, &LEO_STYLES[0], 1.0);
        draw_leo(
            720.0,
            650.0,
            -1.0,
            self.t * 6.0 + 1.5,
            0.0,
            &LEO_STYLES[1],
            1.0,
        );
        text_center(
            "Press Back",
            VIEW_W * 0.5,
            690.0,
            18.0,
            with_alpha(WHITE, 0.8),
        );
    }

    fn draw_story(&self, page: usize, t: f32, full: bool) {
        let def = &self.levels[0];
        let theme = if page >= 2 {
            &self.levels[1].theme
        } else {
            &def.theme
        };
        draw_background(theme, self.t);
        let floor = 600.0;
        let e = ease_out_cubic(t / 1.5);
        match page {
            0 => {
                draw_leo(
                    520.0 + (t * 2.0).sin() * 4.0,
                    floor,
                    1.0,
                    t * 5.0,
                    0.0,
                    &LEO_STYLES[0],
                    1.0,
                );
                draw_nina(640.0, floor, -1.0, 0.0, t);
                crate::render::heart(
                    580.0,
                    470.0 - (t * 2.0).sin() * 6.0,
                    10.0,
                    Color::new(1.0, 0.35, 0.55, 0.9),
                );
            }
            1 => {
                let pod_y = -80.0 + e * 380.0 - smoothstep((t - 1.8) / 1.2) * 450.0;
                let nina_y = if t > 1.6 { pod_y + 140.0 } else { floor };
                draw_leo(460.0, floor, 1.0, 0.0, 0.0, &LEO_STYLES[0], 1.0);
                draw_nina(
                    660.0,
                    nina_y.min(floor),
                    -1.0,
                    0.0,
                    if t > 1.6 { t } else { 0.0 },
                );
                draw_demos_pod(660.0, pod_y, self.t, 0.0, 0.0);
            }
            2 => {
                for i in 0..5 {
                    let x = 150.0 + i as f32 * 240.0 + (t * 60.0 + i as f32 * 90.0) % 120.0;
                    let y = 450.0 - ((t * 3.0 + i as f32).sin()).abs() * 250.0;
                    let tier = [Tier::XL, Tier::L, Tier::M, Tier::L, Tier::XL][i];
                    glossy_orb(x, y, tier.radius(), orb_color(i as u8), 0.0);
                }
                draw_demos_pod(900.0 + t * 120.0, 140.0 - t * 20.0, self.t, 0.0, 0.0);
            }
            _ => {
                let throw = ((t * 2.0).sin() * 0.5 + 0.5).powi(3);
                draw_leo(640.0, floor, 1.0, 0.0, throw, &LEO_STYLES[0], 1.0);
                fancy_headline(
                    "10 STAGES",
                    VIEW_W * 0.5,
                    240.0,
                    72.0 * ease_out_back(t / 0.8),
                    self.t,
                );
            }
        }
        // Caption box with typewriter text.
        glossy_panel(
            120.0,
            20.0,
            1040.0,
            90.0,
            24.0,
            Color::new(0.2, 0.08, 0.35, 0.85),
            Color::new(0.08, 0.02, 0.18, 0.9),
        );
        let text_full = STORY[page];
        let shown: String = text_full
            .chars()
            .take(if full { usize::MAX } else { story_chars(t) })
            .collect();
        text_center(&shown, VIEW_W * 0.5, 74.0, 28.0, WHITE);
        let hint = format!(
            "{} / {}    Confirm: next    Back: skip",
            page + 1,
            STORY.len()
        );
        text_center(&hint, VIEW_W * 0.5, 700.0, 18.0, with_alpha(WHITE, 0.85));
    }

    fn draw_session(&self) {
        let Some(s) = &self.session else { return };
        let theme = &self.levels[s.level].theme;
        draw_world(&s.world, &s.prev, s.alpha, theme, self.t);
        self.fx.draw();
        draw_hud(&s.world, theme, self.t);
        if s.ready > 0.0 {
            let boss = self.levels[s.level].boss.is_some();
            let total = if boss { 2.6 } else { 1.6 };
            let k = ease_out_back(((total - s.ready) / 0.5).min(1.0));
            let a = (s.ready / 0.3).min(1.0);
            if boss {
                let blink = if (self.t * 5.0).sin() > 0.0 { 1.0 } else { 0.6 };
                draw_rectangle(
                    0.0,
                    250.0,
                    VIEW_W,
                    120.0,
                    Color::new(0.6, 0.0, 0.1, 0.55 * a),
                );
                headline(
                    "WARNING",
                    VIEW_W * 0.5,
                    320.0,
                    80.0 * k,
                    Color::new(1.0, 0.3, 0.35, blink * a),
                );
                text_center(
                    "DR. DEMOS APPROACHES",
                    VIEW_W * 0.5,
                    356.0,
                    26.0,
                    with_alpha(WHITE, a),
                );
            } else {
                headline(
                    &format!("STAGE {}", s.level + 1),
                    VIEW_W * 0.5,
                    300.0,
                    72.0 * k,
                    with_alpha(WHITE, a),
                );
                text_center(
                    &self.levels[s.level].name,
                    VIEW_W * 0.5,
                    345.0,
                    32.0,
                    with_alpha(WHITE, a),
                );
                text_center(
                    "GET READY!",
                    VIEW_W * 0.5,
                    395.0,
                    28.0,
                    Color::new(1.0, 0.9, 0.4, a),
                );
            }
        }
    }

    fn draw_pause(&self) {
        draw_rectangle(0.0, 0.0, VIEW_W, VIEW_H, Color::new(0.03, 0.0, 0.1, 0.6));
        headline("PAUSED", VIEW_W * 0.5, 220.0, 72.0, WHITE);
        self.draw_menu_list(&PAUSE_ITEMS, VIEW_W * 0.5, 320.0, 60.0, &[]);
    }

    fn draw_level_clear(&self, t: f32, bonus: u32) {
        let Some(s) = &self.session else { return };
        let k = ease_out_back((t / 0.6).min(1.0));
        draw_rectangle(
            0.0,
            0.0,
            VIEW_W,
            VIEW_H,
            Color::new(0.03, 0.0, 0.1, 0.45 * (t / 0.4).min(1.0)),
        );
        let (w, h) = (620.0 * k, 320.0 * k);
        glossy_panel(
            VIEW_W * 0.5 - w * 0.5,
            180.0,
            w,
            h,
            28.0,
            Color::new(1.0, 0.5, 0.85, 0.92),
            Color::new(0.45, 0.15, 0.8, 0.92),
        );
        if t < 0.4 {
            return;
        }
        fancy_headline("STAGE CLEAR!", VIEW_W * 0.5, 260.0, 64.0, self.t);
        let counted = (bonus as f32 * ease_out_cubic((t - 0.5) / 1.0)) as u32;
        text_center(
            &format!("TIME BONUS  {counted}"),
            VIEW_W * 0.5,
            330.0,
            32.0,
            WHITE,
        );
        text_center(
            &format!("SCORE  {:07}", s.world.score + counted),
            VIEW_W * 0.5,
            380.0,
            36.0,
            WHITE,
        );
        if t > 1.4 {
            let a = (self.t * 3.0).sin() * 0.3 + 0.7;
            let next = if s.level + 2 == LEVEL_COUNT {
                "Confirm: face Dr. Demos"
            } else {
                "Confirm: next stage"
            };
            text_center(next, VIEW_W * 0.5, 450.0, 24.0, with_alpha(WHITE, a));
        }
    }

    fn draw_name_entry(&self, e: &NameEntry, y: f32) {
        text_center(
            "NEW HIGH SCORE! ENTER YOUR INITIALS",
            VIEW_W * 0.5,
            y,
            26.0,
            Color::new(1.0, 0.9, 0.4, 1.0),
        );
        for i in 0..3 {
            let x = VIEW_W * 0.5 - 90.0 + i as f32 * 60.0;
            let hot = i == e.pos;
            let lift = if hot { (self.t * 6.0).sin() * 4.0 } else { 0.0 };
            glossy_panel(
                x,
                y + 20.0 - lift,
                52.0,
                64.0,
                12.0,
                if hot {
                    lighten(orb_color(0), 0.3)
                } else {
                    Color::new(0.4, 0.3, 0.6, 1.0)
                },
                Color::new(0.2, 0.1, 0.35, 1.0),
            );
            text_center(
                &(e.letters[i] as char).to_string(),
                x + 26.0,
                y + 70.0 - lift,
                44.0,
                WHITE,
            );
        }
        text_center(
            "Up/Down: letter   Left/Right: move   Confirm: next",
            VIEW_W * 0.5,
            y + 120.0,
            18.0,
            with_alpha(WHITE, 0.85),
        );
    }

    fn draw_game_over(&self, t: f32, entry: Option<&NameEntry>) {
        let a = (t / 0.8).min(1.0);
        draw_rectangle(
            0.0,
            0.0,
            VIEW_W,
            VIEW_H,
            Color::new(0.05, 0.0, 0.08, 0.65 * a),
        );
        let drop = ease_out_back(a);
        headline(
            "GAME OVER",
            VIEW_W * 0.5,
            120.0 + 120.0 * drop,
            90.0,
            Color::new(1.0, 0.35, 0.45, a),
        );
        if let Some(s) = &self.session {
            text_center(
                &format!("FINAL SCORE  {:07}", s.world.score),
                VIEW_W * 0.5,
                320.0,
                32.0,
                with_alpha(WHITE, a),
            );
        }
        if t < 1.2 {
            return;
        }
        match entry {
            Some(e) => self.draw_name_entry(e, 390.0),
            None => text_center(
                "Press Confirm",
                VIEW_W * 0.5,
                420.0,
                24.0,
                with_alpha(WHITE, (self.t * 3.0).sin() * 0.3 + 0.7),
            ),
        }
    }

    fn draw_victory(&self, t: f32, entry: Option<&NameEntry>) {
        let theme = &self.levels[LEVEL_COUNT - 1].theme;
        draw_background(theme, self.t);
        let floor = 620.0;
        // The hover-pod sputters off into the distance.
        let pod_t = (t / 3.0).min(1.0);
        draw_demos_pod(
            900.0 + pod_t * 600.0,
            140.0 - pod_t * 120.0,
            self.t,
            0.0,
            1.0,
        );
        // Cage opens, Nina runs to Leo.
        let open = smoothstep(t / 1.5);
        let run = smoothstep((t - 1.2) / 1.6);
        draw_cage(880.0, 380.0, 80.0, 90.0, open);
        let nina_x = 920.0 - run * 220.0;
        let nina_floor = if run > 0.0 { floor } else { 465.0 };
        let hop = if run > 0.0 && run < 1.0 {
            (run * std::f32::consts::PI).sin() * 40.0
        } else {
            0.0
        };
        draw_leo(600.0, floor, 1.0, 0.0, 0.0, &LEO_STYLES[0], 1.0);
        draw_nina(
            nina_x,
            nina_floor - hop,
            -1.0,
            if run > 0.0 && run < 1.0 { t * 8.0 } else { 0.0 },
            if run >= 1.0 { t } else { 0.0 },
        );
        self.fx.draw();

        let k = ease_out_back(((t - 2.6) / 0.8).clamp(0.0, 1.0));
        if t > 2.6 {
            fancy_headline("NINA IS SAFE!", VIEW_W * 0.5, 160.0, 80.0 * k, self.t);
        }
        if t > 3.6 {
            text_center(
                "Dr. Demos limps away. The skies of Sky City are clear again.",
                VIEW_W * 0.5,
                230.0,
                24.0,
                WHITE,
            );
        }
        if let Some(s) = &self.session {
            if t > 4.4 {
                text_center(
                    &format!("FINAL SCORE  {:07}", s.world.score),
                    VIEW_W * 0.5,
                    280.0,
                    34.0,
                    WHITE,
                );
            }
        }
        if t > 6.0 {
            match entry {
                Some(e) => {
                    glossy_panel(
                        300.0,
                        300.0,
                        680.0,
                        170.0,
                        24.0,
                        Color::new(0.3, 0.12, 0.5, 0.85),
                        Color::new(0.12, 0.04, 0.25, 0.9),
                    );
                    self.draw_name_entry(e, 330.0);
                }
                None => {
                    let a = (self.t * 3.0).sin() * 0.3 + 0.7;
                    text_center(
                        "THANK YOU FOR PLAYING  -  Press Confirm",
                        VIEW_W * 0.5,
                        340.0,
                        24.0,
                        with_alpha(WHITE, a),
                    );
                }
            }
        }
    }
}

/// Characters of story text revealed after `t` seconds (typewriter effect).
fn story_chars(t: f32) -> usize {
    (t * 40.0) as usize
}

fn play_event_sound(audio: &Audio, ev: &SimEvent) {
    match ev {
        SimEvent::Fire { .. } => audio.play_vol(Sfx::Shoot, 0.6),
        SimEvent::Pop { tier, .. } => audio.play(match tier {
            Tier::XL => Sfx::Pop0,
            Tier::L => Sfx::Pop1,
            Tier::M => Sfx::Pop2,
            Tier::S => Sfx::Pop3,
        }),
        SimEvent::WireBlocked { .. } | SimEvent::AnchorStuck { .. } => {
            audio.play_vol(Sfx::Clink, 0.5)
        }
        SimEvent::PlatformBroken { .. } => audio.play(Sfx::Break),
        SimEvent::PlayerHit { .. } => audio.play(Sfx::Hurt),
        SimEvent::ShieldBroken { .. } => audio.play(Sfx::Shield),
        SimEvent::Pickup { kind, .. } => audio.play(if *kind == DropKind::Freeze {
            Sfx::Freeze
        } else {
            Sfx::Pickup
        }),
        SimEvent::BossHit { .. } => audio.play(Sfx::BossHit),
        SimEvent::BossDefeated { .. } => audio.play(Sfx::Explosion),
        SimEvent::TimeUp => audio.play(Sfx::Hurt),
        SimEvent::LevelCleared => {}
    }
}
