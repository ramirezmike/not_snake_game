use bevy::prelude::*;
use bevy::render::camera::Camera;

use crate::{level::Level, Position, collectable, dude::DudeMeshes, level,
            EntityType, GameObject, holdable, win_flag, moveable,
            level_over, credits, block, camera, path_find, path_find::PathFinder};

pub struct EnvironmentPlugin;
impl Plugin for EnvironmentPlugin {
    fn build(&self, app: &mut AppBuilder) {
        let width = 6;
        let length = 12;
        let height = 12;
        app.insert_resource(Level::new(width, length, height))
           .insert_resource(PathFinder::new(width, length, height))
           .init_resource::<DudeMeshes>()
           .init_resource::<AssetsLoading>()
           .add_plugin(camera::CameraPlugin)
           .add_event::<holdable::LiftHoldableEvent>()
           .add_event::<level::PositionChangeEvent>()
           .add_event::<level_over::LevelOverEvent>()
           .add_system_set(
               SystemSet::on_enter(crate::AppState::Loading)
                         .with_system(load_assets.system())
           )
           .add_system_set(
               SystemSet::on_update(crate::AppState::Loading)
                   .with_system(check_assets_ready.system())
           )
           .add_system_set(
               SystemSet::on_enter(crate::AppState::InGame)
                         .with_system(load_level.system())
                         .with_system(level_over::setup_level_over_screen.system())
           )

           .insert_resource(credits::CreditsDelay(Timer::from_seconds(1.5, false)))
           .insert_resource(level_over::LevelIsOver(false))
           .add_system_set(
               SystemSet::on_update(crate::AppState::InGame)
               .with_system(holdable::lift_holdable.system().label("handle_lift_events"))
               .with_system(holdable::update_held.system().before("handle_lift_events"))
               .with_system(moveable::update_moveable.system().label("handle_moveables"))
               .with_system(win_flag::update_flag.system())
               .with_system(collectable::check_collected.system())
               .with_system(level_over::level_over_check.system())
               .with_system(path_find::show_path.system())
               .with_system(path_find::update_path.system())
               .with_system(path_find::update_graph.system())
               .with_system(path_find::draw_edges.system())
//               .with_system(update_text_position.system())
               .with_system(level::broadcast_changes.system().after("handle_moveables"))
           );
    }
}

pub fn load_assets(
    mut _commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<DudeMeshes>,
    mut loading: ResMut<AssetsLoading>,
) {
    meshes.step1 = asset_server.load("models/dude.glb#Mesh0/Primitive0");
    meshes.material = materials.add(Color::hex(crate::COLOR_DUDE).unwrap().into());
    meshes.enemy_material = materials.add(Color::hex(crate::COLOR_ENEMY).unwrap().into());

    loading.0.push(meshes.step1.clone_untyped());
}


#[derive(Default)]
pub struct AssetsLoading(Vec<HandleUntyped>);
fn check_assets_ready(
    mut state: ResMut<State<crate::AppState>>,
    server: Res<AssetServer>,
    loading: Res<AssetsLoading>,
) {
    println!("Loading...");
    use bevy::asset::LoadState;

    let mut ready = true;

    for handle in loading.0.iter() {
        match server.get_load_state(handle) {
            LoadState::Failed => {
                // one of our assets had an error
            }
            LoadState::Loaded => {
            }
            _ => {
                ready = false;
            }
        }
    }

    if ready {
        state.set(crate::AppState::InGame).unwrap();
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
    asset_server: Res<AssetServer>,
    mut level: ResMut<Level>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mesh = meshes.add(Mesh::from(shape::Plane { size: 1.0 }));
    commands.spawn_bundle(UiCameraBundle::default());
//    let font = asset_server.load("fonts/FiraSans-Bold.ttf");

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
//                .insert(DisplayText(format!("{}", i).to_string()))
//                .id();
//          let follow_text = create_follow_text(entity_id, font.clone());
//          commands.spawn_bundle(follow_text.0).insert(follow_text.1);
        }
    }

//  for i in 0..level.width {
//      for j in ((level.length / 2) + (level.length / 4))..level.length {
//          let block_entity =
//          commands.spawn_bundle(PbrBundle {
//              transform: Transform::from_translation(Vec3::new(i as f32, 2.0, j as f32)),
//              ..Default::default()
//          })
//          .with_children(|parent| {
//              parent.spawn_bundle(PbrBundle {
//                  mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
//                  material: if (i + j + 1) % 2 == 0 { 
//                                materials.add(Color::hex(crate::COLOR_GROUND_1).unwrap().into())
//                            } else {
//                                materials.add(Color::hex(crate::COLOR_GROUND_2).unwrap().into())
//                            },
//                  transform: Transform::from_xyz(0.0, 0.5, 0.0),
//                  ..Default::default()
//              });
//          })
//          .insert(EntityType::Block) // TODO: this should be platform at some point. Dude can't climb platforms, just blocks
//          .insert(Position { x: i, y: 2, z: j })
//          .id();
//          level.set(i, 2, j, Some(GameObject::new(block_entity, EntityType::Block)));
//      }
//  }

    commands.spawn_bundle(LightBundle {
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..Default::default()
    });

    let block_positions = vec!(
        (1.0, 0.0, 3.0),
        (2.0, 0.0, 8.0),
        (2.0, 0.0, 7.0),
        (2.0, 0.0, 6.0),
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
      transform: Transform::from_xyz(level.width as f32 - 1.0, 0.0, level.length as f32 / 2.0),
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
    .insert(Position { x:level.width as i32 - 1, y: 0, z: level.length as i32 / 2 });
}

pub struct DisplayText(pub String);
pub struct FollowText(Entity);

fn create_follow_text(entity: Entity, font: Handle<Font>) -> (TextBundle, FollowText) {
    (TextBundle {
            style: Style {
                align_self: AlignSelf::FlexEnd,
                position_type: PositionType::Absolute,
                position: Rect {
                    bottom: Val::Px(5.0),
                    left: Val::Px(15.0),
                    ..Default::default()
                },
                size: Size {
                    width: Val::Px(200.0),
                    ..Default::default()
                },
                ..Default::default()
            },
            text: Text::with_section(
                "1".to_string(),
                TextStyle {
                    font,
                    font_size: 50.0,
                    color: Color::WHITE,
                    
                },
                TextAlignment {
                    ..Default::default()
                },
            ),
            ..Default::default()
        }, FollowText(entity))
}

fn update_text_position(
    windows: Res<Windows>,
    mut text_query: Query<(&mut Style, &CalculatedSize, &FollowText, &mut Text)>,
    entity_query: Query<(&Transform, &DisplayText)>,
    camera_query: Query<(&Camera, &GlobalTransform), With<crate::camera::Camera>>,
) {
    for (camera, camera_transform) in camera_query.iter() {
        for (mut style, calculated, follow_text, mut text) in text_query.iter_mut() {
            if let Ok((entity_transform, display_text)) = entity_query.get(follow_text.0) {
                text.sections[0].value = display_text.0.clone();
                match camera.world_to_screen(&windows, camera_transform, entity_transform.translation)
                {
                    Some(coords) => {
                        style.position.left = Val::Px(coords.x - calculated.size.width / 2.0);
                        style.position.bottom = Val::Px(coords.y - calculated.size.height / 2.0);
                    }
                    None => {
                        // hide the text when the mesh is behind the camera
                        style.position.bottom = Val::Px(-1000.0);
                    }
                }
            }
        }
    }
}
