use crate::{
    dude, editor::properties, editor::EditorTrashMarker, editor::GameEntity,
    editor::GameEntityType, snake,
};
use bevy::prelude::*;
use bevy_mod_picking::*;

pub fn add_block(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    properties: &Res<properties::Properties>,
    position: &Vec3,
) {
    let color = Color::rgb(
        properties.block.color[0],
        properties.block.color[1],
        properties.block.color[2],
    );
    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            material: materials.add(color.into()),
            transform: Transform::from_xyz(position.x, position.y, position.z),
            ..Default::default()
        })
        .insert(EditorTrashMarker)
        .insert(properties.block.clone())
        .insert(GameEntity {
            entity_type: GameEntityType::Block,
        })
        .insert_bundle(PickableBundle::default());
}

pub fn add_food(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    properties: &Res<properties::Properties>,
    position: &Vec3,
) {
    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Icosphere {
                radius: 0.25,
                subdivisions: 0,
            })),
            material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
            transform: Transform::from_xyz(position.x, position.y, position.z),
            ..Default::default()
        })
        .insert(EditorTrashMarker)
        .insert(properties.food.clone())
        .insert(GameEntity {
            entity_type: GameEntityType::Food,
        })
        .insert_bundle(PickableBundle::default());
}

pub fn add_snake(
    commands: &mut Commands,
    meshes: &ResMut<snake::EnemyMeshes>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    properties: &Res<properties::Properties>,
    position: &Vec3,
) {
    let mut transform = Transform::from_translation(*position);
    transform.apply_non_uniform_scale(Vec3::new(0.50, 0.50, 0.50));
    transform.rotate(Quat::from_axis_angle(
        Vec3::new(0.0, 1.0, 0.0),
        std::f32::consts::FRAC_PI_2,
    ));

    let color = properties.snake.color;
    let color = Color::rgb(color[0], color[1], color[2]);

    commands
        .spawn_bundle(PbrBundle {
            transform,
            mesh: meshes.head.clone(),
            material: materials.add(color.into()),
            ..Default::default()
        })
        .insert(EditorTrashMarker)
        .insert(GameEntity {
            entity_type: GameEntityType::Snake,
        })
        .insert(properties.snake.clone())
        .insert_bundle(PickableBundle::default());
}

pub fn add_not_snake(
    commands: &mut Commands,
    meshes: &mut ResMut<dude::DudeMeshes>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    properties: &Res<properties::Properties>,
    position: &Vec3,
) {
    let mut transform = Transform::from_translation(Vec3::new(position.x, position.y, position.z));
    transform.apply_non_uniform_scale(Vec3::new(dude::SCALE, dude::SCALE, dude::SCALE));

    let color = properties.not_snake.color;
    let color = Color::rgb(color[0], color[1], color[2]);

    commands
        .spawn_bundle(PbrBundle {
            transform,
            mesh: meshes.not_snake.clone(),
            material: materials.add(color.into()),
            ..Default::default()
        })
        .insert(EditorTrashMarker)
        .insert(GameEntity {
            entity_type: GameEntityType::NotSnake,
        })
        .insert(properties.not_snake.clone())
        .insert_bundle(PickableBundle::default());
}
