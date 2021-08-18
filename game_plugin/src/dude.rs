use bevy::prelude::*;
use crate::{Direction, EntityType, GameObject, level::Level, game_controller, sounds,
            dust, snake, environment, Position, holdable, block, moveable, facing::Facing};
use std::collections::HashMap;

pub struct DudePlugin;
impl Plugin for DudePlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system_set(
               SystemSet::on_update(crate::AppState::InGame)
                   .with_system(player_input.system())
                   .with_system(hop_on_snake.system())
                   .with_system(push_block.system())
           );
    }
}

pub static SCALE: f32 = 0.36;
static SPEED: f32 = 0.1;

pub struct SquashQueue {
    pub squashes: Vec::<Squash>,
}

pub struct Squash {
    pub start_scale: Vec3,
    pub target_scale: Vec3,
    pub start_vertical: f32,
    pub target_vertical: f32,
    pub start_horizontal: f32,
    pub target_horizontal: f32,
    pub current_scale_time: f32,
    pub finish_scale_time: f32,
}

pub struct DudeDiedEvent {
    pub death_type: DudeDeath 
}

#[derive(Copy, Clone)]
pub struct KillDudeEvent {
    pub death_type: DudeDeath 
}

#[derive(Copy, Clone, PartialEq)]
pub enum DudeDeath {
    Fall,
    Eaten,
    Electric
}

#[derive(Default)]
pub struct DudeMeshes {
    pub step1: Handle<Mesh>,
    pub head: Handle<Mesh>,
    pub body: Handle<Mesh>,
    pub material: Handle<StandardMaterial>,
}

pub fn spawn_player(
    commands: &mut Commands, 
    meshes: &ResMut<DudeMeshes>, 
    level: &mut ResMut<Level>,
    x: usize,
    y: usize,
    z: usize,
) {
    let mut transform = Transform::from_translation(Vec3::new(x as f32, y as f32, z as f32));
    transform.apply_non_uniform_scale(Vec3::new(SCALE, SCALE, SCALE)); 
    transform.rotate(Quat::from_axis_angle(Vec3::new(0.0, 1.0, 0.0), std::f32::consts::PI));
    let inner_mesh_vertical_offset = 1.0;
    let player_entity = 
    commands.spawn_bundle(PbrBundle {
                transform,
                ..Default::default()
            })
            .insert(Dude)
            .insert(Position { x: x as i32, y: y as i32, z: z as i32 })
            .insert(EntityType::Dude)
            .insert(crate::camera::CameraTarget)
            .insert(holdable::Holder { holding: None })
            .insert(moveable::Moveable::new(SPEED, inner_mesh_vertical_offset))
            .insert(Facing::new(Direction::Right, false))
            .insert(SquashQueue{ squashes: Vec::new() })
            .with_children(|parent|  {
                parent.spawn_bundle(PbrBundle {
                    mesh: meshes.body.clone(),
                    material: meshes.material.clone(),
                    transform: {
                        let mut transform = Transform::from_rotation(Quat::from_axis_angle(Vec3::new(0.0, 1.0, 0.0), 
                                (3.0 * std::f32::consts::PI) / 2.0));

                        transform.translation = Vec3::new(0.0, inner_mesh_vertical_offset, 0.0);
                        transform
                    },
                    ..Default::default()
                });
            }).id();
    level.set(x as i32, y as i32, z as i32, Some(GameObject::new(player_entity, EntityType::Dude)));
}

pub fn handle_squashes( 
    mut squashes: Query<(&mut SquashQueue, &Children)>,
    mut transforms: Query<&mut Transform>,
    time: Res<Time>,
) {
    for (mut queue, children) in squashes.iter_mut() {
        if let Some(mut squash) = queue.squashes.pop() {
            let child_entity = children.last().expect("dude child mesh missing");
            let mut transform = transforms.get_mut(*child_entity).expect("dude child transform missing");
            if squash.current_scale_time >= squash.finish_scale_time {
                // squash/stretch time is done so make sure we're at target scale 
                transform.scale = squash.target_scale;
                transform.translation.y = squash.target_vertical;
                transform.translation.z = squash.target_horizontal;
            } else {
                // continue squashing/stretching
                squash.current_scale_time += time.delta_seconds();

                let target = squash.target_scale;
                let new_scale = squash.start_scale.lerp(target,
                                                        squash.current_scale_time / squash.finish_scale_time);
                if !new_scale.is_nan() {
                    transform.scale = new_scale;
                }

                let mut target = transform.translation.clone();
                target.y = squash.target_vertical;
                target.z = squash.target_horizontal;

                let mut start_vertical = transform.translation.clone();
                start_vertical.y = squash.start_vertical;
                start_vertical.z = squash.start_horizontal;

                let new_vertical = start_vertical.lerp(target,
                                                        squash.current_scale_time / squash.finish_scale_time);
                if !new_vertical.is_nan() {
                    transform.translation = new_vertical;
                }

                queue.squashes.push(squash);
            }
        }
    }
}

fn hop_on_snake(
    enemies: Query<&snake::Enemy>,
    mut dudes: Query<(&Transform, &mut SquashQueue)>,
) {
    for (transform, mut squash_queue) in dudes.iter_mut() {
        let mut below_dude = transform.translation.clone();
        below_dude.y -= 1.0; 

        for enemy in enemies.iter() {
            if enemy.is_in_vec(below_dude) {
                // make dude hop on snake
                if squash_queue.squashes.is_empty() {
                    // squashes are done in reverse
                    squash_queue.squashes.push(Squash {
                        start_scale: Vec3::new(1.0, 1.0, 1.0),
                        target_scale: Vec3::new(1.0, 1.0, 1.0),
                        start_vertical: 1.8,
                        target_vertical: 1.0,
                        start_horizontal: 0.0,
                        target_horizontal: 0.0,
                        current_scale_time: 0.0,
                        finish_scale_time: 0.20,
                    });
                    squash_queue.squashes.push(Squash {
                        start_scale: Vec3::new(1.0, 1.0, 1.0),
                        target_scale: Vec3::new(1.0, 1.0, 1.0),
                        start_vertical: 1.0,
                        target_vertical: 1.8,
                        start_horizontal: 0.0,
                        target_horizontal: 0.0,
                        current_scale_time: 0.0,
                        finish_scale_time: 0.05,
                    });
                }
            }
        }
    }
}

fn player_input(
    keyboard_input: Res<Input<KeyCode>>,
    time: Res<Time>, 
    mut lift_holdable_event_writer: EventWriter<holdable::LiftHoldableEvent>,
    mut dudes: Query<(Entity, &mut moveable::Moveable, &Transform, &Facing, &mut SquashQueue), With<Dude>>, 
    camera: Query<&crate::camera::fly_camera::FlyCamera>,
    mut kill_dude_event_writer: EventWriter<KillDudeEvent>,
    mut action_buffer: Local<Option::<u128>>,
    mut up_buffer: Local<Option::<u128>>,
    mut down_buffer: Local<Option::<u128>>,
    mut right_buffer: Local<Option::<u128>>,
    mut left_buffer: Local<Option::<u128>>,
    axes: Res<Axis<GamepadAxis>>,
    buttons: Res<Input<GamepadButton>>,
    gamepad: Option<Res<game_controller::GameController>>,
    mut create_dust_event_writer: EventWriter<dust::CreateDustEvent>,
) {
    let time_buffer = 100;
//  if keyboard_input.just_pressed(KeyCode::R) {
//      kill_dude_event_writer.send(KillDudeEvent { death_type: DudeDeath::Eaten });
//  }

    // this is for debugging. If we're flying, don't move the player
    if camera.iter().count() > 0 {
        return;
    }

    let time_since_startup = time.time_since_startup().as_millis();
    if let Some(time_since_up) = *up_buffer {
        if time_since_startup - time_since_up > time_buffer {
            *up_buffer = None;
        }
    }
    if let Some(time_since_down) = *down_buffer {
        if time_since_startup - time_since_down > time_buffer {
            *down_buffer = None;
        }
    }
    if let Some(time_since_left) = *left_buffer {
        if time_since_startup - time_since_left > time_buffer {
            *left_buffer = None;
        }
    }
    if let Some(time_since_right) = *right_buffer {
        if time_since_startup - time_since_right > time_buffer {
            *right_buffer = None;
        }
    }
    if let Some(time_since_action) = *action_buffer {
        if time_since_startup - time_since_action > time_buffer {
            *action_buffer = None;
        }
    }

    let pressed_buttons = game_controller::get_pressed_buttons(&axes, &buttons, gamepad);
    for (entity, mut moveable, transform, facing, mut squash_queue) in dudes.iter_mut() {
        if (keyboard_input.just_pressed(KeyCode::Space) 
        || keyboard_input.just_pressed(KeyCode::Return) 
        || keyboard_input.just_pressed(KeyCode::J) 
        || pressed_buttons.contains(&game_controller::GameButton::Action))
        && !moveable.is_moving() && action_buffer.is_none() {
            lift_holdable_event_writer.send(holdable::LiftHoldableEvent(entity, facing.direction));
            *action_buffer = Some(time.time_since_startup().as_millis());
            continue;
        }

        if !action_buffer.is_none() {
            continue;
        }

        let mut move_dir = None;
        if (keyboard_input.pressed(KeyCode::W) 
         || keyboard_input.pressed(KeyCode::Up) 
         || pressed_buttons.contains(&game_controller::GameButton::Up))
           && up_buffer.is_none() {
            move_dir = Some(Direction::Up); 
            *up_buffer = Some(time.time_since_startup().as_millis());
        }
        if (keyboard_input.pressed(KeyCode::S) 
           || keyboard_input.pressed(KeyCode::Down) 
           || pressed_buttons.contains(&game_controller::GameButton::Down))
           && down_buffer.is_none() {
            move_dir = Some(Direction::Down); 
            *down_buffer = Some(time.time_since_startup().as_millis());
        }
        if (keyboard_input.pressed(KeyCode::A) 
           || keyboard_input.pressed(KeyCode::Left) 
           || pressed_buttons.contains(&game_controller::GameButton::Left))
           && left_buffer.is_none() {
            move_dir = Some(Direction::Left); 
            *left_buffer = Some(time.time_since_startup().as_millis());
        }
        if (keyboard_input.pressed(KeyCode::D) 
           || keyboard_input.pressed(KeyCode::Right) 
           || pressed_buttons.contains(&game_controller::GameButton::Right))
           && right_buffer.is_none() {
            move_dir = Some(Direction::Right); 
            *right_buffer= Some(time.time_since_startup().as_millis());
        }

        if let Some(move_dir) = move_dir {
            let mut movement_got_set = false;
            if !moveable.is_moving() {
                moveable.set_movement(move_dir, moveable::MovementType::Step);
                movement_got_set = true; 
            } else {
                // Commented this code.. it was to let the player "cancel" their movement by moving backward
                // but it doesn't really feel right so leaving it out for now
//              match (moveable.get_current_moving_direction().unwrap(), move_dir) {
//                  (Direction::Up, Direction::Down)    |
//                  (Direction::Down, Direction::Up)    |
//                  (Direction::Left, Direction::Right) |
//                  (Direction::Right, Direction::Left) =>  {
//                      println!("forcing movement!");
//                      moveable.force_movement_change(move_dir, moveable::MovementType::Step);
//                      movement_got_set = true; 
//                  },
//                  _ => ()
//              }
            }

            if movement_got_set {
                squash_queue.squashes.clear();

                // squashes are done in reverse
                squash_queue.squashes.push(Squash {
                    start_scale: Vec3::new(0.7, 1.4, 1.0),
                    target_scale: Vec3::new(1.0, 1.0, 1.0),
                    start_vertical: 2.5,
                    target_vertical: 1.0,
                    start_horizontal: 0.0,
                    target_horizontal: 0.0,
                    current_scale_time: 0.0,
                    finish_scale_time: 0.20,
                });
                squash_queue.squashes.push(Squash {
                    start_scale: Vec3::new(1.0, 1.0, 1.0),
                    target_scale: Vec3::new(0.7, 1.4, 1.0),
                    start_vertical: 1.0,
                    target_vertical: 2.5,
                    start_horizontal: 0.0,
                    target_horizontal: 0.0,
                    current_scale_time: 0.0,
                    finish_scale_time: 0.05,
                });

                create_dust_event_writer.send(dust::CreateDustEvent { 
                    position: Position::from_vec(transform.translation),
                    move_away_from: move_dir,
                });
            }
        }
    }
}

fn push_block(
    keyboard_input: Res<Input<KeyCode>>,
    level: Res<Level>,
    dudes: Query<(&Transform, &Position, &Facing)>, 
    mut blocks: Query<(&block::BlockObject, &mut moveable::Moveable)>,

    mut kill_dude_event_writer: EventWriter<KillDudeEvent>,
) {
    for (_transform, position, facing) in dudes.iter() {
        if position.y == 0 {
            // dude has fallen. TODO This should be refactored when I move
            // the dude-relevant Moveable code into here. This is just 
            // here for now since it's convenient?
            kill_dude_event_writer.send(KillDudeEvent { death_type: DudeDeath::Fall });
        }

        if keyboard_input.just_pressed(KeyCode::K) {
            let (x, y, z) = match facing.direction {
                                Direction::Up => (position.x + 1, position.y, position.z),
                                Direction::Down => (position.x - 1, position.y, position.z),
                                Direction::Right => (position.x, position.y, position.z + 1),
                                Direction::Left => (position.x, position.y, position.z - 1),
                                _ => (position.x, position.y, position.z),
                            };

            if level.is_type(x, y, z, Some(EntityType::Block)) {
                if let Some(block) = level.get(x, y, z) {
                    if let Ok((_block, mut moveable)) = blocks.get_mut(block.entity) {
                        moveable.set_movement(facing.direction, moveable::MovementType::Slide);
                        println!("Pushed block {:?}", moveable);
                    }
                }
            }
        }
    }
}

pub fn handle_kill_dude(
    mut commands: Commands,
    mut dudes: Query<Entity, With<Dude>>,
    mut kill_dude_event_reader: EventReader<KillDudeEvent>,
    mut dude_died_event_writer: EventWriter<DudeDiedEvent>,
    mut sound_writer: EventWriter<sounds::SoundEvent>,
) {
    for event in kill_dude_event_reader.iter() {
        println!("Dude kill event made");
        for entity in dudes.iter_mut() {
            commands.entity(entity).insert(environment::Shrink { });
            if event.death_type == DudeDeath::Electric {
                sound_writer.send(sounds::SoundEvent(sounds::Sounds::Shock));
            }
            if event.death_type != DudeDeath::Eaten {
                commands.entity(entity).remove::<moveable::Moveable>();
                commands.entity(entity).remove::<Dude>();

                dude_died_event_writer.send(DudeDiedEvent { death_type: event.death_type });
            }
        }
    }
}

pub fn handle_snake_escapes(
    mut commands: Commands,
    mut dudes: Query<(Entity, &Position), (With<Dude>, With<environment::Shrink>)>,
    mut timers: Local<HashMap::<Entity, (Position, f32)>>,
    mut dude_died_event_writer: EventWriter<DudeDiedEvent>,
    level: Res<Level>,
    time: Res<Time>,
) {
    for (entity, position) in dudes.iter_mut() {
        if let Some((pos, timer)) = &mut timers.get_mut(&entity) {
            *timer += time.delta_seconds();

            if position != pos {
                // dude moved out of the way yay, remove shrink
                commands.entity(entity).remove::<environment::Shrink>();
                commands.entity(entity).insert(environment::Grow { });
                timers.remove(&entity);
                continue;
            } 

            println!("Timer: {:?}", *timer);
            if *timer > 0.15 {
                println!("KILL DUDE");
                // actually kill dude
                timers.remove(&entity);
                commands.entity(entity).remove::<moveable::Moveable>();
                commands.entity(entity).remove::<Dude>();
                dude_died_event_writer.send(DudeDiedEvent { death_type: DudeDeath::Eaten });
            }
        } else {
            // start tracking the shrinkage 
            timers.insert(entity, (position.clone(), 0.0));
        }
    }
}

pub struct Dude;
