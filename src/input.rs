// SPDX-License-Identifier: Apache-2.0
//! Device input: keyboard + gamepad, remappable bindings, and the
//! `InputSource` abstraction that feeds `InputFrame`s to the simulation.

use gilrs::{Axis, Button, Gilrs};
use macroquad::prelude::{get_last_key_pressed, is_key_down, is_key_pressed, KeyCode};
use serde::{Deserialize, Serialize};

use crate::sim::InputFrame;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Action {
    Left,
    Right,
    Up,
    Down,
    Fire,
    Confirm,
    Back,
    Pause,
}

pub const ACTIONS: [Action; 8] = [
    Action::Left,
    Action::Right,
    Action::Up,
    Action::Down,
    Action::Fire,
    Action::Confirm,
    Action::Back,
    Action::Pause,
];

impl Action {
    pub fn label(self) -> &'static str {
        match self {
            Action::Left => "Move left",
            Action::Right => "Move right",
            Action::Up => "Menu up",
            Action::Down => "Menu down",
            Action::Fire => "Fire",
            Action::Confirm => "Confirm",
            Action::Back => "Back",
            Action::Pause => "Pause",
        }
    }
}

/// Keys that can be bound, with their stable names for the save file.
const KEY_NAMES: &[(KeyCode, &str)] = &[
    (KeyCode::Left, "Left"),
    (KeyCode::Right, "Right"),
    (KeyCode::Up, "Up"),
    (KeyCode::Down, "Down"),
    (KeyCode::Space, "Space"),
    (KeyCode::Enter, "Enter"),
    (KeyCode::KpEnter, "KpEnter"),
    (KeyCode::Escape, "Escape"),
    (KeyCode::Backspace, "Backspace"),
    (KeyCode::Tab, "Tab"),
    (KeyCode::LeftShift, "LShift"),
    (KeyCode::RightShift, "RShift"),
    (KeyCode::LeftControl, "LCtrl"),
    (KeyCode::RightControl, "RCtrl"),
    (KeyCode::LeftAlt, "LAlt"),
    (KeyCode::RightAlt, "RAlt"),
    (KeyCode::A, "A"),
    (KeyCode::B, "B"),
    (KeyCode::C, "C"),
    (KeyCode::D, "D"),
    (KeyCode::E, "E"),
    (KeyCode::F, "F"),
    (KeyCode::G, "G"),
    (KeyCode::H, "H"),
    (KeyCode::I, "I"),
    (KeyCode::J, "J"),
    (KeyCode::K, "K"),
    (KeyCode::L, "L"),
    (KeyCode::M, "M"),
    (KeyCode::N, "N"),
    (KeyCode::O, "O"),
    (KeyCode::P, "P"),
    (KeyCode::Q, "Q"),
    (KeyCode::R, "R"),
    (KeyCode::S, "S"),
    (KeyCode::T, "T"),
    (KeyCode::U, "U"),
    (KeyCode::V, "V"),
    (KeyCode::W, "W"),
    (KeyCode::X, "X"),
    (KeyCode::Y, "Y"),
    (KeyCode::Z, "Z"),
    (KeyCode::Key0, "0"),
    (KeyCode::Key1, "1"),
    (KeyCode::Key2, "2"),
    (KeyCode::Key3, "3"),
    (KeyCode::Key4, "4"),
    (KeyCode::Key5, "5"),
    (KeyCode::Key6, "6"),
    (KeyCode::Key7, "7"),
    (KeyCode::Key8, "8"),
    (KeyCode::Key9, "9"),
    (KeyCode::Comma, ","),
    (KeyCode::Period, "."),
    (KeyCode::Slash, "/"),
    (KeyCode::Semicolon, ";"),
];

fn key_from_name(name: &str) -> Option<KeyCode> {
    KEY_NAMES.iter().find(|(_, n)| *n == name).map(|(k, _)| *k)
}

pub fn key_name(key: KeyCode) -> Option<&'static str> {
    KEY_NAMES.iter().find(|(k, _)| *k == key).map(|(_, n)| *n)
}

pub fn button_name(b: Button) -> &'static str {
    match b {
        Button::South => "Pad A",
        Button::East => "Pad B",
        Button::West => "Pad X",
        Button::North => "Pad Y",
        Button::LeftTrigger => "Pad LB",
        Button::RightTrigger => "Pad RB",
        Button::LeftTrigger2 => "Pad LT",
        Button::RightTrigger2 => "Pad RT",
        Button::Start => "Pad Start",
        Button::Select => "Pad Select",
        Button::DPadUp => "Pad Up",
        Button::DPadDown => "Pad Down",
        Button::DPadLeft => "Pad Left",
        Button::DPadRight => "Pad Right",
        _ => "Pad ?",
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Binding {
    pub keys: Vec<String>,
    pub buttons: Vec<Button>,
}

/// One `Binding` per entry of `ACTIONS`, in order.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Bindings(pub Vec<Binding>);

impl Default for Bindings {
    fn default() -> Self {
        let b = |keys: &[&str], buttons: &[Button]| Binding {
            keys: keys.iter().map(|s| s.to_string()).collect(),
            buttons: buttons.to_vec(),
        };
        Bindings(vec![
            b(&["Left", "A"], &[Button::DPadLeft]),
            b(&["Right", "D"], &[Button::DPadRight]),
            b(&["Up", "W"], &[Button::DPadUp]),
            b(&["Down", "S"], &[Button::DPadDown]),
            b(&["Space", "J"], &[Button::South, Button::West]),
            b(&["Enter", "Space"], &[Button::South, Button::Start]),
            b(&["Escape", "Backspace"], &[Button::East]),
            b(&["Escape", "P"], &[Button::Start]),
        ])
    }
}

impl Bindings {
    pub fn is_valid(&self) -> bool {
        self.0.len() == ACTIONS.len()
    }

    pub fn get(&self, a: Action) -> &Binding {
        &self.0[a as usize]
    }

    /// Makes `key` the primary key for `a`.
    pub fn set_key(&mut self, a: Action, key: KeyCode) {
        if let Some(name) = key_name(key) {
            let keys = &mut self.0[a as usize].keys;
            keys.retain(|k| k != name);
            keys.insert(0, name.to_string());
            keys.truncate(2);
        }
    }

    /// Makes `button` the primary gamepad button for `a`.
    pub fn set_button(&mut self, a: Action, button: Button) {
        let buttons = &mut self.0[a as usize].buttons;
        buttons.retain(|b| *b != button);
        buttons.insert(0, button);
        buttons.truncate(2);
    }

    pub fn describe(&self, a: Action) -> String {
        let b = self.get(a);
        let mut parts: Vec<&str> = b.keys.iter().map(String::as_str).collect();
        parts.extend(b.buttons.iter().map(|b| button_name(*b)));
        parts.join(" / ")
    }
}

/// Polls keyboard and gamepads once per frame and answers action queries.
pub struct Controls {
    gilrs: Option<Gilrs>,
    pub bindings: Bindings,
    pad_held: [bool; ACTIONS.len()],
    pad_prev: [bool; ACTIONS.len()],
    /// Most recent gamepad button press this frame (used for rebinding).
    pub last_button: Option<Button>,
    pub last_key: Option<KeyCode>,
}

const STICK_DEADZONE: f32 = 0.5;

impl Controls {
    pub fn new(bindings: Bindings) -> Self {
        let gilrs = match Gilrs::new() {
            Ok(g) => Some(g),
            Err(e) => {
                eprintln!("skypop: gamepad support unavailable: {e}");
                None
            }
        };
        Self {
            gilrs,
            bindings,
            pad_held: [false; ACTIONS.len()],
            pad_prev: [false; ACTIONS.len()],
            last_button: None,
            last_key: None,
        }
    }

    pub fn update(&mut self) {
        self.last_key = get_last_key_pressed();
        self.last_button = None;
        self.pad_prev = self.pad_held;
        self.pad_held = [false; ACTIONS.len()];
        let Some(gilrs) = &mut self.gilrs else { return };
        while let Some(ev) = gilrs.next_event() {
            if let gilrs::EventType::ButtonPressed(b, _) = ev.event {
                self.last_button = Some(b);
            }
        }
        for (_, pad) in gilrs.gamepads() {
            let sx = pad.value(Axis::LeftStickX);
            let sy = pad.value(Axis::LeftStickY);
            for a in ACTIONS {
                let i = a as usize;
                let stick = match a {
                    Action::Left => sx < -STICK_DEADZONE,
                    Action::Right => sx > STICK_DEADZONE,
                    Action::Up => sy > STICK_DEADZONE,
                    Action::Down => sy < -STICK_DEADZONE,
                    _ => false,
                };
                self.pad_held[i] |= stick
                    || self
                        .bindings
                        .get(a)
                        .buttons
                        .iter()
                        .any(|b| pad.is_pressed(*b));
            }
        }
    }

    fn keys(&self, a: Action) -> impl Iterator<Item = KeyCode> + '_ {
        self.bindings
            .get(a)
            .keys
            .iter()
            .filter_map(|k| key_from_name(k))
    }

    pub fn down(&self, a: Action) -> bool {
        self.pad_held[a as usize] || self.keys(a).any(is_key_down)
    }

    pub fn pressed(&self, a: Action) -> bool {
        let i = a as usize;
        (self.pad_held[i] && !self.pad_prev[i]) || self.keys(a).any(is_key_pressed)
    }

    pub fn gameplay_frame(&self) -> InputFrame {
        // A tap that starts and ends within one frame still counts as a shot.
        let fire = self.down(Action::Fire) || self.pressed(Action::Fire);
        InputFrame::new(self.down(Action::Left), self.down(Action::Right), fire)
    }
}

/// Where a player's per-step input comes from. Locally that is the keyboard /
/// gamepad; in LAN co-op (milestone 2) the host wraps frames received from the
/// client in another implementation of this trait.
pub trait InputSource {
    fn sample(&mut self, controls: &Controls) -> InputFrame;
}

pub struct LocalInput;

impl InputSource for LocalInput {
    fn sample(&mut self, controls: &Controls) -> InputFrame {
        controls.gameplay_frame()
    }
}
