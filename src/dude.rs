use bevy::prelude::*;
use crate::{Direction, EntityType, GameObject, level::Level, 
            Position, holdable, block, moveable, facing::Facing};

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

#[derive(Default)]
pub struct DudeMeshes {
    pub step1: Handle<Mesh>,
    pub material: Handle<StandardMaterial>,
}

pub fn spawn_player(
    commands: &mut Commands, 
    meshes: &Res<DudeMeshes>, 
    level: &mut ResMut<Level>,
    x: usize,
    y: usize,
    z: usize,
) {
    let mut transform = Transform::from_translation(Vec3::new(x as f32, y as f32, z as f32));
    transform.apply_non_uniform_scale(Vec3::new(0.25, 0.25, 0.25)); 
    transform.rotate(Quat::from_axis_angle(Vec3::new(0.0, 1.0, 0.0), std::f32::consts::PI));
    let player_entity = 
    commands.spawn_bundle(PbrBundle {
                transform,
                ..Default::default()
            })
            .insert(Dude {
                action_cooldown: Timer::from_seconds(0.15, false),
            })
            .insert(Position { x: x as i32, y: y as i32, z: z as i32 })
            .insert(EntityType::Dude)
            .insert(holdable::Holder { holding: None })
            .insert(moveable::Moveable::new(true, true, 0.1))
            .insert(Facing::new(Direction::Right, false))
            .with_children(|parent|  {
                parent.spawn_bundle(PbrBundle {
                    mesh: meshes.step1.clone(),
                    material: meshes.material.clone(),
                    transform: {
                        let mut transform = Transform::from_rotation(Quat::from_axis_angle(Vec3::new(0.0, 1.0, 0.0), 1.57079632679));
                        transform.translation = Vec3::new(0.0, 0.5, 0.0);
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
    mut dudes: Query<(Entity, &mut Dude, &mut moveable::Moveable, &Facing)>, 
) {
    for (entity, mut dude, mut moveable, facing) in dudes.iter_mut() {
        dude.action_cooldown.tick(time.delta());
        if !dude.action_cooldown.finished() {
            continue;
        }

        if keyboard_input.just_pressed(KeyCode::J) && !moveable.is_moving() {
            lift_holdable_event_writer.send(holdable::LiftHoldableEvent(entity, facing.direction));
            dude.action_cooldown.reset();
            continue;
        }

        let mut move_dir = None;
        if keyboard_input.pressed(KeyCode::W) {
            move_dir = Some(Direction::Up); 
        }
        if keyboard_input.pressed(KeyCode::S) {
            move_dir = Some(Direction::Down); 
        }
        if keyboard_input.pressed(KeyCode::A) {
            move_dir = Some(Direction::Left); 
        }
        if keyboard_input.pressed(KeyCode::D) {
            move_dir = Some(Direction::Right); 
        }

        if let Some(move_dir) = move_dir {
            if !moveable.is_moving()   {
                moveable.set_movement(move_dir, moveable::MovementType::Step);
            }
            dude.action_cooldown.reset();
        }
    }
}

fn push_block(
    keyboard_input: Res<Input<KeyCode>>,
    level: Res<Level>,
    dudes: Query<(&Transform, &Position, &Facing)>, 
    mut blocks: Query<(&block::BlockObject, &mut moveable::Moveable)>,
) {
    for (_transform, position, facing) in dudes.iter() {
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

pub struct Dude {
    action_cooldown: Timer,
}
