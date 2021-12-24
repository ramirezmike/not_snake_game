use bevy::prelude::*;
use crate::{level, level::Level, Position, EntityType, GameObject, dude, snake};

#[derive(Component)]
pub struct Food { 
    pub is_bonus: bool
}
#[derive(Component)]
pub struct FoodInnerMesh { }
pub struct FoodEatenEvent(pub Entity, pub bool);
#[derive(Component)]
pub struct FoodSpawnParticle {
    parent: Entity,
    shadow_id: Option::<Entity>,
    starting_from: Vec3,
    current_movement_time: f32,
    finish_movement_time: f32,
}

pub fn spawn_food(
    commands: &mut Commands,
    level: &mut ResMut<Level>,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    position: Option::<Position>,
    show_shadow: bool,
    is_bonus: bool,
) -> Entity {
    let show_shadow = false;
    let bonus_food_color = Color::hex("97D8B2").unwrap(); 
    let bonus_food_color = Color::rgba(bonus_food_color.r(), bonus_food_color.g(), bonus_food_color.b(), 1.0); 
    let shaded_bonus_food_color = Color::rgba(bonus_food_color.r(), bonus_food_color.g(), bonus_food_color.b(), 0.4);

    let food_color = Color::hex(level.get_palette().food.clone()).unwrap();
    let food_color = Color::rgba(food_color.r(), food_color.g(), food_color.b(), 1.0);
    let shaded_food_color = Color::rgba(food_color.r(), food_color.g(), food_color.b(), 0.4);

    let position = if position.is_some() { position.unwrap() } else { level.get_random_standable(&None, false) };
    let transform = Transform::from_xyz(position.x as f32, position.y as f32, position.z as f32);
    let cube = meshes.add(Mesh::from(shape::Cube { size: 0.1 }));
    let food_id = 
        commands.spawn_bundle(PbrBundle {
          transform,
          ..Default::default()
        })
        .with_children(|parent| {
            let inner_mesh_id = 
                parent.spawn_bundle(PbrBundle {
                    mesh: meshes.add(Mesh::from(shape::Icosphere { radius: 0.25, subdivisions: 0 })),
                    material: {
                        if is_bonus {
                            materials.add(bonus_food_color.into())
                        } else {
                            materials.add(food_color.into())
                        }
                    },
                    transform: Transform::from_xyz(0.0, 0.5, 0.0),
                    visibility: Visibility {
                        is_visible: false
                    },
                    ..Default::default()
                })
                .insert(FoodInnerMesh {})
                .id();

            let mut shadow_mesh_id = None;
            if show_shadow {
                shadow_mesh_id =
                    Some(
                        parent.spawn_bundle(PbrBundle {
                            mesh: meshes.add(Mesh::from(shape::Plane { size: 0.25 })),
                            material: {
                                if is_bonus {
                                    materials.add(shaded_bonus_food_color.into())
                                } else {
                                    materials.add(shaded_food_color.into())
                                }
                            },
                            transform: Transform::from_xyz(0.0, 0.05, 0.0),
                            visibility: Visibility {
                                is_visible: false
                            },
                            ..Default::default()
                        })
                        .insert(FoodInnerMesh {})
                        .id());
            }

            let distance = 10.0;
            let particle_positions = vec!(
                (0.0, distance, 0.0),
                (0.0, -distance, 0.0),

                (distance, 0.0, 0.0),
                (-distance, 0.0, 0.0),
                (distance, distance, distance),
                (-distance, distance, distance),
                (distance, -distance, distance),
                (-distance, -distance, distance),
                (distance, distance, -distance),
                (-distance, distance, -distance),
                (distance, -distance, -distance),
                (-distance, -distance, -distance),

                (0.0, 0.0, distance),
                (0.0, 0.0, -distance),
            );
            for particle_position in particle_positions.iter() {
                let transform = Transform::from_xyz(particle_position.0, particle_position.1 + 0.5, particle_position.2);
                let starting_from = transform.translation.clone();
                parent.spawn_bundle(PbrBundle {
                    mesh: cube.clone(),
                    material: {
                        if is_bonus {
                            materials.add(bonus_food_color.into())
                        } else {
                            materials.add(food_color.into())
                        }
                    },
                    transform,
                    ..Default::default()
                })
                .insert(FoodSpawnParticle { 
                            parent: inner_mesh_id,
                            starting_from,
                            current_movement_time: 0.0,
                            finish_movement_time: 0.5,
                            shadow_id: shadow_mesh_id 
                        });
            }
        })
        .insert(Food {
            is_bonus
        })
        .insert(EntityType::Food)
        .insert(position)
        .id();

    level.set_with_position(position, Some(GameObject::new(food_id, EntityType::Food)));

    food_id
}

pub fn animate_spawn_particles(
    mut commands: Commands,
    mut particles: Query<(Entity, &mut Transform, &mut FoodSpawnParticle)>,
    mut food_visibles: Query<&mut Visibility, With<FoodInnerMesh>>,
    time: Res<Time>,
) {
    let target = Vec3::new(0.0, 0.5, 0.0);
    for (entity, mut transform, mut particle) in particles.iter_mut() {
        if particle.current_movement_time >= particle.finish_movement_time {
            if let Ok(mut visible) = food_visibles.get_mut(particle.parent) {
                visible.is_visible = true;
            }
            if let Some(shadow_entity) = particle.shadow_id {
                if let Ok(mut visible) = food_visibles.get_mut(shadow_entity) {
                    visible.is_visible = true;
                }
            }

            commands.entity(entity).despawn_recursive();
        } else {
            particle.current_movement_time += time.delta_seconds();

            let new_translation = particle.starting_from.lerp(target, 
                                                              particle.current_movement_time / particle.finish_movement_time);
            if !new_translation.is_nan() {
                if transform.translation.distance(target) < transform.translation.distance(new_translation) {
                    transform.translation = target;
                    particle.current_movement_time = particle.finish_movement_time;
                } else {
                    transform.translation = new_translation;
                }
            }
        }
    }
}

pub fn animate_food(
    mut foods: Query<&mut Transform, With<FoodInnerMesh>>,
    time: Res<Time>,
) {
    for mut transform in foods.iter_mut() {
        transform.rotate(Quat::from_rotation_y(time.delta_seconds()));
        transform.rotate(Quat::from_rotation_x(time.delta_seconds()));
    }
}

pub fn handle_food_eaten(
    mut commands: Commands,
    mut food_eaten_event_reader: EventReader<FoodEatenEvent>,
    mut level: ResMut<Level>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    dudes: Query<&Position, With<dude::Dude>>,
    snakes: Query<&Position, (With<snake::Snake>, Without<snake::SnakeBody>)>,
) {
    let mut away_froms = None;
    for event in food_eaten_event_reader.iter() {
        if level.is_food_random() && !event.1 {
            if away_froms.is_none() {
                let mut positions = Vec::new();
                for dude_position in dudes.iter() {
                    positions.push(dude_position.clone());
                }

                for snake_position in snakes.iter() {
                    positions.push(snake_position.clone());
                }

                println!("Positions: {:?}", positions.len());
                away_froms = Some(positions);
            }

            let new_position = level.get_random_standable(&away_froms, false);
            spawn_food(&mut commands, &mut level, &mut meshes, &mut materials, Some(new_position), true, false);
        } 
    }

}

pub fn update_food(
    mut commands: Commands,
    mut foods: Query<(Entity, &Position, &Food)>,
    mut position_change_event_reader: EventReader<level::PositionChangeEvent>,
    mut food_eaten_event_writer: EventWriter<FoodEatenEvent>,
) {
    for position_change in position_change_event_reader.iter() {
        if let Some(game_object) = position_change.1 {
            for (entity, position, food) in foods.iter_mut() {
                if position_change.0 == *position && game_object.entity != entity {
                    commands.entity(entity).despawn_recursive();

                    food_eaten_event_writer.send(FoodEatenEvent(game_object.entity, food.is_bonus));
                }
            }
        }
    }
}
