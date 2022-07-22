use bevy::{ecs::system::SystemParam, prelude::*};
use bevy_kira_audio::{AudioApp, AudioChannel, AudioPlugin, AudioSource};
use std::marker::PhantomData;
use serde::Deserialize;
use bevy::reflect::TypeUuid;
use crate::{level, assets::GameAssets};

pub struct GameAudioPlugin;
impl Plugin for GameAudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_audio_channel::<MusicChannel>()
            .add_audio_channel::<SoundChannel>()
            .add_audio_channel::<ElectricityChannel>()
            .init_resource::<CurrentMusic>()
            .add_plugin(AudioPlugin);
    }
}

pub struct MusicChannel;
pub struct SoundChannel;
pub struct ElectricityChannel;

#[derive(Default)]
pub struct CurrentMusic {
    current_music: Option::<MusicPiece>,
}

#[derive(SystemParam)]
pub struct GameAudio<'w, 's> {
    music_channel: Res<'w, AudioChannel<MusicChannel>>,
    sound_channel: Res<'w, AudioChannel<SoundChannel>>,
    electricity_channel: Res<'w, AudioChannel<ElectricityChannel>>,

    #[system_param(ignore)]
    phantom: PhantomData<&'s ()>,
}

impl<'w, 's> GameAudio<'w, 's> {
    pub fn play_bgm(&mut self, handle: &Handle<AudioSource>) {
        self.music_channel.stop();
        self.music_channel.set_volume(0.5);
        self.music_channel.play_looped(handle.clone());
    }

    pub fn stop_bgm(&mut self) {
        self.music_channel.stop();
    }

    pub fn play_sfx(&mut self, handle: &Handle<AudioSource>) {
        self.sound_channel.set_volume(0.5);
        self.sound_channel.play(handle.clone());
    }

    pub fn play_electricity(&mut self, handle: &Handle<AudioSource>) {
//      self.electricity_channel.set_volume(0.5);
//      self.electricity_channel.play(handle.clone());
    }
}

#[derive(Debug, Clone, Deserialize, TypeUuid)]
#[uuid = "67fadc56-aa9c-4543-8640-a018b74b5052"]
pub struct LevelMusic {
    before: Vec<MusicPiece>,
    during: Vec<MusicPiece>,
    after: Vec<MusicPiece>,
}

impl LevelMusic {
    pub fn new() -> Self {
        LevelMusic {
            before: vec![],
            during: vec![],
            after: vec![],
        }
    }
}

#[derive(Debug, Copy, Clone, Deserialize, TypeUuid, PartialEq)]
#[uuid = "23badc56-aa9c-4543-8640-a018b74b5052"]
pub enum MusicPiece {
    BassDrum,
    BassDrumReverb,
    DrumAndBell,
    LevelOne,
    LevelOne8Bit,
    Halloween,
    Boss,
    Space,
    Hurry,
    Classic,
    Credits,
    Intro,
    Qwerty,
    Organ,
    TickTock,
}

pub fn stop_ingame_music(
    mut audio: GameAudio,
){
    audio.stop_bgm();
}

pub fn play_ingame_music(
    level: Res<level::Level>,
    game_assets: Res<GameAssets>,
    mut audio: GameAudio,
    mut current_music: ResMut<CurrentMusic>,
    state: Res<State<crate::AppState>>,
) {
    let level_music = level.get_music(true);
    println!("audio: Current level {}", level.current_level);

    let next_music = 
    match state.current() {
        &crate::AppState::LevelTitle => {
            println!("selected before music");
            &level_music.before
        },
        &crate::AppState::ScoreDisplay => {
            println!("selected after music");
            &level_music.after
        },
        _ => {
            println!("selected during music");
            &level_music.during
        }
    };


    if let Some(music) = next_music.get(0) {
        println!("Got music");
        if let Some(current) = current_music.current_music {
            println!("C {:?} M {:?}", current, *music);
            if current == *music {
                return;
            }
        } else {
            println!("no current music playing");
        }

        println!("playing music {:?}", music);
        audio.play_bgm(&get_music(&music, &game_assets));
        current_music.current_music = Some(*music);
    } else {
        println!("Stopping existing music");
        current_music.current_music  = None;
        audio.stop_bgm();
    }
}

fn get_music(music_piece: &MusicPiece, game_assets: &GameAssets) -> Handle<AudioSource> {
    match music_piece {
        MusicPiece::BassDrum => game_assets.bass_drum_handle.clone(),
        MusicPiece::BassDrumReverb => game_assets.bass_drum_reverb_handle.clone(),
        MusicPiece::DrumAndBell => game_assets.drum_and_bell_handle.clone(),
        MusicPiece::LevelOne => game_assets.level_one_handle.clone(),
        MusicPiece::LevelOne8Bit => game_assets.level_one_8bit_handle.clone(),
        MusicPiece::Halloween => game_assets.halloween_handle.clone(),
        MusicPiece::Boss => game_assets.boss_handle.clone(),
        MusicPiece::Space => game_assets.space_handle.clone(),
        MusicPiece::Hurry => game_assets.hurry_handle.clone(),
        MusicPiece::Classic => game_assets.classic_handle.clone(),
        MusicPiece::Credits => game_assets.credits_handle.clone(),
        MusicPiece::Intro => game_assets.intro_handle.clone(),
        MusicPiece::Qwerty => game_assets.qwerty_handle.clone(),
        MusicPiece::Organ => game_assets.organ_handle.clone(),
        MusicPiece::TickTock => game_assets.tick_tock_handle.clone(),
    }
}
