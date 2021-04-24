use bevy::prelude::*;
use crate::{level, level::Level, Position, EntityType, GameObject};

pub struct Food { }
pub struct FoodInnerMesh { }
pub struct FoodEatenEvent(pub Entity);

pub fn spawn_food(
    mut commands: Commands,
    mut level: ResMut<Level>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let food_color = Color::hex(crate::COLOR_FOOD).unwrap();
    let food_color = Color::rgba(food_color.r(), food_color.g(), food_color.b(), 1.0);
    let position = level.get_random_standable();
    let transform = Transform::from_xyz(position.x as f32, position.y as f32, position.z as f32);
    let food_id = 
        commands.spawn_bundle(PbrBundle {
          transform,
          ..Default::default()
        })
        .with_children(|parent| {
            parent.spawn_bundle(PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Icosphere { radius: 0.25, subdivisions: 0 })),
                material: materials.add(food_color.into()),
                transform: Transform::from_xyz(0.0, 0.5, 0.0),
                ..Default::default()
            })
            .insert(FoodInnerMesh {});
        })
        .insert(Food {})
        .insert(EntityType::Food)
        .insert(position)
        .id();

    level.set_with_position(position, Some(GameObject::new(food_id, EntityType::Food)));
}

pub fn animate_food(
    mut foods: Query<&mut Transform, With<FoodInnerMesh>>,
    time: Res<Time>,
) {
    for mut transform in foods.iter_mut() {
//        transform.translation.y = 0.5 + (0.2 * time.seconds_since_startup().sin() as f32);
        transform.rotate(Quat::from_rotation_y(time.delta_seconds()));
//        transform.rotate(Quat::from_rotation_z(time.delta_seconds()));
//        transform.rotate(Quat::from_rotation_x(time.delta_seconds()));
    }
}

pub fn update_food(
    mut foods: Query<(Entity, &mut Transform, &mut Position), With<Food>>,
    mut position_change_event_reader: EventReader<level::PositionChangeEvent>,
    mut food_eaten_event_writer: EventWriter<FoodEatenEvent>,
    mut level: ResMut<Level>,
) {
    for position_change in position_change_event_reader.iter() {
        if let Some(game_object) = position_change.1 {
            for (entity, mut transform, mut position) in foods.iter_mut() {
                if position_change.0 == *position && game_object.entity != entity {
                    *position = level.get_random_standable();
                    *transform = Transform::from_xyz(position.x as f32, 
                                                     position.y as f32, 
                                                     position.z as f32);
                    level.set_with_position(*position, Some(GameObject::new(entity, EntityType::Food)));
                    food_eaten_event_writer.send(FoodEatenEvent(game_object.entity));
                }
            }
        }
    }
}
