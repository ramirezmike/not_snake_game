use bevy::prelude::*;
use crate::{Direction, EntityType, GameObject, level::Level, path_find::PathFinder, level,
            Position, holdable, moveable, facing::Facing, food::FoodEatenEvent};

#[derive(Default)]
pub struct EnemyMeshes {
    pub head: Handle<Mesh>,
    pub body: Handle<Mesh>,
    pub material: Handle<StandardMaterial>,
}

pub struct Enemy {
    body_parts: Vec::<Entity>,
    action_cooldown: Timer, 
    pub is_dead: bool,
}

pub struct Snake;
pub struct SnakeBody;
pub struct KillSnakeEvent(pub Entity);

pub fn spawn_enemy(
    commands: &mut Commands, 
    meshes: &Res<EnemyMeshes>, 
    level: &mut ResMut<Level>,
    x: usize,
    y: usize,
    z: usize,
) {
    let position = Vec3::new(x as f32, y as f32, z as f32);
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
                is_dead: false,
            })
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

    let (alive, _):(Vec<AddBodyPartEvent>, Vec<AddBodyPartEvent>) 
        = queued_parts.iter()
                      .partition(|x| snakes.get(x.follow).is_ok());

    let (ready, waiting): (Vec<AddBodyPartEvent>, Vec<AddBodyPartEvent>) 
        = alive.iter()
               .partition(|x| *snakes.get(x.follow).unwrap() != x.target);

    for part_to_add in ready {
        let position = part_to_add.target; 
        let target = Transform::from_xyz(position.x as f32, position.y as f32, position.z as f32);
        let mut transform = Transform::from_translation(target.translation);
        transform.apply_non_uniform_scale(Vec3::new(0.50, 0.50, 0.50)); 
        let mut snake_enemy = snake_enemies.get_mut(part_to_add.snake).unwrap();

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

    if !*is_active {
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

pub fn handle_food_eaten(
    mut food_eaten_event_reader: EventReader<FoodEatenEvent>,
    mut body_part_writer: EventWriter<AddBodyPartEvent>,
    snakes: Query<(Entity, &Enemy, &Transform), With<Snake>>,
    body_parts: Query<&Transform, With<SnakeBody>>,
) {
    for eater in food_eaten_event_reader.iter() {
        if let Ok((entity, snake, transform)) = snakes.get(eater.0) {
            let (follow, target) = match snake.body_parts.last() {
                                      Some(tail) => (*tail, body_parts.get(*tail).unwrap()),
                                      _ => (entity, transform)
                                   };
            body_part_writer.send(AddBodyPartEvent { snake: entity, follow, target: Position::from_vec(target.translation) });
        }
    }
}

pub fn handle_kill_snake(
    mut commands: Commands, 
    mut kill_snake_event_reader: EventReader<KillSnakeEvent>,
    mut snakes: Query<(&mut Enemy, &mut Transform, &mut Position), (With<Snake>, Without<SnakeBody>)>,
    snake_part_positions: Query<&Position, With<SnakeBody>>,
    snake_part_meshes: Query<&Children, Or<(With<SnakeBody>, With<Snake>)>>,
    mut visibles: Query<&mut Visible>,
    mut dying_snakes: Local<Vec::<(Entity, u32, Timer)>>, // entity, number of flashes, timer
    time: Res<Time>,
    mut level: ResMut<Level>,
) {
    let flash_limit = 5;
    for snake_entity in kill_snake_event_reader.iter() {
        println!("Received kill event");
        dying_snakes.push((snake_entity.0, 0, Timer::from_seconds(0.5, true)));
        for body_part in snakes.get_mut(snake_entity.0).unwrap().0.body_parts.iter() {
            dying_snakes.push((*body_part, 0, Timer::from_seconds(0.5, true)));
        }
    }

    let mut temp_dying_snakes = vec!();
    std::mem::swap(&mut temp_dying_snakes, &mut dying_snakes);

    let (dead, mut flashing): (Vec<(Entity, u32, Timer)>, Vec<(Entity, u32, Timer)>) 
        = temp_dying_snakes.into_iter()
                           .partition(|(_, flash_times, _)| *flash_times > flash_limit);

    for dead_snake in dead {
        if let Ok((mut snake_head, mut transform, mut position)) = snakes.get_mut(dead_snake.0) {
            println!("Finding new spot for head");
            // move the head to a new spot
            snake_head.body_parts = vec!();
            let random_standable = level.get_random_standable();

            *position = random_standable;
            transform.translation.x = random_standable.x as f32;
            transform.translation.y = random_standable.y as f32;
            transform.translation.z = random_standable.z as f32;
            level.set(random_standable.x as i32, 
                      random_standable.y as i32, 
                      random_standable.z as i32, 
                      Some(GameObject::new(dead_snake.0, EntityType::Enemy)));
            snake_head.is_dead = false;
        } else {
            println!("despawning tail");
            // despawn the tail
            if let Ok(position) = snake_part_positions.get(dead_snake.0) {
                println!("setting position to none");
                level.set(position.x, position.y, position.z, None);
            }
            commands.entity(dead_snake.0).despawn_recursive();

        }
    }

    // make dying snakes flash
    for mut dying_snake in flashing.iter_mut() {
        if let Ok(snake_part) = snake_part_meshes.get(dying_snake.0) {
            if dying_snake.2.tick(time.delta()).finished() {
                for child in snake_part.iter() {
                    if let Ok(mut child_mesh) = visibles.get_mut(*child) {
                        println!("flashing entity");
                        child_mesh.is_visible = !child_mesh.is_visible;
                        dying_snake.1 += 1;
                        dying_snake.2.reset();
                    }
                }
            }
        } else {
            dying_snake.1 = flash_limit; // couldn't find the part for some reason so just let it disappear next iteration
        }
    }

    *dying_snakes = flashing;
}
