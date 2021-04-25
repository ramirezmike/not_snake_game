use bevy::{prelude::*,};
use crate::{GameObject, EntityType, Position, };
use rand::seq::SliceRandom;

#[derive(Debug)]
pub struct PositionChangeEvent(pub Position, pub Option::<GameObject>);
pub struct NextLevelEvent;

pub struct Level {
    pub width: usize,
    pub length: usize,
    pub height: usize,
    pub game_objects: Vec::<Vec::<Vec::<Option::<GameObject>>>>,
    current_level: usize,
    frame_updates: Vec::<(usize, usize, usize)>,
    level_info: Vec::<LevelInfo>,
}

pub struct LevelInfo {
    pub width: usize,
    pub length: usize,
    pub height: usize,
    level: Vec::<Vec::<Vec::<usize>>>,
    snake_count: usize,
}

impl LevelInfo {
    pub fn new(level: Vec::<Vec::<Vec::<usize>>>, snake_count: usize) -> Self {
        LevelInfo { width: level[0].len(), height: level.len(), length: level[0][0].len(), level, snake_count }
    }
}

impl Level {
    pub fn new() -> Self {
        let empty = 
                    vec![
                        vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                        vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                        vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                        vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                        vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                        vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                    ];
        let empty_3 = 
                    vec![
                        vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                        vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                        vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                    ];
        let level_info = vec!(
            // Level 1
            LevelInfo::new(
                vec![
                    empty.clone(),
                    empty.clone(),
                    empty.clone(),
                    empty.clone(),
                    empty.clone(),
                    empty.clone(),
                    empty.clone(),
                    empty.clone(),
                    empty.clone(),
                    empty.clone(),
                    empty.clone(),
                    empty.clone(),
                    vec![
                        vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                        vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                        vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                        vec![0, 0, 0, 0, 0, 2, 3, 0, 0, 0, 0, 0],
                        vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                        vec![4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                    ],
                    vec![
                        vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
                        vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
                        vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
                        vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
                        vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
                        vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
                    ]
                ],
                0
            ),
            // Level 2
            LevelInfo::new(
                vec![
                    empty.clone(),
                    empty.clone(),
                    empty.clone(),
                    empty.clone(),
                    empty.clone(),
                    empty.clone(),
                    empty.clone(),
                    empty.clone(),
                    empty.clone(),
                    empty.clone(),
                    empty.clone(),
                    vec![
                        vec![4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                        vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                        vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                        vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                        vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                        vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3],
                    ],
                    vec![
                        vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
                        vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
                        vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
                        vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
                        vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
                        vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
                    ]
                ],
                1
            ),
            // Level 3
            LevelInfo::new(
                vec![
                    empty_3.clone(), 
                    empty_3.clone(), 
                    empty_3.clone(), 
                    vec![
                        vec![0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                        vec![4, 0, 2, 0, 0, 5, 0, 0, 0, 0, 0, 3],
                        vec![0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                    ],
                    vec![
                        vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
                        vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
                        vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
                    ]
                ],
                1
            )
        );
        let current_level = 0;
        let width = level_info[current_level].width;
        let length = level_info[current_level].length;
        let height = level_info[current_level].height;

        Level { 
            width,
            length,
            height,
            game_objects: vec![vec![vec![None; length]; height]; width],
            frame_updates: vec!(),
            current_level: 0,
            level_info
        }
    }

    pub fn get_level_info(&self, x: usize, y: usize, z: usize) -> usize {
        // in order to make writing the levels easier, they're stored weird
        // y and x are reversed and the vec is stored [y][x][z]
        let current = &self.level_info[self.current_level];
        current.level[current.height - y - 1][current.width - x - 1][z]
    }

    pub fn is_last_level(&self) -> bool {
        self.current_level == self.level_info.len() - 1
    }

    pub fn change_to_next_level(&mut self) {
        self.current_level += 1;
        self.width = self.level_info[self.current_level].width;
        self.length = self.level_info[self.current_level].length;
        self.height = self.level_info[self.current_level].height;
        self.game_objects = vec![vec![vec![None; self.length]; self.height]; self.width];
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

    pub fn get_random_standable(&self) -> Position {
        let mut standables = vec!();

        for x in 0..self.width {
            for y in 0..self.height {
                for z in 0..self.length {
                    if self.is_standable(x as i32, y as i32, z as i32) 
                    && (!self.is_inbounds(x as i32, y as i32 - 1, z as i32)
                        || !self.is_type(x as i32, y as i32 - 1, z as i32, Some(EntityType::Enemy)))
                    && self.is_type(x as i32, y as i32, z as i32, None) {
                        standables.push((x as i32, y as i32, z as i32));
                    }
                }
            }
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

    pub fn is_enterable_with_vec(&self, position: Vec3) -> bool {
        self.is_type_with_vec(position, None) || self.is_collectable_with_vec(position)
    }

    pub fn is_position_enterable(&self, position: Position) -> bool {
        self.is_position_type(position, None) || self.is_position_collectable(position)
    }

    pub fn is_enterable(&self, x: i32, y: i32, z: i32) -> bool {
        self.is_type(x, y, z, None) || self.is_collectable(x, y, z)
    }

    pub fn is_position_entity(&self, position: &Position) -> bool {
        self.is_entity(position.x, position.y, position.z)
    }

    pub fn is_entity(&self, x: i32, y: i32, z: i32) -> bool {
        match self.get(x, y, z) {
            Some(game_object) => {
                match game_object.entity_type {
                    EntityType::Dude | EntityType::Enemy => true,
                    _ => false
                }
            }
            _ => false
        }
    }

    pub fn is_standable(&self, x: i32, y: i32, z: i32) -> bool {
        // this is awful
        self.is_type(x, y - 1, z, Some(EntityType::Block)) || y - 1 < 0
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
        for x in 0..level.width {
            for y in 0..level.height {
                for z in 0..level.length {
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
    mut level: ResMut<Level>,
) {
    for (position, game_object) in level.drain_frame_updates().iter() {
//        println!("position changed!");
        event_writer.send(PositionChangeEvent(*position, *game_object));
    }
}
