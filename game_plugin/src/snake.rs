use bevy::{
    prelude::*,
    render::{
        pipeline::{PipelineDescriptor, RenderPipeline},
        render_graph::{base, RenderGraph, RenderResourcesNode},
        renderer::RenderResources,
        shader::{ShaderStage, ShaderStages},
    },
};
use crate::{Direction, EntityType, GameObject, level::Level, path_find::PathFinder, dude,
            teleporter, environment, sounds, Position, food::FoodEatenEvent};
use petgraph::{graph::NodeIndex};
use bevy::reflect::{TypeUuid};
use serde::Deserialize;

#[derive(Default)]
pub struct EnemyMeshes {
    pub head: Handle<Mesh>,
    pub body: Handle<Mesh>,
    pub shadow: Handle<Mesh>,
    pub material: Handle<StandardMaterial>,
    pub shadow_material: Handle<StandardMaterial>,
}

#[derive(Debug, Clone, Deserialize, TypeUuid)]
#[uuid = "60cadc56-aa9c-4543-8640-a018b74b5052"] // this needs to be actually generated
pub enum SnakeTarget {
    Normal,
    OnlyFood,
    OnlyDude,
    OnlyRandom,
}

#[derive(Debug, Clone)]
pub struct BodyPosition {
    translation: Vec3,
    rotation: Quat,
}

#[derive(Clone)]
pub struct Enemy {
    body_parts: Vec::<Entity>,
    body_positions: Vec::<BodyPosition>,
    pub speed: f32, 
    movement: Option::<SnakeMovement>,
    pub is_dead: bool,
    up: Vec3,
    forward: Vec3,
    pub is_electric: bool,
    pub current_path: Option<(u32, Vec<NodeIndex<u32>>)>,
}

impl Enemy {
    pub fn get_first_body(&self) -> Position {
        Position::from_vec(self.body_positions[0].translation)
    }

    pub fn is_in_vec(&self, position: Vec3) -> bool {
        for body_position in self.body_positions.iter() {
            if body_position.translation == position {
                return true;
            }
        }

        false
    }
}

pub struct Snake;
pub struct SnakeBody;
pub struct SnakeInnerMesh;
pub struct KillSnakeEvent(pub Entity);
#[derive(Clone)]
struct SnakeMovement {
    target: Vec3,
    starting_from: Vec3,
    current_movement_time: f32,
    finish_movement_time: f32,

    start_rotation: Quat,
    target_rotation: Quat,
    current_rotation_time: f32,
    finish_rotation_time: f32,
}

static INNER_MESH_VERTICAL_OFFSET: f32 = 1.0;

pub fn generate_snake_body(
    commands: &mut Commands,
    meshes: &ResMut<EnemyMeshes>, 
    transform: Transform,
    is_electric: bool,
    game_shaders: &Res<environment::GameShaders>,
) -> Entity {
    commands.spawn_bundle(PbrBundle {
                transform: {
                    let mut t = transform.clone();
                    t.translation.x += 1.0;
                    //t.rotation = Quat::from_axis_angle(Vec3::new(0.0, 1.0, 0.0), std::f32::consts::PI/2.0);
                    t
                },
                ..Default::default()
            })
            .insert(EntityType::Enemy)
            .insert(SnakeBody)
            .insert(Snake)
            .with_children(|parent|  {
                parent.spawn_bundle(PbrBundle {
                    transform: Transform::from_translation(Vec3::new(0.0, INNER_MESH_VERTICAL_OFFSET, 0.0)),
                    ..Default::default()
                }).insert(SnakeInnerMesh)
                .with_children(|inner_parent| {
                    if is_electric {
                        inner_parent.spawn_bundle(PbrBundle {
                            mesh: meshes.body.clone(),
                            material: meshes.material.clone(),
                            render_pipelines: RenderPipelines::from_pipelines(
                                vec![RenderPipeline::new(game_shaders.electric.clone())]
                            ),
                            transform: {
                                let mut t = Transform::from_translation(Vec3::new(0.0, 0.0, 0.0));
                                t.rotate(Quat::from_axis_angle(Vec3::new(0.0, 1.0, 0.0), std::f32::consts::PI / 2.0));
                                t.scale = Vec3::new(1.1, 1.1, 1.1);
                                t
                            },
                            ..Default::default()
                        }).insert(environment::TimeUniform { value: 0.0 });
                    }

                    inner_parent.spawn_bundle(PbrBundle {
                        mesh: meshes.body.clone(),
                        material: meshes.material.clone(),
                        transform: {
                            let mut t = Transform::from_translation(Vec3::new(0.0, 0.0, 0.0));
                            t.rotate(Quat::from_axis_angle(Vec3::new(0.0, 1.0, 0.0), std::f32::consts::PI / 2.0));
                            t
                        },
                        ..Default::default()
                    });
                });
            }).id()
}

pub fn spawn_enemy(
    commands: &mut Commands, 
    meshes: &ResMut<EnemyMeshes>, 
    level: &mut ResMut<Level>,
    game_shaders: &Res<environment::GameShaders>,
    x: usize,
    y: usize,
    z: usize,
    is_electric: bool,
) {
    println!("Creating snake!");
    let position = Vec3::new(x as f32, y as f32, z as f32);
    let mut transform = Transform::from_translation(position);
    transform.apply_non_uniform_scale(Vec3::new(0.50, 0.50, 0.50)); 
    transform.rotate(Quat::from_axis_angle(Vec3::new(0.0, 1.0, 0.0), std::f32::consts::FRAC_PI_2));

    let body_part_entity = generate_snake_body(commands, meshes, transform, is_electric, &game_shaders);
        
    let snake_speed = level.snake_speed();
    let enemy_entity = 
    commands.spawn_bundle(PbrBundle {
                transform,
                ..Default::default()
            })
            .insert(Position { x: position.x as i32, y: position.y as i32, z: position.z as i32 })
            .insert(EntityType::EnemyHead)
            .insert(Snake)
            .insert(Enemy {
                body_parts: vec![body_part_entity],
                body_positions: vec![
                    BodyPosition { translation: Vec3::new(transform.translation.x + 1.0, 
                                                          transform.translation.y, 
                                                          transform.translation.z), 
                                   rotation: Quat::IDENTITY },
                ],
                speed: snake_speed,
                is_electric,
                movement: None,
                is_dead: false,
                up: Vec3::Y,
                forward: -Vec3::X,
                current_path: None,
            })
            .with_children(|parent|  {
                parent.spawn_bundle(PbrBundle {
                    transform: Transform::from_translation(Vec3::new(0.0, INNER_MESH_VERTICAL_OFFSET, 0.0)),
                    ..Default::default()
                }).insert(SnakeInnerMesh)
                .with_children(|inner_parent| {
                    if is_electric {
                        inner_parent.spawn_bundle(PbrBundle {
                            mesh: meshes.head.clone(),
                            material: meshes.material.clone(),
                            render_pipelines: RenderPipelines::from_pipelines(
                                vec![RenderPipeline::new(game_shaders.electric.clone())]
                            ),
                            transform: {
                                let mut t = Transform::from_translation(Vec3::new(0.0, 0.0, 0.0));
                                t.scale = Vec3::new(1.1, 1.1, 1.1);
                                t
                            },
                            ..Default::default()
                        }).insert(environment::TimeUniform { value: 0.0 });
                    } 

                    inner_parent.spawn_bundle(PbrBundle {
                        mesh: meshes.head.clone(),
                        material: meshes.material.clone(),
                        ..Default::default()
                    });
                });
            }).id();
    level.set(position.x as i32, position.y as i32, position.z as i32, Some(GameObject::new(enemy_entity, EntityType::EnemyHead)));
    level.set(position.x as i32 + 1, position.y as i32, position.z as i32, Some(GameObject::new(enemy_entity, EntityType::Enemy)));
}

#[derive(Copy, Clone)]
pub struct AddBodyPartEvent { snake: Entity }

pub fn debug_add_body_part(
    enemies: Query<Entity, With<Enemy>>,
    mut body_part_writer: EventWriter<AddBodyPartEvent>,

    keyboard_input: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut timer: Local<f32>,
) {
    if *timer > 0.2 && keyboard_input.pressed(KeyCode::P) {
        *timer = 0.0;

        for entity in enemies.iter() {
            body_part_writer.send(AddBodyPartEvent { snake: entity });
        }
    }

    *timer += time.delta_seconds();
}

pub fn add_body_parts(
    mut body_part_reader: EventReader<AddBodyPartEvent>,
    mut snake_enemies: Query<&mut Enemy>,
    mut commands: Commands, 
    game_shaders: Res<environment::GameShaders>,
    meshes: ResMut<EnemyMeshes>, 
) {
    for part_to_add in body_part_reader.iter() {
        if let Ok(mut snake_enemy) = snake_enemies.get_mut(part_to_add.snake) {
            let last_position = snake_enemy.body_positions.last().unwrap();
            let mut transform = Transform::from_translation(last_position.translation);
            transform.rotate(Quat::from_axis_angle(Vec3::new(0.0, 1.0, 0.0), std::f32::consts::FRAC_PI_2));
            transform.apply_non_uniform_scale(Vec3::new(0.50, 0.50, 0.50)); 

            let body_part_entity = generate_snake_body(&mut commands, &meshes, transform, snake_enemy.is_electric, &game_shaders);
            snake_enemy.body_parts.push(body_part_entity);
        }
    }
}

pub fn update_enemy(
    time: Res<Time>,
    mut enemies: Query<(Entity, &mut Enemy, &mut Transform, &mut Position, &Children), (Without<SnakeBody>, Without<SnakeInnerMesh>)>,
    mut inner_meshes: Query<&mut Transform, With<SnakeInnerMesh>>,
    path_find: Res<PathFinder>,
    mut level: ResMut<Level>,
    teleporters: Query<&teleporter::Teleporter>,

    dude: Query<&Transform, (With<dude::Dude>, Without<SnakeInnerMesh>, Without<Enemy>)>,
    keyboard_input: Res<Input<KeyCode>>,
    mut is_active: Local<bool>,
    mut timer: Local<f32>,
) {
    // FOR DEBUGGING
    if *timer > 0.2 && keyboard_input.pressed(KeyCode::L) {
        *is_active = !*is_active;
        *timer = 0.0;
    }
    *timer += time.delta_seconds();
    // FOR DEBUGGING

    if !*is_active {
        for (entity, mut enemy, mut transform, mut position, children) in enemies.iter_mut() {
//          if let Ok(dude_transform) = dude.single() {
//              if dude_transform.translation.distance(transform.translation) <= 1.5 {
//                  enemy.movement = None; // set to None so that we detect in the next step to move toward dude
//              }
//          }

            if enemy.movement.is_none() {
                let is_ai_controlled = true;
                let mut new_target = None;
                if is_ai_controlled  {
                    let (_, path) = enemy.current_path.clone().unwrap_or((0, vec!()));
                    let mut found_next = false;
                    let mut found_target = None;

                    for p in path.iter() {
                        if found_next {
                            found_target = Some(p);
                            break;
                        }

                        if path_find.get_position(*p).matches(transform.translation) {
                            found_next = true;
                        }
                    }

                    if let Some(target) = found_target {
                        new_target = Some(path_find.get_position(*target));
                    }
                } else {
                    let mut target = None;
                    let cur=transform.translation;
                    if keyboard_input.pressed(KeyCode::W) {
                        target = Some(Vec3::new(cur.x + 1.0, cur.y, cur.z));
                    }
                    if keyboard_input.pressed(KeyCode::S) {
                        target = Some(Vec3::new(cur.x - 1.0, cur.y, cur.z));
                    }
                    if keyboard_input.pressed(KeyCode::A) {
                        target = Some(Vec3::new(cur.x, cur.y, cur.z - 1.0));
                    }
                    if keyboard_input.pressed(KeyCode::D) {
                        target = Some(Vec3::new(cur.x, cur.y, cur.z + 1.0));
                    }
                    if keyboard_input.pressed(KeyCode::E) {
                        target = Some(Vec3::new(cur.x, cur.y + 1.0, cur.z));
                    }
                    if keyboard_input.pressed(KeyCode::R) {
                        target = Some(Vec3::new(cur.x, cur.y - 1.0, cur.z));
                    }
                    if let Some(target) = target {
                        new_target = Some(Position::from_vec(target));
                    }
                }

                if let Some(target) = new_target {
                    let current = transform.translation.as_i32();
                    let facing = if target.z > current.z { Direction::Right } 
                            else if target.z < current.z { Direction::Left } 
                            else if target.x > current.x { Direction::Up } 
                            else if target.x < current.x { Direction::Down } 
                            else if target.y < current.y { Direction::Beneath } 
                            else { Direction::Above };
                    let starting_from = transform.translation;
                    let target = Vec3::new(target.x as f32, target.y as f32, target.z as f32);

                    let start_rotation = inner_meshes.get_mut(*children.iter().last().unwrap()).unwrap().rotation;
                    let target_rotation = calculate_new_rotation(start_rotation, facing, &mut enemy);

                    enemy.movement = Some(SnakeMovement { 
                                        target, 
                                        starting_from,
                                        current_movement_time: 0.0,
                                        finish_movement_time: enemy.speed,
                                        start_rotation,
                                        target_rotation,
                                        current_rotation_time: 0.0,
                                        finish_rotation_time: enemy.speed,
                                     });

                    // pushes a new history state at the front, and pops one off the end 
                    // and updates the level by setting that spot to None
                    enemy.body_positions.insert(0, BodyPosition { translation: starting_from, rotation: target_rotation });
                    let number_of_body_parts = enemy.body_parts.len();
                    let last_body_position = enemy.body_positions.last().unwrap();
                    level.set_with_vec(last_body_position.translation, None);
                    enemy.body_positions.truncate(number_of_body_parts);
                }
            }

            let mut clone_enemy = enemy.clone();
            if let Some(movement) = &mut enemy.movement {
                // if the spot the snake moved from is the same object then clear it
                if let Some(game_object) = level.get_with_vec(transform.translation) {
                    if game_object.entity == entity {
                        level.set_with_vec(transform.translation, None);
                    }
                }

                if movement.current_movement_time >= movement.finish_movement_time {
                    if level.is_enterable_with_vec(movement.target) {
                        transform.translation = movement.target;
                    }

                    if !teleporters.iter().len() > 0 {
                        for teleporter in teleporters.iter() {
                            if teleporter.position == Position::from_vec(movement.target) {

                                println!("Enemy should teleport!");
                                // teleport
                                let original_target = movement.target.clone();
                                let original_rotation = movement.start_rotation.clone();
                                let new_target = teleporter.target;
                                transform.translation = Vec3::new(new_target.x as f32, 
                                                                  new_target.y as f32, 
                                                                  new_target.z as f32);

                                level.set_with_vec(transform.translation, Some(GameObject::new(entity, EntityType::EnemyHead)));
                                position.update_from_vec(transform.translation);

                                // set new target after teleportation 
                                movement.target = Vec3::new(teleporter.move_to.x as f32, 
                                                            teleporter.move_to.y as f32, 
                                                            teleporter.move_to.z as f32); 

                           //   // change current facing direction to target facing
                           //   if let Some(mut facing) = maybe_facing {
                           //       facing.direction = teleporter.facing;
                           //   }

                                // reset movement start time
                                movement.current_movement_time = 0.0;


                                // update movement of children


                                // set the "start position" after teleportation
                                movement.starting_from = Vec3::new(teleporter.target.x as f32,
                                                                   teleporter.target.y as f32,
                                                                   teleporter.target.z as f32);


                                println!("Setting new facing!");
                                movement.current_rotation_time = movement.finish_rotation_time;

                                let start_rotation = inner_meshes.get_mut(*children.iter().last().unwrap()).unwrap().rotation;
                                let target_rotation = calculate_new_rotation(start_rotation, teleporter.facing, &mut clone_enemy);
                                movement.start_rotation = target_rotation;
                                movement.target_rotation = target_rotation;
                                calculate_new_rotation(start_rotation, teleporter.facing, &mut enemy);
                                for child in children.iter() {
                                    if let Ok(mut inner_mesh) = inner_meshes.get_mut(*child) {
                                        inner_mesh.rotation = target_rotation;
                                    }
                                }

                                // pushes a new history state at the front, and pops one off the end 
                                // and updates the level by setting that spot to None
                                enemy.body_positions.insert(0, 
                                    BodyPosition { translation: original_target, 
                                                   rotation: original_rotation });
                                let number_of_body_parts = enemy.body_parts.len();
                                let last_body_position = enemy.body_positions.last().unwrap();
                                level.set_with_vec(last_body_position.translation, None);
                                enemy.body_positions.truncate(number_of_body_parts);

                                return;
                            }
                        }
                    }

                    enemy.movement = None;
                } else {
                    // keep moving toward target
                    movement.current_movement_time += time.delta_seconds();
                    let new_translation = movement.starting_from.lerp(movement.target, 
                                                                      movement.current_movement_time / movement.finish_movement_time);
                    if !new_translation.is_nan() {
                        if transform.translation.distance(movement.target) < transform.translation.distance(new_translation) {
                            transform.translation = movement.target;
                            movement.current_movement_time = movement.finish_movement_time;
                        } else {
                            transform.translation = new_translation;
                        }
                    }

                    for child in children.iter() {
                        if let Ok(mut inner_mesh) = inner_meshes.get_mut(*child) {
                            // keep rotating toward target
                            movement.current_rotation_time += time.delta_seconds();
                            let new_rotation = movement.start_rotation.lerp(movement.target_rotation, 
                                                                            movement.current_rotation_time / movement.finish_rotation_time);
                            if !new_rotation.is_nan() {
                                if inner_mesh.rotation.angle_between(movement.target_rotation) < inner_mesh.rotation.angle_between(new_rotation) {
                                    inner_mesh.rotation = movement.target_rotation;
                                    movement.current_rotation_time = movement.finish_rotation_time;
                                } else {
                                    inner_mesh.rotation = new_rotation;
                                }
                            }

                        }
                    }
                }

                // need to update level here
                level.set_with_vec(transform.translation, Some(GameObject::new(entity, EntityType::EnemyHead)));
                position.update_from_vec(transform.translation);
            }
        }
    }
}

pub fn update_following(
    mut snakes: Query<&mut Enemy>,
    mut body_parts: Query<(Entity, &mut Transform, &Children), (With<SnakeBody>, Without<Enemy>)>,
    mut inner_meshes: Query<&mut Transform, (With<SnakeInnerMesh>, Without<SnakeBody>, Without<Enemy>)>,
    time: Res<Time>,
    teleporters: Query<&teleporter::Teleporter>,
    mut level: ResMut<Level>,
) {
    for mut snake in snakes.iter_mut() {
        let mut part_index = 0;
        let snake_speed = snake.speed;
        let body_part_entities = snake.body_parts.clone();
        for body_part in body_part_entities.iter() {
            if let Ok((entity, mut transform, children)) = body_parts.get_mut(*body_part) {
                if let Some(target) = &mut snake.body_positions.get_mut(part_index) {

                    let rate = snake_speed / 1.0;
                    let distance = transform.translation.distance(target.translation);
                    let new_translation = transform.translation.lerp(target.translation, time.delta_seconds() / (distance * rate));
                    if !new_translation.is_nan() {
                        if transform.translation.distance(target.translation) < transform.translation.distance(new_translation) {
                            transform.translation = target.translation;

                            if !teleporters.iter().len() > 0 {
                                for teleporter in teleporters.iter() {
                                    if teleporter.position == Position::from_vec(target.translation) {
                                        println!("Body should teleport to {:?}", teleporter.target);

                                        let new_target = teleporter.target;
                                        level.set_with_vec(transform.translation, None);
                                        println!("Body is at {:?}", transform.translation);
                                        transform.translation = Vec3::new(new_target.x as f32, 
                                                                          new_target.y as f32, 
                                                                          new_target.z as f32);
                                        println!("Body is at {:?}", transform.translation);

                                        // set new target after teleportation 
                                        target.translation = Vec3::new(teleporter.move_to.x as f32, 
                                                                       teleporter.move_to.y as f32, 
                                                                       teleporter.move_to.z as f32); 
                                    }
                                }
                            }

                        } else {
                            transform.translation = new_translation;
                        }
                    } 
                    level.set_with_vec(target.translation, Some(GameObject::new(entity, EntityType::Enemy)));


                    for child in children.iter() {
                        if let Ok(mut transform) = inner_meshes.get_mut(*child) {
                            let rotation_rate = (snake_speed * 0.60) / 1.0;
                            let rotation_distance = transform.rotation.angle_between(target.rotation);
                            let new_rotation = transform.rotation.lerp(target.rotation, 
                                                                       time.delta_seconds() / (rotation_distance * rotation_rate));
                            if !new_rotation.is_nan() {
                                if transform.rotation.angle_between(target.rotation) < transform.rotation.angle_between(new_rotation) {
                                    transform.rotation = target.rotation;
                                } else {
                                    transform.rotation = new_rotation;
                                }
                            }
                        }
                    }
                }
            }

            part_index += 1;
        }
    }
}

pub fn handle_food_eaten(
    mut food_eaten_event_reader: EventReader<FoodEatenEvent>,
    mut body_part_writer: EventWriter<AddBodyPartEvent>,
    snakes: Query<Entity, With<Snake>>,
    mut sound_writer: EventWriter<sounds::SoundEvent>,
) {
    for eater in food_eaten_event_reader.iter() {
        if let Ok(entity) = snakes.get(eater.0) {
            body_part_writer.send(AddBodyPartEvent { snake: entity });
            sound_writer.send(sounds::SoundEvent(sounds::Sounds::Bite));
        }
    }
}

// TODO: fix flashing. The inner meshes are likely not getting selected right
pub fn handle_kill_snake(
    mut commands: Commands, 
    mut kill_snake_event_reader: EventReader<KillSnakeEvent>,
    mut snakes: Query<(&mut Enemy, &mut Transform, &mut Position), (With<Snake>, Without<SnakeBody>)>,
    snake_part_transforms: Query<&Transform, With<SnakeBody>>,
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
        if let Ok((snake, _, position)) = snakes.get_mut(snake_entity.0) {
            for body_part in snake.body_parts.iter() {
                dying_snakes.push((*body_part, 0, Timer::from_seconds(0.5, true)));
                if let Ok(transform) = snake_part_transforms.get(*body_part) {
                    let translation = transform.translation.as_i32();
                    level.set(translation.x, translation.y, translation.z, None);
                }
                level.set(position.x, position.y, position.z, None);
            }
        }
    }

    let mut temp_dying_snakes = vec!();
    std::mem::swap(&mut temp_dying_snakes, &mut dying_snakes);

    let (dead, mut flashing): (Vec<(Entity, u32, Timer)>, Vec<(Entity, u32, Timer)>) 
        = temp_dying_snakes.into_iter()
                           .partition(|(_, flash_times, _)| *flash_times > flash_limit);

    for dead_snake in dead {
        if let Ok((mut _snake_head, mut _transform, mut _position)) = snakes.get_mut(dead_snake.0) {
//TODO: maybe this should be removed? Let the snake stay dead until next level
//          println!("Finding new spot for head");
//          // move the head to a new spot
//          snake_head.body_parts = vec!();
//          let random_standable = level.get_random_standable();
//          level.set(position.x, position.y, position.z, None);

//          *position = random_standable;
//          transform.translation.x = random_standable.x as f32;
//          transform.translation.y = random_standable.y as f32;
//          transform.translation.z = random_standable.z as f32;
//          level.set(random_standable.x as i32, 
//                    random_standable.y as i32, 
//                    random_standable.z as i32, 
//                    Some(GameObject::new(dead_snake.0, EntityType::Enemy)));
//          snake_head.is_dead = false;
        } else {
            println!("despawning tail");
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

pub fn detect_dude_on_electric_snake(
    snakes: Query<&Enemy>,
    dudes: Query<&Transform, With<dude::Dude>>,
    mut kill_dude_event_writer: EventWriter<dude::KillDudeEvent>,
) {
    for transform in dudes.iter() {
        let below_dude = Vec3::new(transform.translation.x, transform.translation.y - 1.0, transform.translation.z);
        for snake in snakes.iter() {
            if snake.is_electric && snake.is_in_vec(below_dude) {
                kill_dude_event_writer.send(dude::KillDudeEvent { death_type: dude::DudeDeath::Electric });
                return;
            }
        }
    }
}

pub fn calculate_new_rotation(
    start_rotation: Quat,
    facing: Direction,
    enemy: &mut Enemy
) -> Quat {
    match facing {
        Direction::Right => {
            let mut result = start_rotation;
            if enemy.up == Vec3::Y { // top of head is facing above
                if enemy.forward == -Vec3::X { // forward is down
                    result = result.mul_quat(Quat::from_axis_angle(Vec3::Y, std::f32::consts::PI / 2.0));
                }
                if enemy.forward == Vec3::X { // foward is up
                    result = result.mul_quat(Quat::from_axis_angle(Vec3::Y, -std::f32::consts::PI / 2.0));
                }
            }

            if enemy.up == -Vec3::Y { // top of head is facing beneath
                if enemy.forward == -Vec3::X { // forward is down
                    result = result.mul_quat(Quat::from_axis_angle(Vec3::Y, -std::f32::consts::PI / 2.0));
                }
                if enemy.forward == Vec3::X { // foward is up
                    result = result.mul_quat(Quat::from_axis_angle(Vec3::Y, std::f32::consts::PI / 2.0));
                }
            }

            if enemy.up == -Vec3::Z { // top of head is facing left
                if enemy.forward == Vec3::Y { // forward is facing above
                    result = result.mul_quat(Quat::from_axis_angle(Vec3::X, -std::f32::consts::PI / 2.0));
                    enemy.up = Vec3::Y;
                } 
                if enemy.forward == -Vec3::Y { // forward is facing beneath
                    result = result.mul_quat(Quat::from_axis_angle(Vec3::X, -std::f32::consts::PI / 2.0));
                    enemy.up = -Vec3::Y;
                } 
                if enemy.forward == Vec3::X { // forward is facing up
                    result = result.mul_quat(Quat::from_axis_angle(Vec3::X, -std::f32::consts::PI / 2.0));
                    enemy.up = Vec3::X;
                } 
                if enemy.forward == -Vec3::X { // forward is facing down
                    result = result.mul_quat(Quat::from_axis_angle(Vec3::X, -std::f32::consts::PI / 2.0));
                    enemy.up = -Vec3::X;
                } 
            }

            if enemy.up == Vec3::Z { // top of head is facing right
                if enemy.forward == Vec3::Y { // forward is facing above
                    result = result.mul_quat(Quat::from_axis_angle(Vec3::X, std::f32::consts::PI / 2.0));
                    enemy.up = -Vec3::Y;
                } 
                if enemy.forward == -Vec3::Y { // forward is facing beneath
                    result = result.mul_quat(Quat::from_axis_angle(Vec3::X, std::f32::consts::PI / 2.0));
                    enemy.up = Vec3::Y;
                } 
                if enemy.forward == Vec3::X { // forward is facing up
                    result = result.mul_quat(Quat::from_axis_angle(Vec3::X, std::f32::consts::PI / 2.0));
                    enemy.up = -Vec3::X;
                } 
                if enemy.forward == -Vec3::X { // forward is facing down
                    result = result.mul_quat(Quat::from_axis_angle(Vec3::X, std::f32::consts::PI / 2.0));
                    enemy.up = Vec3::X;
                } 
            }

            if enemy.up == -Vec3::X { // top of head is facing down
                if enemy.forward == Vec3::Y { // forward is facing above
                    result = result.mul_quat(Quat::from_axis_angle(Vec3::Y, -std::f32::consts::PI / 2.0));
                } 
                if enemy.forward == -Vec3::Y { // forward is facing beneath
                    result = result.mul_quat(Quat::from_axis_angle(Vec3::Y, std::f32::consts::PI / 2.0));
                } 
            }

            if enemy.up == Vec3::X { // top of head is facing up
                if enemy.forward == Vec3::Y { // forward is facing above
                    result = result.mul_quat(Quat::from_axis_angle(Vec3::Y, std::f32::consts::PI / 2.0));
                } 
                if enemy.forward == -Vec3::Y { // forward is facing beneath
                    result = result.mul_quat(Quat::from_axis_angle(Vec3::Y, -std::f32::consts::PI / 2.0));
                } 
            }

            enemy.forward = Vec3::Z;

            result 
        },
        Direction::Left => {
            let mut result = start_rotation;
            if enemy.up == Vec3::Y { // top of head is facing above
                if enemy.forward == -Vec3::X { // forward is down
                    result = result.mul_quat(Quat::from_axis_angle(Vec3::Y, -std::f32::consts::PI / 2.0));
                }
                if enemy.forward == Vec3::X { // forward is up
                    result = result.mul_quat(Quat::from_axis_angle(Vec3::Y, std::f32::consts::PI / 2.0));
                }
            }

            if enemy.up == -Vec3::Y { // top of head is facing beneath
                if enemy.forward == -Vec3::X { // forward is down
                    result = result.mul_quat(Quat::from_axis_angle(Vec3::Y, std::f32::consts::PI / 2.0));
                }
                if enemy.forward == Vec3::X { // forward is up
                    result = result.mul_quat(Quat::from_axis_angle(Vec3::Y, -std::f32::consts::PI / 2.0));
                }
            }

            if enemy.up == Vec3::Z { // top of head is facing right
                if enemy.forward == Vec3::Y { // forward is facing above
                    result = result.mul_quat(Quat::from_axis_angle(Vec3::X, -std::f32::consts::PI / 2.0));
                    enemy.up = Vec3::Y;
                } 
                if enemy.forward == -Vec3::Y { // forward is facing beneath
                    result = result.mul_quat(Quat::from_axis_angle(Vec3::X, -std::f32::consts::PI / 2.0));
                    enemy.up = -Vec3::Y;
                } 
                if enemy.forward == Vec3::X { // forward is facing up
                    result = result.mul_quat(Quat::from_axis_angle(Vec3::X, -std::f32::consts::PI / 2.0));
                    enemy.up = Vec3::X;
                } 
                if enemy.forward == -Vec3::X { // forward is facing down
                    result = result.mul_quat(Quat::from_axis_angle(Vec3::X, -std::f32::consts::PI / 2.0));
                    enemy.up = -Vec3::X;
                } 
            }

            if enemy.up == -Vec3::Z { // top of head is facing left
                if enemy.forward == Vec3::Y { // forward is facing above
                    result = result.mul_quat(Quat::from_axis_angle(Vec3::X, std::f32::consts::PI / 2.0));
                    enemy.up = -Vec3::Y;
                } 
                if enemy.forward == -Vec3::Y { // forward is facing beneath
                    result = result.mul_quat(Quat::from_axis_angle(Vec3::X, std::f32::consts::PI / 2.0));
                    enemy.up = Vec3::Y;
                } 
                if enemy.forward == Vec3::X { // forward is facing up
                    result = result.mul_quat(Quat::from_axis_angle(Vec3::X, std::f32::consts::PI / 2.0));
                    enemy.up = -Vec3::X;
                } 
                if enemy.forward == -Vec3::X { // forward is facing down
                    result = result.mul_quat(Quat::from_axis_angle(Vec3::X, std::f32::consts::PI / 2.0));
                    enemy.up = Vec3::X;
                } 
            }

            if enemy.up == -Vec3::X { // top of head is facing down
                if enemy.forward == Vec3::Y { // forward is facing above
                    result = result.mul_quat(Quat::from_axis_angle(Vec3::Y, std::f32::consts::PI / 2.0));
                } 
                if enemy.forward == -Vec3::Y { // forward is facing beneath
                    result = result.mul_quat(Quat::from_axis_angle(Vec3::Y, -std::f32::consts::PI / 2.0));
                } 
            }

            if enemy.up == Vec3::X { // top of head is facing up
                if enemy.forward == Vec3::Y { // forward is facing above
                    result = result.mul_quat(Quat::from_axis_angle(Vec3::Y, -std::f32::consts::PI / 2.0));
                } 
                if enemy.forward == -Vec3::Y { // forward is facing beneath
                    result = result.mul_quat(Quat::from_axis_angle(Vec3::Y, std::f32::consts::PI / 2.0));
                } 
            }

            enemy.forward = -Vec3::Z;

            result 
        }
        Direction::Down => {
            let mut result = start_rotation;
            if enemy.up == Vec3::Y { // top of head is facing above
                if enemy.forward == -Vec3::Z { // forward is left
                    result = result.mul_quat(Quat::from_axis_angle(Vec3::Y, std::f32::consts::PI / 2.0));
                }
                if enemy.forward == Vec3::Z { // forward is right
                    result = result.mul_quat(Quat::from_axis_angle(Vec3::Y, -std::f32::consts::PI / 2.0));
                }
            }

            if enemy.up == -Vec3::Y { // top of head is facing beneath
                if enemy.forward == -Vec3::Z { // forward is left
                    result = result.mul_quat(Quat::from_axis_angle(Vec3::Y, -std::f32::consts::PI / 2.0));
                }
                if enemy.forward == Vec3::Z { // forward is right
                    result = result.mul_quat(Quat::from_axis_angle(Vec3::Y, std::f32::consts::PI / 2.0));
                }
            }

            if enemy.up == Vec3::X { // top of head is facing up
                if enemy.forward == Vec3::Y { // forward is facing above
                    result = result.mul_quat(Quat::from_axis_angle(Vec3::X, -std::f32::consts::PI / 2.0));
                    enemy.up = Vec3::Y;
                } 
                if enemy.forward == -Vec3::Y { // forward is facing beneath
                    result = result.mul_quat(Quat::from_axis_angle(Vec3::X, -std::f32::consts::PI / 2.0));
                    enemy.up = -Vec3::Y;
                } 
                if enemy.forward == -Vec3::Z { // forward is facing left
                    result = result.mul_quat(Quat::from_axis_angle(Vec3::X, -std::f32::consts::PI / 2.0));
                    enemy.up = -Vec3::Z;
                } 
                if enemy.forward == Vec3::Z { // forward is facing right
                    result = result.mul_quat(Quat::from_axis_angle(Vec3::X, -std::f32::consts::PI / 2.0));
                    enemy.up = Vec3::Z;
                } 
            }

            if enemy.up == -Vec3::X { // top of head is facing down
                if enemy.forward == Vec3::Y { // forward is facing above
                    result = result.mul_quat(Quat::from_axis_angle(Vec3::X, std::f32::consts::PI / 2.0));
                    enemy.up = -Vec3::Y;
                } 
                if enemy.forward == -Vec3::Y { // forward is facing beneath
                    result = result.mul_quat(Quat::from_axis_angle(Vec3::X, std::f32::consts::PI / 2.0));
                    enemy.up = Vec3::Y;
                } 
                if enemy.forward == -Vec3::Z { // forward is facing left
                    result = result.mul_quat(Quat::from_axis_angle(Vec3::X, std::f32::consts::PI / 2.0));
                    enemy.up = Vec3::Z;
                } 
                if enemy.forward == Vec3::Z { // forward is facing right
                    result = result.mul_quat(Quat::from_axis_angle(Vec3::X, std::f32::consts::PI / 2.0));
                    enemy.up = -Vec3::Z;
                } 
            }

            if enemy.up == -Vec3::Z { // top of head is facing left
                if enemy.forward == Vec3::Y { // forward is above
                    result = result.mul_quat(Quat::from_axis_angle(Vec3::Y, -std::f32::consts::PI / 2.0));
                }
                if enemy.forward == -Vec3::Y { // forward is below
                    result = result.mul_quat(Quat::from_axis_angle(Vec3::Y, std::f32::consts::PI / 2.0));
                }
            }

            if enemy.up == Vec3::Z { // top of head is facing right
                if enemy.forward == Vec3::Y { // forward is above
                    result = result.mul_quat(Quat::from_axis_angle(Vec3::Y, std::f32::consts::PI / 2.0));
                }
                if enemy.forward == -Vec3::Y { // forward is below
                    result = result.mul_quat(Quat::from_axis_angle(Vec3::Y, -std::f32::consts::PI / 2.0));
                }
            }

            enemy.forward = -Vec3::X;
            result 
        }
        Direction::Up => {
            let mut result = start_rotation;

            if enemy.up == Vec3::Y { // top of head is facing above
                if enemy.forward == -Vec3::Z { // forward is left
                    result = result.mul_quat(Quat::from_axis_angle(Vec3::Y, -std::f32::consts::PI / 2.0));
                }
                if enemy.forward == Vec3::Z { // forward is right
                    result = result.mul_quat(Quat::from_axis_angle(Vec3::Y, std::f32::consts::PI / 2.0));
                }
            }

            if enemy.up == -Vec3::Y { // top of head is facing beneath
                if enemy.forward == -Vec3::Z { // forward is left
                    result = result.mul_quat(Quat::from_axis_angle(Vec3::Y, std::f32::consts::PI / 2.0));
                }
                if enemy.forward == Vec3::Z { // forward is right
                    result = result.mul_quat(Quat::from_axis_angle(Vec3::Y, -std::f32::consts::PI / 2.0));
                }
            }

            if enemy.up == Vec3::X { // top of head is facing up
                if enemy.forward == Vec3::Y { // forward is facing above
                    result = result.mul_quat(Quat::from_axis_angle(Vec3::X, std::f32::consts::PI / 2.0));
                    enemy.up = -Vec3::Y;
                } 
                if enemy.forward == -Vec3::Y { // forward is facing beneath
                    result = result.mul_quat(Quat::from_axis_angle(Vec3::X, std::f32::consts::PI / 2.0));
                    enemy.up = Vec3::Y;
                } 
                if enemy.forward == -Vec3::Z { // forward is facing left
                    result = result.mul_quat(Quat::from_axis_angle(Vec3::X, std::f32::consts::PI / 2.0));
                    enemy.up = Vec3::Z;
                } 
                if enemy.forward == Vec3::Z { // forward is facing right
                    result = result.mul_quat(Quat::from_axis_angle(Vec3::X, std::f32::consts::PI / 2.0));
                    enemy.up = -Vec3::Z;
                } 
            }

            if enemy.up == -Vec3::X { // top of head is facing down
                if enemy.forward == Vec3::Y { // forward is facing above
                    result = result.mul_quat(Quat::from_axis_angle(Vec3::X, -std::f32::consts::PI / 2.0));
                    enemy.up = Vec3::Y;
                } 
                if enemy.forward == -Vec3::Y { // forward is facing beneath
                    result = result.mul_quat(Quat::from_axis_angle(Vec3::X, -std::f32::consts::PI / 2.0));
                    enemy.up = -Vec3::Y;
                } 
                if enemy.forward == -Vec3::Z { // forward is facing left
                    result = result.mul_quat(Quat::from_axis_angle(Vec3::X, -std::f32::consts::PI / 2.0));
                    enemy.up = -Vec3::Z;
                } 
                if enemy.forward == Vec3::Z { // forward is facing right
                    result = result.mul_quat(Quat::from_axis_angle(Vec3::X, -std::f32::consts::PI / 2.0));
                    enemy.up = Vec3::Z;
                } 
            }

            if enemy.up == -Vec3::Z { // top of head is facing left
                if enemy.forward == Vec3::Y { // forward is above
                    result = result.mul_quat(Quat::from_axis_angle(Vec3::Y, std::f32::consts::PI / 2.0));
                }
                if enemy.forward == -Vec3::Y { // forward is below
                    result = result.mul_quat(Quat::from_axis_angle(Vec3::Y, -std::f32::consts::PI / 2.0));
                }
            }

            if enemy.up == Vec3::Z { // top of head is facing right
                if enemy.forward == Vec3::Y { // forward is above
                    result = result.mul_quat(Quat::from_axis_angle(Vec3::Y, -std::f32::consts::PI / 2.0));
                }
                if enemy.forward == -Vec3::Y { // forward is below
                    result = result.mul_quat(Quat::from_axis_angle(Vec3::Y, std::f32::consts::PI / 2.0));
                }
            }

            enemy.forward = Vec3::X;
            result 
        }
        Direction::Above => {
            let mut result = start_rotation;

            if enemy.up == Vec3::Y { // top is facing above
                if enemy.forward == Vec3::X    // forward is up
                || enemy.forward == Vec3::Z    // forward is right
                || enemy.forward == -Vec3::X   // forward is down
                || enemy.forward == -Vec3::Z { // forward is left
                    result = result.mul_quat(Quat::from_axis_angle(Vec3::X, std::f32::consts::PI / 2.0));

                    enemy.up = -enemy.forward;
                    enemy.forward = Vec3::Y;
                }
            }
            if enemy.up == -Vec3::Y { // top is facing beneath
                if enemy.forward == Vec3::X    // forward is up
                || enemy.forward == Vec3::Z    // forward is right
                || enemy.forward == -Vec3::X   // forward is down
                || enemy.forward == -Vec3::Z { // forward is left
                    result = result.mul_quat(Quat::from_axis_angle(Vec3::X, -std::f32::consts::PI / 2.0));

                    enemy.up = enemy.forward;
                    enemy.forward = Vec3::Y;
                }
            }

            if enemy.up == -Vec3::Z { // up is left
                if enemy.forward == Vec3::X {   // forward is up
                    result = result.mul_quat(Quat::from_axis_angle(Vec3::Y, -std::f32::consts::PI / 2.0));
                    enemy.forward = Vec3::Y;
                }
                if enemy.forward == -Vec3::X {   // forward is down
                    result = result.mul_quat(Quat::from_axis_angle(Vec3::Y, std::f32::consts::PI / 2.0));
                    enemy.forward = Vec3::Y;
                }
                if enemy.forward == Vec3::Y {   // forward is above
                    // don't do anything ( this would be going forward ) 
                }
                if enemy.forward == -Vec3::Y {   // forward is below
                    // don't do anything ( this would be going backward ) 
                }
            }

            if enemy.up == Vec3::Z { // up is right
                if enemy.forward == Vec3::X {   // forward is up
                    result = result.mul_quat(Quat::from_axis_angle(Vec3::Y, std::f32::consts::PI / 2.0));
                    enemy.forward = Vec3::Y;
                }
                if enemy.forward == -Vec3::X {   // forward is down
                    result = result.mul_quat(Quat::from_axis_angle(Vec3::Y, -std::f32::consts::PI / 2.0));
                    enemy.forward = Vec3::Y;
                }
                if enemy.forward == Vec3::Y {   // forward is above
                    // don't do anything ( this would be going forward ) 
                }
                if enemy.forward == -Vec3::Y {   // forward is below
                    // don't do anything ( this would be going backward ) 
                }
            }

            if enemy.up == -Vec3::X { // top is facing down
                if enemy.forward == Vec3::Z { // forward is right
                    result = result.mul_quat(Quat::from_axis_angle(Vec3::Y, std::f32::consts::PI / 2.0));
                    enemy.forward = Vec3::Y;
                }
                if enemy.forward == -Vec3::Z { // forward is left
                    result = result.mul_quat(Quat::from_axis_angle(Vec3::Y, -std::f32::consts::PI / 2.0));
                    enemy.forward = Vec3::Y;
                }
            }

            if enemy.up == Vec3::X { // top is facing up
                if enemy.forward == Vec3::Z { // forward is right
                    result = result.mul_quat(Quat::from_axis_angle(Vec3::Y, -std::f32::consts::PI / 2.0));
                    enemy.forward = Vec3::Y;
                }
                if enemy.forward == -Vec3::Z { // forward is left
                    result = result.mul_quat(Quat::from_axis_angle(Vec3::Y, std::f32::consts::PI / 2.0));
                    enemy.forward = Vec3::Y;
                }
            }

            result 
        }
        Direction::Beneath => {
            let mut result = start_rotation;

            if enemy.up == Vec3::Y { // top is facing above
                if enemy.forward == Vec3::X    // forward is up
                || enemy.forward == Vec3::Z    // forward is right
                || enemy.forward == -Vec3::X   // forward is down
                || enemy.forward == -Vec3::Z { // forward is left
                    result = result.mul_quat(Quat::from_axis_angle(Vec3::X, -std::f32::consts::PI / 2.0));

                    enemy.up = enemy.forward;
                    enemy.forward = -Vec3::Y;
                }
            }
            if enemy.up == -Vec3::Y { // top is facing beneath
                if enemy.forward == Vec3::X    // forward is up
                || enemy.forward == Vec3::Z    // forward is right
                || enemy.forward == -Vec3::X   // forward is down
                || enemy.forward == -Vec3::Z { // forward is left
                    result = result.mul_quat(Quat::from_axis_angle(Vec3::X, std::f32::consts::PI / 2.0));

                    enemy.up = -enemy.forward;
                    enemy.forward = -Vec3::Y;
                }
            }

            if enemy.up == -Vec3::Z { // up is left
                if enemy.forward == Vec3::X {   // forward is up
                    result = result.mul_quat(Quat::from_axis_angle(Vec3::Y, std::f32::consts::PI / 2.0));
                    enemy.forward = -Vec3::Y;
                }
                if enemy.forward == -Vec3::X {   // forward is down
                    result = result.mul_quat(Quat::from_axis_angle(Vec3::Y, -std::f32::consts::PI / 2.0));
                    enemy.forward = -Vec3::Y;
                }
                if enemy.forward == Vec3::Y {   // forward is above
                    // don't do anything ( this would be going backward ) 
                }
                if enemy.forward == -Vec3::Y {   // forward is below
                    // don't do anything ( this would be going forward ) 
                }
            }

            if enemy.up == Vec3::Z { // up is right
                if enemy.forward == Vec3::X {   // forward is up
                    result = result.mul_quat(Quat::from_axis_angle(Vec3::Y, -std::f32::consts::PI / 2.0));
                    enemy.forward = -Vec3::Y;
                }
                if enemy.forward == -Vec3::X {   // forward is down
                    result = result.mul_quat(Quat::from_axis_angle(Vec3::Y, std::f32::consts::PI / 2.0));
                    enemy.forward = -Vec3::Y;
                }
                if enemy.forward == Vec3::Y {   // forward is above
                    // don't do anything ( this would be going forward ) 
                }
                if enemy.forward == -Vec3::Y {   // forward is below
                    // don't do anything ( this would be going backward ) 
                }
            }

            if enemy.up == -Vec3::X { // top is facing down
                if enemy.forward == Vec3::Z { // forward is right
                    result = result.mul_quat(Quat::from_axis_angle(Vec3::Y, -std::f32::consts::PI / 2.0));
                    enemy.forward = -Vec3::Y;
                }
                if enemy.forward == -Vec3::Z { // forward is left
                    result = result.mul_quat(Quat::from_axis_angle(Vec3::Y, std::f32::consts::PI / 2.0));
                    enemy.forward = -Vec3::Y;
                }
            }

            if enemy.up == Vec3::X { // top is facing up
                if enemy.forward == Vec3::Z { // forward is right
                    result = result.mul_quat(Quat::from_axis_angle(Vec3::Y, std::f32::consts::PI / 2.0));
                    enemy.forward = -Vec3::Y;
                }
                if enemy.forward == -Vec3::Z { // forward is left
                    result = result.mul_quat(Quat::from_axis_angle(Vec3::Y, -std::f32::consts::PI / 2.0));
                    enemy.forward = -Vec3::Y;
                }
            }

            result 
        }
    }
}
