use bevy::prelude::*;
use bevy_kira_audio::AudioSource;
use crate::{asset_loading};

pub struct AssetsPlugin;
impl Plugin for AssetsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GameAssets::default());
    }
}

#[derive(Default)]
pub struct GameAssets {
    pub font: Handle<Font>,

    pub blip: Handle<AudioSource>,
    pub bevy_icon: asset_loading::GameTexture,

    pub pickup_handle: Vec<Handle<AudioSource>>,
    pub bite_handle: Vec<Handle<AudioSource>>,
    pub flag_spawn_handle: Handle<AudioSource>,
    pub shock_handle: Handle<AudioSource>,
    pub electricity_handle: Handle<AudioSource>,
    pub land_handle: Handle<AudioSource>,
    pub level_end_handle: Handle<AudioSource>,
    pub slide_handle: Handle<AudioSource>,
    pub fall_handle: Handle<AudioSource>,

    pub bass_drum_handle: Handle<AudioSource>,
    pub bass_drum_reverb_handle: Handle<AudioSource>,
    pub drum_and_bell_handle: Handle<AudioSource>,
    pub level_one_handle: Handle<AudioSource>,
    pub level_one_8bit_handle: Handle<AudioSource>,
    pub halloween_handle: Handle<AudioSource>,
    pub classic_handle: Handle<AudioSource>,
    pub boss_handle: Handle<AudioSource>,
    pub space_handle: Handle<AudioSource>,
    pub hurry_handle: Handle<AudioSource>,
    pub qwerty_handle: Handle<AudioSource>,
    pub credits_handle: Handle<AudioSource>,
    pub intro_handle: Handle<AudioSource>,
    pub organ_handle: Handle<AudioSource>,
    pub tick_tock_handle: Handle<AudioSource>,
}
