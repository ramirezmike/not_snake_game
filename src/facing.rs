use bevy::prelude::*;
use crate::Direction;

pub struct Facing {
    pub direction: Direction,
}

impl Facing {
    pub fn new(direction: Direction) -> Self {
        Facing {
            direction
        }
    }
}
