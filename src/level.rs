use bevy::{prelude::*,};
use crate::{GameObject, EntityType, Position};

pub struct Level {
    pub width: i32,
    pub length: i32,
    pub height: i32,
    pub game_objects: Vec::<Vec::<Vec::<Option::<GameObject>>>>
}

impl Level {
    pub fn set_with_vec(&mut self, position: Vec3, game_object: Option::<GameObject>) {
        self.set(position.x as i32, position.y as i32, position.z as i32, game_object);
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
}

pub fn sync_level(mut level: ResMut<Level>, positions: Query<(Entity, &Position, &EntityType)>) {
    level.game_objects = vec![vec![vec![None; 12]; 12]; 6];

    println!("==========================================================================================");
    println!("");
    println!("");
    for (entity, position, entity_type) in positions.iter() {
        level.set(position.x, position.y, position.z, Some(GameObject::new(entity, *entity_type)));
        println!("{:?} {:?}", position, entity_type);
    }
    println!("");
    println!("");
}
