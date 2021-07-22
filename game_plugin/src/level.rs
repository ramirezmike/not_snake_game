use bevy::{prelude::*,};
use crate::{GameObject, EntityType, Position, dude, camera::CameraBehavior, snake, sounds, teleporter};
use rand::seq::SliceRandom;

use bevy::asset::{AssetLoader, LoadContext, LoadedAsset};
use bevy::reflect::{TypeUuid};
use bevy::utils::{BoxedFuture};
use serde::Deserialize;

#[derive(Debug)]
pub struct PositionChangeEvent(pub Position, pub Option::<GameObject>);
pub struct NextLevelEvent;

static HEIGHT_BUFFER: usize = 3;
static INITIAL_LEVEL: usize = 99999;

pub struct Level {
    pub game_objects: Vec::<Vec::<Vec::<Option::<GameObject>>>>,
    pub current_level: usize,
    palette: Palette,
    frame_updates: Vec::<(usize, usize, usize)>,
    level_info: Vec::<LevelInfo>,
    player_death_detected: bool,
}

#[derive(Debug, Clone, Deserialize, TypeUuid)]
#[uuid = "39cadc56-aa9c-4543-8640-a018b74b5052"]
pub struct LevelsAsset {
    pub start_level: usize,
    pub palette: Palette,
    pub levels: Vec::<LevelInfo>,
}

#[derive(Debug, Clone, Deserialize, TypeUuid)]
#[uuid = "99cadc56-aa9c-4543-8640-a018b74b5052"] // this needs to be actually generated 
pub struct Palette {
    pub base: String, 
    pub ground_1: String,
    pub ground_2: String,
    pub dude: String,
    pub enemy: String,
    pub block: String,
    pub flag: String,
    pub food: String,
}

#[derive(Debug, Clone, Deserialize, TypeUuid)]
#[uuid = "49cadc56-aa9c-4543-8640-a018b74b5052"] // this needs to be actually generated
pub struct LevelInfo {
    title: String,
    level: Vec::<Vec::<Vec::<usize>>>,
    is_food_random: bool,
    minimum_food: usize,
    palette: Option::<Palette>,
    snake_speed: Option::<f32>,
    snake_target: Option::<snake::SnakeTarget>, 
    snake_min_length: Option::<usize>,
    camera_x: f32,
    camera_y: f32,
    camera_z: f32,
    camera_rotation_x: f32,
    camera_rotation_y: f32,
    camera_rotation_z: f32,
    camera_rotation_angle: f32,
    camera_behaviors: Vec::<CameraBehavior>,
    camera_cull_x: Option<(f32, f32)>,
    camera_cull_y: Option<(f32, f32)>,
    camera_cull_z: Option<(f32, f32)>,
    teleporter_links: Vec::<teleporter::Teleporter>,
}

#[derive(Default)]
pub struct LevelsAssetLoader;

impl AssetLoader for LevelsAssetLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<(), anyhow::Error>> {
        Box::pin(async move {
            println!("Level asset reloaded");
            let custom_asset = ron::de::from_bytes::<LevelsAsset>(bytes)?;
            load_context.set_default_asset(LoadedAsset::new(custom_asset));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["custom"]
    }
}

#[derive(Default)]
pub struct LevelAssetState {
    pub handle: Handle<LevelsAsset>,
}

impl Level {
    pub fn new() -> Self {
        Level { 
          game_objects: vec!(),
          frame_updates: vec!(),
          palette: Palette {
              base: "000000".to_string(),
              ground_1: "000000".to_string(),
              ground_2: "000000".to_string(),
              dude: "000000".to_string(),
              enemy: "000000".to_string(),
              block: "000000".to_string(),
              flag: "000000".to_string(),
              food: "000000".to_string(),
          },
          current_level: INITIAL_LEVEL,
          level_info: vec!(),
          player_death_detected: false,
        }
    }

    pub fn get_next_level_palette(&self) -> Palette {
        if let Some(info) = self.level_info.get(self.current_level + 1) {
            if let Some(palette) = &info.palette {
                return palette.clone();
            }
        } 

        self.palette.clone()
    }

    pub fn get_palette(&self) -> Palette {
        if let Some(info) = self.level_info.get(self.current_level) {
            if let Some(palette) = &info.palette {
                return palette.clone();
            }
        } 

        self.palette.clone()
    }

    pub fn get_teleporters(&self) -> Vec::<teleporter::Teleporter> {
        if let Some(info) = self.level_info.get(self.current_level) {
            info.teleporter_links.clone()
        } else {
            vec!()
        }
    }

    pub fn width(&self) -> usize {
        if let Some(info) = self.level_info.get(self.current_level) {
            info.level[0].len()
        } else {
            0
        }
    }

    pub fn length(&self) -> usize {
        if let Some(info) = self.level_info.get(self.current_level) {
            info.level[0][0].len()
        } else {
            0
        }
    }

    pub fn height(&self) -> usize {
        HEIGHT_BUFFER + // height buffer
        if let Some(info) = self.level_info.get(self.current_level) {
            info.level.len()
        } else {
            0
        }
    }

    pub fn camera_behaviors(&self) -> &Vec::<CameraBehavior> {
        &self.level_info[self.current_level].camera_behaviors
    }

    pub fn snake_speed(&self) -> f32 {
        self.level_info[self.current_level].snake_speed.unwrap_or(0.5)
    }

    pub fn min_snake_length(&self) -> Option::<usize> {
        self.level_info[self.current_level].snake_min_length
    }

    pub fn snake_target(&self) -> snake::SnakeTarget {
        self.level_info[self.current_level].snake_target.clone().unwrap_or(snake::SnakeTarget::Normal)
    }

    pub fn load_stored_levels(&mut self, asset: LevelsAsset) {
        self.level_info = asset.levels;
        self.palette = asset.palette;
        self.current_level = if self.current_level == INITIAL_LEVEL { asset.start_level } else { self.current_level };
        self.game_objects = vec![vec![vec![None; self.length()]; self.height()]; self.width()];
        self.frame_updates = vec!();
    }

    pub fn get_level_cull_x(&self) -> Option::<(f32, f32)> {
        self.level_info[self.current_level].camera_cull_x
    }

    pub fn get_level_cull_y(&self) -> Option::<(f32, f32)> {
        self.level_info[self.current_level].camera_cull_y
    }

    pub fn get_level_cull_z(&self) -> Option::<(f32, f32)> {
        self.level_info[self.current_level].camera_cull_z
    }

    pub fn get_camera_position(&self) -> Vec3 {
        let current = &self.level_info[self.current_level];
        Vec3::new(current.camera_x, current.camera_y, current.camera_z)
    }

    pub fn get_camera_rotation(&self) -> Quat {
        let current = &self.level_info[self.current_level];
        Quat::from_axis_angle(Vec3::new(current.camera_rotation_x, current.camera_rotation_y, current.camera_rotation_z), 
                              current.camera_rotation_angle)
    }

    pub fn is_food_random(&self) -> bool {
        let current = &self.level_info[self.current_level];
        current.is_food_random
    }

    pub fn get_level_info(&self, x: usize, y: usize, z: usize) -> usize {
        // in order to make writing the levels easier, they're stored weird
        // y and x are reversed and the vec is stored [y][x][z]
        let height = self.height();
        let width = self.width();
        let current = &self.level_info[self.current_level];
        
        if y > current.level.len() - 1 && y < height + 1 {
            0
        } else {
            let height = self.height() - HEIGHT_BUFFER;
            current.level[height - y - 1][width - x - 1][z]
        }
    }

    pub fn is_last_level(&self) -> bool {
        self.current_level == self.level_info.len() - 1
    }

    pub fn change_to_next_level(&mut self) {
        self.current_level += 1;
        self.game_objects = vec![vec![vec![None; self.length()]; self.height()]; self.width()];
    }

    pub fn reset_level(&mut self) {
        self.game_objects = vec![vec![vec![None; self.length()]; self.height()]; self.width()];
    }

    pub fn set_with_vec(&mut self, position: Vec3, game_object: Option::<GameObject>) {
        self.set(position.x as i32, position.y as i32, position.z as i32, game_object);
    }

    pub fn is_inbounds(&self, x: i32, y: i32, z: i32) -> bool {
        x >= 0 && y >= 0 && z >= 0 
        && (x as usize) < self.game_objects.len()
        && (y as usize) < self.game_objects[0].len()
        && (z as usize) < self.game_objects[0][0].len()
    }

    pub fn set_with_position(&mut self, position: Position, game_object: Option::<GameObject>) {
        self.set(position.x, position.y, position.z, game_object);
    }

    pub fn set(&mut self, x: i32, y: i32, z: i32, game_object: Option::<GameObject>) {
        if x < 0 || y < 0 || z < 0 { return; }

        let (x, y, z) = (x as usize, y as usize, z as usize);
        if x < self.game_objects.len()
        && y < self.game_objects[x].len()
        && z < self.game_objects[x][y].len() { 
            if let Some(game_object) = game_object {
                if let Some(current) = self.game_objects[x][y][z] {
                    if current.entity_type == EntityType::EnemyHead && game_object.entity_type == EntityType::Dude
                    || current.entity_type == EntityType::Dude && game_object.entity_type == EntityType::EnemyHead {
                        self.player_death_detected = true;
                    }
                }
            }
            self.game_objects[x][y][z] = game_object;
            self.frame_updates.push((x, y, z));
        }
    }

    pub fn get_with_vec(&self, position: Vec3) -> Option::<GameObject> {
        self.get(position.x as i32, position.y as i32, position.z as i32)
    }

    pub fn get_with_position(&self, position: Position) -> Option::<GameObject> {
        self.get(position.x, position.y, position.z)
    }

    pub fn get(&self, x: i32, y: i32, z: i32) -> Option::<GameObject> {
        if x < 0 || y < 0 || z < 0 { return None; }

        let (x, y, z) = (x as usize, y as usize, z as usize);
        if x < self.game_objects.len()
        && y < self.game_objects[x].len()
        && z < self.game_objects[x][y].len() { 
            self.game_objects[x as usize][y as usize][z as usize]
        } else {
            None
        }
    }

    pub fn get_random_standable(&self, away_froms: &Option::<Vec::<Position>>, allow_path_ignores: bool) -> Position {
        let mut standables = vec!();

        for x in 0..self.width() {
            for y in 0..self.height() {
                for z in 0..self.length() {
                    // I'm sorry, I'm sorry, I'm sorry, I'm sorry
                    if (self.is_standable(x as i32, y as i32, z as i32) || 
                        (!allow_path_ignores || (y > 0 && self.is_type(x as i32, (y - 1) as i32, z as i32, Some(EntityType::PathfindIgnore)))))
                    && (!self.is_inbounds(x as i32, y as i32 - 1, z as i32)
                        || !self.is_type(x as i32, y as i32 - 1, z as i32, Some(EntityType::Enemy)))
                    && self.is_type(x as i32, y as i32, z as i32, None) {
                        standables.push((x as i32, y as i32, z as i32));
                    }
                }
            }
        }

        if let Some(away_froms) = away_froms {
            // return one distances summed up across all the away_froms
            standables.sort_by_key(|standable| {
                let mut distance = 0.0;
                for away_from in away_froms.iter() {
                    let away_from = Vec3::new(away_from.x as f32, away_from.y as f32, away_from.z as f32);
                    distance += Vec3::new(standable.0 as f32, standable.1 as f32, standable.2 as f32).distance(away_from);
                }

                (distance / away_froms.len() as f32) as i32
            });

            // grab latter half which should be further away from every position in away_froms
            standables = standables.drain((standables.len() / 2)..).collect();
        }

        let (x, y, z) = standables.choose(&mut rand::thread_rng()).expect("Nothing was standable");

        Position { x: *x, y: *y, z: *z }
    }

    pub fn is_position_collectable(&self, position: Position) -> bool {
        self.is_collectable(position.x, position.y, position.z)
    }

    pub fn is_collectable_with_vec(&self, position: Vec3) -> bool {
        self.is_collectable(position.x as i32, position.y as i32, position.z as i32)
    }

    pub fn is_collectable(&self, x: i32, y: i32, z: i32) -> bool {
        match self.get(x, y, z) {
            Some(game_object) => {
                match game_object.entity_type {
                    EntityType::WinFlag | EntityType::Food => true,
                    _ => false
                }
            }
            _ => false
        }
    }

    pub fn get_current_minimum_food(&self) -> usize {
        self.level_info[self.current_level].minimum_food
    }

    pub fn get_next_level_title(&self) -> String {
        self.level_info[self.current_level + 1].title.clone()
    }

    pub fn is_enterable_with_vec(&self, position: Vec3) -> bool {
        self.is_type_with_vec(position, None) 
        || self.is_collectable_with_vec(position) 
        || self.is_type_with_vec(position, Some(EntityType::PathfindIgnore))
    }

    pub fn is_position_enterable(&self, position: Position) -> bool {
        self.is_position_type(position, None) || self.is_position_collectable(position)
    }

    pub fn is_enterable(&self, x: i32, y: i32, z: i32) -> bool {
        y != 0 && (self.is_type(x, y, z, Some(EntityType::PathfindIgnore)) 
                  || self.is_type(x, y, z, None) || self.is_collectable(x, y, z))
    }

    pub fn is_position_entity(&self, position: &Position) -> bool {
        self.is_entity(position.x, position.y, position.z)
    }

    pub fn is_entity(&self, x: i32, y: i32, z: i32) -> bool {
        match self.get(x, y, z) {
            Some(game_object) => {
                match game_object.entity_type {
                    EntityType::Dude | EntityType::EnemyHead | EntityType::Enemy => true,
                    _ => false
                }
            }
            _ => false
        }
    }

    pub fn is_standable(&self, x: i32, y: i32, z: i32) -> bool {
        // this is awful
        self.is_type(x, y - 1, z, Some(EntityType::Block)) //|| y - 1 < 0
            || self.is_type(x, y - 1, z, Some(EntityType::Enemy)) 
    }

    pub fn is_position_standable(&self, position: Position) -> bool {
        self.is_standable(position.x, position.y, position.z)
    }

    pub fn is_type_with_vec(&self, position: Vec3, entity_type: Option::<EntityType>) -> bool {
        self.is_type(position.x as i32, position.y as i32, position.z as i32, entity_type)
    }

    pub fn is_with_vec(&self, position: Vec3, game_object: Option::<GameObject>) -> bool {
        self.is(position.x as i32, position.y as i32, position.z as i32, game_object)
    }

    pub fn is(&self, x: i32, y: i32, z: i32, game_object: Option::<GameObject>) -> bool {
        if x < 0 || y < 0 || z < 0 { return false; }

        let (x, y, z) = (x as usize, y as usize, z as usize);

        if let Some(x_objects) = self.game_objects.get(x) {
            if let Some(y_objects) = x_objects.get(y) {
                if let Some(stored_game_object) = y_objects.get(z) {
                    return *stored_game_object == game_object;
                }
            }
        }

        false
    }

    pub fn is_position_type(&self, position: Position, entity_type: Option::<EntityType>) -> bool {
        self.is_type(position.x, position.y, position.z, entity_type)
    }

    pub fn is_type(&self, x: i32, y: i32, z: i32, entity_type: Option::<EntityType>) -> bool {
        if x < 0 || y < 0 || z < 0 { return false; }

        let (x, y, z) = (x as usize, y as usize, z as usize);

        if let Some(x_objects) = self.game_objects.get(x) {
            if let Some(y_objects) = x_objects.get(y) {
                if let Some(game_object) = y_objects.get(z) {
                    return match (game_object, entity_type) {
                        (None, None) => true,
                        (None, _) | (_, None) => false,
                        _ => game_object.unwrap().entity_type == entity_type.unwrap()
                    }
                }
            }
        }

        false
    }

    pub fn drain_frame_updates(&mut self) -> Vec::<(Position, Option::<GameObject>)> {
        use std::mem;

        self.frame_updates.sort_unstable();
        self.frame_updates.dedup();
        let mut frame_updates = vec!();
        mem::swap(&mut self.frame_updates, &mut frame_updates);

        frame_updates
            .into_iter()
            .map(|(x, y, z)| (Position { x: x as i32, y: y as i32, z: z as i32 }, self.game_objects[x][y][z]))
            .collect()
    }
}

pub fn print_level(
    mut time: Local<f32>,
    timer: Res<Time>,
    level: Res<Level>,
) {
    *time += timer.delta_seconds();
    if *time > 1.0 {
        *time = 0.0;
        println!("--------------------------");
        for x in 0..level.width() {
            for y in 0..level.height() {
                for z in 0..level.length() {
                    if let Some(game_object) = level.get(x as i32, y as i32, z as i32) {
                        println!("x: {} y: {} z: {} {:?}", x, y, z, game_object.entity_type);
                    }
                }
            }
        }
        println!("--------------------------");
    }
}

pub fn broadcast_changes(
    mut event_writer: EventWriter<PositionChangeEvent>,
    mut kill_dude_event_writer: EventWriter<dude::KillDudeEvent>,
    mut level: ResMut<Level>,
    mut sound_writer: EventWriter<sounds::SoundEvent>,
) {
    for (position, game_object) in level.drain_frame_updates().iter() {
//        println!("position changed!");
        event_writer.send(PositionChangeEvent(*position, *game_object));
    }
    if level.player_death_detected {
        level.player_death_detected = false;
        kill_dude_event_writer.send(dude::KillDudeEvent { death_type: dude::DudeDeath::Eaten });
        sound_writer.send(sounds::SoundEvent(sounds::Sounds::Bite));
    }
}
