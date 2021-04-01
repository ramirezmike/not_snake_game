use bevy::prelude::*;
use crate::{environment, level::Level, Position, Direction, EntityType, GameObject, };

pub struct Holdable { }
pub struct Holder {
    pub holding: Option::<Entity>
}
pub struct LiftHoldableEvent(pub Entity, pub Direction);

pub fn lift_holdable(
    mut commands: Commands, 
    mut level: ResMut<Level>,
    mut lift_event: EventReader<LiftHoldableEvent>,
    mut holders: Query<(Entity, &mut Holder)>,
    mut positions: Query<&mut Position>,
    mut transforms: Query<&mut Transform>,
) {
    for LiftHoldableEvent(entity, direction) in lift_event.iter() {
        if let Ok((_e, mut holder)) = holders.get_mut(*entity) {
            match holder.holding {
                Some(held_entity) => {
                    let mut drop_successful = false;
                    commands.entity(held_entity)
                            .remove::<environment::BeingHeld>();
                    match (transforms.get_mut(held_entity), positions.get_mut(*entity)) {
                        (Ok(mut transform), Ok(position)) => {
                            transform.translation = position.to_vec();
                            level.set_with_vec(transform.translation, Some(GameObject::new(held_entity, EntityType::Block)));
                            commands.entity(held_entity).insert(position.clone());
                            drop_successful = true; 
                        },
                        _ => ()
                    }

                    if drop_successful {
                        if let (Ok(mut transform), Ok(mut position)) = (transforms.get_mut(*entity), positions.get_mut(*entity)) {
                            println!("Dropping object {} {} {}", position.x, position.y, position.z);
                            position.y += 1;
                            transform.translation.y += 1.0;
                            holder.holding = None;
                        }
                    }
                },
                None => {
                    let mut holder_position: Option::<Position> = None;
                    let mut holdee_position: Option::<Position> = None;

                    if let Ok(position) = positions.get_mut(*entity) {
                        let (x, y, z) = match direction {
                                            Direction::Up => (position.x + 1, position.y, position.z),
                                            Direction::Down => (position.x - 1, position.y, position.z),
                                            Direction::Right => (position.x, position.y, position.z + 1),
                                            Direction::Left => (position.x, position.y, position.z - 1),
                                            _ => (position.x, position.y, position.z)
                                        };
                        holder_position = Some(*position);
                        holdee_position = Some(Position { x, y, z }); 
                    }

                    if let (Some(holder_position), Some(holdee_position)) = (holder_position, holdee_position) {
                        if level.is_position_type(holdee_position, Some(EntityType::Block)) // TODO: need a "is holdable"
                        && level.is_type(holder_position.x, holder_position.y + 1, holder_position.z, None) {
                            if let Some(holdable) = level.get_with_position(holdee_position) {
                                println!("Picking up item {} {} {}", holder_position.x, holder_position.y, holder_position.z);
                                commands.entity(holdable.entity)
                                        .insert(environment::BeingHeld { held_by: *entity });
                                level.set_with_position(holdee_position, None);
                                commands.entity(holdable.entity).remove::<Position>();
                                holder.holding = Some(holdable.entity);
                            }
                        }
                    }
                }
            }
        }
    }
}
