use bevy::prelude::*;
use crate::{Direction, EntityType, GameObject, level::Level, path_find::PathFinder, level,
            environment, Position, holdable, moveable, facing::Facing, food::FoodEatenEvent};

#[derive(Default)]
pub struct EnemyMeshes {
    pub head: Handle<Mesh>,
    pub body: Handle<Mesh>,
    pub shadow: Handle<Mesh>,
    pub material: Handle<StandardMaterial>,
    pub shadow_material: Handle<StandardMaterial>,
}

pub struct BodyPosition {
    translation: Vec3,
    rotation: Quat,
}

pub struct Enemy {
    body_parts: Vec::<Entity>,
    body_positions: Vec::<BodyPosition>,
    action_cooldown: Timer, 
    speed: f32,
    movement: Option::<SnakeMovement>,
    pub is_dead: bool,
    up: Vec3,
    forward: Vec3,
}

pub struct Snake;
pub struct SnakeBody;
pub struct SnakeInnerMesh;
pub struct Follow {
    speed: f32,
    is_active: bool,
    movement: Option::<SnakeMovement>,
    target_position: Option::<Vec3>,
    target_rotation: Option::<Quat>,
}
pub struct KillSnakeEvent(pub Entity);
struct SnakeMovement {
    facing: Direction,
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
    meshes: &Res<EnemyMeshes>, 
    transform: Transform,
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
                    transform: {
                        let mut t = Transform::from_translation(Vec3::new(0.0, INNER_MESH_VERTICAL_OFFSET, 0.0));
                        //t.rotate();
                        t
                    },
                    ..Default::default()
                }).insert(SnakeInnerMesh)
                .with_children(|inner_parent| {
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
    meshes: &Res<EnemyMeshes>, 
    level: &mut ResMut<Level>,
    x: usize,
    y: usize,
    z: usize,
) {
    let position = Vec3::new(x as f32, y as f32, z as f32);
    let mut transform = Transform::from_translation(position);
    transform.apply_non_uniform_scale(Vec3::new(0.50, 0.50, 0.50)); 
    transform.rotate(Quat::from_axis_angle(Vec3::new(0.0, 1.0, 0.0), std::f32::consts::FRAC_PI_2));

    let body_part_entity = generate_snake_body(commands, meshes, transform);
        
    let enemy_entity = 
    commands.spawn_bundle(PbrBundle {
                transform,
                ..Default::default()
            })
            .insert(Position { x: position.x as i32, y: position.y as i32, z: position.z as i32 })
            .insert(EntityType::Enemy)
            .insert(Snake)
            .insert(Enemy {
                body_parts: vec![body_part_entity],
                body_positions: vec![
                    BodyPosition { translation: Vec3::new(transform.translation.x + 1.0, 
                                                          transform.translation.y, 
                                                          transform.translation.z), 
                                   rotation: Quat::IDENTITY },
                ],
                action_cooldown: Timer::from_seconds(0.5, true),
                speed: 0.5,
                movement: None,
                is_dead: false,
                up: Vec3::Y,
                forward: -Vec3::X,
            })
//            .insert(moveable::Moveable::new(false, false, 0.1, inner_mesh_vertical_offset))
            .insert(Facing::new(Direction::Down, true))
            .with_children(|parent|  {
                parent.spawn_bundle(PbrBundle {
                    mesh: meshes.head.clone(),
                    material: meshes.material.clone(),
                    transform: Transform::from_translation(Vec3::new(0.0, INNER_MESH_VERTICAL_OFFSET, 0.0)),
                    ..Default::default()
                })
                .insert(SnakeInnerMesh);

//              parent.spawn_bundle(PbrBundle {
//                  mesh: meshes.shadow.clone(),
//                  material: meshes.shadow_material.clone(),
//                  transform: Transform::from_xyz(0.0, -0.25, 0.0),
//                  ..Default::default()
//              })
//              .insert(environment::Shadow);
            }).id();
    level.set(position.x as i32, position.y as i32, position.z as i32, Some(GameObject::new(enemy_entity, EntityType::Enemy)));
    level.set(position.x as i32 + 1, position.y as i32, position.z as i32, Some(GameObject::new(enemy_entity, EntityType::Enemy)));
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
    mut snake_enemies: Query<(&mut Enemy, &Transform)>,
    mut commands: Commands, 
    meshes: Res<EnemyMeshes>, 
    mut level: ResMut<Level>,
) {
    for part_to_add in body_part_reader.iter() {
        let (mut snake_enemy, snake_transform) = snake_enemies.get_mut(part_to_add.snake).unwrap();

        let last_position = snake_enemy.body_positions.last().unwrap();
        let mut transform = Transform::from_translation(last_position.translation);
        transform.rotate(Quat::from_axis_angle(Vec3::new(0.0, 1.0, 0.0), std::f32::consts::FRAC_PI_2));
        transform.apply_non_uniform_scale(Vec3::new(0.50, 0.50, 0.50)); 

        let body_part_entity = generate_snake_body(&mut commands, &meshes, transform);
        snake_enemy.body_parts.push(body_part_entity);
    }

//    *queued_parts = waiting;
    *queued_parts = vec!();
}

pub fn update_enemy(
    time: Res<Time>,
    mut enemies: Query<(Entity, &mut Enemy, &mut Transform, &mut Position, &Children), (Without<Follow>, Without<SnakeBody>, Without<SnakeInnerMesh>)>,
    mut inner_meshes: Query<&mut Transform, (With<SnakeInnerMesh>, Without<Follow>)>,
    mut follows: Query<(&mut Follow, &mut Transform), (Without<Enemy>, Without<SnakeInnerMesh>)>,
//    mut snake_bodies: Query<&Transform, With<SnakeBody>>,
    path_find: Res<PathFinder>,
    mut level: ResMut<Level>,

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
            if enemy.movement.is_none() {
                let is_ai_controlled = true;
                let mut new_target = None;
                if is_ai_controlled  {
                    let (_, path) = path_find.get_path();
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
//                    println!("AXIS ANGLE: {:?}",start_rotation.to_axis_angle());
//                    println!("Is Identity: {:?}",start_rotation.is_near_identity()); 
                    let is_identity=start_rotation.is_near_identity();
/*
                    let is_180_degrees = |a| a <= (7.0 * std::f32::consts::PI) / 6.0 && a >= (5.0 * std::f32::consts::PI) / 6.0;
                    let is_90_degrees = |a| a <= (2.0 * std::f32::consts::PI) / 3.0 && a >= std::f32::consts::PI / 3.0;
                    let is_0_degrees = |a| a <= std::f32::consts::PI / 6.0 || a >= (11.0 * std::f32::consts::PI) / 6.0;
                    let is_270_degrees = |a:f32| a <= (5.0 * std::f32::consts::PI) / 3.0 && a >= (4.0 * std::f32::consts::PI) / 3.0;
*/


                    // keep building this you can do it! 
                    let target_rotation = 
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
                        };

                  //println!("---------------------------------------------------");
                  //println!("Up: {:?} Forward: {:?}", enemy.up, enemy.forward);


                    enemy.movement = Some(SnakeMovement { 
                                        target, 
                                        facing, 
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

            if let Some(movement) = &mut enemy.movement {
                // if the spot the snake moved from is the same object then clear it
                // eventually we want to handle this separately so that we are just
                // updating all parts of the current snake in the level at once
                if let Some(game_object) = level.get_with_vec(transform.translation) {
                    if game_object.entity == entity {
                        level.set_with_vec(transform.translation, None);
                    }
                }

                if movement.current_movement_time >= movement.finish_movement_time {
                    if level.is_enterable_with_vec(movement.target) {
                        transform.translation = movement.target;
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
                level.set_with_vec(transform.translation, Some(GameObject::new(entity, EntityType::Enemy)));
                position.update_from_vec(transform.translation);
            }
        }
    }
}

pub fn update_following(
    snakes: Query<&Enemy>,
    mut body_parts: Query<(Entity, &mut Transform, &Children), (With<SnakeBody>, Without<Enemy>)>,
    mut inner_meshes: Query<&mut Transform, (With<SnakeInnerMesh>, Without<SnakeBody>, Without<Enemy>)>,
    time: Res<Time>,
    mut level: ResMut<Level>,
) {
    for snake in snakes.iter() {
        let mut part_index = 0;
        //println!("Parts: {} Positions: {}",snake.body_parts.len(), snake.body_positions.len());
        for body_part in snake.body_parts.iter() {
            if let Ok((entity, mut transform, children)) = body_parts.get_mut(*body_part) {
                if let Some(target) = &snake.body_positions.get(part_index) {
                    let rate = snake.speed / 1.0;
                    let distance = transform.translation.distance(target.translation);
                    let new_translation = transform.translation.lerp(target.translation, time.delta_seconds() / (distance * rate));
                    if !new_translation.is_nan() {
                        if transform.translation.distance(target.translation) < transform.translation.distance(new_translation) {
                            transform.translation = target.translation;
                        } else {
                            transform.translation = new_translation;
                        }
                    } 
                    level.set_with_vec(target.translation, Some(GameObject::new(entity, EntityType::Enemy)));


                    for child in children.iter() {
                        if let Ok(mut transform) = inner_meshes.get_mut(*child) {
                            let rotation_rate = (snake.speed * 0.5) / 1.0;
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
            if let Ok(position) = snake_part_positions.get(*body_part) {
                level.set(position.x, position.y, position.z, None);
            }
        }
        if let Ok(position) = snake_part_positions.get(snake_entity.0) {
            level.set(position.x, position.y, position.z, None);
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
            level.set(position.x, position.y, position.z, None);

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
