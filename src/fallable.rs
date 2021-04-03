use bevy::prelude::*;
use crate::{level::Level, Position, block::BlockObject};

pub struct Fallable { }

pub fn update_fallables(
    mut fallables: Query<(&mut BlockObject, &Fallable, &Position)>,
    level: Res<Level>,
) {
    for (mut block_object, _fallable, position) in fallables.iter_mut() {
        if level.is_type(position.x, position.y - 1, position.z, None) {
            block_object.target = Some(crate::Direction::Beneath);
        }
    }
}
