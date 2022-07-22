use bevy::diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;

use crate::{
    block, camera, collectable, credits, dude, dust, food, asset_loading, ui, menus, LAST_LEVEL,
    holdable, level, level::Level, level_over, moveable, path_find, path_find::PathFinder, score,
    snake, audio, teleporter, win_flag, EntityType, GameObject, Position, assets::GameAssets,
};
//use bevy_mod_debugdump::print_schedule_runner;

// material.shaded = false
#[derive(Component, Copy, Clone)]
pub struct CleanupMarker;
#[derive(Component)]
pub struct Shadow;
#[derive(Component)]
pub struct PlatformMesh;
#[derive(Component)]
pub struct BlockMesh;
#[derive(Component)]
pub struct HoldableBlockMesh;
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
            .init_resource::<dude::DudeMeshes>()
            .init_resource::<snake::EnemyMeshes>()
            .init_resource::<camera::CameraMeshes>()
            .init_resource::<win_flag::WinFlagMeshes>()
            .init_resource::<camera::CameraMouthMovement>()
            .init_resource::<camera::CameraBoltMovement>()
            .init_resource::<camera::CameraSpikeMovement>()
            .add_plugin(camera::CameraPlugin)
            .add_plugin(FrameTimeDiagnosticsPlugin)
            .add_event::<holdable::LiftHoldableEvent>()
            .add_event::<level::PositionChangeEvent>()
            .add_event::<level_over::LevelOverEvent>()
            .add_event::<snake::AddBodyPartEvent>()
            .add_event::<snake::KillSnakeEvent>()
            .add_event::<dude::KillDudeEvent>()
            .add_event::<dude::DudeDiedEvent>()
            .add_event::<dust::CreateDustEvent>()
            .add_event::<food::FoodEatenEvent>()
            .add_event::<level::NextLevelEvent>()
            .add_system_set(
                SystemSet::on_enter(crate::AppState::Loading)
                    .with_system(load_assets)
            )
            .add_system_set(
                SystemSet::on_update(crate::AppState::ChangingLevel)
                    .with_system(change_level_screen),
            )
            .add_system_set(
                SystemSet::on_enter(crate::AppState::ResetLevel)
                    .with_system(score::increase_death_count),
            )
//          .add_system_set(
//              SystemSet::on_enter(crate::AppState::RestartLevel)
//                  .with_system(cleanup_environment),
//          )
            .add_system_set(
                SystemSet::on_update(crate::AppState::RestartLevel)
                    .with_system(restart_level),
            )
            .add_system_set(
                SystemSet::on_update(crate::AppState::ResetLevel)
                    .with_system(camera::handle_player_death),
            )
            .add_system_set(
                SystemSet::on_enter(crate::AppState::InGame)
                    .with_system(try_set_level_from_asset.label("load_levels_from_asset"))
                    .with_system(
                        load_level
                            .label("loading_level")
                            .after("load_levels_from_asset"),
                    )
                    .with_system(audio::play_ingame_music.after(load_level))
                    .with_system(
                        crate::camera::create_camera
                            .label("create_camera")
                            .after("loading_level"),
                    )
                    .with_system(create_hud.after("create_camera"))
                    .with_system(set_clear_color.after("loading_level"))
                    .with_system(load_level_into_path_finder.after("loading_level"))
                    .with_system(reset_score)
            )
            .insert_resource(credits::CreditsDelay(Timer::from_seconds(1.5, false)))
            .add_system_set(
                SystemSet::on_update(crate::AppState::InGame)
                    .with_system(holdable::lift_holdable.label("handle_lift_events"))
                    .with_system(holdable::update_held.before("handle_lift_events"))
                    .with_system(moveable::update_moveable.label("handle_moveables"))
                    .with_system(win_flag::update_flag)
                    .with_system(collectable::check_collected)
                    .with_system(update_hud_text_position)
                    .with_system(level_over::level_over_check)
                    //             .with_system(path_find::show_path)
                    .with_system(snake::update_enemy.after(path_find::update_path))
                    .with_system(snake::handle_food_eaten)
                    .with_system(score::handle_food_eaten)
                    .with_system(food::animate_food)
                    .with_system(food::update_food)
                    .with_system(food::handle_food_eaten)
                    .with_system(food::disable_food_shadows)
                    //               .with_system(hide_blocks)
                    //.with_system(light_thing)
                    //              .with_system(snake::add_body_part)
                    .with_system(snake::add_body_parts)
                    .with_system(snake::add_body_to_reach_level_min)
                    .with_system(snake::update_following.after(snake::update_enemy))
                    .with_system(snake::handle_kill_snake.after(snake::update_following))
                    .with_system(dude::handle_squashes)
                    .with_system(camera::handle_player_death)
                    .with_system(dude::handle_kill_dude)
                    .with_system(dude::handle_snake_escapes)
                    .with_system(path_find::update_graph.label("graph_update"))
                    .with_system(path_find::update_path.after("graph_update"))
                    //             .with_system(path_find::draw_edges)
                    //               .with_system(material_test)
                    //               .with_system(level::print_level)
                    //               .with_system(update_text_position)
                    .with_system(level::broadcast_changes.after("handle_moveables"))
                    .with_system(food::animate_spawn_particles)
                    //.with_system(update_fps)
                    .with_system(camera::cull_blocks)
                    .with_system(camera::cull_moveable_blocks)
                    .with_system(snake::detect_dude_on_electric_snake)
                    .with_system(shrink_shrinkables)
                    .with_system(grow_growables)
                    //.with_system(debug_level_over)
                    .with_system(dust::handle_create_dust_event)
                    .with_system(dust::animate_dust), //.with_system(snake::debug_trigger_snake_death)
            );
        //        println!("{}", schedule_graph(&app.app.schedule));

        //        app.set_runner(print_schedule_runner);
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
pub fn shrink_shrinkables(mut shrinkables: Query<&mut Transform, With<Shrink>>, time: Res<Time>) {
    for mut transform in shrinkables.iter_mut() {
        if transform.scale.x <= 0.0 {
            continue;
        }

        transform.scale -= Vec3::splat(time.delta_seconds() * 0.3);
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
    mut assets_handler: asset_loading::AssetsHandler,
    mut meshes: ResMut<Assets<Mesh>>,
    mut dude_meshes: ResMut<dude::DudeMeshes>,
    mut enemy_meshes: ResMut<snake::EnemyMeshes>,
    mut camera_meshes: ResMut<camera::CameraMeshes>,
    mut flag_meshes: ResMut<win_flag::WinFlagMeshes>,
    mut level_asset_state: ResMut<level::LevelAssetState>,

    mut game_assets: ResMut<GameAssets>
) {
    println!("Loading assets");
    assets_handler.add_font(&mut game_assets.font, crate::FONT);

    assets_handler.add_mesh(&mut dude_meshes.step1, "models/dude.glb#Mesh0/Primitive0");

    assets_handler.add_mesh(&mut dude_meshes.body, "models/notsnake.glb#Mesh0/Primitive0");
    assets_handler.add_mesh(&mut dude_meshes.not_snake, "models/notsnake.glb#Mesh0/Primitive0");
    assets_handler.add_mesh(&mut dude_meshes.head, "models/chip.glb#Mesh1/Primitive0");

    assets_handler.add_mesh(&mut enemy_meshes.head, "models/snake.glb#Mesh0/Primitive0");
    assets_handler.add_mesh(&mut enemy_meshes.body, "models/snake.glb#Mesh1/Primitive0");
//  enemy_meshes.shadow = meshes.add(Mesh::from(shape::Plane { size: 0.75 }));

    assets_handler.add_mesh(&mut camera_meshes.bolt, "models/bolt.glb#Mesh0/Primitive0");
    assets_handler.add_mesh(&mut camera_meshes.spikes, "models/spikes.glb#Mesh0/Primitive0");

    assets_handler.add_mesh(&mut flag_meshes.flag, "models/winflag.glb#Mesh0/Primitive0");
    assets_handler.add_asset(&mut level_asset_state.handle, "data/test.custom");

    assets_handler.add_material(&mut game_assets.bevy_icon, "textures/bevy.png", true);

    println!("Level asset state: {:?}", level_asset_state.handle);

    use bevy_kira_audio::AudioSource;
    let mut pickup0 = Handle::<AudioSource>::default();
    let mut pickup1 = Handle::<AudioSource>::default();
    let mut pickup2 = Handle::<AudioSource>::default();
    let mut pickup3 = Handle::<AudioSource>::default();
    let mut pickup4 = Handle::<AudioSource>::default();
    assets_handler.add_audio(&mut pickup0, "sounds/pickup0.ogg");
    assets_handler.add_audio(&mut pickup1, "sounds/pickup1.ogg");
    assets_handler.add_audio(&mut pickup2, "sounds/pickup2.ogg");
    assets_handler.add_audio(&mut pickup3, "sounds/pickup3.ogg");
    assets_handler.add_audio(&mut pickup4, "sounds/pickup4.ogg");
    game_assets.pickup_handle = vec!(pickup0, pickup1, pickup2, pickup3, pickup4);

    let mut bite0 = Handle::<AudioSource>::default();
    let mut bite1 = Handle::<AudioSource>::default();
    let mut bite2 = Handle::<AudioSource>::default();
    let mut bite3 = Handle::<AudioSource>::default();
    assets_handler.add_audio(&mut bite0, "sounds/bite0.ogg");
    assets_handler.add_audio(&mut bite1, "sounds/bite1.ogg");
    assets_handler.add_audio(&mut bite2, "sounds/bite2.ogg");
    assets_handler.add_audio(&mut bite3, "sounds/bite3.ogg");
    game_assets.bite_handle = vec!(bite0, bite1, bite2, bite3);

    assets_handler.add_audio(&mut game_assets.blip, "sounds/blip.wav");
    assets_handler.add_audio(&mut game_assets.flag_spawn_handle, "sounds/flagspawn.ogg");
    assets_handler.add_audio(&mut game_assets.land_handle, "sounds/land.ogg");
    assets_handler.add_audio(&mut game_assets.shock_handle, "sounds/electric.ogg");
    assets_handler.add_audio(&mut game_assets.electricity_handle, "sounds/electricity.ogg");
    assets_handler.add_audio(&mut game_assets.level_end_handle, "sounds/levelend.ogg");
    assets_handler.add_audio(&mut game_assets.slide_handle, "sounds/slide.ogg");
    assets_handler.add_audio(&mut game_assets.fall_handle, "sounds/fall.ogg");

    assets_handler.add_audio(&mut game_assets.intro_handle, "music/intro.ogg");
    assets_handler.add_audio(&mut game_assets.bass_drum_handle, "music/bassdrum.ogg");
    assets_handler.add_audio(&mut game_assets.bass_drum_reverb_handle, "music/bassdrum_reverb.ogg");
    assets_handler.add_audio(&mut game_assets.drum_and_bell_handle, "music/drum_and_bell.ogg");
    assets_handler.add_audio(&mut game_assets.level_one_handle, "music/01.ogg");
    assets_handler.add_audio(&mut game_assets.level_one_8bit_handle, "music/018bit.ogg");
    assets_handler.add_audio(&mut game_assets.halloween_handle, "music/halloween.ogg");
    assets_handler.add_audio(&mut game_assets.classic_handle, "music/classic.ogg");
    assets_handler.add_audio(&mut game_assets.boss_handle, "music/boss.ogg");
    assets_handler.add_audio(&mut game_assets.space_handle, "music/space.ogg");
    assets_handler.add_audio(&mut game_assets.hurry_handle, "music/hurry.ogg");
    assets_handler.add_audio(&mut game_assets.qwerty_handle, "music/qwerty.ogg");
    assets_handler.add_audio(&mut game_assets.credits_handle, "music/credits.ogg");
    assets_handler.add_audio(&mut game_assets.organ_handle, "music/organ.ogg");
    assets_handler.add_audio(&mut game_assets.tick_tock_handle, "music/ticktock.ogg");
}

#[cfg(not(target_arch = "wasm32"))]
fn get_shader_paths() -> (&'static str, &'static str) {
    ("shaders/hot.vert", "shaders/hot.frag")
}

#[cfg(target_arch = "wasm32")]
fn get_shader_paths() -> (&'static str, &'static str) {
    ("shaders/hot_wasm.vert", "shaders/hot_wasm.frag")
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
            EntityType::EnemyHead
            | EntityType::Enemy
            | EntityType::Dude
            | EntityType::Block
            | EntityType::WinFlag
            | EntityType::Platform
            | EntityType::Food => {
                commands.entity(entity).despawn_recursive();
            }
            _ => commands.entity(entity).despawn(),
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

pub fn reset_score(mut score: ResMut<score::Score>) {
    score.current_level = 0;
}

pub fn set_clear_color(level: Res<Level>, mut clear_color: ResMut<ClearColor>) {
    let palette = &level.get_palette();
    clear_color.0 = Color::hex(palette.base.clone()).unwrap();
}

pub fn load_level_into_path_finder(level: Res<Level>, mut path_finder: ResMut<PathFinder>) {
    path_finder.load_level(&level);
}

pub fn try_set_level_from_asset(
    mut level: ResMut<Level>,
    level_asset_state: Res<level::LevelAssetState>,
    levels_asset: ResMut<Assets<level::LevelsAsset>>,
    mut state: ResMut<State<crate::AppState>>,
) {
    println!("Starting to load level...");
    let levels_asset = levels_asset.get(&level_asset_state.handle);
    if let Some(level_asset) = levels_asset {
        level.load_stored_levels(level_asset.clone());
    } else {
        // try again later?
        println!("failed to load level");
        state.set(crate::AppState::Loading).unwrap();
        return;
    }
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
    mut audio: audio::GameAudio,
    game_assets: Res<GameAssets>,
    state: Res<State<crate::AppState>>,
) {
    println!("resetting level");
    level.reset_level();

    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.50,
    });

    let palette = &level.get_palette();

    dude_meshes.material = materials.add(StandardMaterial {
        base_color: Color::hex(palette.dude.clone()).unwrap().into(),
        perceptual_roughness: 1.0,
        metallic: 0.4,
        reflectance: 0.0,
        ..Default::default()
    });

    let enemy_color = Color::hex(palette.enemy.clone()).unwrap();
    enemy_meshes.material = materials.add(StandardMaterial {
        base_color: enemy_color,
        perceptual_roughness: 1.0,
        metallic: 0.4,
        reflectance: 0.0,
        ..Default::default()
    });

    let plane = meshes.add(Mesh::from(shape::Plane { size: 1.0 }));
    let cube = meshes.add(Mesh::from(shape::Cube { size: 1.0 }));
    let flag_color = Color::hex(palette.flag.clone()).unwrap();
    let flag_color = Color::rgba(flag_color.r(), flag_color.g(), flag_color.b(), 0.7);

    let block_material = materials.add(StandardMaterial {
        base_color: Color::hex(palette.block.clone()).unwrap().into(),
        reflectance: 0.0,
        ..Default::default()
    });
    let mut ground_1_material = materials.add(StandardMaterial {
        base_color: Color::hex(palette.ground_1.clone()).unwrap().into(),
        reflectance: 0.0,
        ..Default::default()
    });
    let mut ground_2_material = materials.add(StandardMaterial {
        base_color: Color::hex(palette.ground_2.clone()).unwrap().into(),
        reflectance: 0.0,
        ..Default::default()
    });

    if *state.current() == crate::AppState::MainMenu {
        ground_1_material = materials.add(StandardMaterial {
            base_color: Color::BLACK,
            unlit: true,
            reflectance: 0.0,
            ..Default::default()
        });
        ground_2_material = materials.add(StandardMaterial {
            base_color: Color::BLACK,
            unlit: true,
            reflectance: 0.0,
            ..Default::default()
        });
    }

    commands.spawn_bundle(UiCameraBundle::default())
            .insert(CleanupMarker);
    let space_scale = 0.9;

    for x in 0..level.width() {
        for y in 0..level.height() {
            for z in 0..level.length() {
                match level.get_level_info(x, y, z) {
                    14 => {
                        let entity = commands
                            .spawn_bundle(PbrBundle {
                                material: ground_1_material.clone(),
                                visibility: Visibility { is_visible: false },
                                ..Default::default()
                            })
                            .insert(CleanupMarker)
                            .id();
                        level.set(
                            x as i32,
                            y as i32,
                            z as i32,
                            Some(GameObject::new(entity, EntityType::Block)),
                        );
                    }
                    item @ 1 | item @ 8 | item @ 9 => {
                        // platform
                        let entity_type = if item == 9 {
                            EntityType::UnstandableBlock
                        } else {
                            EntityType::Block
                        };
                        let mut block = commands
                            .spawn_bundle(PbrBundle {
                                mesh: if y == 0 { plane.clone() } else { cube.clone() },
                                material: if item == 1 {
                                    ground_2_material.clone()
                                } else {
                                    ground_1_material.clone()
                                },
                                transform: {
                                    let mut transform = Transform::from_translation(Vec3::new(
                                        x as f32,
                                        if y == 0 {
                                            y as f32 + 0.5
                                        } else {
                                            y as f32 + 0.0
                                        },
                                        z as f32,
                                    ));
                                    transform.scale.x = space_scale;
                                    if y != 0 {
                                        transform.scale.y = space_scale;
                                    }
                                    transform.scale.z = space_scale;
                                    transform
                                },
                                ..Default::default()
                            });

                        block
                            .insert(entity_type)
                            .insert(CleanupMarker)
                            .insert(BlockMesh);

                        if level.current_level == LAST_LEVEL {
                            block.insert(bevy::pbr::NotShadowCaster);
                        } 

                        let entity = block.id();
                        level.set(
                            x as i32,
                            y as i32,
                            z as i32,
                            Some(GameObject::new(entity, entity_type)),
                        );

                        if y == 0 {
                            commands
                                .spawn_bundle(PbrBundle {
                                    mesh: cube.clone(),
                                    material: ground_1_material.clone(),
                                    transform: {
                                        let height = 30.0;
                                        let mut transform = Transform::from_translation(Vec3::new(
                                            x as f32,
                                            ((y as f32) + 0.5) - (height / 2.0) - 0.0001,
                                            z as f32,
                                        ));

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
                                })
                                .insert(CleanupMarker)
                                .insert(PlatformMesh);
                        }
                    }
                    2 => {
                        // moveable block
                        let inner_mesh_vertical_offset = 0.0;
                        let mut block = commands
                            .spawn_bundle(PbrBundle {
                                transform: Transform::from_xyz(x as f32, y as f32, z as f32),
                                material: block_material.clone(),
                                ..Default::default()
                            });

                            block
                            .with_children(|parent| {
                                let mut inner =
                                parent.spawn_bundle(PbrBundle {
                                    mesh: cube.clone(),
                                    material: block_material.clone(),
                                    transform: {
                                        let mut transform = Transform::from_xyz(
                                            0.0,
                                            inner_mesh_vertical_offset,
                                            0.0,
                                        );

                                        transform.scale.x = space_scale;
                                        transform.scale.y = space_scale;
                                        transform.scale.z = space_scale;
                                        transform
                                    },
                                    ..Default::default()
                                });

                                inner.insert(HoldableBlockMesh);

                                if level.current_level == LAST_LEVEL {
                                    inner.insert(bevy::pbr::NotShadowCaster);
                                } 
                            })
                            .insert(CleanupMarker)
                            .insert(EntityType::Block)
                            .insert(holdable::Holdable {})
                            .insert(Position {
                                x: x as i32,
                                y: y as i32,
                                z: z as i32,
                            })
                            .insert(block::BlockObject {})
                            .insert(moveable::Moveable::new(0.1, inner_mesh_vertical_offset));

                        if level.current_level == LAST_LEVEL {
                            block.insert(bevy::pbr::NotShadowCaster);
                        } 

                        let block_entity = block.id();
                        level.set(
                            x as i32,
                            y as i32,
                            z as i32,
                            Some(GameObject::new(block_entity, EntityType::Block)),
                        );
                    }
                    3 => {
                        // win_flag
                        let mut transform = Transform::from_xyz(x as f32, y as f32, z as f32);
                        transform.apply_non_uniform_scale(Vec3::new(0.25, 0.25, 0.25));
                        let position = Position::from_vec(transform.translation);

                        let entity = commands
                            .spawn_bundle(PbrBundle {
                                transform,
                                ..Default::default()
                            })
                            .with_children(|parent| {
                                parent
                                    .spawn_bundle(PbrBundle {
                                        mesh: flag_meshes.flag.clone(),
                                        material: materials.add(flag_color.into()),
                                        visibility: Visibility { is_visible: false },
                                        transform: {
                                            let mut t = Transform::from_xyz(0.0, 0.0, 0.0);
                                            t.apply_non_uniform_scale(Vec3::new(1.0, 0.1, 1.0));
                                            t
                                        },
                                        ..Default::default()
                                    })
                                    .insert(win_flag::WinFlagInnerMesh {});
                            })
                            .insert(CleanupMarker)
                            .insert(collectable::Collectable { collected: false })
                            .insert(win_flag::WinFlag {})
                            .insert(EntityType::WinFlag)
                            .insert(position)
                            .id();

                        level.set(
                            x as i32,
                            y as i32,
                            z as i32,
                            Some(GameObject::new(entity, EntityType::WinFlag)),
                        );
                    }
                    7 => {
                        let entity = commands
                            .spawn_bundle(PbrBundle {
                                ..Default::default()
                            })
                            .insert(CleanupMarker)
                            .id();
                        level.set(
                            x as i32,
                            y as i32,
                            z as i32,
                            Some(GameObject::new(entity, EntityType::PathfindIgnore)),
                        );
                    }
                    11 => dude::spawn_player(&mut commands, &dude_meshes, &mut level, x, y, z, CleanupMarker),
                    item @ 5 | item @ 10 => {
                        snake::spawn_enemy(
                            &mut commands,
                            &enemy_meshes,
                            &mut level,
                            x,
                            y,
                            z,
                            item == 10,
                            CleanupMarker
                        );

                        if item == 10 {
                            audio.play_electricity(&game_assets.electricity_handle);
                        }
                    }
                    13 => {
                        let id = food::spawn_food(
                            &mut commands,
                            &mut level,
                            &mut meshes,
                            &mut materials,
                            Some(Position {
                                x: x as i32,
                                y: y as i32,
                                z: z as i32,
                            }),
                            false,
                            CleanupMarker,
                        );
                        level.set(
                            x as i32,
                            y as i32,
                            z as i32,
                            Some(GameObject::new(id, EntityType::PathfindIgnore)),
                        );
                    }
                    item @ 4 | item @ 6 | item @ 12 => {
                        food::spawn_food(
                            &mut commands,
                            &mut level,
                            &mut meshes,
                            &mut materials,
                            Some(Position {
                                x: x as i32,
                                y: y as i32,
                                z: z as i32,
                            }),
                            false, //item == 6 || item == 4,
                            CleanupMarker,
                        );
                    }
                    _ => (),
                }
            }
        }
    }

    for teleporter in level.get_teleporters() {
        teleporter::spawn_teleporter(&mut commands, teleporter, CleanupMarker);
    }

    if level.is_food_random() {
        food::spawn_food(
            &mut commands,
            &mut level,
            &mut meshes,
            &mut materials,
            None,
            false,
            CleanupMarker,
        );
    }

    //    println!("Level is Loaded... Number of items{:?}", entities.iter().len());

    level_ready.0 = true;
}

fn create_hud(
    mut commands: Commands,
    state: Res<State<crate::AppState>>,
    asset_server: Res<AssetServer>,
    text_scaler: ui::text_size::TextScaler,
) {
    if *state.current() == crate::AppState::MainMenu {
        return;
    }

    commands.spawn_bundle(UiCameraBundle::default())
            .insert(CleanupMarker);

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
                    font_size: text_scaler.scale(menus::DEFAULT_FONT_SIZE * 0.7),
                    color: Color::WHITE,
                },
                TextAlignment {
                    ..Default::default()
                },
            ),
            ..Default::default()
        })
        .insert(CleanupMarker)
        .insert(FollowText);

//  commands
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
//      .insert(CleanupMarker)
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
    mut text_query: Query<&mut Text, With<FollowText>>,
    score: Res<score::Score>,
    level: Res<Level>,
) {
    for mut text in text_query.iter_mut() {
        if level.current_level == crate::LOST_SCORE_LEVEL {
            text.sections[0].value = format!(
                "Score: {} / {}",
                "{UNDEFINED}",
                level.get_current_minimum_food()
            )
            .into();
        } else {
            text.sections[0].value = format!(
                "Score: {} / {}",
                score.current_level,
                level.get_current_minimum_food()
            )
            .into();
        }
    }
}

pub fn hide_blocks(mut blocks: Query<&mut Visibility>, keyboard_input: Res<Input<KeyCode>>) {
    if keyboard_input.just_pressed(KeyCode::T) {
        for mut visible in blocks.iter_mut() {
            visible.is_visible = false;
        }
    }
}

pub fn light_thing(mut light: Query<&mut PointLight>, keyboard_input: Res<Input<KeyCode>>) {
    if keyboard_input.just_pressed(KeyCode::T) {
        if let Ok(mut light) = light.get_single_mut() {
            light.intensity += 1.0;
            println!("Intense: {}", light.intensity);
        }
    }

    if keyboard_input.just_pressed(KeyCode::G) {
        if let Ok(mut light) = light.get_single_mut() {
            light.intensity -= 1.0;
            println!("Intense: {}", light.intensity);
        }
    }

    if keyboard_input.just_pressed(KeyCode::Y) {
        if let Ok(mut light) = light.get_single_mut() {
            light.range += 100.0;
            println!("Range : {}", light.range);
        }
    }

    if keyboard_input.just_pressed(KeyCode::H) {
        if let Ok(mut light) = light.get_single_mut() {
            light.range -= 100.0;
            println!("Range : {}", light.range);
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
