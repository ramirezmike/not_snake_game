use bevy::prelude::*;
use bevy::render::camera::Camera;

use crate::{level::Level, Position, collectable, dude, snake, level,
            EntityType, GameObject, holdable, win_flag, moveable, food, score,
            level_over, credits, block, camera, path_find, path_find::PathFinder};

// material.shaded = false
pub struct Shadow;
pub struct PlatformMesh;
pub struct LevelReady(pub bool);
pub struct EnvironmentPlugin;
impl Plugin for EnvironmentPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.insert_resource(Level::new())
           .insert_resource(PathFinder::new())
           .insert_resource(LevelReady(false))
           .insert_resource(score::Score::new())
           .init_resource::<dude::DudeMeshes>()
           .init_resource::<snake::EnemyMeshes>()
           .init_resource::<camera::CameraMouthMovement>()
           .init_resource::<AssetsLoading>()
           .add_plugin(camera::CameraPlugin)
           .add_event::<holdable::LiftHoldableEvent>()
           .add_event::<level::PositionChangeEvent>()
           .add_event::<level_over::LevelOverEvent>()
           .add_event::<snake::AddBodyPartEvent>()
           .add_event::<snake::KillSnakeEvent>()
           .add_event::<dude::KillDudeEvent>()
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
               SystemSet::on_exit(crate::AppState::ChangingLevel)
                   .with_system(cleanup_change_level_screen.system())
           )
           .add_system_set(
               SystemSet::on_enter(crate::AppState::ResetLevel)
                   .with_system(cleanup_change_level_screen.system())
           )
           .add_system_set(
               SystemSet::on_update(crate::AppState::ResetLevel)
                   .with_system(camera::handle_player_death.system())
           )
           .add_system_set(
               SystemSet::on_enter(crate::AppState::InGame)
                         .with_system(load_level.system())
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
               .with_system(score::handle_food_eaten.system())
               .with_system(food::animate_food.system())
               .with_system(food::update_food.system())
//              .with_system(snake::add_body_part.system())
               .with_system(snake::add_body_parts.system())
               .with_system(snake::update_following.system())
               .with_system(snake::handle_kill_snake.system())
               .with_system(camera::handle_player_death.system())
               .with_system(dude::handle_kill_dude.system())
               .with_system(path_find::update_graph.system().label("graph_update"))
               .with_system(path_find::update_path.system().after("graph_update"))
               .with_system(path_find::draw_edges.system())
//               .with_system(material_test.system())
//               .with_system(level::print_level.system())
//               .with_system(update_text_position.system())
               .with_system(level::broadcast_changes.system().after("handle_moveables"))
           );
    }
}

/*
pub fn material_test(
    mut materials: Query<&mut StandardMaterial>,
) {
    for mut material in materials.iter_mut() {
        material.unlit = true; 
    }
}
*/

pub fn cleanup_change_level_screen(
    mut commands: Commands,
    level_over_text: Query<(Entity, &level_over::LevelOverText)>,
) {
    for (entity, _text) in level_over_text.iter() {
        commands.entity(entity).despawn();
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
    if *timer > 1.0 {
        level.change_to_next_level();
        state.set(crate::AppState::InGame).unwrap();
        *timer = 0.0; 
    }
}

pub fn load_assets(
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut dude_meshes: ResMut<dude::DudeMeshes>,
    mut enemy_meshes: ResMut<snake::EnemyMeshes>,
    mut loading: ResMut<AssetsLoading>,
) {
    dude_meshes.step1 = asset_server.load("models/dude.glb#Mesh0/Primitive0");
    dude_meshes.material = materials.add(Color::hex(crate::COLOR_DUDE).unwrap().into());

    enemy_meshes.head = asset_server.load("models/snake.glb#Mesh0/Primitive0");
    enemy_meshes.body = asset_server.load("models/snake.glb#Mesh1/Primitive0");
    let enemy_color = Color::hex(crate::COLOR_ENEMY).unwrap();
    enemy_meshes.material = materials.add(enemy_color.into());
    enemy_meshes.shadow_material = materials.add(Color::rgba(enemy_color.r(), enemy_color.g(), enemy_color.b(), 0.4).into());
    enemy_meshes.shadow = meshes.add(Mesh::from(shape::Plane { size: 0.75 }));

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
    mut level_ready: ResMut<LevelReady>,
    entities: Query<(Entity, &EntityType)>,
    lights: Query<Entity, With<Light>>,
    platforms: Query<Entity, With<PlatformMesh>>,
    ui_cameras: Query<Entity, With<bevy::render::camera::OrthographicProjection>>,
) {
    level_ready.0 = false;
    for (entity, entity_type) in entities.iter() {
        println!("Despawning... {:?}", entity_type);
        match entity_type {
            EntityType::EnemyHead | EntityType::Enemy | EntityType::Dude | EntityType::Block | 
            EntityType::WinFlag | EntityType::Platform | EntityType::Food => {
                commands.entity(entity).despawn_recursive();
            }
            _ => commands.entity(entity).despawn()
        }
    }

    for entity in lights.iter() {
        commands.entity(entity).despawn_recursive();
    }

    for entity in ui_cameras.iter() {
        commands.entity(entity).despawn_recursive();
    }

    for entity in platforms.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

pub fn load_level(
    mut commands: Commands,
    mut level: ResMut<Level>,
    mut path_finder: ResMut<PathFinder>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut score: ResMut<score::Score>,
    mut level_ready: ResMut<LevelReady>,
    dude_meshes: Res<dude::DudeMeshes>,
    enemy_meshes: Res<snake::EnemyMeshes>, 
    entities: Query<Entity>,
    transforms: Query<&Transform>,
) {
    println!("Starting to load level...");
    score.total = score.current_level;
    score.current_level = 0;
    level.reset_level();
    path_finder.load_level(&level);
    let plane = meshes.add(Mesh::from(shape::Plane { size: 1.0 }));
    let cube = meshes.add(Mesh::from(shape::Cube { size: 1.0 }));
    let ground_1_material = materials.add(Color::hex(crate::COLOR_GROUND_1).unwrap().into());
    let ground_2_material = materials.add(Color::hex(crate::COLOR_GROUND_2).unwrap().into());
    let block_material = materials.add(Color::hex(crate::COLOR_BLOCK).unwrap().into());
    let flag_color = Color::hex(crate::COLOR_FLAG).unwrap();
    let flag_color = Color::rgba(flag_color.r(), flag_color.g(), flag_color.b(), 1.0);
                    
    commands.spawn_bundle(UiCameraBundle::default());
    let space_scale = 0.9;

    for x in 0..level.width {
        for y in 0..level.height {
            for z in 0..level.length {
                match level.get_level_info(x, y, z) {
                    1 => { // platform
                        let entity =
                            commands.spawn_bundle(PbrBundle {
                                mesh: if y == 0 { plane.clone() } else { cube.clone() },
                                material: if (x + z + 1) % 2 == 0 { ground_2_material.clone() } else { ground_2_material.clone() },
                                transform: { 
                                    let mut transform = 
                                    Transform::from_translation(Vec3::new(x as f32, 
                                                                         if y == 0 { y as f32 + 1.0 } else { y as f32 + 0.5 }, 
                                                                          z as f32));
                                    transform.scale.x = space_scale;
                                    if y != 0 {
                                        transform.scale.y = space_scale;
                                    }
                                    transform.scale.z = space_scale;
                                    transform
                                },
                                ..Default::default()
                            })
                            .insert(EntityType::Block)
                            .id();
                        level.set(x as i32, y as i32, z as i32, Some(GameObject::new(entity, EntityType::Block)));

                        if y == 0 {
                            commands.spawn_bundle(PbrBundle {
                                mesh: cube.clone(),
                                material: ground_1_material.clone(),
                                transform: {
                                    let height = 30.0;
                                    let mut transform =
                                        Transform::from_translation(Vec3::new(x as f32, 
                                                                             ((y + 1) as f32) - (height / 2.0) - 0.0001, 
                                                                              z as f32));

                                    transform.scale.x = space_scale;
                                    if y != 0 {
                                        transform.scale.y = space_scale;
                                    } else {
                                        transform.scale.y = height;
                                    }
                                    transform.scale.z = space_scale;
                                    transform
                                },
                                ..Default::default()
                            }).insert(PlatformMesh);
                        }
                    },
                    2 => { // moveable block
                        let inner_mesh_vertical_offset = 0.5;
                        let block_entity =
                            commands.spawn_bundle(PbrBundle {
                              transform: Transform::from_xyz(x as f32, y as f32, z as f32),
                              ..Default::default()
                            })
                            .with_children(|parent| {
                                parent.spawn_bundle(PbrBundle {
                                    mesh: cube.clone(),
                                    material: block_material.clone(),
                                    transform: {
                                        let mut transform = Transform::from_xyz(0.0, inner_mesh_vertical_offset, 0.0);

                                        transform.scale.x = space_scale;
                                        transform.scale.y = space_scale;
                                        transform.scale.z = space_scale;
                                        transform
                                    },
                                    ..Default::default()
                                });
                            })
                            .insert(EntityType::Block)
                            .insert(holdable::Holdable {})
                            .insert(Position { x: x as i32, y: y as i32, z: z as i32 })
                            .insert(block::BlockObject { })
                            .insert(moveable::Moveable::new(0.1, inner_mesh_vertical_offset))
                            .id();
                        level.set(x as i32, y as i32, z as i32, Some(GameObject::new(block_entity, EntityType::Block)));
                    },
                    3 => { // win_flag
                        let transform = Transform::from_xyz(x as f32, y as f32, z as f32);
                        println!("{} {} {}", x, y, z);
                        let position = Position::from_vec(transform.translation);
                        let entity =
                            commands.spawn_bundle(PbrBundle {
                              transform,
                              ..Default::default()
                            })
                            .with_children(|parent| {
                                parent.spawn_bundle(PbrBundle {
                                    mesh: meshes.add(Mesh::from(shape::Icosphere { radius: 0.25, subdivisions: 0 })),
                                    material: materials.add(flag_color.into()),
                                    visible: Visible {
                                        is_visible: false,
                                        is_transparent: false,
                                    },
                                    transform: Transform::from_xyz(0.0, 0.5, 0.0),
                                    ..Default::default()
                                })
                                .insert(win_flag::WinFlagInnerMesh {});
                            })
                            .insert(collectable::Collectable { collected: false }) 
                            .insert(win_flag::WinFlag {})
                            .insert(EntityType::WinFlag)
                            .insert(position)
                            .id();

                        level.set(x as i32, y as i32, z as i32, Some(GameObject::new(entity, EntityType::WinFlag)));
                    },
                    4 => dude::spawn_player(&mut commands, &dude_meshes, &mut level, x, y, z),
                    5 => snake::spawn_enemy(&mut commands, &enemy_meshes, &mut level, x, y, z),
                    6 => {
                        food::spawn_food(&mut commands, &mut level, &mut meshes, &mut materials, 
                                         Some(Position{ x: x as i32, y: y as i32, z: z as i32 }))
                    }
                    _ => ()
                }
            }
        }
    }

    commands.spawn_bundle(LightBundle {
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..Default::default()
    });

    if level.is_food_random() {
        food::spawn_food(&mut commands, &mut level, &mut meshes, &mut materials, None);
    }

    println!("Level is Loaded... Number of items{:?} {:?}", entities.iter().len(), transforms.iter().len());

    level_ready.0 = true;
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
