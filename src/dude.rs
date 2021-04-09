use bevy::prelude::*;
use crate::{Direction, EntityType, GameObject, level::Level, path_find::PathFinder,
            Position, holdable, block, moveable, facing::Facing};

pub struct DudePlugin;
impl Plugin for DudePlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system_set(
               SystemSet::on_enter(crate::AppState::InGame)
                   .with_system(spawn_player.system())
                   .with_system(spawn_enemy.system())
           )
           .add_system_set(
               SystemSet::on_update(crate::AppState::InGame)
                   .with_system(player_input.system())
                   .with_system(update_enemy.system())
                   .with_system(push_block.system())
           );
    }
}

#[derive(Default)]
pub struct DudeMeshes {
    pub step1: Handle<Mesh>,
    pub material: Handle<StandardMaterial>,
}

pub struct Enemy {
    action_cooldown: Timer, 
}
fn spawn_enemy(
    mut commands: Commands, 
    meshes: Res<DudeMeshes>, 
    mut level: ResMut<Level>,
) {
    let position = Vec3::new(0.0, 0.0, 11.0);
    let mut transform = Transform::from_translation(position);
    transform.apply_non_uniform_scale(Vec3::new(0.25, 0.25, 0.25)); 
    transform.rotate(Quat::from_axis_angle(Vec3::new(0.0, 1.0, 0.0), std::f32::consts::PI));
    let enemy_entity = 
    commands.spawn_bundle(PbrBundle {
                transform,
                ..Default::default()
            })
            .insert(Position { x: position.x as i32, y: position.y as i32, z: position.z as i32 })
            .insert(EntityType::Enemy)
            .insert(Enemy {
                action_cooldown: Timer::from_seconds(1.0, true),
            })
            .insert(holdable::Holder { holding: None })
            .insert(moveable::Moveable::new(true, true, 0.1))
            .insert(Facing::new(Direction::Left))
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
    level.set(position.x as i32, position.y as i32, position.z as i32, Some(GameObject::new(enemy_entity, EntityType::Enemy)));
}

fn update_enemy(
    time: Res<Time>,
    mut enemies: Query<(&mut Enemy, &Transform, &mut moveable::Moveable)>,
    path_find: Res<PathFinder>
) {
    for (mut enemy, transform, mut moveable) in enemies.iter_mut() {
        if enemy.action_cooldown.tick(time.delta()).finished() {
            let (_, path) = path_find.get_path();
            let mut found_next = false;
            let mut new_target = None;

            for p in path.iter() {
                if found_next {
                    new_target = Some(p);
                    break;
                }

                if path_find.get_position(*p).matches(transform.translation) {
                    found_next = true;
                }
            }

            if let Some(target) = new_target {
                let target = path_find.get_position(*target);
                let current = transform.translation.as_i32();
                if target.z > current.z {
                    println!("Going right");
                    moveable.set_movement(Direction::Right, moveable::MovementType::Step);
                } else if target.z < current.z {
                    println!("Going left");
                    moveable.set_movement(Direction::Left, moveable::MovementType::Step);
                } else if target.x > current.x {
                    println!("Going up");
                    moveable.set_movement(Direction::Up, moveable::MovementType::Step);
                } else if target.x < current.x {
                    println!("Going down");
                    moveable.set_movement(Direction::Down, moveable::MovementType::Step);
                }
            }
        }
    }
}

fn spawn_player(
    mut commands: Commands, 
    meshes: Res<DudeMeshes>, 
    mut level: ResMut<Level>,
) {
    let mut transform = Transform::from_translation(Vec3::new(0.0, 0.0, 0.0));
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
            .insert(Position { x: 0, y: 0, z: 0 })
            .insert(EntityType::Dude)
            .insert(holdable::Holder { holding: None })
            .insert(moveable::Moveable::new(true, true, 0.1))
            .insert(Facing::new(Direction::Right))
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
    level.set(0, 0, 0, Some(GameObject::new(player_entity, EntityType::Dude)));
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
