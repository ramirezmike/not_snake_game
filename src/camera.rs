use bevy::prelude::*;
//  use bevy_mod_picking::*;
//  use bevy_rapier3d::rapier::geometry::ColliderBuilder;
//  use bevy_rapier3d::rapier::dynamics::{RigidBodyBuilder,RigidBodySet};
//  use bevy_rapier3d::physics::RigidBodyHandleComponent;
//  use bevy_rapier3d::rapier::math::Isometry;
use crate::{level::Level, dude};

pub mod fly_camera;

use fly_camera::*;

pub struct CameraPlugin;
impl Plugin for CameraPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app//.add_plugin(PickingPlugin)

           .add_system_set(
               SystemSet::on_exit(crate::AppState::Loading)
                         .with_system(create_camera.system())
           )
           .add_system_set(
               SystemSet::on_enter(crate::AppState::ChangingLevel)
                         .with_system(destroy_camera.system())
           )
           .add_system_set(
               SystemSet::on_exit(crate::AppState::ChangingLevel)
                         .with_system(create_camera.system())
           )
           .add_system_set(
               SystemSet::on_update(crate::AppState::InGame)
                         .with_system(toggle_fly.system())
           )
           .add_plugin(FlyCameraPlugin)
           
//           .add_system(update_camera.system())
           //.add_system(update_camera_collisions.system());
           ;
    }
}
    
fn update_camera(cameras: Query<(Entity, &Camera, &Transform)>) {
    for (_e, _camera, transform) in cameras.iter() {
        println!("Position: {:?} Rotation: {:?} {:?}", transform.translation, transform.rotation.to_axis_angle(), transform.rotation);
    }
}

fn destroy_camera(
    mut commands: Commands,
    cameras: Query<Entity, With<Camera>>
) {
    for camera in cameras.iter() {
        commands.entity(camera).despawn_recursive();
    }
}

#[derive(Debug, PartialEq)]
pub enum MovementStep { Start, Middle, Loading, End }
impl Default for MovementStep {
    fn default() -> Self { MovementStep::Start }
}

#[derive(Default)]
pub struct CameraMouthMovement {
    moving: bool,
    current_movement_time: f32,
    current_movement_step: MovementStep,  
}

pub struct CameraMouth {
    start: Vec3,
    middle: Vec3,
    end: Vec3,
}

fn create_camera(
    mut commands: Commands,
    mut windows: ResMut<Windows>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    level: Res<Level>,
) {
    println!("Creating camera!");
    let mut transform = Transform::default();
    transform.translation = level.get_camera_position();
    transform.rotation = level.get_camera_rotation();

    let plane = meshes.add(Mesh::from(shape::Cube { size: 1.0 }));
    let mut material:StandardMaterial = Color::hex(crate::COLOR_ENEMY).unwrap().into();
    material.unlit = true;
    let block_material = materials.add(material);
    commands
        .spawn_bundle(PerspectiveCameraBundle {
            transform, 
            ..Default::default()
        })
        .with_children(|parent| {
            let distance_from_camera = -1.5;
            let distance_from_each_other = 1.0;
            let start_buffer = 5.00;
            let end_buffer = 0.45;
            let upper_left = CameraMouth {
                start: Vec3::new(-distance_from_each_other * start_buffer, distance_from_each_other * start_buffer, distance_from_camera),
                middle: Vec3::new(-distance_from_each_other, distance_from_each_other, distance_from_camera),
                end: Vec3::new(-distance_from_each_other * end_buffer, distance_from_each_other * end_buffer, distance_from_camera),
            };
            let upper_right = CameraMouth {
                start: Vec3::new(distance_from_each_other * start_buffer, distance_from_each_other * start_buffer, distance_from_camera),
                middle: Vec3::new(distance_from_each_other, distance_from_each_other, distance_from_camera),
                end: Vec3::new(distance_from_each_other * end_buffer, distance_from_each_other * end_buffer, distance_from_camera),
            };
            let lower_left = CameraMouth {
                start: Vec3::new(-distance_from_each_other * start_buffer, -distance_from_each_other * start_buffer, distance_from_camera),
                middle: Vec3::new(-distance_from_each_other, -distance_from_each_other, distance_from_camera),
                end: Vec3::new(-distance_from_each_other * end_buffer, -distance_from_each_other * end_buffer, distance_from_camera),
            };
            let lower_right = CameraMouth {
                start: Vec3::new(distance_from_each_other * start_buffer, -distance_from_each_other * start_buffer, distance_from_camera),
                middle: Vec3::new(distance_from_each_other, -distance_from_each_other, distance_from_camera),
                end: Vec3::new(distance_from_each_other * end_buffer, -distance_from_each_other * end_buffer, distance_from_camera),
            };
            parent.spawn_bundle(PbrBundle {
                mesh: plane.clone(),
                material: block_material.clone(),
                transform: Transform::from_translation(upper_left.start),
                ..Default::default()
            }).insert(upper_left);
            parent.spawn_bundle(PbrBundle {
                mesh: plane.clone(),
                material: block_material.clone(),
                transform: Transform::from_translation(upper_right.start),
                ..Default::default()
            }).insert(upper_right);
            parent.spawn_bundle(PbrBundle {
                mesh: plane.clone(),
                material: block_material.clone(),
                transform: Transform::from_translation(lower_left.start),
                ..Default::default()
            }).insert(lower_left);
            parent.spawn_bundle(PbrBundle {
                mesh: plane.clone(),
                material: block_material.clone(),
                transform: Transform::from_translation(lower_right.start),
                ..Default::default()
            }).insert(lower_right);
        })
        .insert(Camera)
 //       .with(PickSource::default());
            ;

    let window = windows.get_primary_mut().unwrap();
    window.set_cursor_lock_mode(true);
    window.set_cursor_visibility(false);
}

#[derive(Bundle)]
struct Player { }

pub struct Camera;

pub fn handle_player_death(
    mut state: ResMut<State<crate::AppState>>,
    time: Res<Time>,
    mut mouth_pieces: Query<(&CameraMouth, &mut Transform)>,
    mut kill_dude_event_reader: EventReader<dude::KillDudeEvent>,
    mut mouth_movement: ResMut<CameraMouthMovement>, 
) {
    if !mouth_movement.moving {
        mouth_movement.moving = kill_dude_event_reader.iter().count() > 0;
    }

    if mouth_movement.moving {
        mouth_movement.current_movement_time += time.delta_seconds();
        let mut movement_completed = false;
        for (piece, mut transform) in  mouth_pieces.iter_mut() {
            let (target, speed) = match mouth_movement.current_movement_step {
                                      MovementStep::Start => (piece.middle, 0.5),
                                      MovementStep::Middle => (piece.end, 1.5),
                                      MovementStep::Loading => (piece.end, 0.5),
                                      MovementStep::End => (piece.start, 1.0),
                                  };

            if mouth_movement.current_movement_step != MovementStep::Loading {
                let new_translation = transform.translation.lerp(target, 
                                                                 mouth_movement.current_movement_time / speed);
                if !new_translation.is_nan() {
                    if transform.translation.distance(target) < transform.translation.distance(new_translation) {
                        transform.translation = target;
                    } else {
                        transform.translation = new_translation;
                    }
                }
            }

            if mouth_movement.current_movement_time >= speed {
                movement_completed = true;
            }
        }

        if movement_completed {
            mouth_movement.current_movement_time = 0.0; 
            mouth_movement.current_movement_step = match mouth_movement.current_movement_step  {
                                                       MovementStep::Start => MovementStep::Middle,
                                                       MovementStep::Middle => {
                                                           state.set(crate::AppState::ResetLevel).unwrap();
                                                           MovementStep::Loading
                                                       },
                                                       MovementStep::Loading => {
                                                           state.set(crate::AppState::InGame).unwrap();
                                                           MovementStep::End
                                                       }
                                                       MovementStep::End => {
                                                           mouth_movement.moving = false;    
                                                           MovementStep::Start
                                                       }
                                                   };
        }
    }
}

fn toggle_fly(
    mut commands: Commands, 
    keys: Res<Input<KeyCode>>, 
    mut windows: ResMut<Windows>,
    mut camera: Query<(Entity, &mut Camera, Option<&FlyCamera>, &mut Transform)>,
    mut cooldown: Local<f32>,
    timer: Res<Time>,
) {
    *cooldown += timer.delta_seconds();

    if *cooldown < 2.0 {
        return;
    }

    if keys.just_pressed(KeyCode::F) {
        println!("PRESSED F");
        let window = windows.get_primary_mut().unwrap();
        for (e, _, f, mut t) in camera.iter_mut() {
            match f {
                Some(_) => {
                    commands.entity(e).remove::<FlyCamera>();
                    window.set_cursor_lock_mode(false);
                    window.set_cursor_visibility(true);
                },
                None => {
                    let mut fly_camera = FlyCamera::default();
                    fly_camera.key_forward = KeyCode::W; 
                    fly_camera.key_backward = KeyCode::S; 
                    fly_camera.key_left = KeyCode::A; 
                    fly_camera.key_right = KeyCode::D; 
                    commands.entity(e).insert(fly_camera);
                    t.translation = Vec3::new(-6.867214, 5.8081317, 5.4974184);
                    t.rotation = Quat::from_xyzw(-0.14680715, -0.6914177, -0.14668213, 0.692007);
                    window.set_cursor_lock_mode(true);
                    window.set_cursor_visibility(false);
                },
            }
        }

        *cooldown = 0.0;
    }
}
