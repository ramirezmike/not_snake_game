use bevy::prelude::*;
use bevy_kira_audio::{Audio, AudioChannel, AudioPlugin, AudioSource};
use std::collections::HashMap;

pub struct SoundEvent(pub Sounds);

pub struct AudioState {
    channels: HashMap<AudioChannel, ChannelAudioState>,
    sound_channel: AudioChannel,
    pickup_handle: Handle<AudioSource>,
    bite_handle: Handle<AudioSource>,
    jump_handle: Handle<AudioSource>,
    land_handle: Handle<AudioSource>,
    level_end_handle: Handle<AudioSource>,
    slide_handle: Handle<AudioSource>,
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
}

impl AudioState {
    pub fn new(asset_server: &Res<AssetServer>) -> AudioState {
        let mut channels = HashMap::new();
        let sound_channel = AudioChannel::new("first".to_owned());
        channels.insert(
            sound_channel.clone(),
            ChannelAudioState::default(),
        );
        channels.insert(
            AudioChannel::new("second".to_owned()),
            ChannelAudioState::default(),
        );
        channels.insert(
            AudioChannel::new("third".to_owned()),
            ChannelAudioState::default(),
        );

        AudioState {
            sound_channel,
            channels,
            pickup_handle: asset_server.load("sounds/pickup.wav"),
            bite_handle: asset_server.load("sounds/bite.wav"),
            jump_handle: asset_server.load("sounds/jump.wav"),
            land_handle: asset_server.load("sounds/land.wav"),
            level_end_handle: asset_server.load("sounds/levelend.wav"),
            slide_handle: asset_server.load("sounds/slide.wav"),
        }
    }

    pub fn get_sound_handles(&self) -> Vec<HandleUntyped> {
        vec!(
            self.pickup_handle.clone_untyped(),
            self.bite_handle.clone_untyped(),
            self.jump_handle.clone_untyped(),
            self.land_handle.clone_untyped(),
            self.level_end_handle.clone_untyped(),
            self.slide_handle.clone_untyped(),
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
                            };
        audio.play_in_channel(sound_to_play, &self.sound_channel);
    }
}

pub fn play_sounds(
    audio: Res<Audio>,
    mut audio_state: ResMut<AudioState>,
    mut sound_reader: EventReader<SoundEvent>
) {
    for sound in sound_reader.iter() {
        audio_state.play(&sound.0, &audio);
    }
}
