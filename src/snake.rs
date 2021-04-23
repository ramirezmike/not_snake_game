use bevy::prelude::*;
use crate::{Direction, EntityType, GameObject, level::Level, path_find::PathFinder, level,
            Position, holdable, block, moveable, facing::Facing};

#[derive(Default)]
pub struct EnemyMeshes {
    pub head: Handle<Mesh>,
    pub body: Handle<Mesh>,
    pub material: Handle<StandardMaterial>,
}

pub struct Enemy {
    body_parts: Vec::<Entity>,
    action_cooldown: Timer, 
}

pub struct Snake;
pub struct SnakeBody;

pub fn spawn_enemy(
    mut commands: Commands, 
    meshes: Res<EnemyMeshes>, 
    mut level: ResMut<Level>,
) {
    let position = Vec3::new(0.0, 0.0, 11.0);
    let mut transform = Transform::from_translation(position);
    transform.apply_non_uniform_scale(Vec3::new(0.50, 0.50, 0.50)); 
    transform.rotate(Quat::from_axis_angle(Vec3::new(0.0, 1.0, 0.0), std::f32::consts::PI));
    let enemy_entity = 
    commands.spawn_bundle(PbrBundle {
                transform,
                ..Default::default()
            })
            .insert(Position { x: position.x as i32, y: position.y as i32, z: position.z as i32 })
            .insert(EntityType::Enemy)
            .insert(Snake)
            .insert(Enemy {
                body_parts: vec!(),
                action_cooldown: Timer::from_seconds(0.5, true),
            })
            .insert(holdable::Holder { holding: None })
            .insert(moveable::Moveable::new(false, false, 0.1))
            .insert(Facing::new(Direction::Right))
            .with_children(|parent|  {
                parent.spawn_bundle(PbrBundle {
                    mesh: meshes.head.clone(),
                    material: meshes.material.clone(),
                    transform: {
                        let mut transform = Transform::from_rotation(Quat::from_axis_angle(Vec3::new(0.0, 1.0, 0.0), 0.0));
                        transform.translation = Vec3::new(0.0, 1.0, 0.0);
                        transform
                    },
                    ..Default::default()
                });
            }).id();
    level.set(position.x as i32, position.y as i32, position.z as i32, Some(GameObject::new(enemy_entity, EntityType::Enemy)));
}

#[derive(Copy, Clone)]
pub struct AddBodyPartEvent {
    snake: Entity,
    follow: Entity,
    target: Position
}

pub fn debug_add_body_part(
    enemies: Query<(Entity, &Enemy, &Transform)>,
    body_parts: Query<&Transform, With<SnakeBody>>,
    mut body_part_writer: EventWriter<AddBodyPartEvent>,

    keyboard_input: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut timer: Local<f32>,
) {
    if *timer > 0.2 && keyboard_input.pressed(KeyCode::P) {
        *timer = 0.0;

        for (entity, enemy, transform) in enemies.iter() {
            let (follow, target) = match enemy.body_parts.last() {
                                      Some(tail) => (*tail, body_parts.get(*tail).unwrap()),
                                      _ => (entity, transform)
                                   };
            body_part_writer.send(AddBodyPartEvent { snake: entity, follow, target: Position::from_vec(target.translation) });
        }
    }

    *timer += time.delta_seconds();
}

pub fn add_body_parts(
    mut body_part_reader: EventReader<AddBodyPartEvent>,
    mut queued_parts: Local<Vec::<AddBodyPartEvent>>,
    snakes: Query<&Position, With<Snake>>,
    mut snake_enemies: Query<&mut Enemy>,
    mut commands: Commands, 
    meshes: Res<EnemyMeshes>, 
    mut level: ResMut<Level>,
) {
    for event in body_part_reader.iter() {
        queued_parts.push(AddBodyPartEvent { snake: event.snake, follow: event.follow, target: event.target });
    }

    let (ready, waiting): (Vec<AddBodyPartEvent>, Vec<AddBodyPartEvent>) 
        = queued_parts.iter()
                      .partition(|x| *snakes.get(x.follow).unwrap() != x.target);

    for part_to_add in ready {
        let position = part_to_add.target; 
        let target = Transform::from_xyz(position.x as f32, position.y as f32, position.z as f32);
        let mut transform = Transform::from_translation(target.translation);
        transform.apply_non_uniform_scale(Vec3::new(0.50, 0.50, 0.50)); 
        let mut snake_enemy = snake_enemies.get_mut(part_to_add.snake).unwrap();

        // TODO: we might not need this and can treat everything as Enemy
        let entity_type = if snake_enemy.body_parts.len() > 1 { EntityType::Block } else { EntityType::Enemy }; 

        let enemy_entity = 
        commands.spawn_bundle(PbrBundle {
                    transform,
                    ..Default::default()
                })
                .insert(Position { x: transform.translation.x as i32, 
                                   y: transform.translation.y as i32, 
                                   z: transform.translation.z as i32 })
                .insert(entity_type)
                .insert(SnakeBody)
                .insert(Snake)
                .insert(moveable::Moveable::new(false, false, 0.1))
                .with_children(|parent|  {
                    parent.spawn_bundle(PbrBundle {
                        mesh: meshes.body.clone(),
                        material: meshes.material.clone(),
                        transform: {
                            let mut transform = Transform::from_rotation(Quat::from_axis_angle(Vec3::new(0.0, 1.0, 0.0), 0.0));
                            transform.translation = Vec3::new(0.0, 1.0, 0.0);
                            transform
                        },
                        ..Default::default()
                    });
                }).id();
        
        snake_enemy.body_parts.push(enemy_entity);

        level.set(transform.translation.x as i32, 
                  transform.translation.y as i32, 
                  transform.translation.z as i32, 
                  Some(GameObject::new(enemy_entity, EntityType::Enemy)));
    }

    *queued_parts = waiting;
}

pub fn add_body_part(
    mut enemies: Query<(&mut Enemy, &mut moveable::Moveable)>,
    body_parts: Query<(&Transform, &SnakeBody)>,
    mut commands: Commands, 
    meshes: Res<EnemyMeshes>, 
    mut level: ResMut<Level>,

    mut position_change_event_reader: EventReader<level::PositionChangeEvent>,
) {
    for level::PositionChangeEvent(position, game_object) in position_change_event_reader.iter() {
        if let Some(game_object) = game_object {
            match game_object.entity_type {
                EntityType::Enemy => {
                    if let Ok((mut enemy, mut moveable)) = enemies.get_mut(game_object.entity) {
                        let target = if let Some(tail) = enemy.body_parts.last() {
                            println!("# of parts {} ", body_parts.iter().count());
                                        body_parts.get(*tail).unwrap().0.clone()
                                     } else {
                                        Transform::from_xyz(position.x as f32, position.y as f32, position.z as f32)
                                     };
                        let target = Transform::from_xyz(position.x as f32, position.y as f32, position.z as f32);
                        let mut transform = Transform::from_translation(target.translation);
                        transform.apply_non_uniform_scale(Vec3::new(0.50, 0.50, 0.50)); 
                        let enemy_entity = 
                        commands.spawn_bundle(PbrBundle {
                                    transform,
                                    ..Default::default()
                                })
                                .insert(Position { x: transform.translation.x as i32, 
                                                   y: transform.translation.y as i32, 
                                                   z: transform.translation.z as i32 })
                                .insert(EntityType::Enemy)
                                .insert(SnakeBody)
                                .insert(moveable::Moveable::new(false, false, 0.1))
                                .with_children(|parent|  {
                                    parent.spawn_bundle(PbrBundle {
                                        mesh: meshes.body.clone(),
                                        material: meshes.material.clone(),
                                        transform: {
                                            let mut transform = Transform::from_rotation(Quat::from_axis_angle(Vec3::new(0.0, 1.0, 0.0), 0.0));
                                            transform.translation = Vec3::new(0.0, 1.0, 0.0);
                                            transform
                                        },
                                        ..Default::default()
                                    });
                                }).id();
                        enemy.body_parts.push(enemy_entity);

                        level.set(transform.translation.x as i32, 
                                  transform.translation.y as i32, 
                                  transform.translation.z as i32, 
                                  Some(GameObject::new(enemy_entity, EntityType::Enemy)));
                    }
                }, 
                _ => ()
            }
        }
    }
}

pub fn update_enemy(
    time: Res<Time>,
    mut enemies: Query<(&mut Enemy, &Transform, &mut moveable::Moveable), Without<SnakeBody>>,
    mut snake_bodies: Query<(&mut moveable::Moveable, &Transform), With<SnakeBody>>,
    path_find: Res<PathFinder>,

    keyboard_input: Res<Input<KeyCode>>,
    mut is_active: Local<bool>,
    mut timer: Local<f32>,
) {
    if *timer > 0.2 && keyboard_input.pressed(KeyCode::L) {
        *is_active = !*is_active;
        *timer = 0.0;
    }

    *timer += time.delta_seconds();

    if *is_active {
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

                let set_movement = 
                    |target: Position, current: IVec3, 
                     moveable: &mut moveable::Moveable, 
                     movement_type: moveable::MovementType| {

                    if target.z > current.z {
                        println!("Going right");
                        moveable.set_movement(Direction::Right, movement_type);
                    } else if target.z < current.z {
                        println!("Going left");
                        moveable.set_movement(Direction::Left, movement_type);
                    } else if target.x > current.x {
                        println!("Going up");
                        moveable.set_movement(Direction::Up, movement_type);
                    } else if target.x < current.x {
                        println!("Going down");
                        moveable.set_movement(Direction::Down, movement_type);
                    } else if target.y < current.y {
                        println!("Going below");
                        moveable.set_movement(Direction::Beneath, movement_type);
                    } else if target.y > current.y {
                        println!("Going above");
                        moveable.set_movement(Direction::Above, movement_type);
                    }
                };

                if let Some(target) = new_target {
                    let target = path_find.get_position(*target);
                    let current = transform.translation.as_i32();
                    set_movement(target, current, &mut moveable, moveable::MovementType::Step); 

                    let mut previous_position = current;
                    for snake_body in enemy.body_parts.iter() {
                        if let Ok((mut moveable, transform)) = snake_bodies.get_mut(*snake_body) {
                            let target = Position::from_vec(previous_position.as_f32());
                            let current = transform.translation.as_i32();
                            set_movement(target, current, &mut moveable, moveable::MovementType::Force); 
                            previous_position = current; 
                        }
                    }
                }
            }
        }
    }
}

