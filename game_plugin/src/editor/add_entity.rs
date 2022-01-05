use bevy::prelude::*;
use bevy_mod_picking::*;
use crate::{dude, snake, editor::GameEntity, editor::GameEntityType, editor::EditorTrashMarker};

pub fn add_block(
    commands: &mut Commands,
    mut meshes: &mut ResMut<Assets<Mesh>>,
    mut materials: &mut ResMut<Assets<StandardMaterial>>,
    position: &Vec3,
) {
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
        transform: Transform::from_xyz(position.x, position.y, position.z),
        ..Default::default()
    })
    .insert(EditorTrashMarker)
    .insert(GameEntity {
        entity_type: GameEntityType::Block,
    })
    .insert_bundle(PickableBundle::default());
}

pub fn add_snake(
    commands: &mut Commands, 
    meshes: &ResMut<snake::EnemyMeshes>, 
    position: &Vec3,
) {
    let mut transform = Transform::from_translation(*position);
    transform.apply_non_uniform_scale(Vec3::new(0.50, 0.50, 0.50)); 
    transform.rotate(Quat::from_axis_angle(Vec3::new(0.0, 1.0, 0.0), std::f32::consts::FRAC_PI_2));

    commands.spawn_bundle(PbrBundle {
                transform,
                ..Default::default()
            })
            .insert(GameEntity {
                entity_type: GameEntityType::Snake,
            })
            .insert(EditorTrashMarker)
            .insert_bundle(PickableBundle::default())
            .with_children(|parent|  {
                let parent_entity = parent.parent_entity();
                parent.spawn_bundle(PbrBundle {
                    transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
                    ..Default::default()
                })
                .with_children(|inner_parent| {
                    inner_parent.spawn_bundle(PbrBundle {
                        mesh: meshes.head.clone(),
                        material: meshes.material.clone(),
                        ..Default::default()
                    });
                });
            });
}

pub fn add_not_snake(
    commands: &mut Commands,
    mut meshes: &mut ResMut<dude::DudeMeshes>,
    position: &Vec3,
) {
    let mut transform = Transform::from_translation(Vec3::new(position.x, position.y, position.z));
    transform.apply_non_uniform_scale(Vec3::new(dude::SCALE, dude::SCALE, dude::SCALE)); 
    transform.rotate(Quat::from_axis_angle(Vec3::new(0.0, 1.0, 0.0), std::f32::consts::PI));

    commands.spawn_bundle(PbrBundle {
                transform,
                ..Default::default()
            })
            .insert(GameEntity {
                entity_type: GameEntityType::NotSnake,
            })
            .insert_bundle(PickableBundle::default())
            .insert(EditorTrashMarker)
            .with_children(|parent|  {
                parent.spawn_bundle(PbrBundle {
                    mesh: meshes.body.clone(),
                    material: meshes.material.clone(),
                    transform: {
                        let mut transform = Transform::from_rotation(Quat::from_axis_angle(Vec3::new(0.0, 1.0, 0.0), 
                                (3.0 * std::f32::consts::PI) / 2.0));

                        transform.translation = Vec3::new(0.0, 0.0, 0.0);
                        transform
                    },
                    ..Default::default()
                });
            });
}