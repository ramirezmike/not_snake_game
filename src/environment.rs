use bevy::prelude::*;

use crate::{level::Level, Position, collectable,
            EntityType, GameObject, holdable, win_flag, moveable,
            level_over, credits, level, block, camera};

pub struct EnvironmentPlugin;
impl Plugin for EnvironmentPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.insert_resource(Level {
               width: 6,
               length: 12,
               height: 12,
               game_objects: vec![vec![vec![None; 12]; 12]; 6],
           })

           .add_plugin(camera::CameraPlugin)
           .add_event::<holdable::LiftHoldableEvent>()
           .add_event::<level_over::LevelOverEvent>()
           .add_system_set(
               SystemSet::on_enter(crate::AppState::InGame)
                         .with_system(load_level.system())
                         .with_system(level_over::setup_level_over_screen.system())
           )

           .insert_resource(credits::CreditsDelay(Timer::from_seconds(1.5, false)))
           .insert_resource(level_over::LevelIsOver(false))
           .add_system_set(
               SystemSet::on_update(crate::AppState::InGame)
               .with_system(holdable::lift_holdable.system())
               .with_system(holdable::update_held.system())
               .with_system(block::update_block.system())
               .with_system(moveable::update_moveable.system())
               .with_system(win_flag::update_flag.system())
               .with_system(collectable::check_collected.system())
               .with_system(level_over::level_over_check.system())
           );
    }
}

pub fn cleanup_environment(
    mut commands: Commands, 
    entities: Query<(Entity, &EntityType)>,
    level_over_text: Query<(Entity, &level_over::LevelOverText)>,
) {
    for (entity, entity_type) in entities.iter() {
        println!("Despawning... {:?}", entity_type);
        match entity_type {
            EntityType::Dude | EntityType::Block | EntityType::WinFlag | EntityType::Platform => {
                commands.entity(entity).despawn_recursive();
            }
            _ => commands.entity(entity).despawn()
        }
    }

    for (entity, _text) in level_over_text.iter() {
        commands.entity(entity).despawn();
    }
}

pub fn load_level(
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
            })
            .insert(EntityType::Platform);
        }
    }

    for i in 0..level.width {
        for j in ((level.length / 2) + (level.length / 4))..level.length {
            let block_entity =
            commands.spawn_bundle(PbrBundle {
                transform: Transform::from_translation(Vec3::new(i as f32, 2.0, j as f32)),
                ..Default::default()
            })
            .with_children(|parent| {
                parent.spawn_bundle(PbrBundle {
                    mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
                    material: if (i + j + 1) % 2 == 0 { 
                                  materials.add(Color::hex(crate::COLOR_GROUND_1).unwrap().into())
                              } else {
                                  materials.add(Color::hex(crate::COLOR_GROUND_2).unwrap().into())
                              },
                    transform: Transform::from_xyz(0.0, 0.5, 0.0),
                    ..Default::default()
                });
            })
            .insert(EntityType::Block) // TODO: this should be platform at some point. Dude can't climb platforms, just blocks
            .insert(Position { x: i, y: 2, z: j })
            .id();
            level.set(i, 2, j, Some(GameObject::new(block_entity, EntityType::Block)));
        }
    }

    commands.spawn_bundle(LightBundle {
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..Default::default()
    });

    let block_positions = vec!(
        (1.0, 0.0, 3.0),
        (2.0, 0.0, 8.0),
        (2.0, 1.0, 8.0),
        (3.0, 0.0, 2.0),
        (4.0, 0.0, 1.0),
    );

    for block in block_positions.iter() {
        let block_entity =
            commands.spawn_bundle(PbrBundle {
              transform: Transform::from_xyz(block.0, block.1, block.2),
              ..Default::default()
            })
            .with_children(|parent| {
                parent.spawn_bundle(PbrBundle {
                    mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
                    material: materials.add(Color::hex(crate::COLOR_BLOCK).unwrap().into()),
                    transform: Transform::from_xyz(0.0, 0.5, 0.0),
                    ..Default::default()
                });
            })
            .insert(EntityType::Block)
            .insert(holdable::Holdable {})
            .insert(Position { x: block.0 as i32, y: block.1 as i32, z: block.2 as i32 })
            .insert(block::BlockObject { })
            .insert(moveable::Moveable::new(true, false, 0.1))
            .id();
        level.set(block.0 as i32, block.1 as i32, block.2 as i32, Some(GameObject::new(block_entity, EntityType::Block)));
    }

    let flag_color = Color::hex(crate::COLOR_FLAG).unwrap();
    let flag_color = Color::rgba(flag_color.r(), flag_color.g(), flag_color.b(), 1.0);
    commands.spawn_bundle(PbrBundle {
      transform: Transform::from_xyz(((level.width - 1) / 2) as f32, 3.0, (level.length - 1) as f32),
      ..Default::default()
    })
    .with_children(|parent| {
        parent.spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Icosphere { radius: 0.25, subdivisions: 0 })),
            material: materials.add(flag_color.into()),
            transform: Transform::from_xyz(0.0, 0.5, 0.0),
            ..Default::default()
        })
        .insert(win_flag::WinFlagInnerMesh {});
    })
    .insert(collectable::Collectable { collected: false }) 
    .insert(win_flag::WinFlag {})
    .insert(EntityType::WinFlag)
    .insert(Position { x:((level.width - 1) / 2), y: 3, z: level.length - 1 });
}
