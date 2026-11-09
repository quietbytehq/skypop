// SPDX-License-Identifier: Apache-2.0
//! Every sound effect is synthesised at start-up into an in-memory WAV, so the
//! game ships no audio files.

use macroquad::audio::{load_sound_from_bytes, play_sound, PlaySoundParams, Sound};

const RATE: u32 = 22_050;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sfx {
    Pop0,
    Pop1,
    Pop2,
    Pop3,
    Shoot,
    Clink,
    Break,
    Hurt,
    Shield,
    Pickup,
    Freeze,
    BossHit,
    Explosion,
    Clear,
    GameOver,
    Victory,
    MenuMove,
    MenuSelect,
    Warning,
}

const ALL: [Sfx; 19] = [
    Sfx::Pop0,
    Sfx::Pop1,
    Sfx::Pop2,
    Sfx::Pop3,
    Sfx::Shoot,
    Sfx::Clink,
    Sfx::Break,
    Sfx::Hurt,
    Sfx::Shield,
    Sfx::Pickup,
    Sfx::Freeze,
    Sfx::BossHit,
    Sfx::Explosion,
    Sfx::Clear,
    Sfx::GameOver,
    Sfx::Victory,
    Sfx::MenuMove,
    Sfx::MenuSelect,
    Sfx::Warning,
];

pub struct Audio {
    sounds: Vec<Option<Sound>>,
    pub volume: f32,
}

impl Audio {
    pub async fn new(volume: f32) -> Self {
        let mut sounds = Vec::with_capacity(ALL.len());
        for sfx in ALL {
            let wav = encode_wav(&synth(sfx));
            sounds.push(load_sound_from_bytes(&wav).await.ok());
        }
        Self { sounds, volume }
    }

    pub fn play(&self, sfx: Sfx) {
        self.play_vol(sfx, 1.0);
    }

    pub fn play_vol(&self, sfx: Sfx, gain: f32) {
        if self.volume <= 0.0 {
            return;
        }
        if let Some(Some(s)) = self.sounds.get(sfx as usize) {
            play_sound(
                s,
                PlaySoundParams {
                    looped: false,
                    volume: (self.volume * gain).clamp(0.0, 1.0),
                },
            );
        }
    }
}

/// Deterministic noise source for the synthesiser.
struct Noise(u32);

impl Noise {
    fn next(&mut self) -> f32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 17;
        self.0 ^= self.0 << 5;
        (self.0 as f32 / u32::MAX as f32) * 2.0 - 1.0
    }
}

fn buffer(seconds: f32) -> Vec<f32> {
    vec![0.0; (seconds * RATE as f32) as usize]
}

/// Adds a tone sweeping `f0 -> f1` with an exponential decay envelope.
/// `shape`: 0 = sine, 1 = square, 2 = triangle.
#[allow(clippy::too_many_arguments)]
fn tone(buf: &mut [f32], start: f32, len: f32, f0: f32, f1: f32, amp: f32, decay: f32, shape: u8) {
    let s0 = (start * RATE as f32) as usize;
    let n = (len * RATE as f32) as usize;
    let mut phase = 0.0f32;
    for i in 0..n {
        let Some(slot) = buf.get_mut(s0 + i) else {
            break;
        };
        let t = i as f32 / n as f32;
        let f = f0 + (f1 - f0) * t;
        phase = (phase + f / RATE as f32).fract();
        let wave = match shape {
            1 => {
                if phase < 0.5 {
                    0.6
                } else {
                    -0.6
                }
            }
            2 => 4.0 * (phase - 0.5).abs() - 1.0,
            _ => (phase * std::f32::consts::TAU).sin(),
        };
        let attack = (i as f32 / (RATE as f32 * 0.004)).min(1.0);
        *slot += wave * amp * attack * (-decay * t).exp();
    }
}

/// Adds low-passed noise with a decay envelope.
fn noise(buf: &mut [f32], start: f32, len: f32, amp: f32, decay: f32, smooth: f32, seed: u32) {
    let s0 = (start * RATE as f32) as usize;
    let n = (len * RATE as f32) as usize;
    let mut rng = Noise(seed);
    let mut lp = 0.0;
    for i in 0..n {
        let Some(slot) = buf.get_mut(s0 + i) else {
            break;
        };
        let t = i as f32 / n as f32;
        lp += (rng.next() - lp) * smooth;
        *slot += lp * amp * (-decay * t).exp();
    }
}

fn arpeggio(buf: &mut [f32], notes: &[f32], step: f32, len: f32, amp: f32, shape: u8) {
    for (i, f) in notes.iter().enumerate() {
        tone(buf, i as f32 * step, len, *f, *f, amp, 4.0, shape);
        tone(
            buf,
            i as f32 * step,
            len,
            *f * 2.0,
            *f * 2.0,
            amp * 0.25,
            6.0,
            0,
        );
    }
}

fn synth(sfx: Sfx) -> Vec<f32> {
    match sfx {
        Sfx::Pop0 | Sfx::Pop1 | Sfx::Pop2 | Sfx::Pop3 => {
            let k = match sfx {
                Sfx::Pop0 => 0.0,
                Sfx::Pop1 => 1.0,
                Sfx::Pop2 => 2.0,
                _ => 3.0,
            };
            let mut b = buffer(0.28);
            let f = 260.0 * 1.35f32.powf(k);
            tone(&mut b, 0.0, 0.16, f * 2.2, f, 0.55, 5.0, 0);
            noise(&mut b, 0.0, 0.09, 0.5, 7.0, 0.5, 7 + k as u32);
            tone(&mut b, 0.02, 0.22, f * 3.0, f * 3.6, 0.12, 6.0, 2);
            b
        }
        Sfx::Shoot => {
            let mut b = buffer(0.2);
            tone(&mut b, 0.0, 0.18, 380.0, 1400.0, 0.28, 3.0, 1);
            noise(&mut b, 0.0, 0.05, 0.15, 5.0, 0.8, 3);
            b
        }
        Sfx::Clink => {
            let mut b = buffer(0.15);
            tone(&mut b, 0.0, 0.12, 2200.0, 1900.0, 0.25, 8.0, 0);
            b
        }
        Sfx::Break => {
            let mut b = buffer(0.4);
            noise(&mut b, 0.0, 0.35, 0.7, 6.0, 0.9, 11);
            tone(&mut b, 0.0, 0.2, 2600.0, 1800.0, 0.15, 6.0, 2);
            b
        }
        Sfx::Hurt => {
            let mut b = buffer(0.55);
            tone(&mut b, 0.0, 0.5, 420.0, 90.0, 0.5, 3.0, 1);
            noise(&mut b, 0.0, 0.25, 0.5, 5.0, 0.3, 19);
            b
        }
        Sfx::Shield => {
            let mut b = buffer(0.4);
            tone(&mut b, 0.0, 0.35, 900.0, 300.0, 0.35, 4.0, 2);
            noise(&mut b, 0.0, 0.2, 0.3, 6.0, 0.6, 23);
            b
        }
        Sfx::Pickup => {
            let mut b = buffer(0.4);
            arpeggio(&mut b, &[660.0, 880.0, 1320.0], 0.07, 0.18, 0.3, 2);
            b
        }
        Sfx::Freeze => {
            let mut b = buffer(0.7);
            for (i, f) in [1760.0, 2093.0, 2637.0, 3136.0].iter().enumerate() {
                tone(&mut b, i as f32 * 0.05, 0.5, *f, *f * 0.98, 0.14, 4.0, 0);
            }
            b
        }
        Sfx::BossHit => {
            let mut b = buffer(0.25);
            tone(&mut b, 0.0, 0.2, 180.0, 70.0, 0.55, 4.0, 1);
            noise(&mut b, 0.0, 0.1, 0.4, 6.0, 0.7, 29);
            b
        }
        Sfx::Explosion => {
            let mut b = buffer(1.6);
            noise(&mut b, 0.0, 1.5, 0.9, 3.5, 0.08, 31);
            tone(&mut b, 0.0, 1.0, 120.0, 30.0, 0.5, 3.0, 0);
            b
        }
        Sfx::Clear => {
            let mut b = buffer(1.0);
            arpeggio(
                &mut b,
                &[523.0, 659.0, 784.0, 1047.0, 784.0, 1047.0],
                0.1,
                0.3,
                0.28,
                2,
            );
            b
        }
        Sfx::GameOver => {
            let mut b = buffer(1.4);
            arpeggio(&mut b, &[392.0, 349.0, 311.0, 262.0], 0.25, 0.5, 0.3, 2);
            b
        }
        Sfx::Victory => {
            let mut b = buffer(2.2);
            let notes = [523.0, 523.0, 659.0, 784.0, 659.0, 784.0, 1047.0, 1047.0];
            arpeggio(&mut b, &notes, 0.16, 0.45, 0.28, 2);
            tone(&mut b, 1.3, 0.9, 523.0, 523.0, 0.18, 2.0, 0);
            tone(&mut b, 1.3, 0.9, 659.0, 659.0, 0.14, 2.0, 0);
            b
        }
        Sfx::MenuMove => {
            let mut b = buffer(0.08);
            tone(&mut b, 0.0, 0.06, 880.0, 990.0, 0.18, 5.0, 2);
            b
        }
        Sfx::MenuSelect => {
            let mut b = buffer(0.25);
            arpeggio(&mut b, &[784.0, 1175.0], 0.06, 0.15, 0.25, 2);
            b
        }
        Sfx::Warning => {
            let mut b = buffer(1.3);
            for i in 0..3 {
                tone(&mut b, i as f32 * 0.42, 0.35, 440.0, 330.0, 0.3, 1.5, 1);
            }
            b
        }
    }
}

fn encode_wav(samples: &[f32]) -> Vec<u8> {
    let data_len = (samples.len() * 2) as u32;
    let mut out = Vec::with_capacity(44 + data_len as usize);
    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&(36 + data_len).to_le_bytes());
    out.extend_from_slice(b"WAVEfmt ");
    out.extend_from_slice(&16u32.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes()); // PCM
    out.extend_from_slice(&1u16.to_le_bytes()); // mono
    out.extend_from_slice(&RATE.to_le_bytes());
    out.extend_from_slice(&(RATE * 2).to_le_bytes());
    out.extend_from_slice(&2u16.to_le_bytes());
    out.extend_from_slice(&16u16.to_le_bytes());
    out.extend_from_slice(b"data");
    out.extend_from_slice(&data_len.to_le_bytes());
    for s in samples {
        let v = (s.clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
        out.extend_from_slice(&v.to_le_bytes());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_sfx_synthesises_a_valid_wav() {
        for sfx in ALL {
            let s = synth(sfx);
            assert!(!s.is_empty());
            assert!(s.iter().all(|v| v.is_finite()));
            let wav = encode_wav(&s);
            assert_eq!(&wav[0..4], b"RIFF");
            assert_eq!(wav.len(), 44 + s.len() * 2);
        }
    }
}
