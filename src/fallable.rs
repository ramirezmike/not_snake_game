use bevy::prelude::*;
use crate::{level::Level, Position, Direction, BoxObject};

pub struct Fallable { }

pub fn update_fallables(
    mut fallables: Query<(&mut BoxObject, &Fallable, &Position)>,
    level: Res<Level>,
) {
    for (mut box_object, _fallable, position) in fallables.iter_mut() {
        if level.is_type(position.x, position.y - 1, position.z, None) {
            box_object.target = Some(Direction::Beneath);
        }
    }
}
