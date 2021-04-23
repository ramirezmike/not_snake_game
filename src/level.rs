use bevy::{prelude::*,};
use crate::{GameObject, EntityType, Position, environment, moveable::Moveable};
use std::collections::HashMap;


#[derive(Debug)]
pub struct PositionChangeEvent(pub Position, pub Option::<GameObject>);

pub struct Level {
    pub width: usize,
    pub length: usize,
    pub height: usize,
    pub game_objects: Vec::<Vec::<Vec::<Option::<GameObject>>>>,
    frame_updates: Vec::<(usize, usize, usize)>
}

impl Level {
    pub fn new(width: usize, length: usize, height: usize) -> Self {
        Level { 
            width,
            length,
            height,
            game_objects: vec![vec![vec![None; length]; height]; width],
            frame_updates: vec!()
        }
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
                    EntityType::WinFlag => true,
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
