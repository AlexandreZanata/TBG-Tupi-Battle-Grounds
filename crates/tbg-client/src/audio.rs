//! Shared browser/native sound effects and persistent mute/volume preferences.
//! Sounds are original synthesized WAV files from `examples/make_sfx.rs`.

use bevy::prelude::*;

use crate::config;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Sound {
    Deploy,
    Melee,
    Shot,
    Cannon,
    Heal,
    Evolve,
    Synergy,
    StrongholdHit,
    Victory,
    Defeat,
    Click,
}

pub const SOUNDS: [(Sound, &str); 11] = [
    (Sound::Deploy, "deploy"),
    (Sound::Melee, "melee"),
    (Sound::Shot, "shot"),
    (Sound::Cannon, "cannon"),
    (Sound::Heal, "heal"),
    (Sound::Evolve, "evolve"),
    (Sound::Synergy, "synergy"),
    (Sound::StrongholdHit, "stronghold_hit"),
    (Sound::Victory, "victory"),
    (Sound::Defeat, "defeat"),
    (Sound::Click, "click"),
];

/// Minimum seconds between two plays of the same sound (combat spam).
const THROTTLE: f64 = 0.09;

#[derive(Resource, Default)]
pub struct Sounds {
    queue: Vec<Sound>,
    last: Vec<(Sound, f64)>,
    pub muted: bool,
}

impl Sounds {
    pub fn play(&mut self, s: Sound) {
        if !self.muted && !self.queue.contains(&s) {
            self.queue.push(s);
        }
    }

    pub fn toggle_mute(&mut self) {
        self.muted = !self.muted;
        config::save_flag("tbg_muted", self.muted);
    }

    /// Sounds due now, honoring the per-sound throttle.
    fn drain(&mut self, now: f64) -> Vec<Sound> {
        let mut out = Vec::new();
        for s in std::mem::take(&mut self.queue) {
            let recent = self.last.iter().any(|(l, t)| *l == s && now - t < THROTTLE);
            if !recent {
                self.last.retain(|(l, _)| *l != s);
                self.last.push((s, now));
                out.push(s);
            }
        }
        out
    }
}

pub fn plugin(app: &mut App) {
    let muted = config::load_flag("tbg_muted");
    app.insert_resource(Sounds { muted, ..default() });
    let volume =
        crate::platform::storage::get(crate::platform::storage::Scope::Local, "audio.volume")
            .and_then(|v| v.parse::<u8>().ok())
            .unwrap_or(100)
            .min(100);
    app.insert_resource(bevy::audio::GlobalVolume::new(bevy::audio::Volume::Linear(
        f32::from(volume) / 100.,
    )));
    app.add_systems(Startup, playback::load)
        .add_systems(Update, playback::play);
}

mod playback {
    use bevy::prelude::*;

    use super::{Sound, Sounds, SOUNDS};

    #[derive(Resource)]
    pub(super) struct Clips(Vec<(Sound, Handle<AudioSource>)>);

    pub(super) fn load(mut commands: Commands, assets: Res<AssetServer>) {
        let clips = SOUNDS
            .iter()
            .map(|(s, name)| (*s, assets.load(format!("audio/{name}.wav"))))
            .collect();
        commands.insert_resource(Clips(clips));
    }

    pub(super) fn play(
        mut commands: Commands,
        time: Res<Time>,
        mut sounds: ResMut<Sounds>,
        clips: Res<Clips>,
    ) {
        for s in sounds.drain(time.elapsed_secs_f64()) {
            if let Some((_, h)) = clips.0.iter().find(|(c, _)| *c == s) {
                commands.spawn((AudioPlayer::new(h.clone()), PlaybackSettings::DESPAWN));
            }
        }
    }
}
