use bevy::prelude::*;

use crate::{dude, Position, EntityType, level_over};

pub struct Collectable { 
    pub collected: bool 
}

pub fn check_collected(
    mut collectables: Query<(&mut Collectable, &Position, &EntityType)>,
    dudes: Query<(&dude::Dude, &Position)>,
    mut level_over_event_writer: EventWriter<level_over::LevelOverEvent>,
) {
    for (mut collectable, collectable_position, collectable_entity_type) in collectables.iter_mut().filter(|x| !x.0.collected) {
        for (_dude, dude_position) in dudes.iter() {
            if collectable_position == dude_position {
                collectable.collected = true;
                match collectable_entity_type {
                    EntityType::WinFlag => level_over_event_writer.send(level_over::LevelOverEvent {}),
                    _ => ()
                }
            }
        }
    }
}
