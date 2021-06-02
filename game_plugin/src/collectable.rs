use bevy::prelude::*;

use crate::{dude, Position, EntityType, level_over, level, score::Score, sounds};

pub struct Collectable { 
    pub collected: bool 
}

pub fn check_collected(
    mut collectables: Query<(&mut Collectable, &Position, &EntityType)>,
    dudes: Query<(&dude::Dude, &Position)>,
    score: Res<Score>,
    level: ResMut<level::Level>,
    mut sound_writer: EventWriter<sounds::SoundEvent>,
    mut level_over_event_writer: EventWriter<level_over::LevelOverEvent>,
) {
    for (mut collectable, collectable_position, collectable_entity_type) in collectables.iter_mut().filter(|x| !x.0.collected) {
        for (_dude, dude_position) in dudes.iter() {
            if collectable_position == dude_position {
                match collectable_entity_type {
                    EntityType::WinFlag => {
                        if score.current_level >= level.get_current_minimum_food() {
                            level_over_event_writer.send(level_over::LevelOverEvent {});
                            sound_writer.send(sounds::SoundEvent(sounds::Sounds::LevelEnd));
                            collectable.collected = true;
                        }
                    }
                    _ => ()
                }
            }
        }
    }
}
