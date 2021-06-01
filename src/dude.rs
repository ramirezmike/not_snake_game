use bevy::prelude::*;
use crate::{Direction, EntityType, GameObject, level::Level, game_controller,
            environment, Position, holdable, block, moveable, facing::Facing};

pub struct DudePlugin;
impl Plugin for DudePlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system_set(
               SystemSet::on_update(crate::AppState::InGame)
                   .with_system(player_input.system())
                   .with_system(push_block.system())
           );
    }
}

pub struct KillDudeEvent {
    pub death_type: DudeDeath 
}

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
    transform.apply_non_uniform_scale(Vec3::new(0.36, 0.36, 0.36)); 
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
            .insert(moveable::Moveable::new(0.1, inner_mesh_vertical_offset))
            .insert(Facing::new(Direction::Right, false))
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

fn player_input(
    keyboard_input: Res<Input<KeyCode>>,
    time: Res<Time>, 
    mut lift_holdable_event_writer: EventWriter<holdable::LiftHoldableEvent>,
    mut dudes: Query<(Entity, &mut moveable::Moveable, &Facing), With<Dude>>, 
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
) {
    let time_buffer = 100;
    if keyboard_input.just_pressed(KeyCode::R) {
        kill_dude_event_writer.send(KillDudeEvent { death_type: DudeDeath::Fall });
    }

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
    for (entity, mut moveable, facing) in dudes.iter_mut() {
        if (keyboard_input.just_pressed(KeyCode::J) || pressed_buttons.contains(&game_controller::GameButton::Action))
           && !moveable.is_moving() && action_buffer.is_none() {
            lift_holdable_event_writer.send(holdable::LiftHoldableEvent(entity, facing.direction));
            *action_buffer = Some(time.time_since_startup().as_millis());
            continue;
        }

        if !action_buffer.is_none() {
            continue;
        }

        let mut move_dir = None;
        if (keyboard_input.pressed(KeyCode::W) || pressed_buttons.contains(&game_controller::GameButton::Up))
           && up_buffer.is_none() {
            move_dir = Some(Direction::Up); 
            *up_buffer = Some(time.time_since_startup().as_millis());
        }
        if (keyboard_input.pressed(KeyCode::S) || pressed_buttons.contains(&game_controller::GameButton::Down))
           && down_buffer.is_none() {
            move_dir = Some(Direction::Down); 
            *down_buffer = Some(time.time_since_startup().as_millis());
        }
        if (keyboard_input.pressed(KeyCode::A) || pressed_buttons.contains(&game_controller::GameButton::Left))
           && left_buffer.is_none() {
            move_dir = Some(Direction::Left); 
            *left_buffer = Some(time.time_since_startup().as_millis());
        }
        if (keyboard_input.pressed(KeyCode::D) || pressed_buttons.contains(&game_controller::GameButton::Right))
           && right_buffer.is_none() {
            move_dir = Some(Direction::Right); 
            *right_buffer= Some(time.time_since_startup().as_millis());
        }

        if let Some(move_dir) = move_dir {
            if !moveable.is_moving()   {
                moveable.set_movement(move_dir, moveable::MovementType::Step);
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
            // here for now since it's convenient
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
) {
    for _event in kill_dude_event_reader.iter() {
        for entity in dudes.iter_mut() {
            commands.entity(entity).remove::<moveable::Moveable>();
            commands.entity(entity).insert(environment::Shrink { });
        }
    }
}

pub struct Dude;
