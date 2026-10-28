//! Generates the game's sound effects as small WAV files (original work,
//! no third-party audio): `cargo run -p tbg-client --example make_sfx`.
//! Output: crates/tbg-client/assets/audio/*.wav (22.05 kHz, mono, 16-bit).

use std::f32::consts::TAU;
use std::fs;
use std::path::Path;

const RATE: f32 = 22_050.0;

struct Noise(u32);

impl Noise {
    fn next(&mut self) -> f32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 17;
        self.0 ^= self.0 << 5;
        (self.0 as f32 / u32::MAX as f32) * 2.0 - 1.0
    }
}

/// Attack/decay envelope over `len` seconds.
fn env(t: f32, len: f32, attack: f32) -> f32 {
    if t < attack {
        t / attack
    } else {
        (1.0 - (t - attack) / (len - attack)).max(0.0).powf(1.6)
    }
}

/// `voice(t) -> sample` rendered for `len` seconds.
fn render(len: f32, mut voice: impl FnMut(f32) -> f32) -> Vec<f32> {
    (0..(len * RATE) as usize)
        .map(|i| voice(i as f32 / RATE))
        .collect()
}

fn tone(freq: impl Fn(f32) -> f32, len: f32, square: bool) -> Vec<f32> {
    let mut phase = 0.0;
    render(len, move |t| {
        phase += freq(t) / RATE;
        let s = (phase * TAU).sin();
        let s = if square { s.signum() * 0.6 } else { s };
        s * env(t, len, 0.005)
    })
}

fn noise_hit(len: f32, low: f32, seed: u32) -> Vec<f32> {
    let mut n = Noise(seed);
    let mut lp = 0.0;
    render(len, move |t| {
        lp += (n.next() - lp) * low;
        lp * 2.5 * env(t, len, 0.002)
    })
}

fn notes(freqs: &[f32], each: f32) -> Vec<f32> {
    freqs
        .iter()
        .flat_map(|f| tone(|_| *f, each, false))
        .collect()
}

fn mix(a: Vec<f32>, b: Vec<f32>) -> Vec<f32> {
    let n = a.len().max(b.len());
    (0..n)
        .map(|i| a.get(i).unwrap_or(&0.0) + b.get(i).unwrap_or(&0.0))
        .collect()
}

fn write(dir: &Path, name: &str, samples: &[f32]) {
    let data: Vec<u8> = samples
        .iter()
        .flat_map(|s| (((s * 0.5).clamp(-1.0, 1.0) * 32_767.0) as i16).to_le_bytes())
        .collect();
    let mut wav = Vec::new();
    wav.extend(b"RIFF");
    wav.extend((36 + data.len() as u32).to_le_bytes());
    wav.extend(b"WAVEfmt ");
    wav.extend(16u32.to_le_bytes());
    wav.extend(1u16.to_le_bytes()); // PCM
    wav.extend(1u16.to_le_bytes()); // mono
    wav.extend((RATE as u32).to_le_bytes());
    wav.extend((RATE as u32 * 2).to_le_bytes());
    wav.extend(2u16.to_le_bytes());
    wav.extend(16u16.to_le_bytes());
    wav.extend(b"data");
    wav.extend((data.len() as u32).to_le_bytes());
    wav.extend(data);
    fs::write(dir.join(format!("{name}.wav")), wav).expect("write wav");
}

fn main() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/audio");
    fs::create_dir_all(&dir).expect("create assets/audio");
    write(
        &dir,
        "deploy",
        &tone(|t| 300.0 + 900.0 * t / 0.18, 0.18, true),
    );
    write(&dir, "melee", &noise_hit(0.12, 0.25, 7));
    write(&dir, "shot", &noise_hit(0.16, 0.6, 11));
    write(
        &dir,
        "cannon",
        &mix(
            noise_hit(0.35, 0.08, 13),
            tone(|t| 90.0 - 40.0 * t, 0.35, false),
        ),
    );
    write(&dir, "heal", &notes(&[660.0, 880.0], 0.1));
    write(&dir, "evolve", &notes(&[523.0, 659.0, 784.0, 1046.0], 0.07));
    write(&dir, "synergy", &notes(&[440.0, 660.0], 0.09));
    write(
        &dir,
        "stronghold_hit",
        &mix(noise_hit(0.25, 0.12, 17), tone(|_| 70.0, 0.25, false)),
    );
    write(
        &dir,
        "victory",
        &notes(&[523.0, 659.0, 784.0, 1046.0, 1046.0], 0.12),
    );
    write(&dir, "defeat", &notes(&[392.0, 330.0, 262.0, 196.0], 0.16));
    write(&dir, "click", &tone(|_| 1200.0, 0.03, true));
    println!("wrote 11 sound effects to {}", dir.display());
}
