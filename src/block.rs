use bevy::prelude::*;

use crate::{holdable, level, Position, GameObject, EntityType, moveable::Moveable};

pub struct BlockObject { }

pub fn update_block(
    mut blocks: Query<(Entity, &BlockObject, &mut Moveable, &mut Position, &mut Transform), Without<holdable::BeingHeld>>, 
    mut level: ResMut<level::Level>,
    time: Res<Time>, 
) {
    for (entity, mut block_object, mut moveable, mut position, mut transform) in blocks.iter_mut() {
//      if !moveable.target_dir.is_some() { 
//          // this is a terrible hack to handle when
//          // a block ends up stuck in a spot that doesn't match
//          // it's current position. This is a concurrency problem that
//          // should be addresed. 
//          transform.translation = Vec3::new(position.x as f32, 
//                                            position.y as f32, 
//                                            position.z as f32);
//          continue; 
//      }

//      let current = transform.translation;
//      let target_translation = match moveable.target_dir.unwrap() {
//                                   crate::Direction::Beneath
//                                       => Transform::from_xyz(current.x, 
//                                                              current.y - 1.0, 
//                                                              current.z),
//                                   crate::Direction::Above
//                                       => Transform::from_xyz(current.x, 
//                                                              current.y + 1.0, 
//                                                              current.z),
//                                   crate::Direction::Up 
//                                       => Transform::from_xyz(current.x + 1.0, 
//                                                              current.y, 
//                                                              current.z),
//                                   crate::Direction::Down 
//                                       => Transform::from_xyz(current.x - 1.0, 
//                                                              current.y, 
//                                                              current.z),
//                                   crate::Direction::Right 
//                                       => Transform::from_xyz(current.x, 
//                                                              current.y, 
//                                                              current.z + 1.0),
//                                   crate::Direction::Left 
//                                       => Transform::from_xyz(current.x, 
//                                                              current.y, 
//                                                              current.z - 1.0),
//                               }.translation;

//      if !position.matches(target_translation) {
//          level.set(position.x, position.y, position.z, None);
//      }

//      if level.is_type_with_vec(target_translation, None) 
//          || level.is_with_vec(target_translation, Some(GameObject::new(entity, EntityType::Block))) {
//          let target_position = Vec3::new(target_translation.x - transform.translation.x,
//                                          target_translation.y - transform.translation.y,
//                                          target_translation.z - transform.translation.z).normalize();
//           
//          level.set_with_vec(target_position, Some(GameObject::new(entity, EntityType::Block)));
//          transform.translation += target_position * 0.01 * time.delta().subsec_millis() as f32;
//      } else {
//          println!("Can't move!");
//          moveable.target_dir = None;
//          transform.translation = Vec3::new(position.x as f32, position.y as f32, position.z as f32);
//          level.set(position.x, position.y, position.z, Some(GameObject::new(entity, EntityType::Block)));
//      }

//      position.x = transform.translation.x as i32;
//      position.y = transform.translation.y as i32;
//      position.z = transform.translation.z as i32;
    }
}
