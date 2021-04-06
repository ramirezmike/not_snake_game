use bevy::prelude::*;
use crate::{Direction, level::Level, Position, EntityType, GameObject, facing::Facing};

#[derive(Debug)]
pub struct Moveable {
    // (position, current movement time, finish movement time, Direction, original_position, movement_type)
    target_position: Option::<(Vec3, f32, f32, Direction, Vec3, MovementType)>, 
    queued_movement: Option::<(Direction, MovementType)>,
    gravity: bool,
    can_climb: bool,
    is_climbing: bool,
    movement_speed: f32,
}

#[derive(Debug)]
pub enum MovementType {
    Step,
    Slide
}

impl Moveable {
    pub fn new(gravity: bool, can_climb: bool, movement_speed: f32) -> Self {
        Moveable {
            target_position: None,
            queued_movement: None,
            is_climbing: false,
            gravity,
            can_climb,
            movement_speed,
        }
    }

    pub fn set_movement(&mut self, direction: Direction, movement_type: MovementType) {
        let is_falling = self.gravity 
                      && self.queued_movement.is_some() 
                      && self.queued_movement.as_ref().unwrap().0 == Direction::Beneath;
        if !is_falling && !self.is_climbing {
            self.queued_movement = Some((direction, movement_type));
        }
    }

    pub fn is_moving(&self) -> bool {
        self.target_position.is_some()
    }

    pub fn get_current_direction(&self) -> Option::<Direction> {
        if let Some(target_position) = &self.target_position {
            Some(target_position.3)
        } else {
            None
        }
    }
}

pub fn update_moveable(
    mut moveables: Query<(Entity, &mut Moveable, &mut Transform, &mut Position, &EntityType, Option::<&mut Facing>)>,
    mut level: ResMut<Level>,
    time: Res<Time>,
) {
    for (entity, mut moveable, mut transform, mut position, entity_type, maybe_facing) in moveables.iter_mut() {
        if let Some(target_position) = &mut moveable.target_position {
//            println!("{:?} {:?} {:?}",transform.translation, target_position.1, target_position.2);
            level.set_with_vec(transform.translation, None);
            if target_position.1 >= target_position.2 {
                //  check if the target is still valid
                if level.is_enterable_with_vec(target_position.0) {
                    transform.translation = target_position.0;
                } 
                moveable.target_position = None;
            } else {
                // try to keep moving toward target
                target_position.1 += time.delta_seconds();
                let new_translation = target_position.4.lerp(target_position.0, target_position.1 / target_position.2);
                if !new_translation.is_nan() {
                    if transform.translation.distance(target_position.0) < transform.translation.distance(new_translation) {
                        transform.translation = target_position.0;
                        target_position.1 = target_position.2;
                    } else {
                        transform.translation = new_translation;
                    }
                }
            }
            // need to update level here
            level.set_with_vec(transform.translation, Some(GameObject::new(entity, *entity_type)));
            position.from_vec(transform.translation);

            // at the end here we should check that the target is still valid and
            // if not then re-set what the target should be? this should check distance
            // and only do anything if you're one space away from hitting something
            // it should also check every space between current spot and the target spot
            // that way if something appears within the path after it has been set
            // it'll know about it and react accordingly 
            // maybe this should throw an event when things collide?

        } else if let Some(queued_movement) = &moveable.queued_movement {
            match queued_movement.1 {
                MovementType::Step => {
                /*
                    if step then check if position in direction is empty/collectable
                    (maybe should have "is moveable"?) and then if it's good, set the
                    moveable's target to be one step in the direction of the queued
                    movement and reset the current_movement_timer.
                */
                    let mut ignore_movement = false;
                    let target_position = 
                        match queued_movement.0 { 
                            Direction::Up => (position.x + 1, position.y, position.z),
                            Direction::Down => (position.x - 1, position.y, position.z),
                            Direction::Right => (position.x, position.y, position.z + 1),
                            Direction::Left => (position.x, position.y, position.z - 1),
                            Direction::Beneath => (position.x, position.y - 1, position.z),
                            Direction::Above => (position.x, position.y + 1, position.z),
                        };
                    let target_position = IVec3::new(target_position.0, target_position.1, target_position.2).as_f32();

                    let target_is_enterable = level.is_enterable_with_vec(target_position);
                    if let Some(mut facing) = maybe_facing {
                        let previous_facing = facing.direction;
                        facing.direction = 
                            match queued_movement.0 {
                                Direction::Up | Direction::Down |
                                Direction::Right | Direction::Left => queued_movement.0,
                                _ => facing.direction
                            };
                        transform.rotation = 
                            match facing.direction {
                                Direction::Up => Quat::from_axis_angle(Vec3::Y, -std::f32::consts::FRAC_PI_2),
                                Direction::Down => Quat::from_axis_angle(Vec3::Y, std::f32::consts::FRAC_PI_2),
                                Direction::Right => Quat::from_axis_angle(Vec3::Y, std::f32::consts::PI),
                                Direction::Left => Quat::from_axis_angle(Vec3::Y, 0.0),
                                _ => transform.rotation
                            };

                        // if we're currently not facing a wall/cliff then just turn toward it
                        let below_target_is_enterable 
                            = level.is_enterable_with_vec(Vec3::new(target_position.x, target_position.y - 1.0, target_position.z));

                        let is_vertical_movement = queued_movement.0 == Direction::Above || queued_movement.0 == Direction::Beneath;
                        if !is_vertical_movement && previous_facing != queued_movement.0 && (below_target_is_enterable || !target_is_enterable) {
                            ignore_movement = true; 
                        }
                    }

                    if !ignore_movement  {
                        if target_is_enterable {
                            moveable.target_position = 
                                Some((target_position, 0.0, 
                                      moveable.movement_speed, queued_movement.0,
                                      transform.translation, MovementType::Step));
                        } else if moveable.can_climb {
                            let above_moveable = IVec3::new(position.x, position.y + 1, position.z).as_f32(); 
                            let above_target = Vec3::new(target_position.x, target_position.y + 1.0, target_position.z);
                            if level.is_enterable_with_vec(above_moveable) && level.is_enterable_with_vec(above_target) {
                                let movement_speed = moveable.movement_speed;
                                let direction = queued_movement.0;
                                moveable.target_position = 
                                    Some((above_moveable, 0.0, 
                                          movement_speed, Direction::Above,
                                          transform.translation, MovementType::Step));
                                moveable.set_movement(direction, MovementType::Step);
                                moveable.is_climbing = true;
                                continue;
                            }
                        }
                    }
                },
                MovementType::Slide=> {
                /*
                    if slide then find next invalid position in direction (one that is
                    not empty/collectable) and then set the target to be the position
                    beforoe that and reset the current_movement_timer.
                */
                    let mut target_position = *position;
                    let mut keep_going = true;
                    let mut number_of_steps = 0;
                    while keep_going {
                        let next_position = 
                            match queued_movement.0 { 
                                Direction::Up => (target_position.x + 1, target_position.y, target_position.z),
                                Direction::Down => (target_position.x - 1, target_position.y, target_position.z),
                                Direction::Right => (target_position.x, target_position.y, target_position.z + 1),
                                Direction::Left => (target_position.x, target_position.y, target_position.z - 1),
                                Direction::Beneath => (target_position.x, target_position.y - 1, target_position.z),
                                Direction::Above => (target_position.x, target_position.y + 1, target_position.z),
                            };
                        if level.is_enterable(next_position.0, next_position.1, next_position.2) {
                            target_position = Position { x: next_position.0, y: next_position.1, z: next_position.2 };
                            number_of_steps += 1; 
                        } else {
                            keep_going = false;
                        }
                    }

                    let target_position = IVec3::new(target_position.x, target_position.y, target_position.z).as_f32();
                    moveable.target_position = 
                        Some((target_position, 0.0, 
                              moveable.movement_speed * number_of_steps as f32, 
                              queued_movement.0, transform.translation, MovementType::Slide));
                }
            }

            // Queued movement should be handled at this point
            moveable.queued_movement = None;
            moveable.is_climbing = false;
        }

        if moveable.gravity && !moveable.is_climbing //&& moveable.target_position.is_none()
            // need to add or movement_type is slide for blocks
        && level.is_enterable_with_vec(IVec3::new(position.x, position.y - 1, position.z).as_f32()) {
            moveable.set_movement(Direction::Beneath, MovementType::Step);
        }
    }
}
