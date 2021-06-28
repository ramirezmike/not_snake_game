use bevy::prelude::*;
use crate::{level, level::Level, Direction, Position, EntityType, GameObject, environment::HudFoodMesh};
use rand::{thread_rng, Rng};

pub struct Dust {
    pub move_toward: Vec3,
}

pub struct CreateDustEvent {
    pub position: Position,
    pub move_away_from: Direction,
}

pub fn animate_dust(
    mut commands: Commands,
    mut dusts: Query<(&Dust, &mut Transform, &Handle<StandardMaterial>, &Parent)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    time: Res<Time>,
) {
    for (dust, mut transform, material, parent) in dusts.iter_mut() {
        transform.rotate(Quat::from_rotation_x(time.delta_seconds()));
        transform.rotate(Quat::from_rotation_y(time.delta_seconds()));
        transform.scale *= 1.0 - (time.delta_seconds() * 0.9);

        let target = transform.translation.lerp(dust.move_toward, time.delta_seconds() * 0.2);
        if !target.is_nan() {
            transform.translation = target;
        }

        transform.translation.y += time.delta_seconds() * 0.5;

        let mut despawn_entity = true; // if the material doesn't exist, just despawn
        if let Some(material) = materials.get_mut(material) {
            let a = material.base_color.a();
            if a > 0.0 {
                despawn_entity = false;
                material.base_color.set_a(a - (time.delta_seconds() * 1.25));
            }
        } 

        if despawn_entity {
            commands.entity(**parent).despawn_recursive();
        }
    }
}

pub fn handle_create_dust_event(
    mut commands: Commands,
    mut create_dust_event_reader: EventReader<CreateDustEvent>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    ground_materials: Query<&Handle<StandardMaterial>>,
    level: Res<Level>,
) {
    for event in create_dust_event_reader.iter() {
        let position =  event.position;

        let ground = level.get(position.x, position.y - 1, position.z);
        if let Some(ground) = ground {
            if ground.entity_type != EntityType::Block
            && ground.entity_type != EntityType::Platform 
            && ground.entity_type != EntityType::UnstandableBlock {
                continue;
            }

            let mut color = None;
            if let Ok(ground_material) = ground_materials.get(ground.entity) {
                if let Some(material) = materials.get_mut(ground_material) {
                    color = Some(material.base_color.clone());
                }
            }
            
            if color.is_none() {
                continue;
            }

            let color = color.unwrap();
            let transform = Transform::from_xyz(position.x as f32, position.y as f32, position.z as f32);

            for _ in 0..3 {
                let inner_mesh_x = thread_rng().gen_range(-25..25) as f32 / 100.0;
                let inner_mesh_z = thread_rng().gen_range(-25..25) as f32 / 100.0;

                let color = Color::rgba(color.r(), color.g(), color.b(), 0.7 + inner_mesh_x.abs());
                let move_toward = match event.move_away_from {
                                      Direction::Up => Vec3::new(-1.0, 0.0, 0.0),
                                      Direction::Down => Vec3::new(1.0, 0.0, 0.0),
                                      Direction::Left => Vec3::new(0.0, 0.0, 1.0),
                                      Direction::Right => Vec3::new(0.0, 0.0, -1.0),
                                      _ => Vec3::new(0.0, 0.0, 0.0),
                                  };

                commands.spawn_bundle(PbrBundle {
                  transform,
                  ..Default::default()
                })
                .with_children(|parent| {
                    parent.spawn_bundle(PbrBundle {
                        mesh: meshes.add(Mesh::from(shape::Cube { size: 0.15 })),
                        material: materials.add(color.into()),
                        transform: {
                            let mut t = Transform::from_xyz(inner_mesh_x, 0.1, inner_mesh_z);
                            t.rotate(Quat::from_rotation_x(inner_mesh_z));
                            t.rotate(Quat::from_rotation_y(inner_mesh_x));
                            t
                        },
                        visible: Visible {
                            is_visible: true,
                            is_transparent: true,
                        },
                        ..Default::default()
                    })
                    .insert(Dust { move_toward });
                });
            }
        }
    }
}
