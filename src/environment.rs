use bevy::prelude::*;
use bevy::render::camera::Camera;

use crate::{level::Level, Position, collectable, dude::DudeMeshes, snake, level,
            EntityType, GameObject, holdable, win_flag, moveable, food,
            level_over, credits, block, camera, path_find, path_find::PathFinder};

pub struct EnvironmentPlugin;
impl Plugin for EnvironmentPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.insert_resource(Level::new())
           .insert_resource(PathFinder::new())
           .init_resource::<DudeMeshes>()
           .init_resource::<snake::EnemyMeshes>()
           .init_resource::<AssetsLoading>()
           .add_plugin(camera::CameraPlugin)
           .add_event::<holdable::LiftHoldableEvent>()
           .add_event::<level::PositionChangeEvent>()
           .add_event::<level_over::LevelOverEvent>()
           .add_event::<snake::AddBodyPartEvent>()
           .add_event::<snake::KillSnakeEvent>()
           .add_event::<food::FoodEatenEvent>()
           .add_event::<level::NextLevelEvent>()
           .add_system_set(
               SystemSet::on_enter(crate::AppState::Loading)
                         .with_system(load_assets.system())
           )
           .add_system_set(
               SystemSet::on_update(crate::AppState::Loading)
                   .with_system(check_assets_ready.system())
           )
           .add_system_set(
               SystemSet::on_update(crate::AppState::ChangingLevel)
                   .with_system(change_level_screen.system())
           )
           .add_system_set(
               SystemSet::on_enter(crate::AppState::InGame)
                         .with_system(load_level.system())
                         .with_system(snake::spawn_enemy.system())
                         .with_system(level_over::setup_level_over_screen.system())
           )

           .insert_resource(credits::CreditsDelay(Timer::from_seconds(1.5, false)))
           .add_system_set(
               SystemSet::on_update(crate::AppState::InGame)
               .with_system(holdable::lift_holdable.system().label("handle_lift_events"))
               .with_system(holdable::update_held.system().before("handle_lift_events"))
               .with_system(moveable::update_moveable.system().label("handle_moveables"))
               .with_system(win_flag::update_flag.system())
               .with_system(collectable::check_collected.system())
               .with_system(level_over::level_over_check.system())
               .with_system(path_find::show_path.system())
               .with_system(snake::update_enemy.system())
               .with_system(snake::handle_food_eaten.system())
               .with_system(food::animate_food.system())
               .with_system(food::update_food.system())
//              .with_system(snake::add_body_part.system())
               .with_system(snake::debug_add_body_part.system())
               .with_system(snake::add_body_parts.system())
               .with_system(snake::handle_kill_snake.system())
               .with_system(path_find::update_path.system())
               .with_system(path_find::update_graph.system())
               .with_system(path_find::draw_edges.system())
//             .with_system(level::print_level.system())
//               .with_system(update_text_position.system())
               .with_system(level::broadcast_changes.system().after("handle_moveables"))
           );
    }
}

pub fn change_level_screen(
    mut state: ResMut<State<crate::AppState>>,
    time: Res<Time>,
    mut level: ResMut<Level>,
    mut timer: Local<f32>,
) {
    *timer += time.delta_seconds();

    println!("changing level...");
    if *timer > 2.0 {
        level.change_to_next_level();
        state.set(crate::AppState::InGame).unwrap();
    }
}

pub fn load_assets(
    mut _commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut dude_meshes: ResMut<DudeMeshes>,
    mut enemy_meshes: ResMut<snake::EnemyMeshes>,
    mut loading: ResMut<AssetsLoading>,
) {
    dude_meshes.step1 = asset_server.load("models/dude.glb#Mesh0/Primitive0");
    dude_meshes.material = materials.add(Color::hex(crate::COLOR_DUDE).unwrap().into());

    enemy_meshes.head = asset_server.load("models/snake.glb#Mesh0/Primitive0");
    enemy_meshes.body = asset_server.load("models/snake.glb#Mesh1/Primitive0");
    enemy_meshes.material = materials.add(Color::hex(crate::COLOR_ENEMY).unwrap().into());

    loading.0.push(dude_meshes.step1.clone_untyped());
    loading.0.push(enemy_meshes.head.clone_untyped());
    loading.0.push(enemy_meshes.body.clone_untyped());
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
            EntityType::Enemy | EntityType::Dude | EntityType::Block | 
            EntityType::WinFlag | EntityType::Platform | EntityType::Food => {
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
    mut path_finder: ResMut<PathFinder>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    path_finder.load_level(&level);
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

    for i in 0..level.width {
        for j in ((level.length / 2) + (level.length / 4))..level.length {
            let block_entity =
            commands.spawn_bundle(PbrBundle {
                transform: Transform::from_translation(Vec3::new(i as f32, 0.0, j as f32)),
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
            .insert(Position { x: i as i32, y: 0, z: j as i32})
            .id();
            level.set(i as i32, 0, j as i32, Some(GameObject::new(block_entity, EntityType::Block)));
        }
    }

    commands.spawn_bundle(LightBundle {
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..Default::default()
    });

    let block_positions = vec!(
        (1.0, 0.0, 3.0),
        (2.0, 0.0, 3.0),
        (2.0, 0.0, 4.0),
        (2.0, 0.0, 6.0),
        (2.0, 0.0, 0.0),
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
    let transform = Transform::from_xyz(level.width as f32 / 2.0 - 1.0, 3.0, (level.length - 1)as f32);
    let position = Position::from_vec(transform.translation);
    commands.spawn_bundle(PbrBundle {
      transform,
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
    .insert(position);

    food::spawn_food(commands, level, meshes, materials);
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
