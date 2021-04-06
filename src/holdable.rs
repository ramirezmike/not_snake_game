use bevy::prelude::*;
use crate::{level::Level, Position, Direction, EntityType, GameObject, facing::Facing};

pub struct Holdable { }
pub struct Holder {
    pub holding: Option::<Entity>
}
pub struct BeingHeld {
    pub held_by: Entity
}
pub struct LiftHoldableEvent(pub Entity, pub Direction);

pub fn lift_holdable(
    mut commands: Commands, 
    mut level: ResMut<Level>,
    mut lift_event: EventReader<LiftHoldableEvent>,
    mut holders: Query<(Entity, &mut Holder, Option::<&mut Facing>)>,
    mut positions: Query<&mut Position>,
    mut transforms: Query<&mut Transform>,
) {
    for LiftHoldableEvent(entity, mut direction) in lift_event.iter() {
        if let Ok((_e, mut holder, maybe_facing)) = holders.get_mut(*entity) {
            match holder.holding {
                Some(held_entity) => {
                    let mut new_holder_position: Option<Position> = None;
                    let mut new_holder_rotation: Option<Quat> = None;
                    if let (Ok(_transform), Ok(position)) = (transforms.get_mut(*entity), positions.get_mut(*entity)) {
                        let mut directions_to_try = vec!(Direction::Right, Direction::Left, Direction::Up, Direction::Down);
                        directions_to_try.sort_by_key(|d| if *d == direction { 0 } else { 1 });

                        for direction_to_try in directions_to_try {
                            new_holder_position = 
                                match direction_to_try {
                                    Direction::Right => Some(Position { x: position.x, y: position.y, z: position.z - 1 }),
                                    Direction::Left => Some(Position { x: position.x, y: position.y, z: position.z + 1 }),
                                    Direction::Up => Some(Position { x: position.x - 1, y: position.y, z: position.z }),
                                    Direction::Down => Some(Position { x: position.x + 1, y: position.y, z: position.z }),
                                    _ => None
                                };

                            if let Some(potential_holder_position) = new_holder_position {
                                if level.is_position_type(potential_holder_position, None) 
                                || level.is_position_collectable(potential_holder_position) {
                                    new_holder_rotation = 
                                        match direction_to_try {
                                            Direction::Up => Some(Quat::from_axis_angle(Vec3::Y, -std::f32::consts::FRAC_PI_2)),
                                            Direction::Down => Some(Quat::from_axis_angle(Vec3::Y, std::f32::consts::FRAC_PI_2)),
                                            Direction::Right => Some(Quat::from_axis_angle(Vec3::Y, std::f32::consts::PI)),
                                            Direction::Left => Some(Quat::from_axis_angle(Vec3::Y, 0.0)),
                                            _ => None
                                        };
                                    direction = direction_to_try; 
                                    break; // we good
                                } else {
                                    new_holder_position = None; // try again
                                }
                            }
                        }
                    } 

                    if new_holder_position.is_none() {
                        continue;
                    }

                    let mut drop_successful = false;
                    commands.entity(held_entity)
                            .remove::<BeingHeld>();
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
                            *position = new_holder_position.unwrap();
                            transform.translation.x = position.x as f32;
                            transform.translation.y = position.y as f32;
                            transform.translation.z = position.z as f32;
                            transform.rotation = new_holder_rotation.unwrap();
                            level.set_with_vec(transform.translation, Some(GameObject::new(held_entity, EntityType::Block)));
                            
                            if let Some(mut facing) = maybe_facing {
                                facing.direction = direction; 
                                // this doesn't work
                                //maybe_dude.target = Some((transform.translation, direction)); 
                            }

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
                                        .insert(BeingHeld { held_by: *entity });
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

pub fn update_held(
    mut holdables: Query<(&mut Transform, &BeingHeld)>, 
    holders: Query<(Entity, &Transform), Without<BeingHeld>>
) {
    for (mut holdable_transform, being_held) in holdables.iter_mut() {
        if let Ok((_entity, transform)) = holders.get(being_held.held_by) {
            holdable_transform.translation.x = transform.translation.x;
            holdable_transform.translation.y = transform.translation.y + 1.0;
            holdable_transform.translation.z = transform.translation.z;
        }
    }
}
