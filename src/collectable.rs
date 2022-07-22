use bevy::prelude::*;
use crate::{dude, level, level_over, score::Score, audio, EntityType, Position, assets::GameAssets};

#[derive(Component)]
pub struct Collectable {
    pub collected: bool,
}

pub fn check_collected(
    mut collectables: Query<(&mut Collectable, &Position, &EntityType)>,
    dudes: Query<(&dude::Dude, &Position)>,
    score: Res<Score>,
    level: ResMut<level::Level>,
    mut level_over_event_writer: EventWriter<level_over::LevelOverEvent>,
    game_assets: Res<GameAssets>,
    mut audio: audio::GameAudio,
) {
    for (mut collectable, collectable_position, collectable_entity_type) in
        collectables.iter_mut().filter(|x| !x.0.collected)
    {
        for (_dude, dude_position) in dudes.iter() {
            if collectable_position == dude_position {
                match collectable_entity_type {
                    EntityType::WinFlag => {
                        if score.current_level >= level.get_current_minimum_food() {
                            level_over_event_writer.send(level_over::LevelOverEvent {});

                            audio.play_sfx(&game_assets.level_end_handle);
                            collectable.collected = true;
                        }
                    }
                    _ => (),
                }
            }
        }
    }
}
