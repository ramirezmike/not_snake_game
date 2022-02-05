use bevy::{ecs::system::SystemParam, prelude::*};
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
pub struct LevelFile {
    properties: properties::Properties,
    blocks: Vec::<BlockEntity>,
    not_snakes: Vec::<NotSnakeEntity>,
    snakes: Vec::<SnakeEntity>,
    foods: Vec::<FoodEntity>,
}

impl LevelFile {
    pub fn new() -> Self {
        LevelFile {
            properties: properties::Properties::new(), 
            blocks: vec!(),
            not_snakes: vec!(),
            snakes: vec!(),
            foods: vec!(),
        }
    }
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
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut editor_state: EditorState,
    mut new_level_event_reader: EventReader<NewLevelEvent>,
) {
    if new_level_event_reader.iter().count() == 0 { return; }

    editor_state.despawn_entities(&mut commands);

    *editor_state.properties = properties::Properties::new();

    add_entity::add_block(
        &mut commands,
        &mut meshes,
        &mut materials,
        &editor_state.properties.block,
        &Vec3::new(0.0, 0.5, 0.0),
    );
}

fn load_level_event_handler(
    mut commands: Commands,
    mut load_event_reader: EventReader<LoadLevelEvent>,
    mut editor_state: EditorState,
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

    editor_state.despawn_entities(&mut commands);

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
        *editor_state.properties = level.properties.clone();
        load_level(&mut commands, meshes, materials, &level);
    }
}

pub fn load_level(
    mut commands: &mut Commands,
    mut meshes: (ResMut<dude::DudeMeshes>, ResMut<snake::EnemyMeshes>, ResMut<Assets<Mesh>>),
    mut materials: ResMut<Assets<StandardMaterial>>,
    level: &LevelFile,
) {
    let (mut not_snake_meshes, mut snake_meshes, mut basic_meshes) = meshes;
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

// TODO: Move this to the editor root
#[derive(SystemParam)]
pub struct EditorState<'w, 's> {
    properties: ResMut<'w, properties::Properties>,
    blocks: Query<'w, 's, (Entity, &'static properties::block::BlockProperties, &'static Transform)>,
    not_snakes: Query<'w, 's, (Entity, &'static properties::not_snake::NotSnakeProperties, &'static Transform)>,
    snakes: Query<'w, 's, (Entity, &'static properties::snake::SnakeProperties, &'static Transform)>,
    foods: Query<'w, 's, (Entity, &'static properties::food::FoodProperties, &'static Transform)>,
}

impl <'w, 's> EditorState<'w, 's> {
    fn despawn_entities(&self, mut commands: &mut Commands) {
        let block_entities = self.blocks.iter().map(|block| block.0);
        let not_snake_entities =self.not_snakes.iter().map(|not_snake| not_snake.0);
        let snake_entities =self.snakes.iter().map(|snake| snake.0);
        let food_entities =self.foods.iter().map(|food| food.0);
        let all_entities = block_entities.chain(not_snake_entities)
                                         .chain(snake_entities)
                                         .chain(food_entities);
        
        all_entities.for_each(|entity| commands.entity(entity).despawn_recursive());
    }

    pub fn is_empty(&self) -> bool {
        self.blocks.is_empty() &&
        self.not_snakes.is_empty() &&
        self.snakes.is_empty() &&
        self.foods.is_empty() 
    }

    pub fn to_level_file(&self) -> LevelFile {
        LevelFile {
            properties: self.properties.as_ref().clone(),
            blocks: self
                          .blocks
                          .iter()
                          .map(|(_, block_property, transform)| {
                              BlockEntity {
                                  properties: block_property.clone(),
                                  position: transform.translation.to_array(),
                              }
                          })
                          .collect(),
            not_snakes: self
                                  .not_snakes.iter()
                                  .map(|(_, not_snake_property, transform)| {
                                      NotSnakeEntity {
                                          properties: not_snake_property.clone(),
                                          position: transform.translation.to_array(),
                                      }
                                  })
                                  .collect(),
            snakes: self.snakes.iter()
                          .map(|(_, snake_property, transform)| {
                              SnakeEntity {
                                  properties: snake_property.clone(),
                                  position: transform.translation.to_array(),
                              }
                          })
                          .collect(),
            foods: self.foods.iter()
                        .map(|(_, food_property, transform)| {
                            FoodEntity {
                                properties: food_property.clone(),
                                position: transform.translation.to_array(),
                            }
                        })
                        .collect(),
        }
    }
}

fn save_level_event_handler(
    mut save_event_reader: EventReader<SaveLevelEvent>,
    asset_server_settings: Res<AssetServerSettings>,
    editor_state: EditorState,
) {
    if save_event_reader.iter().count() == 0 { return; }

    let level_file = editor_state.to_level_file();

    if let Ok(serialized_level) = serde_json::to_string(&level_file) {
        let save_file_path = FileDialog::new()
                                    .set_directory(&format!("{}/levels", asset_server_settings.asset_folder))
                                    .set_file_name(&format!("{}.lvl", editor_state.properties.level_title))
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
