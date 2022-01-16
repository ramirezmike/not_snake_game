use crate::editor::{cleanup_editor, editor_camera, GameEntity, GameEntityType};
use crate::level::LevelInfo;
use crate::{
    dude, dust, environment, food, game_controller, holdable, level, moveable, path_find, snake,
    sounds, AppState,
};
use bevy::prelude::*;

pub struct EditorPlayPlugin;
impl Plugin for EditorPlayPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_enter(AppState::EditorPlay)
                .with_system(load_current_editor_level.label("load_level_from_editor"))
                .with_system(cleanup_editor.after("load_level_from_editor"))
                .with_system(
                    environment::load_level
                        .label("loading_level")
                        .after("load_level_from_editor"),
                )
                .with_system(environment::load_level_into_path_finder.after("loading_level")),
        )
        .insert_resource(CurrentEditorLevel { level_info: None })
        .add_system_set(
            SystemSet::on_update(AppState::EditorPlay)
                .with_system(editor_camera::update_camera)
                .with_system(snake::update_enemy)
                .with_system(snake::add_body_parts)
                .with_system(snake::add_body_to_reach_level_min)
                .with_system(snake::handle_food_eaten)
                .with_system(food::animate_food)
                .with_system(food::update_food)
                .with_system(food::handle_food_eaten)
                .with_system(moveable::update_moveable.label("handle_moveables"))
                .with_system(holdable::lift_holdable.label("handle_lift_events"))
                .with_system(holdable::update_held.before("handle_lift_events"))
                .with_system(level::broadcast_changes.after("handle_moveables"))
                .with_system(snake::update_following)
                .with_system(dude::handle_squashes)
                .with_system(path_find::update_graph.label("graph_update"))
                .with_system(path_find::update_path.after("graph_update"))
                .with_system(food::animate_spawn_particles)
                .with_system(game_controller::gamepad_connections)
                .with_system(environment::shrink_shrinkables)
                .with_system(environment::grow_growables)
                .with_system(dust::handle_create_dust_event)
                .with_system(dust::animate_dust),
        );
    }
}

fn load_current_editor_level(
    mut level: ResMut<level::Level>,
    mut current_editor_level: ResMut<CurrentEditorLevel>,
    game_entities: Query<(&Transform, &GameEntity)>,
) {
    current_editor_level.level_info = None; // TODO remove this

    println!("Loading editor level");
    level.load_stored_levels(level::LevelsAsset {
        start_level: 0,
        palette: level::Palette {
            base: "b7b7a4".to_string(),
            ground_1: "463c5e".to_string(),
            ground_2: "6b705c".to_string(), //"444774",
            dude: "f3a787".to_string(),
            enemy: "ff4f69".to_string(),
            block: "d8e2dc".to_string(), //"85daeb",
            flag: "fdfe89".to_string(),
            food: "fdfe89".to_string(),
        },
        levels: vec![convert_state_to_level(&game_entities)],
    });
}

pub struct CurrentEditorLevel {
    level_info: Option<LevelInfo>,
}

pub fn convert_state_to_level(game_entities: &Query<(&Transform, &GameEntity)>) -> LevelInfo {
    let mut min_x = f32::MAX;
    let mut max_x = f32::MIN;
    let mut min_y = f32::MAX;
    let mut max_y = f32::MIN;
    let mut min_z = f32::MAX;
    let mut max_z = f32::MIN;

    for (transform, _) in game_entities.iter() {
        println!("Spot: {:?}", transform.translation);
        min_x = min_x.min(transform.translation.x);
        max_x = max_x.max(transform.translation.x);
        min_y = min_y.min(transform.translation.y - 0.5);
        max_y = max_y.max(transform.translation.y - 0.5);
        min_z = min_z.min(transform.translation.z);
        max_z = max_z.max(transform.translation.z);
    }

    println!("min x {} max x {}", min_x, max_x);
    println!("min y {} max y {}", min_y, max_y);
    println!("min z {} max z {}", min_z, max_z);

    let x_length = ((max_x - min_x) + 1.0).abs() as usize;
    let y_length = ((max_y - min_y) + 6.0).abs() as usize;
    let z_length = ((max_z - min_z) + 1.0).abs() as usize;

    println!("X len {} Y len {} Z len {}", x_length, y_length, z_length);

    let mut level = vec![vec![vec![0; z_length]; x_length]; y_length];

    for (transform, game_entity) in game_entities.iter() {
        // shift everything so bottom left of the editor coordinates is 0, 0, 0
        let x_index = (transform.translation.x - min_x) as usize;
        let y_index = y_length - 1 - (transform.translation.y - min_y - 0.5) as usize; // y is inverted (also in editor has 0.5 offset)
        let z_index = (transform.translation.z - min_z) as usize;
        println!("Storing X {} Y {} Z {}", x_index, y_index, z_index);

        level[y_index][x_index][z_index] = match game_entity.entity_type {
            GameEntityType::Block => 1,
            GameEntityType::Snake => 5,
            GameEntityType::NotSnake => 11,
        };
    }

    println!("Printing level for debug");
    for y in level.iter() {
        for x in y.iter() {
            for z in x.iter() {
                print!("{:?}", z);
            }
            print!("\n");
        }
        print!("\n\n");
    }

    LevelInfo {
        title: "Editor Level".to_string(),
        level,
        score_text: vec![],
        level_text: vec![],
        is_food_random: true,
        minimum_food: 10,
        palette: None,
        snake_speed: None,
        snake_target: None,
        snake_min_length: Some(5),
        camera_x: -7.8,
        camera_y: 6.93746,
        camera_z: 4.988021,
        camera_rotation_x: -0.1940206,
        camera_rotation_y: -0.9615429,
        camera_rotation_z: -0.1944001,
        camera_rotation_angle: 1.6119548,
        camera_behaviors: vec![],
        camera_cull_x: None,
        camera_cull_y: None,
        camera_cull_z: None,
        teleporter_links: vec![],
        music: sounds::LevelMusic::new(),
    }
}
