use bevy::{prelude::*,};
use bevy::app::AppExit;
use bevy::app::Events;

mod camera;
pub mod environment;
mod dude;

use camera::*;
use environment::*;
use dude::*;

pub static COLOR_BASE: &str = "343f56";
pub static COLOR_GROUND_1: &str = "387c6d";
pub static COLOR_GROUND_2: &str = "f8f5f1";
pub static COLOR_BOX: &str = "e9896a";

fn main() {
    App::build()
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(ClearColor(Color::hex(COLOR_BASE).unwrap()))
        .add_plugins(DefaultPlugins)
        .add_plugin(EnvironmentPlugin)
        .add_plugin(DudePlugin)
        .add_startup_system(setup.system())
        .add_system(exit.system())
        .add_plugin(CameraPlugin)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
//  commands.spawn_bundle(PbrBundle {
//      mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
//      material: materials.add(Color::hex(COLOR_BOX).unwrap().into()),
//      transform: Transform::from_xyz(0.0, 0.5, 0.0),
//      ..Default::default()
//  });
    // light
    commands.spawn_bundle(LightBundle {
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..Default::default()
    });
}

fn exit(keys: Res<Input<KeyCode>>, mut exit: ResMut<Events<AppExit>>) {
    if keys.just_pressed(KeyCode::Q) {
        exit.send(AppExit);
    }
}
