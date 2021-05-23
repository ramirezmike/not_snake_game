use bevy::prelude::*;
use crate::{food::FoodEatenEvent, Dude, sounds};
use bevy_kira_audio::Audio;

pub struct Score {
    pub total: usize,
    pub current_level: usize,
}

impl Score {
    pub fn new() -> Self {
        Score {
            total: 0,
            current_level: 0
        }
    }
}

pub fn handle_food_eaten(
    mut score: ResMut<Score>,
    mut food_eaten_event_reader: EventReader<FoodEatenEvent>,
    dude: Query<Entity, With<Dude>>,
    mut sound_writer: EventWriter<sounds::SoundEvent>,
) {
    for eater in food_eaten_event_reader.iter() {
        if let Ok(_) = dude.get(eater.0) {
            score.current_level += 1;
            println!("Food: {} Total: {}", score.current_level, score.total);
            sound_writer.send(sounds::SoundEvent(sounds::Sounds::Pickup));
        }
    }
}
