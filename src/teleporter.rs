use crate::{Direction, Position};
use bevy::prelude::*;
use bevy::reflect::TypeUuid;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, TypeUuid, Component)]
#[uuid = "00aadc56-aa9c-4543-8640-a018b74b5052"] // this needs to be actually generated
pub struct Teleporter {
    pub position: Position, // starting spot
    pub target: Position,   // teleport to this spot
    pub move_to: Position,  // move to here after teleporting
    pub facing: Direction,
}

pub fn spawn_teleporter<T: Component>(commands: &mut Commands, teleporter: Teleporter, cleanup_marker: T) {
    commands.spawn().insert(teleporter).insert(cleanup_marker);
}
