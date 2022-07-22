use crate::{
    block, dust, environment, facing::Facing, game_controller, holdable, level::Level, moveable,
    direction, snake, audio, Direction, EntityType, GameObject, Position, assets::GameAssets,
};
use bevy::prelude::*;
use bevy_utils::Instant;
use std::collections::HashMap;
use leafwing_input_manager::prelude::*;

pub struct DudePlugin;
impl Plugin for DudePlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_update(crate::AppState::InGame)
                .with_system(player_input.after("handle_input"))
                .with_system(
                    handle_controllers
                        .label("handle_input")
                        .after("store_controller_inputs"),
                )
                .with_system(pause_game.after(handle_controllers))
                .with_system(hop_on_snake)
                .with_system(push_block),
        )
        .add_plugin(InputManagerPlugin::<PlayerAction>::default());
    }
}

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug)]
pub enum PlayerAction {
    Up,
    Down,
    Left,
    Right,

    ActionUp,
    ActionDown,
    ActionLeft,
    ActionRight,

    Pause,
    Debug1,
    Debug2,
}

impl PlayerAction {
    const DIRECTIONS: [Self; 4] = [
        PlayerAction::Up,
        PlayerAction::Down,
        PlayerAction::Left,
        PlayerAction::Right,
    ];

    fn direction(self) -> direction::Direction {
        match self {
            PlayerAction::Up => direction::Direction::UP,
            PlayerAction::Down => direction::Direction::DOWN,
            PlayerAction::Left => direction::Direction::LEFT,
            PlayerAction::Right => direction::Direction::RIGHT,
            _ => direction::Direction::NEUTRAL,
        }
    }
}


pub static SCALE: f32 = 0.36;
static SPEED: f32 = 0.1;

#[derive(Component)]
pub struct SquashQueue {
    pub squashes: Vec<Squash>,
}

#[derive(Component)]
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
    pub death_type: DudeDeath,
}

#[derive(Copy, Clone)]
pub struct KillDudeEvent {
    pub death_type: DudeDeath,
}

#[derive(Copy, Clone, PartialEq)]
pub enum DudeDeath {
    Fall,
    Eaten,
    Electric,
}

#[derive(Default)]
pub struct DudeMeshes {
    pub step1: Handle<Mesh>,
    pub head: Handle<Mesh>,
    pub body: Handle<Mesh>,
    pub not_snake: Handle<Mesh>,
    pub material: Handle<StandardMaterial>,
}

pub fn spawn_player<T: Component>(
    commands: &mut Commands,
    meshes: &ResMut<DudeMeshes>,
    level: &mut ResMut<Level>,
    x: usize,
    y: usize,
    z: usize,
    cleanup_marker: T
) {
    let player_entity = create_not_snake(commands, meshes, x as isize, y as isize, z as isize, cleanup_marker);
    level.set(
        x as i32,
        y as i32,
        z as i32,
        Some(GameObject::new(player_entity, EntityType::Dude)),
    );
}

pub fn create_not_snake<T: Component>(
    commands: &mut Commands,
    meshes: &ResMut<DudeMeshes>,
    x: isize,
    y: isize,
    z: isize,
    cleanup_marker: T
) -> Entity {
    let mut transform = Transform::from_translation(Vec3::new(x as f32, y as f32, z as f32));
    transform.apply_non_uniform_scale(Vec3::new(SCALE, SCALE, SCALE));
    transform.rotate(Quat::from_axis_angle(Vec3::new(0.0, 1.0, 0.0), std::f32::consts::PI));

    let inner_mesh_vertical_offset = 0.0;
    commands
        .spawn_bundle(PbrBundle {
            transform,
            ..Default::default()
        })
        .insert(Dude)
        .insert(cleanup_marker)
        .insert(Position {
            x: transform.translation.x as i32,
            y: transform.translation.y as i32,
            z: transform.translation.z as i32,
        })
        .insert_bundle(InputManagerBundle {
            input_map: default_input_map(),
            action_state: ActionState::default(),
        })
        .insert(EntityType::Dude)
        .insert(crate::camera::CameraTarget)
        .insert(holdable::Holder { holding: None })
        .insert(moveable::Moveable::new(SPEED, inner_mesh_vertical_offset))
        .insert(Facing::new(Direction::Right, false))
        .insert(SquashQueue {
            squashes: Vec::new(),
        })
        .with_children(|parent| {
            parent.spawn_bundle(PbrBundle {
                mesh: meshes.body.clone(),
                material: meshes.material.clone(),
                ..Default::default()
            });
        })
        .id()
}

pub fn handle_squashes(
    mut squashes: Query<(&mut SquashQueue, &Children)>,
    mut transforms: Query<&mut Transform>,
    time: Res<Time>,
) {
    for (mut queue, children) in squashes.iter_mut() {
        if let Some(mut squash) = queue.squashes.pop() {
            let child_entity = children.last().expect("dude child mesh missing");
            let mut transform = transforms
                .get_mut(*child_entity)
                .expect("dude child transform missing");
            if squash.current_scale_time >= squash.finish_scale_time {
                // squash/stretch time is done so make sure we're at target scale
                transform.scale = squash.target_scale;
                transform.translation.y = squash.target_vertical;
                transform.translation.z = squash.target_horizontal;
            } else {
                // continue squashing/stretching
                squash.current_scale_time += time.delta_seconds();

                let target = squash.target_scale;
                let new_scale = squash
                    .start_scale
                    .lerp(target, squash.current_scale_time / squash.finish_scale_time);
                if !new_scale.is_nan() {
                    transform.scale = new_scale;
                }

                let mut target = transform.translation.clone();
                target.y = squash.target_vertical;
                target.z = squash.target_horizontal;

                let mut start_vertical = transform.translation.clone();
                start_vertical.y = squash.start_vertical;
                start_vertical.z = squash.start_horizontal;

                let new_vertical = start_vertical
                    .lerp(target, squash.current_scale_time / squash.finish_scale_time);
                if !new_vertical.is_nan() {
                    transform.translation = new_vertical;
                }

                queue.squashes.push(squash);
            }
        }
    }
}

fn hop_on_snake(enemies: Query<&snake::Enemy>, mut dudes: Query<(&Transform, &mut SquashQueue)>) {
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

fn pause_game(
    mut state: ResMut<State<crate::AppState>>,
    mut controllers: ResMut<game_controller::GameController>,
    mut action_state: Query<&mut ActionState<PlayerAction>>,
) {
    let mut action_state = action_state.single_mut();

    if action_state.just_pressed(PlayerAction::Pause) {
        println!("pushing to pause");
        state.push(crate::AppState::Pause).unwrap();

        let now = Instant::now();
        action_state.tick(now);
        controllers.clear_presses();
    }
}

fn handle_controllers(
    controllers: Res<game_controller::GameController>,
    mut players: Query<(Entity, &mut ActionState<PlayerAction>), With<Dude>>,
) {
    for (_, mut action_state) in players.iter_mut() {
        for (_, pressed) in controllers.pressed.iter() {
            if pressed.contains(&game_controller::GameButton::Left) {
                action_state.release(PlayerAction::Left);
                action_state.press(PlayerAction::Left);
            }
            if pressed.contains(&game_controller::GameButton::Right) {
                action_state.release(PlayerAction::Right);
                action_state.press(PlayerAction::Right);
            }
            if pressed.contains(&game_controller::GameButton::Up) {
                action_state.release(PlayerAction::Up);
                action_state.press(PlayerAction::Up);
            }
            if pressed.contains(&game_controller::GameButton::Down) {
                action_state.release(PlayerAction::Down);
                action_state.press(PlayerAction::Down);
            }
        }

        for (_, just_pressed) in controllers.just_pressed.iter() {
            if just_pressed.contains(&game_controller::GameButton::ActionUp) {
                action_state.release(PlayerAction::ActionUp);
                action_state.press(PlayerAction::ActionUp);
            }
            if just_pressed.contains(&game_controller::GameButton::ActionDown) {
                action_state.release(PlayerAction::ActionDown);
                action_state.press(PlayerAction::ActionDown);
            }
            if just_pressed.contains(&game_controller::GameButton::ActionRight) {
                action_state.release(PlayerAction::ActionRight);
                action_state.press(PlayerAction::ActionRight);
            }
            if just_pressed.contains(&game_controller::GameButton::ActionLeft) {
                action_state.release(PlayerAction::ActionLeft);
                action_state.press(PlayerAction::ActionLeft);
            }
            if just_pressed.contains(&game_controller::GameButton::Start) {
                action_state.release(PlayerAction::Pause);
                action_state.press(PlayerAction::Pause);
            }
        }
    }
}

fn default_input_map() -> InputMap<PlayerAction> {
    use PlayerAction::*;
    let mut input_map = InputMap::default();

    input_map.set_gamepad(Gamepad(0));

    input_map.insert(Pause, KeyCode::Escape);

    // Movement
    input_map.insert(Up, KeyCode::Up);
    input_map.insert(Up, KeyCode::W);
    input_map.insert(Up, GamepadButtonType::DPadUp);

    input_map.insert(Down, KeyCode::Down);
    input_map.insert(Down, KeyCode::S);
    input_map.insert(Down, GamepadButtonType::DPadDown);

    input_map.insert(Left, KeyCode::Left);
    input_map.insert(Left, KeyCode::A);
    input_map.insert(Left, GamepadButtonType::DPadLeft);

    input_map.insert(Right, KeyCode::Right);
    input_map.insert(Right, KeyCode::D);
    input_map.insert(Right, GamepadButtonType::DPadRight);

    // Actions
    input_map.insert(ActionUp, KeyCode::I);
    input_map.insert(ActionUp, GamepadButtonType::North);

    input_map.insert(ActionDown, KeyCode::K);
    input_map.insert(ActionDown, GamepadButtonType::South);

    input_map.insert(ActionLeft, KeyCode::J);
    input_map.insert(ActionLeft, GamepadButtonType::West);

    input_map.insert(ActionRight, KeyCode::L);
    input_map.insert(ActionRight, GamepadButtonType::East);

//  input_map.insert(Debug1, KeyCode::R);
//  input_map.insert(Debug2, KeyCode::F);

    input_map
}
 

fn player_input(
    action_state: Query<&ActionState<PlayerAction>>,
    time: Res<Time>,
    mut lift_holdable_event_writer: EventWriter<holdable::LiftHoldableEvent>,
    mut dudes: Query<
        (
            Entity,
            &mut moveable::Moveable,
            &Transform,
            &Facing,
            &mut SquashQueue,
        ),
        With<Dude>,
    >,
    // camera: Query<&crate::camera::fly_camera::FlyCamera>,
    mut kill_dude_event_writer: EventWriter<KillDudeEvent>,
    mut level_over_event_writer: EventWriter<crate::level_over::LevelOverEvent>,

    mut action_buffer: Local<Option<u128>>,
    mut up_buffer: Local<Option<u128>>,
    mut down_buffer: Local<Option<u128>>,
    mut right_buffer: Local<Option<u128>>,
    mut left_buffer: Local<Option<u128>>,
    mut create_dust_event_writer: EventWriter<dust::CreateDustEvent>,
    mut state: ResMut<State<crate::AppState>>,
) {
    let time_buffer = 100;
    //  // this is for debugging. If we're flying, don't move the player
    //  if camera.iter().count() > 0 {
    //      return;
    //  }

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

    let action_state = action_state.single();

    if action_state.just_pressed(PlayerAction::Debug1) {
        kill_dude_event_writer.send(KillDudeEvent { death_type: DudeDeath::Eaten });
    }
    if action_state.just_pressed(PlayerAction::Debug2) {
        level_over_event_writer.send(crate::level_over::LevelOverEvent {});
    }

    for (entity, mut moveable, transform, facing, mut squash_queue) in dudes.iter_mut() {
        if action_state.just_pressed(PlayerAction::ActionDown)
           && !moveable.is_moving()
            && action_buffer.is_none()
        {
            lift_holdable_event_writer.send(holdable::LiftHoldableEvent(entity, facing.direction));
            *action_buffer = Some(time.time_since_startup().as_millis());
            continue;
        }

        if !action_buffer.is_none() {
            continue;
        }

        let mut move_dir = None;
        if action_state.pressed(PlayerAction::Up) && up_buffer.is_none() {
            move_dir = Some(Direction::Up);
            *up_buffer = Some(time.time_since_startup().as_millis());
        }
        if action_state.pressed(PlayerAction::Down) && down_buffer.is_none() {
            move_dir = Some(Direction::Down);
            *down_buffer = Some(time.time_since_startup().as_millis());
        }
        if action_state.pressed(PlayerAction::Left) && left_buffer.is_none() {
            move_dir = Some(Direction::Left);
            *left_buffer = Some(time.time_since_startup().as_millis());
        }
        if action_state.pressed(PlayerAction::Right) && right_buffer.is_none() {
            move_dir = Some(Direction::Right);
            *right_buffer = Some(time.time_since_startup().as_millis());
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
                    start_vertical: 1.5,
                    target_vertical: 0.0,
                    start_horizontal: 0.0,
                    target_horizontal: 0.0,
                    current_scale_time: 0.0,
                    finish_scale_time: 0.20,
                });
                squash_queue.squashes.push(Squash {
                    start_scale: Vec3::new(1.0, 1.0, 1.0),
                    target_scale: Vec3::new(0.7, 1.4, 1.0),
                    start_vertical: 0.0,
                    target_vertical: 1.5,
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
            kill_dude_event_writer.send(KillDudeEvent {
                death_type: DudeDeath::Fall,
            });
        }

//      if keyboard_input.just_pressed(KeyCode::K) {
//          let (x, y, z) = match facing.direction {
//              Direction::Up => (position.x + 1, position.y, position.z),
//              Direction::Down => (position.x - 1, position.y, position.z),
//              Direction::Right => (position.x, position.y, position.z + 1),
//              Direction::Left => (position.x, position.y, position.z - 1),
//              _ => (position.x, position.y, position.z),
//          };

//          if level.is_type(x, y, z, Some(EntityType::Block)) {
//              if let Some(block) = level.get(x, y, z) {
//                  if let Ok((_block, mut moveable)) = blocks.get_mut(block.entity) {
//                      moveable.set_movement(facing.direction, moveable::MovementType::Slide);
//                      println!("Pushed block {:?}", moveable);
//                  }
//              }
//          }
//      }
    }
}

pub fn handle_kill_dude(
    mut commands: Commands,
    mut dudes: Query<Entity, With<Dude>>,
    mut kill_dude_event_reader: EventReader<KillDudeEvent>,
    mut dude_died_event_writer: EventWriter<DudeDiedEvent>,
    game_assets: Res<GameAssets>,
    mut audio: audio::GameAudio,
) {
    for event in kill_dude_event_reader.iter() {
        println!("Dude kill event made");
        for entity in dudes.iter_mut() {
            commands.entity(entity).insert(environment::Shrink {});
            if event.death_type == DudeDeath::Electric {
                audio.play_sfx(&game_assets.shock_handle);
            }
            if event.death_type != DudeDeath::Eaten {
                commands.entity(entity).remove::<moveable::Moveable>();
                commands.entity(entity).remove::<Dude>();

                dude_died_event_writer.send(DudeDiedEvent {
                    death_type: event.death_type,
                });
            }
        }
    }
}

pub fn handle_snake_escapes(
    mut commands: Commands,
    mut dudes: Query<(Entity, &Position), (With<Dude>, With<environment::Shrink>)>,
    mut timers: Local<HashMap<Entity, (Position, f32)>>,
    mut dude_died_event_writer: EventWriter<DudeDiedEvent>,
    time: Res<Time>,
) {
    for (entity, position) in dudes.iter_mut() {
        if let Some((pos, timer)) = &mut timers.get_mut(&entity) {
            *timer += time.delta_seconds();

            if position != pos {
                // dude moved out of the way yay, remove shrink
                commands.entity(entity).remove::<environment::Shrink>();
                commands.entity(entity).insert(environment::Grow {});
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
                dude_died_event_writer.send(DudeDiedEvent {
                    death_type: DudeDeath::Eaten,
                });
            }
        } else {
            // start tracking the shrinkage
            timers.insert(entity, (position.clone(), 0.0));
        }
    }
}

#[derive(Component)]
pub struct Dude;
