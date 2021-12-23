use bevy::prelude::*;
use serde::Deserialize;
use bevy::reflect::{TypeUuid};
use bevy::pbr::NotShadowCaster;
use crate::{level::Level, dude, environment};
use bevy::render::camera::PerspectiveProjection;

#[derive(Component)]
pub struct CameraTarget;

#[derive(Debug, Clone, Deserialize, TypeUuid)]
#[uuid = "59cadc56-aa9c-4543-8640-a018b74b5052"] // this needs to be actually generated
pub enum CameraBehavior {
    Static,
    LockFollowX(f32, f32),
    LockFollowY(f32, f32, f32),
    FollowY(f32),
    FollowZ(f32),
    LooseFollowX(f32),
    MoveToX(f32),
}

#[derive(Default)]
pub struct CameraMeshes {
    pub bolt: Handle<Mesh>,
    pub spikes: Handle<Mesh>,
}

pub struct CameraPlugin;
impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
               SystemSet::on_update(crate::AppState::InGame)
                         //.with_system(toggle_fly.system())
           )
           .add_system_set(
               SystemSet::on_enter(crate::AppState::InGame)
                         .with_system(reset_camera_on_enter_ingame.system())
           )
           .add_system_set(
               SystemSet::on_update(crate::AppState::MainMenu)
                         //.with_system(toggle_fly.system())
           )
           .add_system(update_camera.system());
    }
}

pub fn reset_camera_on_enter_ingame(
    mut main_camera: Query<&mut MainCamera>,
) {
    for mut camera in main_camera.iter_mut() {
        camera.current_followx_target = None;
        camera.current_followy_target = None;
    }
}

pub fn cull_blocks(
    level: Res<Level>,
    camera: Query<&Transform, With<MainCamera>>,
    mut blocks: Query<(&Transform, &mut Visibility), 
                        (Without<MainCamera>, 
                            Or<(With<environment::BlockMesh>, With<environment::PlatformMesh>)>)>
) {
    let cull_x = level.get_level_cull_x();
    let cull_y = level.get_level_cull_y();
    let cull_z = level.get_level_cull_z();

    for camera_transform in camera.iter() {
        for (block_transform, mut visible) in blocks.iter_mut() {
            let mut is_visible = true;
            if let Some((min_x, max_x)) = cull_x {
                is_visible = block_transform.translation.x < camera_transform.translation.x + max_x 
                                  && block_transform.translation.x > camera_transform.translation.x - min_x;
            } 
            if let Some((min_y, max_y)) = cull_y {
                is_visible = (block_transform.translation.y < camera_transform.translation.y + max_y 
                                  && block_transform.translation.y > camera_transform.translation.y - min_y)
                                // hack to show bottom level blocks which are much lower than 0 if we happen
                                // to be close enough to 0
                            || ((camera_transform.translation.y - min_y) < 0.0 && block_transform.translation.y < 0.0);
            } 
            if let Some((min_z, max_z)) = cull_z {
                is_visible = block_transform.translation.z < camera_transform.translation.z + max_z 
                                  && block_transform.translation.z > camera_transform.translation.z - min_z;
            } 
            visible.is_visible = is_visible;
        }
    }
}

fn lerp(a: f32, b: f32, f: f32) -> f32 {
    return a + f * (b - a);
}
    
fn update_camera(
    mut cameras: Query<(Entity, &mut MainCamera, &mut Transform)>,
    level: Res<Level>,
    target: Query<&Transform, (With<CameraTarget>, Without<MainCamera>)>,
    keyboard_input: Res<Input<KeyCode>>,
    time: Res<Time>,
) {
//  if keyboard_input.just_pressed(KeyCode::P) {
//      for (_e, _camera, transform) in cameras.iter_mut() {
//          let translation = transform.translation;
//          let (rotation, axis) = transform.rotation.to_axis_angle();
//          println!("camera_x: {:?},", translation.x); 
//          println!("camera_y: {:?},", translation.y); 
//          println!("camera_z: {:?},", translation.z); 
//          println!("camera_rotation_x: {:?},", rotation.x); 
//          println!("camera_rotation_y: {:?},", rotation.y); 
//          println!("camera_rotation_z: {:?},", rotation.z); 
//          println!("camera_rotation_angle: {:?},", axis); 
//      }
//  }

    let is_menu = level.current_level == 0;

    for (_, mut main_camera, mut camera_transform) in cameras.iter_mut() {
        if let Ok(target_transform) = target.get_single() {
            for behavior in level.camera_behaviors() {
                match behavior {
                    CameraBehavior::LockFollowX(min, max) => {
                        match &mut main_camera.current_followx_target {
                            Some(movement) => {
                                movement.current_movement_time += time.delta_seconds();

                                let new_translation = lerp(movement.starting_from, movement.target, 
                                                           movement.current_movement_time / movement.finish_movement_time);
                                                     
                                if movement.current_movement_time > movement.finish_movement_time {
                                    camera_transform.translation.x = movement.target;
                                    main_camera.current_followx_target = None;
                                } else if !new_translation.is_nan() {
                                    camera_transform.translation.x = new_translation;
                                }
                            },
                            None => {
                                let x_distance = target_transform.translation.x - camera_transform.translation.x;
                                if x_distance > *max || x_distance < *min {
                                    main_camera.current_followx_target = Some(
                                        CameraMovement {
                                            target: target_transform.translation.x 
                                                    - ((*max - *min) / 2.0)
                                                    - *min,
                                            starting_from: camera_transform.translation.x,
                                            current_movement_time: 0.0,
                                            finish_movement_time: 0.5,
                                        }
                                    );
                                } 
                            }
                        }
                    },
                    CameraBehavior::LockFollowY(min, max, offset) => {
                        match &mut main_camera.current_followy_target {
                            Some(movement) => {
                                movement.current_movement_time += time.delta_seconds();

                                let new_translation = lerp(movement.starting_from, movement.target, 
                                                           movement.current_movement_time / movement.finish_movement_time);
                                                     
                                if movement.current_movement_time > movement.finish_movement_time {
                                    camera_transform.translation.y = movement.target;
                                    main_camera.current_followy_target = None;
                                } else if !new_translation.is_nan() {
                                    camera_transform.translation.y = new_translation;
                                }
                            },
                            None => {
                                let y_distance = target_transform.translation.y - camera_transform.translation.y;
                                if y_distance > *max || y_distance < *min {
                                    main_camera.current_followy_target = Some(
                                        CameraMovement {
                                            target: target_transform.translation.y + offset,
                                            starting_from: camera_transform.translation.y,
                                            current_movement_time: 0.0,
                                            finish_movement_time: 0.5,
                                        }
                                    );
                                } 
                            }
                        }
                    },
                    CameraBehavior::FollowY(offset) => {
                        camera_transform.translation.y += 
                            (target_transform.translation.y - camera_transform.translation.y + offset) 
                           * if is_menu { 0.4 } else { 0.8 } 
                           * time.delta_seconds();
                    },
                    CameraBehavior::LooseFollowX(offset) => {
                        camera_transform.translation.x += 
                            (target_transform.translation.x - camera_transform.translation.x + offset) 
                           * 1.8 
                           * time.delta_seconds();
                    },
                    CameraBehavior::MoveToX(offset) => {
                        camera_transform.translation.x += 
                            (offset - camera_transform.translation.x) 
                           * 1.8 
                           * time.delta_seconds();
                    },
                    CameraBehavior::FollowZ(offset) => {
                        camera_transform.translation.z += 
                            (target_transform.translation.z - camera_transform.translation.z + offset) 
                           * 0.8 
                           * time.delta_seconds();
                    },
                    CameraBehavior::Static => (),
                }
            }
        }
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

#[derive(Component)]
pub struct CameraMouth {
    start: Vec3,
    middle: Vec3,
    end: Vec3,
}

#[derive(Default)]
pub struct CameraBoltMovement {
    moving: bool,
    current_movement_time: f32,
    current_movement_step: MovementStep,  
}

#[derive(Component)]
pub struct CameraBolt {
    start: Vec3,
    middle: Vec3,
    end: Vec3,
}

#[derive(Default)]
pub struct CameraSpikeMovement {
    moving: bool,
    current_movement_time: f32,
    current_movement_step: MovementStep,  
}

#[derive(Component)]
pub struct CameraSpike {
    start: Vec3,
    middle: Vec3,
    end: Vec3,
}

pub fn create_camera(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    camera_meshes: ResMut<CameraMeshes>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut cameras: Query<&mut Transform, With<MainCamera>>,
    level: Res<Level>,
    level_ready: Res<environment::LevelReady>,
) {
    if !level_ready.0 {
        return; // level isn't loaded so we'll try again later
    }

    let mut transform = Transform::default();
    transform.translation = level.get_camera_position();
    transform.rotation = level.get_camera_rotation();

    if let Ok(mut camera_transform) = cameras.get_single_mut() {
        *camera_transform = transform;
    } else {
        println!("Creating camera!");

        let plane = meshes.add(Mesh::from(shape::Cube { size: 1.35 }));
        let mut material: StandardMaterial = Color::hex(level.get_palette().enemy.clone()).unwrap().into();
        material.unlit = true;
        let block_material = materials.add(material);

        let mut material: StandardMaterial = Color::hex("C8C96B").unwrap().into();
        material.unlit = true;
        let electric_material = materials.add(material);

        let mut material: StandardMaterial = Color::hex("000000").unwrap().into();
        material.unlit = true;
        let spike_material = materials.add(material);

        commands
            .spawn_bundle(PerspectiveCameraBundle {
                transform, 
                ..Default::default()
            })
            .with_children(|parent| {
                let distance_from_camera = -1.7;
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
                }).insert(upper_left)
                  .insert(NotShadowCaster);
                parent.spawn_bundle(PbrBundle {
                    mesh: plane.clone(),
                    material: block_material.clone(),
                    transform: Transform::from_translation(upper_right.start),
                    ..Default::default()
                }).insert(upper_right)
                  .insert(NotShadowCaster);
                parent.spawn_bundle(PbrBundle {
                    mesh: plane.clone(),
                    material: block_material.clone(),
                    transform: Transform::from_translation(lower_left.start),
                    ..Default::default()
                }).insert(lower_left)
                  .insert(NotShadowCaster);
                parent.spawn_bundle(PbrBundle {
                    mesh: plane.clone(),
                    material: block_material.clone(),
                    transform: Transform::from_translation(lower_right.start),
                    ..Default::default()
                }).insert(lower_right)
                  .insert(NotShadowCaster);


                // Bolt 
                let distance_from_camera = -5.7;

                let camera_bolt = CameraBolt {
                    start: Vec3::new(0.0, 10.0, distance_from_camera),
                    middle: Vec3::new(0.0, 2.0, distance_from_camera),
                    end: Vec3::new(1.0, 10.0, 20.0),
                };
                parent.spawn_bundle(PbrBundle {
                    mesh: camera_meshes.bolt.clone(),
                    material: electric_material.clone(),
                    transform: {
                        let mut t = Transform::from_translation(camera_bolt.start);
                        t.rotation = Quat::from_axis_angle(Vec3::Y, (3.0 * std::f32::consts::PI) / 2.0);
                        t
                    },
                    ..Default::default()
                }).insert(camera_bolt)
                  .insert(NotShadowCaster);

                // Spike
                let distance_from_camera = -3.0;

                let camera_spike = CameraSpike {
                    start: Vec3::new(0.0, 3.0, distance_from_camera),
                    middle: Vec3::new(0.0, 2.0, distance_from_camera),
                    end: Vec3::new(0.0, -1.0, distance_from_camera),
                };
                parent.spawn_bundle(PbrBundle {
                    mesh: camera_meshes.spikes.clone(),
                    material: spike_material.clone(),
                    transform: {
                        let mut t = Transform::from_translation(camera_spike.start);
                        t.rotation = Quat::from_axis_angle(Vec3::Y, (3.0 * std::f32::consts::PI) / 2.0);
                        t
                    },
                    ..Default::default()
                }).insert(camera_spike)
                  .insert(NotShadowCaster);

                // directional 'sun' light
                const HALF_SIZE: f32 = 100.0;
                parent.spawn_bundle(DirectionalLightBundle {
                    directional_light: DirectionalLight {
                        // Configure the projection to better fit the scene
                        shadow_projection: OrthographicProjection {
                            left: -HALF_SIZE,
                            right: HALF_SIZE,
                            bottom: -HALF_SIZE,
                            top: HALF_SIZE,
                            near: -10.0 * HALF_SIZE,
                            far: 10.0 * HALF_SIZE,
                            ..Default::default()
                        },
                        shadows_enabled: true,
                        ..Default::default()
                    },
                    transform: Transform {
                        rotation: Quat::from_rotation_x(-std::f32::consts::FRAC_PI_4),
                        ..Default::default()
                    },
                    ..Default::default()
                });
            })
            .insert(MainCamera {
                current_followx_target: None,
                current_followy_target: None
            });

    //   let window = windows.get_primary_mut().unwrap();
    //  window.set_cursor_lock_mode(true);
    //  window.set_cursor_visibility(false);

    }
//  // destroy any existing main cameras
//  for camera in cameras.iter() {
//      println!("destroying camera");
//      commands.entity(camera).despawn_recursive();
//  }

}

#[derive(Bundle)]
struct Player { }

pub struct CameraMovement {
    target: f32,
    starting_from: f32,
    current_movement_time: f32,
    finish_movement_time: f32,
}

#[derive(Component)]
pub struct MainCamera {
    pub current_followx_target: Option<CameraMovement>,
    pub current_followy_target: Option<CameraMovement>,
}

static DEFAULT_FOV: f32 = 0.7853982; 

pub fn handle_player_death(
    mut state: ResMut<State<crate::AppState>>,
    time: Res<Time>,
    mut mouth_pieces: Query<(&CameraMouth, &mut Transform), (Without<CameraBolt>, Without<CameraSpike>)>,
    mut bolt_pieces: Query<(&CameraBolt, &mut Transform), (Without<CameraMouth>, Without<CameraSpike>)>,
    mut spike_pieces: Query<(&CameraSpike, &mut Transform), (Without<CameraMouth>, Without<CameraBolt>)>,
    mut dude_died_event_reader: EventReader<dude::DudeDiedEvent>,
    mut mouth_movement: ResMut<CameraMouthMovement>, 
    mut bolt_movement: ResMut<CameraBoltMovement>, 
    mut spike_movement: ResMut<CameraSpikeMovement>, 
    mut perspective: Query<&mut PerspectiveProjection>,
)  {
    if !mouth_movement.moving && !bolt_movement.moving && !spike_movement.moving {
        for kill_dude_event in dude_died_event_reader.iter() {
            match kill_dude_event.death_type {
                dude::DudeDeath::Eaten => mouth_movement.moving = true, 
                dude::DudeDeath::Electric => bolt_movement.moving = true, 
                dude::DudeDeath::Fall => spike_movement.moving = true, 
            }
        }
    }

    if mouth_movement.moving {
        mouth_movement.current_movement_time += time.delta_seconds();
        let mut movement_completed = false;
        for (piece, mut transform) in mouth_pieces.iter_mut() {
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

    if bolt_movement.moving {
        bolt_movement.current_movement_time += time.delta_seconds();
        let mut movement_completed = false;
        let default_scale = Vec3::new(1.0, 1.0, 1.0);
        for (piece, mut transform) in bolt_pieces.iter_mut() {
            let (target, scale, speed) = match bolt_movement.current_movement_step {
                                      MovementStep::Start => (piece.middle, default_scale, 0.5),
                                      MovementStep::Middle => (piece.middle, piece.end, 1.5),
                                      MovementStep::Loading => (piece.middle, piece.end, 0.5),
                                      MovementStep::End => (piece.start, default_scale, 1.0),
                                  };

            if bolt_movement.current_movement_step != MovementStep::Loading {
                let new_translation = transform.translation.lerp(target, 
                                                                 bolt_movement.current_movement_time / speed);
                if !new_translation.is_nan() {
                    if transform.translation.distance(target) < transform.translation.distance(new_translation) {
                        transform.translation = target;
                    } else {
                        transform.translation = new_translation;
                    }
                }

                let new_scale = transform.scale.lerp(scale, bolt_movement.current_movement_time / speed);
                if !new_scale.is_nan() {
                    if transform.scale.distance(scale) < transform.scale.distance(new_scale) {
                        transform.scale = scale;
                    } else {
                        transform.scale = new_scale;
                    }
                }
            }

            if bolt_movement.current_movement_time >= speed {
                movement_completed = true;
            }
        }

        if movement_completed {
            bolt_movement.current_movement_time = 0.0; 
            bolt_movement.current_movement_step = match bolt_movement.current_movement_step  {
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
                                                           bolt_movement.moving = false;    
                                                           MovementStep::Start
                                                       }
                                                   };
        }
    }

    if spike_movement.moving {
        spike_movement.current_movement_time += time.delta_seconds();
        let mut movement_completed = false;
        for (piece, mut transform) in spike_pieces.iter_mut() {
            let (target, speed) = match spike_movement.current_movement_step {
                                      MovementStep::Start => (piece.middle, 0.5),
                                      MovementStep::Middle => (piece.end, 1.5),
                                      MovementStep::Loading => (piece.end, 0.5),
                                      MovementStep::End => (piece.start, 1.0),
                                  };

            if spike_movement.current_movement_step != MovementStep::Loading {
                let new_translation = transform.translation.lerp(target, 
                                                                 spike_movement.current_movement_time / speed);
                if !new_translation.is_nan() {
                    if transform.translation.distance(target) < transform.translation.distance(new_translation) {
                        transform.translation = target;
                    } else {
                        transform.translation = new_translation;
                    }
                }
            }

            if spike_movement.current_movement_time >= speed {
                movement_completed = true;
            }
        }

        if movement_completed {
            spike_movement.current_movement_time = 0.0; 
            spike_movement.current_movement_step = match spike_movement.current_movement_step  {
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
                                                           spike_movement.moving = false;    
                                                           MovementStep::Start
                                                       }
                                                   };
        }
    }
}
