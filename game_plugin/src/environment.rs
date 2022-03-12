use bevy::prelude::*;
use bevy::reflect::{TypeUuid};
use bevy::render::camera::Camera;
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, Diagnostics};
use bevy_kira_audio::{Audio, AudioPlugin};

use crate::{level::Level, Position, collectable, dude, snake, level, 
            EntityType, GameObject, holdable, win_flag, moveable, food, score,
            camera::MainCamera, sounds, game_controller, teleporter, dust,
            level_over, credits, block, camera, path_find, path_find::PathFinder};
//use bevy_mod_debugdump::print_schedule_runner;

// material.shaded = false
#[derive(Component)]
pub struct Shadow;
#[derive(Component)]
pub struct PlatformMesh;
#[derive(Component)]
pub struct BlockMesh;
pub struct LevelReady(pub bool);
pub struct GameOver(pub bool);
pub struct EnvironmentPlugin;
impl Plugin for EnvironmentPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Level::new())
           .insert_resource(PathFinder::new())
           .insert_resource(LevelReady(false))
           .insert_resource(GameOver(false))
           .insert_resource(score::Score::new())
           .insert_resource(sounds::CollectSounds::new())
           .init_resource::<dude::DudeMeshes>()
           .init_resource::<snake::EnemyMeshes>()
           .init_resource::<camera::CameraMeshes>()
           .init_resource::<win_flag::WinFlagMeshes>()
           .init_resource::<camera::CameraMouthMovement>()
           .init_resource::<camera::CameraBoltMovement>()
           .init_resource::<camera::CameraSpikeMovement>()
           .init_resource::<AssetsLoading>()
           .add_plugin(AudioPlugin)
           .add_plugin(camera::CameraPlugin)
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
           .add_event::<dude::DudeDiedEvent>()
           .add_event::<dust::CreateDustEvent>()
           .add_event::<food::FoodEatenEvent>()
           .add_event::<level::NextLevelEvent>()
           .add_system_set(
               SystemSet::on_enter(crate::AppState::Loading)
                         .with_system(load_assets.system())
                         .with_system(cleanup_environment.system())
           )
           .add_system_set(
               SystemSet::on_exit(crate::AppState::Loading)
                         .with_system(sounds::load_rest_of_sounds)
           )
           .add_system_set(
               SystemSet::on_update(crate::AppState::Loading)
                   .with_system(check_assets_ready.system())
           )
           .add_system_set(
               SystemSet::on_enter(crate::AppState::ScoreDisplay)
                   .with_system(sounds::stop_electricity.system())
                   .with_system(sounds::play_after_music.system())
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
                   .with_system(sounds::set_level_music.system().label("level_music"))
                   .with_system(sounds::play_before_music.system().after("level_music"))
                   .with_system(level_over::setup_level_over_screen.system())
           )
           .add_system_set(
               SystemSet::on_exit(crate::AppState::LevelTitle)
                   .with_system(sounds::play_fanfare.system())
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
                   .with_system(sounds::pause_music.system())
                   .with_system(crate::pause::setup_menu.system())
           )
           .add_system_set(
               SystemSet::on_update(crate::AppState::Pause)
                   .with_system(crate::pause::pause_menu.system())
           )
           .add_system_set(
               SystemSet::on_exit(crate::AppState::Pause)
                   .with_system(crate::pause::cleanup_pause_menu.system())
                   .with_system(sounds::unpause_music.system())
           )
           .add_system_set(
               SystemSet::on_update(crate::AppState::ResetLevel)
                   .with_system(camera::handle_player_death.system())
           )
           .add_system_set(
               SystemSet::on_enter(crate::AppState::InGame)
                         .with_system(load_level.system().label("loading_level"))
                         .with_system(crate::camera::create_camera.label("create_camera").after("loading_level"))
                         .with_system(create_hud.after("create_camera"))
                         .with_system(set_clear_color.system().after("loading_level"))
                         .with_system(load_level_into_path_finder.system().after("loading_level"))
                         .with_system(reset_score.system())
                         .with_system(disable_shadows.after("loading_level"))
                         .with_system(sounds::reset_sounds.system())
                         .with_system(sounds::play_ingame_music.system())
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
               .with_system(score::handle_kill_dude.system())
               .with_system(food::animate_food.system())
               .with_system(food::update_food.system())
               .with_system(food::handle_food_eaten.system())
//               .with_system(hide_blocks.system())
               .with_system(light_thing.system())
//              .with_system(snake::add_body_part.system())
               .with_system(snake::add_body_parts.system())
               .with_system(snake::add_body_to_reach_level_min.system())
               .with_system(snake::update_following.system())
               .with_system(sounds::adjust_electricity_volume.system())
               .with_system(snake::handle_kill_snake.system())
               .with_system(dude::handle_squashes.system())
               .with_system(camera::handle_player_death.system())
               .with_system(dude::handle_kill_dude.system())
               .with_system(dude::handle_snake_escapes.system())
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
//              .with_system(update_fps.system())
               .with_system(camera::cull_blocks.system())
               .with_system(snake::detect_dude_on_electric_snake.system())
               .with_system(shrink_shrinkables.system())
               .with_system(grow_growables.system())
               //.with_system(debug_level_over.system())
               .with_system(dust::handle_create_dust_event.system())
               .with_system(dust::animate_dust.system())
               //.with_system(snake::debug_trigger_snake_death.system())
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

fn disable_shadows(
    mut commands: Commands,
    mut lights: Query<&mut DirectionalLight>,
    level: Res<Level>,
) {
    if level.current_level == 16 {
        for mut light in lights.iter_mut() {
            light.shadows_enabled = false; 
        }
    }
}

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

#[derive(Component)]
pub struct Grow;
pub fn grow_growables(
    mut commands: Commands,
    mut growables: Query<(Entity, &mut Transform), With<Grow>>,
    time: Res<Time>,
) {
    for (entity, mut transform) in growables.iter_mut() {
        if transform.scale.x >= dude::SCALE {
            transform.scale = Vec3::new(dude::SCALE, dude::SCALE, dude::SCALE);
            commands.entity(entity).remove::<Grow>();
            continue;
        }

        transform.scale += Vec3::splat(time.delta_seconds() * 0.3);
    }
}

#[derive(Component)]
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

    if *timer > 0.2 {
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
) {
    let audio_state = sounds::AudioState::new(&asset_server);
    let font_handle: Handle<Font> = asset_server.load(crate::FONT);

    dude_meshes.step1 = asset_server.load("models/dude.glb#Mesh0/Primitive0");
    dude_meshes.body = asset_server.load("models/chip.glb#Mesh0/Primitive0");
    dude_meshes.head = asset_server.load("models/chip.glb#Mesh1/Primitive0");

    enemy_meshes.head = asset_server.load("models/snake.glb#Mesh0/Primitive0");
    enemy_meshes.body = asset_server.load("models/snake.glb#Mesh1/Primitive0");
    enemy_meshes.shadow = meshes.add(Mesh::from(shape::Plane { size: 0.75 }));

    camera_meshes.bolt = asset_server.load("models/bolt.glb#Mesh0/Primitive0");
    camera_meshes.spikes = asset_server.load("models/spikes.glb#Mesh0/Primitive0");

    flag_meshes.flag = asset_server.load("models/winflag.glb#Mesh0/Primitive0");
    level_asset_state.handle = asset_server.load("data/test.custom");

    loading.0.append(&mut audio_state.get_sound_handles());
    loading.0.push(level_asset_state.handle.clone_untyped());
    loading.0.push(font_handle.clone_untyped());

    loading.0.push(dude_meshes.step1.clone_untyped());
    loading.0.push(dude_meshes.head.clone_untyped());
    loading.0.push(dude_meshes.body.clone_untyped());
    loading.0.push(enemy_meshes.head.clone_untyped());
    loading.0.push(enemy_meshes.body.clone_untyped());
    loading.0.push(flag_meshes.flag.clone_untyped());
    loading.0.push(camera_meshes.bolt.clone_untyped());
    loading.0.push(camera_meshes.spikes.clone_untyped());

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
    use bevy::asset::LoadState;

    let mut ready = true;

    for handle in loading.0.iter() {
        match server.get_load_state(handle) {
            LoadState::Failed => {
                // one of our assets had an error
                println!("Failed to load {:?}", handle);
            }
            LoadState::Loaded => (),
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
    dust: Query<Entity, With<dust::Dust>>,
    platforms: Query<Entity, With<PlatformMesh>>,
    ui_cameras: Query<Entity, With<bevy::render::camera::OrthographicProjection>>,
    texts: Query<Entity, With<Text>>,
    teleporters: Query<Entity, With<teleporter::Teleporter>>,
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

    for entity in ui_cameras.iter() {
        commands.entity(entity).despawn_recursive();
    }

    for entity in platforms.iter() {
        commands.entity(entity).despawn_recursive();
    }
    
    for entity in texts.iter() {
        commands.entity(entity).despawn_recursive();
    }

    for entity in dust.iter() {
        commands.entity(entity).despawn_recursive();
    }

    for entity in teleporters.iter() {
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
    audio: Res<Audio>,
    mut audio_state: ResMut<sounds::AudioState>,
    camera: Query<Entity, With<camera::MainCamera>>,
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

    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.50,
    });

    let palette = &level.get_palette();

    dude_meshes.material = materials.add(Color::hex(palette.dude.clone()).unwrap().into());
    let enemy_color = Color::hex(palette.enemy.clone()).unwrap();
    enemy_meshes.material = materials.add(StandardMaterial {
                                base_color: enemy_color,
                                perceptual_roughness: 1.0,
                                metallic: 0.4,
                                ..Default::default()
                            });

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
                        reflectance: 0.0,
                        ..Default::default()
                    });
        ground_2_material = 
        materials.add(StandardMaterial {
                        base_color: Color::BLACK,
                        unlit: true,
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
                    14 => {
                        let entity = commands.spawn_bundle(PbrBundle { ..Default::default() }).id();
                        level.set(x as i32, y as i32, z as i32, Some(GameObject::new(entity, EntityType::Block)));
                    },
                    item @ 1 | item @ 8 | item @ 9 => { // platform
                        let entity_type = if item == 9 {
                                              EntityType::UnstandableBlock
                                          } else {
                                              EntityType::Block
                                          };
                        let mut block =
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
                            });
                            block.insert(entity_type)
                                 .insert(BlockMesh);
                        if level.current_level == 16 {
                            block.insert(bevy::pbr::NotShadowCaster)
                                 .insert(bevy::pbr::NotShadowReceiver);
                        } 
                        let entity = block.id();

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
                        let mut block =
                            commands.spawn_bundle(PbrBundle {
                              transform: Transform::from_xyz(x as f32, y as f32, z as f32),
                                material: block_material.clone(),
                              ..Default::default()
                            });
                            block
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
                            });

                            block
                            .insert(EntityType::Block)
                            .insert(holdable::Holdable {})
                            .insert(Position { x: x as i32, y: y as i32, z: z as i32 })
                            .insert(block::BlockObject { })
                            .insert(moveable::Moveable::new(0.1, inner_mesh_vertical_offset));

                        if level.current_level == 16 {
                            block.insert(bevy::pbr::NotShadowCaster)
                                 .insert(bevy::pbr::NotShadowReceiver);
                        } 

                        let block_entity = block.id();

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
                                    visibility: Visibility {
                                        is_visible: false,
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
                    7 => {
                        let entity = commands.spawn_bundle(PbrBundle { ..Default::default() }).id();
                        level.set(x as i32, y as i32, z as i32, Some(GameObject::new(entity, EntityType::PathfindIgnore)));
                    },
                    11 => dude::spawn_player(&mut commands, &dude_meshes, &mut level, x, y, z),
                    item @ 5 | item @ 10 => {
                        snake::spawn_enemy(&mut commands, &enemy_meshes, &mut level, x, y, z, item == 10);

                        if item == 10 {
                            audio_state.play_electricity(&audio);
                        }
                    },
                    13 => {
                        let id = 
                            food::spawn_food(&mut commands, &mut level, &mut meshes, &mut materials, 
                                             Some(Position{ x: x as i32, y: y as i32, z: z as i32 }), false, false);
                        level.set(x as i32, y as i32, z as i32, Some(GameObject::new(id, EntityType::PathfindIgnore)));
                    }
                    item @ 4 | item @ 6 | item @ 12 => {
                        food::spawn_food(&mut commands, &mut level, &mut meshes, &mut materials, 
                                         Some(Position{ x: x as i32, y: y as i32, z: z as i32 }), item == 6 || item == 4, item == 4);
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

//    println!("Level is Loaded... Number of items{:?}", entities.iter().len());

    level_ready.0 = true;
}

fn create_hud(
    mut commands: Commands,
    state: Res<State<crate::AppState>>,
    main_camera: Query<Entity, With<camera::MainCamera>>, 
    asset_server: Res<AssetServer>,
    mut level: ResMut<Level>,
) {
    if *state.current() == crate::AppState::MainMenu {
        return;
    }

    commands.spawn_bundle(UiCameraBundle::default());

    // Set up UI labels for clarity
    let font = asset_server.load(crate::FONT);

    commands
        .spawn_bundle(TextBundle {
            style: Style {
                align_self: AlignSelf::FlexEnd,
                ..Default::default()
            },
            text: Text::with_section(
                "Score".to_string(),
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

//      commands
//      .spawn_bundle(TextBundle {
//          style: Style {
//              align_self: AlignSelf::FlexEnd,
//              ..Default::default()
//          },
//          // Use `Text` directly
//          text: Text {
//              // Construct a `Vec` of `TextSection`s
//              sections: vec![
//                  TextSection {
//                      value: "FPS: ".to_string(),
//                      style: TextStyle {
//                          font: font.clone(),
//                          font_size: 20.0,
//                          color: Color::WHITE,
//                      },
//                  },
//                  TextSection {
//                      value: "".to_string(),
//                      style: TextStyle {
//                          font: font.clone(),
//                          font_size: 20.0,
//                          color: Color::GOLD,
//                      },
//                  },
//              ],
//              ..Default::default()
//          },
//          ..Default::default()
//      })
//      .insert(FpsText);


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
    mut text_query: Query<(&mut Style, &CalculatedSize, &mut Text), With<FollowText>>,
    score: Res<score::Score>,
    level: Res<Level>,
) {
    for (mut style, calculated, mut text) in text_query.iter_mut() {
        if level.current_level == 14 {
            text.sections[0].value = format!("Score: {} / {}", "{UNDEFINED}", 
                                                               level.get_current_minimum_food()).into();
        } else {
            text.sections[0].value = format!("Score: {} / {}", score.current_level, 
                                                               level.get_current_minimum_food()).into();
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
    mut blocks: Query<&mut Visibility>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    if keyboard_input.just_pressed(KeyCode::T) {
        for mut visible in blocks.iter_mut() {
            visible.is_visible = false;
        }
    }
}

pub fn light_thing(
    mut light: Query<&mut PointLight>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    if keyboard_input.just_pressed(KeyCode::T) {
        if let Ok(mut light) = light.get_single_mut() {
            light.intensity += 1.0;
            println!("Intense: {}",light.intensity);
        }
    }

    if keyboard_input.just_pressed(KeyCode::G) {
        if let Ok(mut light) = light.get_single_mut() {
            light.intensity -= 1.0;
            println!("Intense: {}",light.intensity);
        }
    }

    if keyboard_input.just_pressed(KeyCode::Y) {
        if let Ok(mut light) = light.get_single_mut() {
            light.range += 100.0;
            println!("Range : {}",light.range);
        }
    }

    if keyboard_input.just_pressed(KeyCode::H) {
        if let Ok(mut light) = light.get_single_mut() {
            light.range -= 100.0;
            println!("Range : {}",light.range);
        }
    }
}

pub struct DisplayText(pub String);
#[derive(Component)]
pub struct FollowText;
#[derive(Component)]
pub struct FpsText;

pub fn debug_level_over(
    keyboard_input: Res<Input<KeyCode>>,
    mut level_over_event_writer: EventWriter<level_over::LevelOverEvent>,
) {
    if keyboard_input.just_pressed(KeyCode::N) {
        level_over_event_writer.send(level_over::LevelOverEvent {});
    }
}
