use bevy::app::AppExit;
use bevy::ecs::event::Events;
use bevy::prelude::*;
use bevy::reflect::TypeUuid;
use bevy::window::{PresentMode, WindowMode};
use serde::Deserialize;
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};

mod asset_loading;
mod audio;
mod assets;
mod camera;
mod direction;
mod game_controller;
mod level_over;
mod menus;
mod pause;
mod score;
mod splash;
mod title_screen;
mod ui;

pub mod block;
pub mod collectable;
pub mod credits;
pub mod dude;
pub mod dust;
pub mod environment;
pub mod facing;
pub mod fallable;
pub mod food;
pub mod holdable;
pub mod level;
pub mod moveable;
pub mod path_find;
pub mod snake;
pub mod teleporter;
pub mod win_flag;

use dude::*;
use environment::*;

pub static COLOR_BLACK: &str = "000000";
pub const FONT: &str = "fonts/monogram.ttf";
pub const LAST_LEVEL: usize = 16;
pub const LOST_SCORE_LEVEL: usize = 14;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum AppState {
    MainMenu,
    Loading,
    Pause,
    InGame,
    ScoreDisplay,
    LevelTitle,
    ChangingLevel,
    ResetLevel,
    RestartLevel,
    Splash,
    Credits,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_event::<credits::CreditsEvent>()
        .add_state(AppState::Loading)
        .add_plugin(assets::AssetsPlugin)
        .add_plugin(asset_loading::AssetLoadingPlugin)
        .insert_resource(Msaa { samples: 1 })
        .insert_resource(WindowDescriptor {
            title: "Not Snake".to_string(),
//          width: 1280.0,
//          height: 1024.0,
            resizable: false,
            mode: WindowMode::Windowed,
            present_mode: PresentMode::Fifo,
            ..default()
        })
        .add_plugin(DudePlugin)
        .add_plugin(EnvironmentPlugin)
        .add_plugin(audio::GameAudioPlugin)
        .add_plugin(game_controller::GameControllerPlugin)
        .add_plugin(level_over::LevelOverPlugin)
        .add_plugin(pause::PausePlugin)
        .add_plugin(score::ScorePlugin)
        .add_plugin(splash::SplashPlugin)
        .add_plugin(title_screen::TitlePlugin)
        .add_plugin(ui::text_size::TextSizePlugin)
//      .add_plugin(LogDiagnosticsPlugin::default())
//      .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
//        .add_startup_system(fullscreen_app)
        .add_system_set(
            SystemSet::on_enter(AppState::MainMenu)
                .with_system(environment::try_set_level_from_asset)
                .with_system(environment::load_level.label("loading_level").after(try_set_level_from_asset))
                .with_system(audio::play_ingame_music.after(environment::load_level))
                .with_system(camera::create_camera.after("loading_level"))
                .with_system(environment::set_clear_color.after("loading_level"))
                .with_system(environment::load_level_into_path_finder.after("loading_level")),
        )
        .add_system_set(
            SystemSet::on_update(AppState::MainMenu)
                .with_system(holdable::lift_holdable.label("handle_lift_events"))
                .with_system(holdable::update_held.before("handle_lift_events"))
                .with_system(moveable::update_moveable.label("handle_moveables"))
                .with_system(snake::add_body_to_reach_level_min)
                .with_system(collectable::check_collected)
                .with_system(snake::update_enemy.after(path_find::update_path))
                .with_system(snake::handle_food_eaten)
                .with_system(score::handle_food_eaten)
                .with_system(food::animate_food)
                .with_system(food::animate_spawn_particles)
                .with_system(food::update_food)
                .with_system(food::handle_food_eaten)
                .with_system(snake::add_body_parts)
                .with_system(snake::update_following.after(snake::update_enemy))
                .with_system(snake::handle_kill_snake.after(snake::update_following))
                .with_system(path_find::update_graph.label("graph_update"))
                .with_system(path_find::update_path.after("graph_update"))
                .with_system(level::broadcast_changes.after("handle_moveables"))
        )
        .add_system_set(
            SystemSet::on_exit(AppState::MainMenu)
                .with_system(cleanup::<environment::CleanupMarker>)
                .with_system(audio::stop_ingame_music)
        )
        .add_system_set(
            SystemSet::on_enter(AppState::Credits)
                .with_system(audio::play_ingame_music.after(credits::setup_credits))
                .with_system(credits::setup_credits),
        )
        .add_system_set(
            SystemSet::on_exit(AppState::Credits)
                .with_system(despawn_everything),
        )
        .add_system_set(
            SystemSet::on_update(AppState::Credits).with_system(credits::update_credits),
        )
        .add_system_set(
            SystemSet::on_update(AppState::InGame).with_system(credits::show_credits),
        )
        .add_system_set(
            SystemSet::on_exit(AppState::InGame)
                .with_system(cleanup::<environment::CleanupMarker>)
                .with_system(title_screen::release_all_presses)
                //.with_system(environment::cleanup_environment),
        )
        .init_resource::<level::LevelAssetState>()
        .add_asset::<level::LevelsAsset>()
        .init_asset_loader::<level::LevelsAssetLoader>()
        //.add_startup_system(setup)
        //.add_system(print_on_load)
        .add_system(exit)
        .run();
}

fn exit(keys: Res<Input<KeyCode>>, mut exit: ResMut<Events<AppExit>>) {
    if keys.just_pressed(KeyCode::Q) {
        exit.send(AppExit);
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct GameObject {
    pub entity: Entity,
    pub entity_type: EntityType,
}

impl GameObject {
    pub fn new(entity: Entity, entity_type: EntityType) -> Self {
        GameObject {
            entity,
            entity_type,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Debug, Component)]
pub enum EntityType {
    Block,
    UnstandableBlock,
    Dude,
    Enemy,
    EnemyHead,
    Platform,
    WinFlag,
    Food,
    PathfindIgnore,
}

#[derive(Copy, Clone, Debug, PartialEq, Deserialize, TypeUuid)]
#[uuid = "939adc56-aa9c-4543-8640-a018b74b5052"] // this needs to be actually generated
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
    Beneath,
    Above,
}

#[derive(Copy, Clone, Debug, PartialEq, Deserialize, TypeUuid, Component)]
#[uuid = "93cadc56-aa9c-4543-8640-a018b74b5052"] // this needs to be actually generated
pub struct Position {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}
impl Position {
    pub fn from_vec(v: Vec3) -> Position {
        Position {
            x: v.x as i32,
            y: v.y as i32,
            z: v.z as i32,
        }
    }
    pub fn to_vec(&self) -> Vec3 {
        Vec3::new(self.x as f32, self.y as f32, self.z as f32)
    }
    pub fn update_from_vec(&mut self, v: Vec3) {
        self.x = v.x as i32;
        self.y = v.y as i32;
        self.z = v.z as i32;
    }
    pub fn matches(&self, v: Vec3) -> bool {
        v.x as i32 == self.x && v.y as i32 == self.y && v.z as i32 == self.z
    }
}

pub fn fullscreen_app(mut windows: ResMut<Windows>) {
    let window = windows.get_primary_mut().unwrap();
    println!("Setting fullscreen...");
    window.set_maximized(true);
    window.set_mode(WindowMode::BorderlessFullscreen);
}

pub fn cleanup<T: Component>(mut commands: Commands, entities: Query<Entity, With<T>>) {
    for entity in entities.iter() {
        commands.get_or_spawn(entity).despawn_recursive();
    }
}


pub fn despawn_everything(
    mut commands: Commands,
    entities: Query<Entity>,
) {
    for entity in entities.iter() {
        commands.get_or_spawn(entity).despawn_recursive();
    }
}
