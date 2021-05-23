use bevy::prelude::*;
use bevy_kira_audio::{Audio, AudioChannel, AudioPlugin, AudioSource};
use std::collections::HashMap;

pub struct SoundEvent(pub Sounds);

pub struct AudioState {
    channels: HashMap<AudioChannel, ChannelAudioState>,
    sound_channel: AudioChannel,
    pickup_handle: Handle<AudioSource>,
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
    Pickup
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
            pickup_handle: asset_server.load("sounds/pickup.wav")
        }
    }

    pub fn get_sound_handles(&self) -> Vec<HandleUntyped> {
        vec!(
            self.pickup_handle.clone_untyped()
        )
    }

    pub fn play(&mut self, sound: &Sounds, audio: &Res<Audio>) {
        let mut channel_audio_state = self.channels.get_mut(&self.sound_channel).unwrap();
        channel_audio_state.paused = false;
        channel_audio_state.stopped = false;

        let sound_to_play = match sound {
                                Sounds::Pickup => self.pickup_handle.clone(),
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
