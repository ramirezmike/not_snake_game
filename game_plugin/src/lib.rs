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
pub mod hud_pass;
pub mod sounds;
pub mod game_controller;
pub mod pause;
pub mod teleporter;
pub mod dust;
mod menu;

use camera::*;
use environment::*;
use dude::*;

pub static COLOR_BLACK: &str = "000000";

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
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
    Credits,
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_plugins(DefaultPlugins)
//         .add_plugin(DebugLinesPlugin)
           .init_resource::<menu::ButtonMaterials>()
           .add_event::<credits::CreditsEvent>()

//           .add_state(AppState::MainMenu)
           .add_state(AppState::Loading)
           .add_system_set(SystemSet::on_exit(AppState::Loading)
                                .with_system(fullscreen_app.system()))
           .add_system_set(SystemSet::on_enter(AppState::MainMenu))
           .add_system_set(
               SystemSet::on_enter(AppState::MainMenu)
                         .with_system(environment::load_level.system().label("loading_level"))
                         .with_system(sounds::play_ingame_music.system().after("loading_level"))
                         .with_system(menu::setup_menu.system().after("loading_level"))
                         .with_system(camera::create_camera.system().after("loading_level"))
                         .with_system(environment::set_clear_color.system().after("loading_level"))
                         .with_system(environment::load_level_into_path_finder.system().after("loading_level"))
           )
           .add_system_set(
               SystemSet::on_update(AppState::MainMenu)
                   .with_system(menu::menu.system())

                   .with_system(holdable::lift_holdable.system().label("handle_lift_events"))
                   .with_system(holdable::update_held.system().before("handle_lift_events"))
                   .with_system(moveable::update_moveable.system().label("handle_moveables"))
                   .with_system(collectable::check_collected.system())
                   .with_system(snake::update_enemy.system())
                   .with_system(snake::handle_food_eaten.system())
                   .with_system(score::handle_food_eaten.system())
                   .with_system(food::animate_food.system())
                   .with_system(food::animate_spawn_particles.system())
                   .with_system(food::update_food.system())
                   .with_system(food::handle_food_eaten.system())
                   .with_system(snake::add_body_parts.system())
                   .with_system(snake::update_following.system())
                   .with_system(snake::handle_kill_snake.system())
                   .with_system(path_find::update_graph.system().label("graph_update"))
                   .with_system(path_find::update_path.system().after("graph_update"))
                   .with_system(level::broadcast_changes.system().after("handle_moveables"))
                   .with_system(game_controller::gamepad_connections.system())
           )
           .add_system_set(
               SystemSet::on_exit(AppState::MainMenu)
                            .with_system(menu::cleanup_menu.system())
                            .with_system(environment::cleanup_environment.system())
           )
           .add_system_set(
               SystemSet::on_enter(AppState::Credits)
               .with_system(environment::cleanup_change_level_screen.system())
               .with_system(sounds::play_credits_music.system())
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

           .add_system(exit.system());
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

#[derive(Copy, Clone, PartialEq, Debug)]
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

#[derive(Copy, Clone, Debug, PartialEq, Deserialize, TypeUuid)]
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
    window.set_maximized(true);
    window.set_mode(WindowMode::BorderlessFullscreen);
}
