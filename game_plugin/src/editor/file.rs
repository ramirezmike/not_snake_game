use bevy::prelude::*;
use bevy::asset::AssetServerSettings;
use serde::{Serialize, Deserialize};
use crate::editor::{properties, add_entity, GameEntity};
use crate::{AppState, dude, snake};
use std::{fs, path::PathBuf};
use rfd::FileDialog;

pub struct SaveLevelEvent;
pub struct LoadLevelEvent;
pub struct NewLevelEvent;

pub struct EditorFilePlugin;
impl Plugin for EditorFilePlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_update(AppState::Editor)
                .with_system(save_level_event_handler)
                .with_system(load_level_event_handler)
                .with_system(new_level_event_handler)
        )
        .add_event::<NewLevelEvent>()
        .add_event::<LoadLevelEvent>()
        .add_event::<SaveLevelEvent>();
    }
}

#[derive(Serialize, Deserialize)]
struct LevelFile {
    properties: properties::Properties,
    blocks: Vec::<BlockEntity>,
    not_snakes: Vec::<NotSnakeEntity>,
    snakes: Vec::<SnakeEntity>,
    foods: Vec::<FoodEntity>,
}

#[derive(Serialize, Deserialize)]
struct BlockEntity {
    properties: properties::block::BlockProperties,
    position: [f32; 3],
}

#[derive(Serialize, Deserialize)]
struct NotSnakeEntity {
    properties: properties::not_snake::NotSnakeProperties,
    position: [f32; 3],
}

#[derive(Serialize, Deserialize)]
struct SnakeEntity {
    properties: properties::snake::SnakeProperties,
    position: [f32; 3],
}

#[derive(Serialize, Deserialize)]
struct FoodEntity {
    properties: properties::food::FoodProperties,
    position: [f32; 3],
}

fn new_level_event_handler(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut properties: ResMut<properties::Properties>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    existing_entities: Query<Entity, With<GameEntity>>,
    mut new_level_event_reader: EventReader<NewLevelEvent>,
) {
    if new_level_event_reader.iter().count() == 0 { return; }

    // remove anything that is currently in the editor
    for entity in existing_entities.iter() {
        commands.entity(entity).despawn_recursive();
    }

    *properties = properties::Properties::new();

    add_entity::add_block(
        &mut commands,
        &mut meshes,
        &mut materials,
        &properties.block,
        &Vec3::new(0.0, 0.5, 0.0),
    );
}

fn load_level_event_handler(
    mut commands: Commands,
    mut load_event_reader: EventReader<LoadLevelEvent>,
    mut properties: ResMut<properties::Properties>,
    existing_entities: Query<Entity, With<GameEntity>>,
    asset_server_settings: Res<AssetServerSettings>,
    mut meshes: (ResMut<dude::DudeMeshes>, ResMut<snake::EnemyMeshes>, ResMut<Assets<Mesh>>),
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if load_event_reader.iter().count() == 0 { return; }


    let load_file_path = FileDialog::new()
                                .set_directory(&format!("{}/levels", asset_server_settings.asset_folder))
                                .pick_file();
    if load_file_path.is_none() {
        println!("Error selecting file to load");
        return;
    }
    let load_file_path = load_file_path.expect("Just checked that file is populated");

    // remove anything that is currently in the editor
    for entity in existing_entities.iter() {
        commands.entity(entity).despawn_recursive();
    }

    let mut loaded_level = None;
    match fs::read_to_string(load_file_path) {
        Ok(file) => {
            match serde_json::from_str::<LevelFile>(&file) {
                Ok(level) => {
                    println!("Level loaded successfully");
                    loaded_level = Some(level);
                },
                Err(e) => error!("Error deserializing level: {:?}", e),
            }
        }
        Err(e) => error!("Error loading level: {:?}", e),
    }

    if let Some(level) = loaded_level {
        let (mut not_snake_meshes, mut snake_meshes, mut basic_meshes) = meshes;
        *properties = level.properties.clone();

        for block in level.blocks.iter() {
            add_entity::add_block(
                &mut commands,
                &mut basic_meshes,
                &mut materials,
                &block.properties,
                &Vec3::from_slice(&block.position),
            );
        }

        for not_snake in level.not_snakes.iter() {
            add_entity::add_not_snake(
                &mut commands,
                &mut not_snake_meshes,
                &mut materials,
                &not_snake.properties,
                &Vec3::from_slice(&not_snake.position),
            );
        }

        for snake in level.snakes.iter() {
            add_entity::add_snake(
                &mut commands,
                &mut snake_meshes,
                &mut materials,
                &snake.properties,
                &Vec3::from_slice(&snake.position),
            );
        }

        for food in level.foods.iter() {
            add_entity::add_food(
                &mut commands,
                &mut basic_meshes,
                &mut materials,
                &food.properties,
                &Vec3::from_slice(&food.position),
            );
        }
    }
}

fn save_level_event_handler(
    mut save_event_reader: EventReader<SaveLevelEvent>,
    asset_server_settings: Res<AssetServerSettings>,
    properties: Res<properties::Properties>,
    blocks: Query<(&properties::block::BlockProperties, &Transform)>,
    not_snakes: Query<(&properties::not_snake::NotSnakeProperties, &Transform)>,
    snakes: Query<(&properties::snake::SnakeProperties, &Transform)>,
    foods: Query<(&properties::food::FoodProperties, &Transform)>,
) {
    if save_event_reader.iter().count() == 0 { return; }

    let level_file = LevelFile {
        properties: properties.as_ref().clone(),
        blocks: blocks.iter()
                      .map(|(block_property, transform)| {
                          BlockEntity {
                              properties: block_property.clone(),
                              position: transform.translation.to_array(),
                          }
                      })
                      .collect(),
        not_snakes: not_snakes.iter()
                              .map(|(not_snake_property, transform)| {
                                  NotSnakeEntity {
                                      properties: not_snake_property.clone(),
                                      position: transform.translation.to_array(),
                                  }
                              })
                              .collect(),
        snakes: snakes.iter()
                      .map(|(snake_property, transform)| {
                          SnakeEntity {
                              properties: snake_property.clone(),
                              position: transform.translation.to_array(),
                          }
                      })
                      .collect(),
        foods: foods.iter()
                    .map(|(food_property, transform)| {
                        FoodEntity {
                            properties: food_property.clone(),
                            position: transform.translation.to_array(),
                        }
                    })
                    .collect(),
    };

    if let Ok(serialized_level) = serde_json::to_string(&level_file) {
        let save_file_path = FileDialog::new()
                                    .set_directory(&format!("{}/levels", asset_server_settings.asset_folder))
                                    .set_file_name(&format!("{}.lvl", properties.level_title))
                                    .save_file();
        if let Some(save_file_path) = save_file_path {
            match fs::write(save_file_path, serialized_level) {
                Err(e) => error!("Error storing level: {:?}", e),
                _ => println!("File saved successfully")
            }
        }
    } else {
        println!("Error serializing level");
    }
}
