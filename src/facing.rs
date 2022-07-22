use crate::Direction;
use bevy::prelude::*;

#[derive(Debug, Component)]
pub struct Facing {
    pub direction: Direction,
    pub can_face_verticals: bool,
}

impl Facing {
    pub fn new(direction: Direction, can_face_verticals: bool) -> Self {
        Facing {
            direction,
            can_face_verticals,
        }
    }
}
