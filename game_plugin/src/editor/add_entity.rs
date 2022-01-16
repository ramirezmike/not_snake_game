use crate::{dude, editor::EditorTrashMarker, editor::GameEntity, editor::GameEntityType, snake};
use bevy::prelude::*;
use bevy_mod_picking::*;

pub fn add_block(
    commands: &mut Commands,
    mut meshes: &mut ResMut<Assets<Mesh>>,
    mut materials: &mut ResMut<Assets<StandardMaterial>>,
    position: &Vec3,
) {
    commands
        .spawn_bundle(PbrBundle {
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

#[derive(Component)]
pub struct SnakeProperties {
    color: Color,
}

pub fn add_snake(
    commands: &mut Commands, 
    meshes: &ResMut<snake::EnemyMeshes>, 
    materials: &mut ResMut<Assets<StandardMaterial>>,
    position: &Vec3
) {
    let mut transform = Transform::from_translation(*position);
    transform.apply_non_uniform_scale(Vec3::new(0.50, 0.50, 0.50));
    transform.rotate(Quat::from_axis_angle(
        Vec3::new(0.0, 1.0, 0.0),
        std::f32::consts::FRAC_PI_2,
    ));
    let snake_color = Color::hex("ff4f69").unwrap();

    commands
        .spawn_bundle(PbrBundle {
            transform,
            ..Default::default()
        })
        .insert(EditorTrashMarker)
        .with_children(|parent| {
            let parent_entity = parent.parent_entity();
            parent
                .spawn_bundle(PbrBundle {
                    transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
                    ..Default::default()
                })
                .with_children(|inner_parent| {
                    inner_parent.spawn_bundle(PbrBundle {
                        mesh: meshes.head.clone(),
                        material: materials.add(snake_color.into()),
                        ..Default::default()
                    })
                    .insert(GameEntity {
                        entity_type: GameEntityType::Snake,
                    })
                    .insert(SnakeProperties {
                        color: snake_color.clone(),
                    })
                    .insert_bundle(PickableBundle::default());
                });
        });
}

#[derive(Component)]
pub struct NotSnakeProperties {
    color: Color,
}

pub fn add_not_snake(
    commands: &mut Commands,
    mut meshes: &mut ResMut<dude::DudeMeshes>,
    mut materials: &mut ResMut<Assets<StandardMaterial>>,
    position: &Vec3,
) {
    let mut transform = Transform::from_translation(Vec3::new(position.x, position.y, position.z));
    transform.apply_non_uniform_scale(Vec3::new(dude::SCALE, dude::SCALE, dude::SCALE));
    transform.rotate(Quat::from_axis_angle(
        Vec3::new(0.0, 1.0, 0.0),
        std::f32::consts::PI,
    ));

    let not_snake_color = Color::hex("f3a787").unwrap();

    commands
        .spawn_bundle(PbrBundle {
            transform,
            ..Default::default()
        })
        .insert(EditorTrashMarker)
        .with_children(|parent| {
            parent
                .spawn_bundle(PbrBundle {
                    mesh: meshes.body.clone(),
                    material: materials.add(not_snake_color.into()),
                    transform: {
                        let mut transform = Transform::from_rotation(Quat::from_axis_angle(
                            Vec3::new(0.0, 1.0, 0.0),
                            (3.0 * std::f32::consts::PI) / 2.0,
                        ));

                        transform.translation = Vec3::new(0.0, 0.0, 0.0);
                        transform
                    },
                    ..Default::default()
                })
                .insert(GameEntity {
                    entity_type: GameEntityType::NotSnake,
                })
                .insert(NotSnakeProperties {
                    color: not_snake_color.clone(),
                })
                .insert_bundle(PickableBundle::default());
        });
}
