use bevy::{prelude::*,};
use bevy::app::AppExit;
use bevy::app::Events;
use bevy_prototype_debug_lines::*;
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};

mod camera;
pub mod environment;
pub mod level;
pub mod holdable;
pub mod fallable;
pub mod moveable;
pub mod facing;
pub mod dude;
pub mod snake;
pub mod level_over;
pub mod credits;
pub mod block;
pub mod collectable;
pub mod win_flag;
pub mod food;
pub mod score;
pub mod path_find;
pub mod hud_pass;
pub mod sounds;
pub mod game_controller;
mod menu;

use camera::*;
use environment::*;
use dude::*;

pub static COLOR_BLACK: &str = "000000";

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum AppState {
    MainMenu,
    Loading,
    InGame,
    ChangingLevel,
    ResetLevel,
    Credits,
}

fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_plugin(DebugLinesPlugin)
        .insert_resource(Msaa { samples: 1 })
        .insert_resource(ClearColor(Color::hex(COLOR_BLACK).unwrap()))
        .init_resource::<menu::ButtonMaterials>()
        .add_event::<credits::CreditsEvent>()

//       .add_state(AppState::MainMenu)
//        .add_state(AppState::InGame)
        .add_state(AppState::Loading)

        .add_system_set(SystemSet::on_enter(AppState::MainMenu).with_system(menu::setup_menu.system()))
        .add_system_set(SystemSet::on_update(AppState::MainMenu).with_system(menu::menu.system()))
        .add_system_set(SystemSet::on_exit(AppState::MainMenu).with_system(menu::cleanup_menu.system()))
        .add_system_set(
            SystemSet::on_enter(AppState::Credits)
            .with_system(environment::cleanup_change_level_screen.system())
            .with_system(credits::setup_credits.system())
        )
        .add_system_set(SystemSet::on_update(AppState::Credits).with_system(credits::update_credits.system()))
        .add_system_set(SystemSet::on_update(AppState::InGame).with_system(credits::show_credits.system()))
        .add_system_set(SystemSet::on_exit(AppState::InGame).with_system(environment::cleanup_environment.system()))
        .add_plugin(EnvironmentPlugin)
        .add_plugin(DudePlugin)

        .init_resource::<level::LevelAssetState>()
        .add_asset::<level::LevelsAsset>()
        .init_asset_loader::<level::LevelsAssetLoader>()
      //.add_startup_system(setup.system())
      //.add_system(print_on_load.system())

        .add_system(exit.system())
//      .add_plugin(LogDiagnosticsPlugin::default())
//      .add_plugin(FrameTimeDiagnosticsPlugin::default())
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
    pub entity_type: EntityType
}

impl GameObject {
    pub fn new(entity: Entity, entity_type: EntityType) -> Self {
        GameObject { entity, entity_type }
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum EntityType {
    Block,
    Dude,
    Enemy,
    EnemyHead,
    Platform,
    WinFlag,
    Food,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Direction {
    Up, Down, Left, Right, Beneath, Above
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Position { pub x: i32, pub y: i32, pub z: i32 }
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
