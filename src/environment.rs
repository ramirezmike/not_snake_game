use bevy::prelude::*;

use crate::{level::Level, Position, Direction, 
            EntityType, GameObject, holdable};

pub struct EnvironmentPlugin;
impl Plugin for EnvironmentPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.insert_resource(Level {
               width: 6,
               length: 12,
               height: 12,
               game_objects: vec![vec![vec![None; 12]; 12]; 6],
           })
           .add_startup_system(create_environment.system())
           .add_event::<holdable::LiftHoldableEvent>()
           .add_system(holdable::lift_holdable.system())
           .add_system(update_held_blocks.system())
           .add_system(update_box.system())
           .add_system(crate::level::sync_level.system());
    }
}

pub fn create_environment(
    mut commands: Commands,
    mut level: ResMut<Level>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mesh = meshes.add(Mesh::from(shape::Plane { size: 1.0 }));

    for i in 0..level.width {
        for j in 0..level.length {
            commands.spawn_bundle(PbrBundle {
                mesh: mesh.clone(),
                material: if (i + j + 1) % 2 == 0 { 
                              materials.add(Color::hex(crate::COLOR_GROUND_1).unwrap().into())
                          } else {
                              materials.add(Color::hex(crate::COLOR_GROUND_2).unwrap().into())
                          },
                transform: Transform::from_translation(Vec3::new(i as f32, 0.0, j as f32)),
                ..Default::default()
            });
        }
    }

    commands.spawn_bundle(LightBundle {
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..Default::default()
    });

    let block_entity =
        commands.spawn_bundle(PbrBundle {
          transform: Transform::from_xyz(2.0, 0.0, 5.0),
          ..Default::default()
        })
        .with_children(|parent| {
            parent.spawn_bundle(PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
                material: materials.add(Color::hex(crate::COLOR_BOX).unwrap().into()),
                transform: Transform::from_xyz(0.0, 0.5, 0.0),
                ..Default::default()
            });
        })
        .insert(EntityType::Block)
        .insert(holdable::Holdable {})
        .insert(Position { x: 2, y: 0, z: 5 })
        .insert(BoxObject { target: None })
        .id();
    level.set(2, 0, 5, Some(GameObject::new(block_entity, EntityType::Block)));

    let block_entity =
        commands.spawn_bundle(PbrBundle {
          transform: Transform::from_xyz(1.0, 0.0, 5.0),
          ..Default::default()
        })
        .with_children(|parent| {
            parent.spawn_bundle(PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
                material: materials.add(Color::hex(crate::COLOR_BOX).unwrap().into()),
                transform: Transform::from_xyz(0.0, 0.5, 0.0),
                ..Default::default()
            });
        })
        .insert(EntityType::Block)
        .insert(holdable::Holdable {})
        .insert(Position { x: 1, y: 0, z: 5 })
        .insert(BoxObject { target: None })
        .id();
    level.set(1, 0, 5, Some(GameObject::new(block_entity, EntityType::Block)));

    let block_entity =
        commands.spawn_bundle(PbrBundle {
          transform: Transform::from_xyz(1.0, 0.0, 6.0),
          ..Default::default()
        })
        .with_children(|parent| {
            parent.spawn_bundle(PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
                material: materials.add(Color::hex(crate::COLOR_BOX).unwrap().into()),
                transform: Transform::from_xyz(0.0, 0.5, 0.0),
                ..Default::default()
            });
        })
        .insert(EntityType::Block)
        .insert(holdable::Holdable {})
        .insert(Position { x: 1, y: 0, z: 6 })
        .insert(BoxObject { target: None })
        .id();
    level.set(1, 0, 6, Some(GameObject::new(block_entity, EntityType::Block)));
}

pub struct BoxObject { 
    pub target: Option::<Direction>,
}

pub struct BeingHeld {
    pub held_by: Entity
}

fn update_held_blocks(
    mut boxes: Query<(&BoxObject, &mut Transform, &BeingHeld)>, 
    holders: Query<(Entity, &Transform), Without<BeingHeld>>
) {
    for (_box_object, mut box_transform, being_held) in boxes.iter_mut() {
        if let Ok((_entity, transform)) = holders.get(being_held.held_by) {
            box_transform.translation.x = transform.translation.x;
            box_transform.translation.y = transform.translation.y + 1.0;
            box_transform.translation.z = transform.translation.z;
        }
    }
}

fn update_box(
    mut boxes: Query<(Entity, &mut BoxObject, &mut Position, &mut Transform), Without<BeingHeld>>, 
    mut level: ResMut<Level>,
    time: Res<Time>, 
) {
    for (entity, mut box_object, mut position, mut transform) in boxes.iter_mut() {
        if level.is_type(position.x, position.y - 1, position.z, None) {
            println!("FALLLLLLLLLLL");
            box_object.target = Some(Direction::Beneath);
        }

        if !box_object.target.is_some() { 
            // this is a terrible hack to handle when
            // a box ends up stuck in a spot that doesn't match
            // it's current position. This is a concurrency problem that
            // should be addresed. 
            transform.translation = Vec3::new(position.x as f32, 
                                              position.y as f32, 
                                              position.z as f32);
            continue; 
        }

        let current = transform.translation;
        let target_translation = match box_object.target.unwrap() {
                                     Direction::Beneath
                                         => Transform::from_xyz(current.x, 
                                                                current.y - 1.0, 
                                                                current.z),
                                     Direction::Above
                                         => Transform::from_xyz(current.x, 
                                                                current.y + 1.0, 
                                                                current.z),
                                     Direction::Up 
                                         => Transform::from_xyz(current.x + 1.0, 
                                                                current.y, 
                                                                current.z),
                                     Direction::Down 
                                         => Transform::from_xyz(current.x - 1.0, 
                                                                current.y, 
                                                                current.z),
                                     Direction::Right 
                                         => Transform::from_xyz(current.x, 
                                                                current.y, 
                                                                current.z + 1.0),
                                     Direction::Left 
                                         => Transform::from_xyz(current.x, 
                                                                current.y, 
                                                                current.z - 1.0),
                                 }.translation;

        if level.is_type_with_vec(target_translation, None) 
            || level.is_with_vec(target_translation, Some(GameObject::new(entity, EntityType::Block))) {
            let target_position = Vec3::new(target_translation.x - transform.translation.x,
                                            target_translation.y - transform.translation.y,
                                            target_translation.z - transform.translation.z).normalize();
             
            level.set(position.x, position.y, position.z, None);
            level.set_with_vec(target_position, Some(GameObject::new(entity, EntityType::Block)));
            transform.translation += target_position * 0.01 * time.delta().subsec_millis() as f32;
        } else {
            println!("Can't move!");
            box_object.target = None;
            position.x = transform.translation.x as i32;
            position.y = transform.translation.y as i32;
            position.z = transform.translation.z as i32;
            transform.translation = Vec3::new(position.x as f32, position.y as f32, position.z as f32);
            level.set(position.x, position.y, position.z, Some(GameObject::new(entity, EntityType::Block)));
        }
    }
}
