use bevy::prelude::*;
use bevy_kira_audio::{Audio, AudioChannel, AudioPlugin, AudioSource};
use std::collections::HashMap;
use serde::Deserialize;
use bevy::reflect::{TypeUuid};
use crate::{level, dude, snake};

pub struct SoundEvent(pub Sounds);
pub struct CollectSounds {
    snake: usize,
    dude: usize,
}

impl CollectSounds {
    pub fn new() -> Self {
        CollectSounds {
            snake: 0,
            dude: 0,
        }
    }
}

#[derive(Debug, Clone, Deserialize, TypeUuid)]
#[uuid = "23badc56-aa9c-4543-8640-a018b74b5052"]
pub enum MusicPiece {
    BassDrum,
    BassDrumReverb,
    DrumAndBell,
    LevelOne,
}

#[derive(Debug, Clone, Deserialize, TypeUuid)]
#[uuid = "67fadc56-aa9c-4543-8640-a018b74b5052"]
pub struct LevelMusic {
    before: Vec::<MusicPiece>,
    during: Vec::<MusicPiece>,
    after: Vec::<MusicPiece>,
}

impl LevelMusic {
    pub fn new() -> Self {
        LevelMusic {
            before: vec!(),
            during: vec!(),
            after: vec!(),
        }
    }
}

pub struct AudioState {
    channels: HashMap<AudioChannel, ChannelAudioState>,
    sound_channel: AudioChannel,
    music_channel: AudioChannel,

    electricity_channel: AudioChannel,
    pickup_handle: Vec::<Handle<AudioSource>>,
    bite_handle: Vec::<Handle<AudioSource>>,
    flag_spawn_handle: Handle<AudioSource>,
    jump_handle: Handle<AudioSource>,
    shock_handle: Handle<AudioSource>,
    electricity_handle: Handle<AudioSource>,
    land_handle: Handle<AudioSource>,
    level_end_handle: Handle<AudioSource>,
    slide_handle: Handle<AudioSource>,
    fall_handle: Handle<AudioSource>,

    bass_drum_handle: Handle<AudioSource>,
    bass_drum_reverb_handle: Handle<AudioSource>,
    drum_and_bell_handle: Handle<AudioSource>,
    level_one_handle: Handle<AudioSource>,
}

struct ChannelAudioState {
    stopped: bool,
    paused: bool,
    loop_started: bool,
    volume: f32,
}

impl Default for ChannelAudioState {
    fn default() -> Self {
        ChannelAudioState {
            volume: 1.0,
            stopped: true,
            loop_started: false,
            paused: false,
        }
    }
}

pub enum Sounds {
    Pickup,
    Bite,
    FlagSpawn,
    Jump,
    Land,
    LevelEnd,
    Slide,
    Fall,
    Shock,
}

impl AudioState {
    pub fn new(asset_server: &Res<AssetServer>) -> AudioState {
        let mut channels = HashMap::new();
        let sound_channel = AudioChannel::new("first".to_owned());
        let music_channel = AudioChannel::new("music".to_owned());
        let electricity_channel = AudioChannel::new("electricity".to_owned());

        channels.insert(
            sound_channel.clone(),
            ChannelAudioState::default(),
        );
        channels.insert(
            music_channel.clone(),
            ChannelAudioState::default(),
        );
        channels.insert(
            electricity_channel.clone(), 
            ChannelAudioState::default(),
        );

        AudioState {
            sound_channel,
            music_channel,
            electricity_channel, 
            channels,
            pickup_handle: vec!(
                asset_server.load("sounds/pickup0.wav"),
                asset_server.load("sounds/pickup1.wav"),
                asset_server.load("sounds/pickup2.wav"),
                asset_server.load("sounds/pickup3.wav"),
                asset_server.load("sounds/pickup4.wav"),
            ),
            bite_handle: vec!(
                asset_server.load("sounds/bite0.wav"),
                asset_server.load("sounds/bite1.wav"),
                asset_server.load("sounds/bite2.wav"),
                asset_server.load("sounds/bite3.wav"),
            ),
            flag_spawn_handle: asset_server.load("sounds/flagspawn.wav"),
            jump_handle: asset_server.load("sounds/jump.wav"),
            land_handle: asset_server.load("sounds/land.wav"),
            shock_handle: asset_server.load("sounds/electric.wav"),
            electricity_handle: asset_server.load("sounds/electricity.wav"),
            level_end_handle: asset_server.load("sounds/levelend.wav"),
            slide_handle: asset_server.load("sounds/slide.wav"),
            fall_handle: asset_server.load("sounds/fall.wav"),

            bass_drum_handle: asset_server.load("music/bassdrum.wav"),
            bass_drum_reverb_handle: asset_server.load("music/bassdrum_reverb.wav"),
            drum_and_bell_handle: asset_server.load("music/drum_and_bell.wav"),
            level_one_handle: asset_server.load("music/01.wav"),
        }
    }

    pub fn get_sound_handles(&self) -> Vec<HandleUntyped> {
        let mut sounds = 
            vec!(
                self.flag_spawn_handle.clone_untyped(),
                self.jump_handle.clone_untyped(),
                self.land_handle.clone_untyped(),
                self.shock_handle.clone_untyped(),
                self.electricity_handle.clone_untyped(),
                self.level_end_handle.clone_untyped(),
                self.slide_handle.clone_untyped(),
                self.fall_handle.clone_untyped(),

                self.bass_drum_handle.clone_untyped(),
                self.bass_drum_reverb_handle.clone_untyped(),
                self.drum_and_bell_handle.clone_untyped(),
                self.level_one_handle.clone_untyped(),
            );

        self.pickup_handle.iter().for_each(|s| {
            sounds.push(s.clone_untyped());
        });
        self.bite_handle.iter().for_each(|s| {
            sounds.push(s.clone_untyped());
        });

        sounds
    }

    pub fn play(&mut self, 
        sound: &Sounds, 
        audio: &Res<Audio>, 
        collect_sounds_tracker: &mut ResMut<CollectSounds>,
    ) {
        let mut channel_audio_state = self.channels.get_mut(&self.sound_channel).unwrap();
        channel_audio_state.paused = false;
        channel_audio_state.stopped = false;

        let sound_to_play = match sound {
                                Sounds::Pickup => {
                                    let sound = self.pickup_handle[collect_sounds_tracker.dude % 5].clone();
                                    collect_sounds_tracker.dude += 1;
                                    sound
                                },
                                Sounds::Bite => {
                                    let sound = self.bite_handle[collect_sounds_tracker.snake % 4].clone();
                                    collect_sounds_tracker.snake += 1;
                                    sound
                                },
                                Sounds::FlagSpawn => self.flag_spawn_handle.clone(),
                                Sounds::Jump => self.jump_handle.clone(),
                                Sounds::Land => self.land_handle.clone(),
                                Sounds::LevelEnd => self.level_end_handle.clone(),
                                Sounds::Slide => self.slide_handle.clone(),
                                Sounds::Shock => self.shock_handle.clone(),
                                Sounds::Fall => self.fall_handle.clone(),
                            };
        audio.play_in_channel(sound_to_play, &self.sound_channel);
    }

    pub fn start_music_channels(&mut self, audio: &Res<Audio>) {
        AudioState::start_music_channel(&mut self.channels, audio, &self.bass_drum_handle, &self.music_channel);
    }

    fn start_music_channel(
        channels: &mut HashMap<AudioChannel, ChannelAudioState>,
        audio: &Res<Audio>, 
        handle: &Handle<AudioSource>, 
        channel: &AudioChannel
    ) {
        let mut channel_audio_state = channels.get_mut(channel).unwrap();
        channel_audio_state.paused = false;
        channel_audio_state.stopped = false;

        audio.set_volume_in_channel(0.0, channel);
        audio.play_looped_in_channel(handle.clone(), channel);
    }

//  pub fn play_music(&mut self, audio: &Res<Audio>, musics: Vec::<MusicPiece>) {
//      audio.set_volume_in_channel(1.0, channel);
//  }

    pub fn play_electricity(&mut self, audio: &Res<Audio>) {
        let mut channel_audio_state = self.channels.get_mut(&self.electricity_channel).unwrap();
        channel_audio_state.paused = false;
        channel_audio_state.stopped = false;

        audio.play_looped_in_channel(self.electricity_handle.clone(), &self.electricity_channel);
    }

    pub fn play_fanfare_in_channel(
        &self,
        audio: &Res<Audio>, 
        music: &MusicPiece,
        channel: &AudioChannel
    ) {
        let handle = 
            match music {
                MusicPiece::BassDrum => Some(&self.bass_drum_handle),
                _ => None
            };

        if let Some(handle) = handle {
            println!("Playing a fanfare..");
            audio.play_in_channel(handle.clone(), channel);
        }
    }

    pub fn play_music_in_channel(
        &self,
        audio: &Res<Audio>, 
        music: &MusicPiece,
        channel: &AudioChannel
    ) {
        let handle = 
            match music {
                MusicPiece::BassDrum => Some(&self.bass_drum_handle),
                MusicPiece::BassDrumReverb => Some(&self.bass_drum_reverb_handle),
                MusicPiece::DrumAndBell => Some(&self.drum_and_bell_handle),
                MusicPiece::LevelOne => Some(&self.level_one_handle),
                _ => None
            };

        if let Some(handle) = handle {
            println!("Playing a music..");
            audio.play_looped_in_channel(handle.clone(), channel);
        }
    }
}

pub fn play_sounds(
    audio: Res<Audio>,
    mut audio_state: ResMut<AudioState>,
    mut sound_reader: EventReader<SoundEvent>,
    mut collect_sounds_tracker: ResMut<CollectSounds>,
) {
    for sound in sound_reader.iter() {
        audio_state.play(&sound.0, &audio, &mut collect_sounds_tracker);
    }
}

pub fn play_ingame_music (
    level: Res<level::Level>,
    audio: Res<Audio>,
    audio_state: Res<AudioState>,
) {
    println!("Stopping existing music");
    audio.stop_channel(&audio_state.music_channel);

    let level_music = level.get_music(true);
    if let Some(music) = level_music.during.get(0) {
        println!("playing during music");
        audio_state.play_music_in_channel(&audio, &music, &audio_state.music_channel);
    } 
}

pub fn play_fanfare (
    level: Res<level::Level>,
    audio: Res<Audio>,
    audio_state: Res<AudioState>,
) {
    let level_music = level.get_music(false);
    if let Some(music) = level_music.during.get(1) {
        println!("playing during music");
        audio_state.play_fanfare_in_channel(&audio, &music, &audio_state.music_channel);
    } 
}

pub fn play_before_music (
    level: Res<level::Level>,
    audio: Res<Audio>,
    audio_state: Res<AudioState>,
) {
    println!("Stopping existing music");
    audio.stop_channel(&audio_state.music_channel);

    let level_music = level.get_music(false);
    if let Some(music) = level_music.before.get(0) {
        println!("playing before music");
        audio_state.play_music_in_channel(&audio, &music, &audio_state.music_channel);
    } 
}

pub fn play_after_music (
    level: Res<level::Level>,
    audio: Res<Audio>,
    audio_state: Res<AudioState>,
) {
    println!("Stopping existing music");
    audio.stop_channel(&audio_state.music_channel);

    let level_music = level.get_music(true);
    if let Some(music) = level_music.after.get(0) {
        println!("playing after music");
        audio_state.play_music_in_channel(&audio, &music, &audio_state.music_channel);
    }
}

pub fn pause_music (
    level: Res<level::Level>,
    audio: Res<Audio>,
    audio_state: Res<AudioState>,
) {
    let level_music = level.get_music(true);
    if let Some(_) = level_music.during.get(0) {
        audio.pause_channel(&audio_state.music_channel);
    } 
}

pub fn unpause_music (
    level: Res<level::Level>,
    audio: Res<Audio>,
    audio_state: Res<AudioState>,
) {
    let level_music = level.get_music(true);
    if let Some(_) = level_music.during.get(0) {
        audio.resume_channel(&audio_state.music_channel);
    } 
}

pub fn stop_electricity(
    audio_state: Res<AudioState>,
    audio: Res<Audio>
) {
    audio.stop_channel(&audio_state.electricity_channel);
}

pub fn adjust_electricity_volume(
    audio_state: Res<AudioState>,
    audio: Res<Audio>,
    enemies: Query<(&Transform, &snake::Enemy)>,
    dudes: Query<&Transform, With<dude::Dude>>,
) {
    audio.set_volume_in_channel(0.0, &audio_state.electricity_channel);
    for dude_transform in dudes.iter() {
        for (enemy_transform, enemy) in enemies.iter() {
            if enemy.is_electric {
                let distance = dude_transform.translation.distance(enemy_transform.translation);

                if distance < 1.5 {
                    audio.set_volume_in_channel(0.75, &audio_state.electricity_channel);
                } else if distance < 3.5 {
                    audio.set_volume_in_channel(0.6, &audio_state.electricity_channel);
                } else if distance < 5.5 {
                    audio.set_volume_in_channel(0.4, &audio_state.electricity_channel);
                } else {
                    audio.set_volume_in_channel(0.2, &audio_state.electricity_channel);
                }
            }
        }
    }
}


// this kinda isn't needed anymore but leaving it to just set the volume
pub fn set_level_music(
    audio: Res<Audio>,
    audio_state: Res<AudioState>,
) {
    audio.set_volume_in_channel(1.0, &audio_state.music_channel);
}

pub fn reset_sounds(
    mut collect_sounds_tracker: ResMut<CollectSounds>,
) {
    collect_sounds_tracker.dude = 0;
    collect_sounds_tracker.snake = 0;
}
