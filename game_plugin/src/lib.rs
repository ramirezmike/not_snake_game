use bevy::{prelude::*,};
use bevy::app::AppExit;
use bevy::app::Events;
//use bevy_prototype_debug_lines::*;
use bevy::reflect::{TypeUuid};
use bevy::window::WindowMode;
use serde::Deserialize;

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
pub mod sounds;
pub mod game_controller;
pub mod pause;
pub mod teleporter;
pub mod dust;
mod menu;
mod editor;

use camera::*;
use environment::*;
use dude::*;

pub static COLOR_BLACK: &str = "000000";
pub const FONT: &str = "fonts/monogram.ttf";

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum AppState {
    MainMenu,
    Loading,
    Editor,
    EditorPlay,
    Pause,
    InGame,
    ScoreDisplay,
    LevelTitle,
    ChangingLevel,
    ResetLevel,
    RestartLevel,
    Credits,
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(DefaultPlugins)
//         .add_plugin(DebugLinesPlugin)
           .add_event::<credits::CreditsEvent>()

//           .add_state(AppState::MainMenu)
//           .add_state(AppState::Editor)
           .add_state(AppState::Loading)
           .add_system_set(SystemSet::on_exit(AppState::Loading)
                                .with_system(fullscreen_app)
           )
           .add_system_set(
               SystemSet::on_enter(AppState::MainMenu)
                         .with_system(environment::load_level.label("loading_level"))
                         .with_system(sounds::play_ingame_music.after("loading_level"))
                         .with_system(menu::setup_menu.after("loading_level"))
                         .with_system(camera::create_camera.after("loading_level"))
                         .with_system(environment::set_clear_color.after("loading_level"))
                         .with_system(environment::load_level_into_path_finder.after("loading_level"))
           )
           .add_system_set(
               SystemSet::on_update(AppState::MainMenu)
                   .with_system(menu::menu)

                   .with_system(holdable::lift_holdable.label("handle_lift_events"))
                   .with_system(holdable::update_held.before("handle_lift_events"))
                   .with_system(moveable::update_moveable.label("handle_moveables"))
                   .with_system(collectable::check_collected)
                   .with_system(snake::update_enemy)
                   .with_system(snake::handle_food_eaten)
                   .with_system(score::handle_food_eaten)
                   .with_system(food::animate_food)
                   .with_system(food::animate_spawn_particles)
                   .with_system(food::update_food)
                   .with_system(food::handle_food_eaten)
                   .with_system(snake::add_body_parts)
                   .with_system(snake::update_following)
                   .with_system(snake::handle_kill_snake)
                   .with_system(path_find::update_graph.label("graph_update"))
                   .with_system(path_find::update_path.after("graph_update"))
                   .with_system(level::broadcast_changes.after("handle_moveables"))
                   .with_system(game_controller::gamepad_connections)
           )
           .add_system_set(
               SystemSet::on_exit(AppState::MainMenu)
                            .with_system(menu::cleanup_menu)
                            .with_system(environment::cleanup_environment)
           )
           .add_system_set(
               SystemSet::on_enter(AppState::Credits)
               .with_system(environment::cleanup_change_level_screen)
               .with_system(sounds::play_credits_music)
               .with_system(credits::setup_credits)
           )
           .add_system_set(SystemSet::on_update(AppState::Credits).with_system(credits::update_credits))
           .add_system_set(SystemSet::on_update(AppState::InGame).with_system(credits::show_credits))
           .add_system_set(SystemSet::on_exit(AppState::InGame).with_system(environment::cleanup_environment))
           .add_plugin(EnvironmentPlugin)
           .add_plugin(DudePlugin)
           .add_plugin(editor::EditorPlugin)

           .init_resource::<level::LevelAssetState>()
           .add_asset::<level::LevelsAsset>()
           .init_asset_loader::<level::LevelsAssetLoader>()
          //.add_startup_system(setup)
          //.add_system(print_on_load)

           .add_system(exit);
    }
}

fn exit(keys: Res<Input<KeyCode>>, mut exit: ResMut<Events<AppExit>>) {
//  if keys.just_pressed(KeyCode::Q) {
//      exit.send(AppExit);
//  }
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
    Up, Down, Left, Right, Beneath, Above
}

#[derive(Copy, Clone, Debug, PartialEq, Deserialize, TypeUuid, Component)]
#[uuid = "93cadc56-aa9c-4543-8640-a018b74b5052"] // this needs to be actually generated
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

pub fn fullscreen_app(
    mut windows: ResMut<Windows>,
) {
    let window = windows.get_primary_mut().unwrap();
    println!("Setting fullscreen...");
//  window.set_maximized(true);
//  window.set_mode(WindowMode::BorderlessFullscreen);
}
