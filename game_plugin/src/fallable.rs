use bevy::prelude::*;
use crate::{level::Level, Position, moveable::Moveable, Dude};

#[derive(Component)]
pub struct Fallable { 
    pub is_jumping: bool 
}

pub fn update_fallables(
    mut fallables: Query<(&mut Moveable, &Fallable, &Position, Option::<&Dude>)>,
    level: Res<Level>,
) {
//  for (mut moveable, fallable, position, maybe_dude) in fallables.iter_mut() {
//      if !fallable.is_jumping && level.is_type(position.x, position.y - 1, position.z, None) {
//          if maybe_dude.is_none() && moveable.target_dir.is_none() { // this is disgusting
//              moveable.target_dir = Some(crate::Direction::Beneath);
//          } else if moveable.target_pos.is_none() {
//              let target_pos = Vec3::new(position.x as f32, (position.y - 1) as f32, position.z as f32);
//              moveable.target_pos = Some(target_pos);
//          }
//      }
//  }
}
