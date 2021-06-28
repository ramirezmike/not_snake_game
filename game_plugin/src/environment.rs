use bevy::{
    prelude::*,
    render::{
        pipeline::{PipelineDescriptor},
        render_graph::{base, RenderGraph, RenderResourcesNode},
        renderer::RenderResources,
        shader::{ShaderStage, ShaderStages},
    },
};
use bevy::reflect::{TypeUuid};
use bevy::render::camera::Camera;
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, Diagnostics};
use bevy_kira_audio::{Audio, AudioPlugin};

use crate::{level::Level, Position, collectable, dude, snake, level, hud_pass,
            EntityType, GameObject, holdable, win_flag, moveable, food, score,
            camera::MainCamera, sounds, game_controller, teleporter, dust,
            level_over, credits, block, camera, path_find, path_find::PathFinder};
//use bevy_mod_debugdump::print_schedule_runner;

// material.shaded = false
pub struct Shadow;
pub struct PlatformMesh;
pub struct BlockMesh;
pub struct LevelReady(pub bool);
pub struct GameOver(pub bool);
pub struct EnvironmentPlugin;
impl Plugin for EnvironmentPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.insert_resource(Level::new())
           .insert_resource(PathFinder::new())
           .insert_resource(LevelReady(false))
           .insert_resource(GameOver(false))
           .insert_resource(score::Score::new())
           .init_resource::<crate::pause::PauseButtonMaterials>()
           .init_resource::<dude::DudeMeshes>()
           .init_resource::<snake::EnemyMeshes>()
           .init_resource::<camera::CameraMeshes>()
           .init_resource::<win_flag::WinFlagMeshes>()
           .init_resource::<GameShaders>()
           .init_resource::<camera::CameraMouthMovement>()
           .init_resource::<camera::CameraBoltMovement>()
           .init_resource::<camera::CameraSpikeMovement>()
           .init_resource::<AssetsLoading>()
           .add_plugin(AudioPlugin)
           .add_plugin(camera::CameraPlugin)
           .add_plugin(hud_pass::HUDPassPlugin)
           .add_event::<holdable::LiftHoldableEvent>()
           .add_event::<level::PositionChangeEvent>()
           .add_event::<level_over::LevelOverEvent>()
           .add_event::<snake::AddBodyPartEvent>()
           .add_event::<snake::KillSnakeEvent>()
           .add_event::<sounds::SoundEvent>()
           .add_plugin(FrameTimeDiagnosticsPlugin::default())
//           .add_plugin(LogDiagnosticsPlugin::default())
           .add_plugin(FrameTimeDiagnosticsPlugin::default())
           .add_event::<dude::KillDudeEvent>()
           .add_event::<dust::CreateDustEvent>()
           .add_event::<food::FoodEatenEvent>()
           .add_event::<level::NextLevelEvent>()
           .add_system_set(
               SystemSet::on_enter(crate::AppState::Loading)
                         .with_system(load_assets.system())
                         .with_system(cleanup_environment.system())
           )
           .add_system_set(
               SystemSet::on_update(crate::AppState::Loading)
                   .with_system(check_assets_ready.system())
           )
           .add_system_set(
               SystemSet::on_enter(crate::AppState::ScoreDisplay)
                   .with_system(score::setup_score_screen.system())
           )
           .add_system_set(
               SystemSet::on_update(crate::AppState::ScoreDisplay)
                   .with_system(score::displaying_score.system())
           )
           .add_system_set(
               SystemSet::on_exit(crate::AppState::ScoreDisplay)
                   .with_system(cleanup_change_level_screen.system())
           )
           .add_system_set(
               SystemSet::on_enter(crate::AppState::LevelTitle)
                   .with_system(level_over::setup_level_over_screen.system())
           )
           .add_system_set(
               SystemSet::on_update(crate::AppState::LevelTitle)
                   .with_system(level_over::displaying_title.system())
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
               SystemSet::on_enter(crate::AppState::RestartLevel)
                   .with_system(cleanup_environment.system())
           )
           .add_system_set(
               SystemSet::on_update(crate::AppState::RestartLevel)
                   .with_system(restart_level.system())
           )
           .add_system_set(
               SystemSet::on_enter(crate::AppState::Pause)
                   .with_system(crate::pause::setup_menu.system())
           )
           .add_system_set(
               SystemSet::on_update(crate::AppState::Pause)
                   .with_system(crate::pause::pause_menu.system())
           )
           .add_system_set(
               SystemSet::on_exit(crate::AppState::Pause)
                   .with_system(crate::pause::cleanup_pause_menu.system())
           )
           .add_system_set(
               SystemSet::on_update(crate::AppState::ResetLevel)
                   .with_system(camera::handle_player_death.system())
           )
           .add_system_set(
               SystemSet::on_enter(crate::AppState::InGame)
                         .with_system(load_level.system().label("loading_level"))
                         .with_system(crate::camera::create_camera.system().after("loading_level"))
                         .with_system(set_clear_color.system().after("loading_level"))
                         .with_system(load_level_into_path_finder.system().after("loading_level"))
                         .with_system(reset_score.system())
                         .with_system(level_over::setup_level_over_screen.system().after("loading_level"))
           )

           .insert_resource(credits::CreditsDelay(Timer::from_seconds(1.5, false)))
           .add_system_set(
               SystemSet::on_update(crate::AppState::InGame)
               .with_system(holdable::lift_holdable.system().label("handle_lift_events"))
               .with_system(holdable::update_held.system().before("handle_lift_events"))
               .with_system(moveable::update_moveable.system().label("handle_moveables"))
               .with_system(pause_game.system())
               .with_system(win_flag::update_flag.system())
               .with_system(collectable::check_collected.system())
               .with_system(update_hud_text_position.system())
               .with_system(level_over::level_over_check.system())
//             .with_system(path_find::show_path.system())
               .with_system(snake::update_enemy.system())
               .with_system(snake::handle_food_eaten.system())
               .with_system(score::handle_food_eaten.system())
               .with_system(food::animate_food.system())
               .with_system(food::update_food.system())
               .with_system(food::handle_food_eaten.system())
//               .with_system(hide_blocks.system())
//               .with_system(light_thing.system())
//              .with_system(snake::add_body_part.system())
               .with_system(snake::add_body_parts.system())
               .with_system(snake::update_following.system())
               .with_system(snake::handle_kill_snake.system())
               .with_system(dude::handle_squashes.system())
               .with_system(camera::handle_player_death.system())
               .with_system(dude::handle_kill_dude.system())
               .with_system(path_find::update_graph.system().label("graph_update"))
               .with_system(path_find::update_path.system().after("graph_update"))
//             .with_system(path_find::draw_edges.system())
//               .with_system(material_test.system())
//               .with_system(level::print_level.system())
//               .with_system(update_text_position.system())
               .with_system(level::broadcast_changes.system().after("handle_moveables"))
               .with_system(food::animate_spawn_particles.system())
               .with_system(sounds::play_sounds.system())
               .with_system(game_controller::gamepad_connections.system())
               .with_system(update_fps.system())
               .with_system(camera::cull_blocks.system())
               .with_system(animate_shader.system())
               .with_system(snake::detect_dude_on_electric_snake.system())
               .with_system(shrink_shrinkables.system())
               .with_system(dust::handle_create_dust_event.system())
               .with_system(dust::animate_dust.system())
           );
//        println!("{}", schedule_graph(&app.app.schedule));

//        app.set_runner(print_schedule_runner);
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

pub fn restart_level(
    mut state: ResMut<State<crate::AppState>>,
    mut timer: Local<f32>,
    time: Res<Time>,
) {
    *timer += time.delta_seconds();

    if *timer > 1.0 {
        state.set(crate::AppState::InGame).unwrap();
        *timer = 0.0; 
    }
}

pub struct Shrink;
pub fn shrink_shrinkables(
    mut shrinkables: Query<&mut Transform, With<Shrink>>,
    time: Res<Time>,
) {
    for mut transform in shrinkables.iter_mut() {
        if transform.scale.x <= 0.0 {
            continue;
        }

        transform.scale -= Vec3::splat(time.delta_seconds() * 0.3);
    }
}

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
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut dude_meshes: ResMut<dude::DudeMeshes>,
    mut enemy_meshes: ResMut<snake::EnemyMeshes>,
    mut camera_meshes: ResMut<camera::CameraMeshes>,
    mut flag_meshes: ResMut<win_flag::WinFlagMeshes>,
    mut loading: ResMut<AssetsLoading>,
    mut level_asset_state: ResMut<level::LevelAssetState>, 
    mut pipelines: ResMut<Assets<PipelineDescriptor>>,
    mut game_shaders: ResMut<GameShaders>,
    mut render_graph: ResMut<RenderGraph>,

) {
    dude_meshes.step1 = asset_server.load("models/dude.glb#Mesh0/Primitive0");
    dude_meshes.body = asset_server.load("models/chip.glb#Mesh0/Primitive0");
    dude_meshes.head = asset_server.load("models/chip.glb#Mesh1/Primitive0");

    enemy_meshes.head = asset_server.load("models/snake.glb#Mesh0/Primitive0");
    enemy_meshes.body = asset_server.load("models/snake.glb#Mesh1/Primitive0");
    enemy_meshes.shadow = meshes.add(Mesh::from(shape::Plane { size: 0.75 }));

    camera_meshes.bolt = asset_server.load("models/bolt.glb#Mesh0/Primitive0");
    camera_meshes.spikes = asset_server.load("models/spikes.glb#Mesh0/Primitive0");

    flag_meshes.flag = asset_server.load("models/winflag.glb#Mesh0/Primitive0");

    // Create a new shader pipeline.
    let shader_paths = get_shader_paths();
    game_shaders.electric = pipelines.add(PipelineDescriptor::default_config(ShaderStages {
        vertex: asset_server.load::<Shader, _>(shader_paths.0),
        fragment: Some(asset_server.load::<Shader, _>(shader_paths.1)),
    }));


    // Add a `RenderResourcesNode` to our `RenderGraph`. This will bind `TimeComponent` to our
    // shader.
    render_graph.add_system_node(
        "time_uniform",
        RenderResourcesNode::<TimeUniform>::new(true),
    );

    // Add a `RenderGraph` edge connecting our new "time_component" node to the main pass node. This
    // ensures that "time_component" runs before the main pass.
    render_graph
        .add_node_edge("time_uniform", base::node::MAIN_PASS)
        .unwrap();

    let audio_state = sounds::AudioState::new(&asset_server);

    loading.0.push(dude_meshes.step1.clone_untyped());
    loading.0.push(dude_meshes.head.clone_untyped());
    loading.0.push(dude_meshes.body.clone_untyped());
    loading.0.push(enemy_meshes.head.clone_untyped());
    loading.0.push(enemy_meshes.body.clone_untyped());
    loading.0.push(flag_meshes.flag.clone_untyped());
    loading.0.push(camera_meshes.bolt.clone_untyped());
    loading.0.push(camera_meshes.spikes.clone_untyped());

//    loading.0.push(game_shaders.electric.clone_untyped());
    loading.0.append(&mut audio_state.get_sound_handles());

    level_asset_state.handle = asset_server.load("data/test.custom");
    asset_server.watch_for_changes().unwrap();
    commands.insert_resource(audio_state);
}

#[cfg(not(target_arch = "wasm32"))]
fn get_shader_paths() -> (&'static str, &'static str) {
    ("shaders/hot.vert", "shaders/hot.frag")
}

#[cfg(target_arch = "wasm32")]
fn get_shader_paths() -> (&'static str, &'static str) {
    ("shaders/hot_wasm.vert", "shaders/hot_wasm.frag")
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
        state.set(crate::AppState::MainMenu).unwrap();
    }
}

// this could probably just be one large query with ORs on the Withs
pub fn cleanup_environment(
    mut commands: Commands, 
    mut level_ready: ResMut<LevelReady>,
    entities: Query<(Entity, &EntityType)>,
    lights: Query<Entity, With<Light>>,
    platforms: Query<Entity, With<PlatformMesh>>,
    hud_entities: Query<Entity, With<hud_pass::HUDPass>>,
    hud_cameras: Query<Entity, With<Hud3DCamera>>,
    ui_cameras: Query<Entity, With<bevy::render::camera::OrthographicProjection>>,
    texts: Query<Entity, With<Text>>,
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

    // light is on camera so don't need to despawn now?
//  for entity in lights.iter() {
//      commands.entity(entity).despawn_recursive();
//  }

    for entity in hud_entities.iter() {
        commands.entity(entity).despawn_recursive();
    }

    for entity in ui_cameras.iter() {
        commands.entity(entity).despawn_recursive();
    }

    for entity in hud_cameras.iter() {
        commands.entity(entity).despawn_recursive();
    }

    for entity in platforms.iter() {
        commands.entity(entity).despawn_recursive();
    }
    
    for entity in texts.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

pub fn reset_score(
    mut score: ResMut<score::Score>,
) {
    score.current_level = 0;
}

pub fn set_clear_color(
    level: Res<Level>,
    mut clear_color: ResMut<ClearColor>,
) {
    let palette = &level.get_palette();
    clear_color.0 = Color::hex(palette.base.clone()).unwrap();
}

pub fn load_level_into_path_finder(
    level: Res<Level>,
    mut path_finder: ResMut<PathFinder>,
) {
    path_finder.load_level(&level);
}

pub fn load_level(
    mut commands: Commands,
    mut level: ResMut<Level>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut level_ready: ResMut<LevelReady>,
    mut dude_meshes: ResMut<dude::DudeMeshes>,
    mut enemy_meshes: ResMut<snake::EnemyMeshes>,
    flag_meshes: ResMut<win_flag::WinFlagMeshes>,
    game_shaders: Res<GameShaders>,
    audio: Res<Audio>,
    mut audio_state: ResMut<sounds::AudioState>,
//  entities: Query<Entity>,
    level_asset_state: Res<level::LevelAssetState>, 
    levels_asset: ResMut<Assets<level::LevelsAsset>>,
    asset_server: Res<AssetServer>,
    mut state: ResMut<State<crate::AppState>>,
) {
    println!("Starting to load level...");
    let levels_asset = levels_asset.get(&level_asset_state.handle);
    if let Some(level_asset) = levels_asset  {
        level.load_stored_levels(level_asset.clone());
    } else {
        // try again later?
        println!("failed to load level");
        state.set(crate::AppState::Loading).unwrap();
        return;
    }

    level.reset_level();
    audio_state.stop_electricity(&audio);

    let palette = &level.get_palette();

    dude_meshes.material = materials.add(Color::hex(palette.dude.clone()).unwrap().into());
    let enemy_color = Color::hex(palette.enemy.clone()).unwrap();
    enemy_meshes.material = materials.add(enemy_color.into());
    enemy_meshes.shadow_material = materials.add(Color::rgba(enemy_color.r(), enemy_color.g(), enemy_color.b(), 0.4).into());

    let plane = meshes.add(Mesh::from(shape::Plane { size: 1.0 }));
    let cube = meshes.add(Mesh::from(shape::Cube { size: 1.0 }));
    let mut ground_1_material = materials.add(Color::hex(palette.ground_1.clone()).unwrap().into());
    let mut ground_2_material = materials.add(Color::hex(palette.ground_2.clone()).unwrap().into());
    let block_material = materials.add(Color::hex(palette.block.clone()).unwrap().into());
    let flag_color = Color::hex(palette.flag.clone()).unwrap();
    let flag_color = Color::rgba(flag_color.r(), flag_color.g(), flag_color.b(), 0.7);

    if *state.current() == crate::AppState::MainMenu {
        ground_1_material = 
        materials.add(StandardMaterial {
                        base_color: Color::BLACK,
                        unlit: true,
                        roughness: 1.0,
                        reflectance: 0.0,
                        ..Default::default()
                    });
        ground_2_material = 
        materials.add(StandardMaterial {
                        base_color: Color::BLACK,
                        unlit: true,
                        roughness: 1.0,
                        reflectance: 0.0,
                        ..Default::default()
                    });
    }
                    
    commands.spawn_bundle(UiCameraBundle::default());
    let space_scale = 0.9;

    for x in 0..level.width() {
        for y in 0..level.height() {
            for z in 0..level.length() {
                match level.get_level_info(x, y, z) {
                    item @ 1 | item @ 8 | item @ 9 => { // platform
                        let entity_type = if item == 9 {
                                              EntityType::UnstandableBlock
                                          } else {
                                              EntityType::Block
                                          };
                        let entity =
                            commands.spawn_bundle(PbrBundle {
                                mesh: if y == 0 { plane.clone() } else { cube.clone() },
                                material: if item == 1 { ground_2_material.clone() } else { ground_1_material.clone() },
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
                            .insert(entity_type)
                            .insert(BlockMesh)
                            .id();
                        level.set(x as i32, y as i32, z as i32, Some(GameObject::new(entity, entity_type)));

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
                                material: block_material.clone(),
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
                        let mut transform = Transform::from_xyz(x as f32, y as f32, z as f32);
                        transform.apply_non_uniform_scale(Vec3::new(0.25, 0.25, 0.25)); 
                        let position = Position::from_vec(transform.translation);

                        let entity =
                            commands.spawn_bundle(PbrBundle {
                              transform,
                              ..Default::default()
                            })
                            .with_children(|parent| {
                                parent.spawn_bundle(PbrBundle {
                                    mesh: flag_meshes.flag.clone(),
                                    material: materials.add(flag_color.into()),
                                    visible: Visible {
                                        is_visible: false,
                                        is_transparent: true,
                                    },
                                    transform: {
                                        let mut t = Transform::from_xyz(0.0, 1.50, 0.0);
                                        t.apply_non_uniform_scale(Vec3::new(1.0, 0.1, 1.0)); 
                                        t
                                    },
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
                    11 => dude::spawn_player(&mut commands, &dude_meshes, &mut level, x, y, z),
                    item @ 5 | item @ 10 => {
                        snake::spawn_enemy(&mut commands, &enemy_meshes, &mut level, &game_shaders, x, y, z, item == 10);

                        if item == 10 {
                            audio_state.play_electricity(&audio);
                        }
                    },
                    item @ 4 | item @ 6 | item @ 7 => {
                        food::spawn_food(&mut commands, &mut level, &mut meshes, &mut materials, 
                                         Some(Position{ x: x as i32, y: y as i32, z: z as i32 }), item == 6 || item == 4, item == 4)
                    },
                    _ => ()
                }
            }
        }
    }

    for teleporter in level.get_teleporters() {
        teleporter::spawn_teleporter(&mut commands, teleporter);
    }

    if level.is_food_random() {
        food::spawn_food(&mut commands, &mut level, &mut meshes, &mut materials, None, true, false);
    }


    if *state.current() != crate::AppState::MainMenu {
        create_hud(&mut commands, &mut meshes, &mut materials, &asset_server, &level);
    }

//    println!("Level is Loaded... Number of items{:?}", entities.iter().len());

    level_ready.0 = true;
}

pub struct Hud3DCamera;
pub struct HudFoodMesh;
fn create_hud(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    asset_server: &Res<AssetServer>,
    level: &ResMut<Level>,
) {
    commands.spawn_bundle(UiCameraBundle::default());
    let food_color = Color::hex(level.get_palette().food.clone()).unwrap();
    let food_color = Color::rgba(food_color.r(), food_color.g(), food_color.b(), 1.0);
    commands.spawn_bundle(hud_pass::HUDPbrBundle {
        mesh: meshes.add(Mesh::from(shape::Icosphere { radius: 0.25, subdivisions: 0 })),
        material: materials.add(food_color.into()),
        transform: Transform::from_translation(Vec3::new(0.0, 5.2, -9.5)),
        ..Default::default()
    })
    .insert(HudFoodMesh);

    commands.spawn_bundle(hud_pass::HUDCameraBundle {
        transform: Transform::from_translation(Vec3::new(-15.0, 0.0, 0.0))
            .looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    })
    .insert(Hud3DCamera);

    // Set up UI labels for clarity
    let font = asset_server.load("fonts/FiraSans-Bold.ttf");

    commands
        .spawn_bundle(TextBundle {
            style: Style {
                align_self: AlignSelf::FlexEnd,
                position_type: PositionType::Absolute,
                position: Rect {
                    bottom: Val::Px(5.0),
                    left: Val::Px(15.0),
                    ..Default::default()
                },
                size: Size {
                    //width: Val::Px(200.0),
                    ..Default::default()
                },
                ..Default::default()
            },
            text: Text::with_section(
                "blah".to_string(),
                TextStyle {
                    font: font.clone(),
                    font_size: 50.0,
                    color: Color::WHITE,
                },
                TextAlignment {
                    ..Default::default()
                }
            ),
            ..Default::default()
        })
        .insert(FollowText);

        commands
        .spawn_bundle(TextBundle {
            style: Style {
                align_self: AlignSelf::FlexEnd,
                ..Default::default()
            },
            // Use `Text` directly
            text: Text {
                // Construct a `Vec` of `TextSection`s
                sections: vec![
                    TextSection {
                        value: "FPS: ".to_string(),
                        style: TextStyle {
                            font: font.clone(),
                            font_size: 20.0,
                            color: Color::WHITE,
                        },
                    },
                    TextSection {
                        value: "".to_string(),
                        style: TextStyle {
                            font: font.clone(),
                            font_size: 20.0,
                            color: Color::GOLD,
                        },
                    },
                ],
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(FpsText);


    println!("Added HUD");
}

pub fn update_fps(diagnostics: Res<Diagnostics>, mut query: Query<&mut Text, With<FpsText>>) {
    for mut text in query.iter_mut() {
        if let Some(fps) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
            if let Some(average) = fps.average() {
                text.sections[1].value = format!("{:.2}", average);
            }
        }
    }
}

pub fn update_hud_text_position(
    windows: Res<Windows>,
    mut text_query: Query<(&mut Style, &CalculatedSize, &mut Text), With<FollowText>>,
    mesh_query: Query<&Transform, With<HudFoodMesh>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Hud3DCamera>>,
    score: Res<score::Score>,
    level: Res<Level>,
) {
    for (camera, camera_transform) in camera_query.iter() {
        for mesh_position in mesh_query.iter() {
            for (mut style, calculated, mut text) in text_query.iter_mut() {
                text.sections[0].value = format!("{} / {}", score.current_level, level.get_current_minimum_food()).into();
                match camera.world_to_screen(&windows, camera_transform, mesh_position.translation)
                {
                    Some(coords) => {
                        style.position.left = Val::Px(coords.x + 100.0 - calculated.size.width / 2.0);
                        style.position.bottom = Val::Px(coords.y - calculated.size.height / 2.0);
                    }
                    None => {
                        // A hack to hide the text when the cube is behind the camera
                        style.position.bottom = Val::Px(-1000.0);
                    }
                }
            }
        }
    }
}

pub fn pause_game(
    mut state: ResMut<State<crate::AppState>>,
    keyboard_input: Res<Input<KeyCode>>,
    axes: Res<Axis<GamepadAxis>>,
    buttons: Res<Input<GamepadButton>>,
    gamepad: Option<Res<game_controller::GameController>>,
) {
    let pressed_buttons = game_controller::get_pressed_buttons(&axes, &buttons, gamepad);
    if keyboard_input.just_pressed(KeyCode::Escape) || pressed_buttons.contains(&game_controller::GameButton::Start) {
        state.push(crate::AppState::Pause).unwrap();
    }
}

pub fn hide_blocks(
    mut blocks: Query<&mut Visible>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    if keyboard_input.just_pressed(KeyCode::T) {
        for mut visible in blocks.iter_mut() {
            visible.is_visible = false;
        }
    }
}

pub fn light_thing(
    mut light: Query<&mut Light>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    if keyboard_input.just_pressed(KeyCode::T) {
        if let Ok(mut light) = light.single_mut() {
            light.intensity += 1.0;
            println!("Intense: {}",light.intensity);
        }
    }

    if keyboard_input.just_pressed(KeyCode::G) {
        if let Ok(mut light) = light.single_mut() {
            light.intensity -= 1.0;
            println!("Intense: {}",light.intensity);
        }
    }

    if keyboard_input.just_pressed(KeyCode::Y) {
        if let Ok(mut light) = light.single_mut() {
            light.range += 100.0;
            println!("Range : {}",light.range);
        }
    }

    if keyboard_input.just_pressed(KeyCode::H) {
        if let Ok(mut light) = light.single_mut() {
            light.range -= 100.0;
            println!("Range : {}",light.range);
        }
    }
}

#[derive(Bundle)]
pub struct MyLightBundle {
    light: bevy::pbr::AmbientLight
}

pub struct DisplayText(pub String);
pub struct FollowText;
pub struct FpsText;

#[derive(RenderResources, Default, TypeUuid)]
#[uuid = "463e4b8a-d555-4fc2-ba9f-4c880063ba92"]
pub struct TimeUniform {
    pub value: f32,
}

#[derive(Default)]
pub struct GameShaders {
    pub electric: Handle<PipelineDescriptor>
}

pub fn animate_shader(time: Res<Time>, mut query: Query<&mut TimeUniform>) {
    for mut time_uniform in query.iter_mut() {
        time_uniform.value = time.seconds_since_startup() as f32;
    }
}
