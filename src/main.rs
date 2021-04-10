use bevy::{prelude::*,};
use bevy::app::AppExit;
use bevy::app::Events;
use bevy_prototype_debug_lines::*;

mod camera;
pub mod environment;
pub mod level;
pub mod holdable;
pub mod fallable;
pub mod moveable;
pub mod facing;
pub mod dude;
pub mod level_over;
pub mod credits;
pub mod block;
pub mod collectable;
pub mod win_flag;
pub mod path_find;
mod menu;

use camera::*;
use environment::*;
use dude::*;

// 01
//  pub static COLOR_BLACK: &str = "000000";
//  pub static COLOR_BASE: &str = "343f56";//"";
//  pub static COLOR_GROUND_1: &str = "387c6d";
//  pub static COLOR_GROUND_2: &str = "214A41";//"f8f5f1";
//  pub static COLOR_DUDE: &str = "f8f5f1";
//  pub static COLOR_BLOCK: &str = "e9896a";
//  pub static COLOR_FLAG: &str = "80E895"; //"92DB56"; //40DBB7

// 02
pub static COLOR_BLACK: &str = "000000";
pub static COLOR_BASE: &str = "142d4c";//"";
pub static COLOR_GROUND_1: &str = "142d4c";
pub static COLOR_GROUND_2: &str = "385170";//"f8f5f1";
pub static COLOR_DUDE: &str = "ececec";
pub static COLOR_BLOCK: &str = "9fd3c7";
pub static COLOR_FLAG: &str = "7A5C57"; //"92DB56"; //40DBB7

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum AppState {
    MainMenu,
    Loading,
    InGame,
    Credits,
}

fn main() {
    App::build()
        .add_plugins(DefaultPlugins)
        .add_plugin(DebugLinesPlugin)
        .insert_resource(Msaa { samples: 1 })
        .insert_resource(ClearColor(Color::hex(COLOR_BASE).unwrap()))
        .init_resource::<menu::ButtonMaterials>()
        .add_event::<credits::CreditsEvent>()
//        .add_state(AppState::MainMenu)
//        .add_state(AppState::InGame)
        .add_state(AppState::Loading)
        .add_system_set(SystemSet::on_enter(AppState::MainMenu).with_system(menu::setup_menu.system()))
        .add_system_set(SystemSet::on_update(AppState::MainMenu).with_system(menu::menu.system()))
        .add_system_set(SystemSet::on_exit(AppState::MainMenu).with_system(menu::cleanup_menu.system()))
        .add_system_set(SystemSet::on_enter(AppState::Credits).with_system(credits::setup_credits.system()))
        .add_system_set(SystemSet::on_update(AppState::Credits).with_system(credits::update_credits.system()))
        .add_system_set(SystemSet::on_update(AppState::InGame).with_system(credits::show_credits.system()))
        .add_system_set(SystemSet::on_exit(AppState::InGame).with_system(environment::cleanup_environment.system()))
        .add_plugin(EnvironmentPlugin)
        .add_plugin(DudePlugin)
        .add_system(exit.system())
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
    Platform,
    WinFlag,
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
