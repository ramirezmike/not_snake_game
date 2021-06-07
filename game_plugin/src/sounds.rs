use bevy::prelude::*;
use bevy_kira_audio::{Audio, AudioChannel, AudioPlugin, AudioSource};
use std::collections::HashMap;

pub struct SoundEvent(pub Sounds);

pub struct AudioState {
    channels: HashMap<AudioChannel, ChannelAudioState>,
    sound_channel: AudioChannel,
    music_channel: AudioChannel,
    electricity_channel: AudioChannel,
    pickup_handle: Handle<AudioSource>,
    bite_handle: Handle<AudioSource>,
    jump_handle: Handle<AudioSource>,
    shock_handle: Handle<AudioSource>,
    electricity_handle: Handle<AudioSource>,
    land_handle: Handle<AudioSource>,
    level_end_handle: Handle<AudioSource>,
    slide_handle: Handle<AudioSource>,
    fall_handle: Handle<AudioSource>,
    music_1_handle: Handle<AudioSource>,
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
        channels.insert(
            AudioChannel::new("third".to_owned()),
            ChannelAudioState::default(),
        );

        AudioState {
            sound_channel,
            music_channel,
            electricity_channel, 
            channels,
            pickup_handle: asset_server.load("sounds/pickup.wav"),
            bite_handle: asset_server.load("sounds/bite.wav"),
            jump_handle: asset_server.load("sounds/jump.wav"),
            land_handle: asset_server.load("sounds/land.wav"),
            shock_handle: asset_server.load("sounds/electric.wav"),
            electricity_handle: asset_server.load("sounds/electricity.wav"),
            level_end_handle: asset_server.load("sounds/levelend.wav"),
            slide_handle: asset_server.load("sounds/slide.wav"),
            fall_handle: asset_server.load("sounds/fall.wav"),
            music_1_handle: asset_server.load("music/drum_and_bell.wav"),
        }
    }

    pub fn get_sound_handles(&self) -> Vec<HandleUntyped> {
        vec!(
            self.pickup_handle.clone_untyped(),
            self.bite_handle.clone_untyped(),
            self.jump_handle.clone_untyped(),
            self.land_handle.clone_untyped(),
            self.shock_handle.clone_untyped(),
            self.electricity_handle.clone_untyped(),
            self.level_end_handle.clone_untyped(),
            self.slide_handle.clone_untyped(),
            self.fall_handle.clone_untyped(),
            self.music_1_handle.clone_untyped(),
        )
    }

    pub fn play(&mut self, sound: &Sounds, audio: &Res<Audio>) {
        let mut channel_audio_state = self.channels.get_mut(&self.sound_channel).unwrap();
        channel_audio_state.paused = false;
        channel_audio_state.stopped = false;

        let sound_to_play = match sound {
                                Sounds::Pickup => self.pickup_handle.clone(),
                                Sounds::Bite => self.bite_handle.clone(),
                                Sounds::Jump => self.jump_handle.clone(),
                                Sounds::Land => self.land_handle.clone(),
                                Sounds::LevelEnd => self.level_end_handle.clone(),
                                Sounds::Slide => self.slide_handle.clone(),
                                Sounds::Shock => self.shock_handle.clone(),
                                Sounds::Fall => self.fall_handle.clone(),
                            };
        audio.play_in_channel(sound_to_play, &self.sound_channel);
    }

    pub fn play_music(&mut self, audio: &Res<Audio>) {
        let mut channel_audio_state = self.channels.get_mut(&self.music_channel).unwrap();
        channel_audio_state.paused = false;
        channel_audio_state.stopped = false;

//        audio.play_looped_in_channel(self.music_1_handle.clone(), &self.music_channel);
    }

    pub fn play_electricity(&mut self, audio: &Res<Audio>) {
        let mut channel_audio_state = self.channels.get_mut(&self.electricity_channel).unwrap();
        channel_audio_state.paused = false;
        channel_audio_state.stopped = false;

        audio.play_looped_in_channel(self.electricity_handle.clone(), &self.electricity_channel);
    }

    pub fn stop_electricity(&mut self, audio: &Res<Audio>) {
        audio.stop_channel(&self.electricity_channel);
    }
}

pub fn play_sounds(
    audio: Res<Audio>,
    mut audio_state: ResMut<AudioState>,
    mut sound_reader: EventReader<SoundEvent>,
    mut playing_music: Local<bool>,
) {
    for sound in sound_reader.iter() {
        audio_state.play(&sound.0, &audio);
    }

    if !*playing_music {
        audio_state.play_music(&audio);
        *playing_music = true;
    }
}
