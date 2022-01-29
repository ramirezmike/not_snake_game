use crate::editor::interface::{EntityAction, EntitySelection};
use crate::editor::properties::{
    block::BlockProperties, not_snake::NotSnakeProperties, Properties,
};
use crate::editor::{add_entity, GameEntity, GameEntityType};
use crate::{dude, snake};
use bevy::prelude::*;
use bevy_mod_picking::*;
use std::collections::HashMap;

pub fn handle_entity_click_events(
    mut commands: Commands,
    mut events: EventReader<PickingEvent>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut dude_meshes: ResMut<dude::DudeMeshes>,
    mut enemy_meshes: ResMut<snake::EnemyMeshes>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut items_with_selections: Query<&mut Selection>,
    properties: Res<Properties>,
    picking_cameras: Query<&PickingCamera>,
    entity_action: Res<EntityAction>,
    entity_selection: Res<EntitySelection>,
    entities: Query<&Transform>,
) {
    for event in events.iter() {
        match event {
            PickingEvent::Clicked(_) => {
                for picking_camera in picking_cameras.iter() {
                    if let Some((entity, intersection)) = picking_camera.intersect_top() {
                        match *entity_action {
                            EntityAction::Select => {
                                // ¯\_(ツ)_/¯
                            }
                            EntityAction::Delete => {
                                // TODO: Need to prevent deleting the last item
                                //       that exists.
                                commands.entity(entity).despawn_recursive();
                            }
                            EntityAction::Add => {
                                if let Ok(transform) = entities.get(entity) {
                                    let mut selected_position = transform.translation;
                                    match convert_normal_to_face(&intersection.normal()) {
                                        Face::Above => selected_position.y += 1.0,
                                        Face::Below => selected_position.y -= 1.0,
                                        Face::Up => selected_position.x += 1.0,
                                        Face::Down => selected_position.x -= 1.0,
                                        Face::Left => selected_position.z -= 1.0,
                                        Face::Right => selected_position.z += 1.0,
                                        Face::None => {
                                            // invalid, don't do anything
                                            return;
                                        }
                                    }

                                    match *entity_selection {
                                        EntitySelection::Snake => add_entity::add_snake(
                                            &mut commands,
                                            &mut enemy_meshes,
                                            &mut materials,
                                            &properties.snake,
                                            &selected_position,
                                        ),
                                        EntitySelection::NotSnake => add_entity::add_not_snake(
                                            &mut commands,
                                            &mut dude_meshes,
                                            &mut materials,
                                            &properties.not_snake,
                                            &selected_position,
                                        ),
                                        EntitySelection::Food => add_entity::add_food(
                                            &mut commands,
                                            &mut meshes,
                                            &mut materials,
                                            &properties.food,
                                            &selected_position,
                                        ),
                                        _ => add_entity::add_block(
                                            &mut commands,
                                            &mut meshes,
                                            &mut materials,
                                            &properties.block,
                                            &selected_position,
                                        ),
                                    }
                                }

                                // deselect everything when clicking with Add
                                items_with_selections
                                    .iter_mut()
                                    .for_each(|mut s| s.set_selected(false));
                            }
                        }
                    }
                }
            }
            _ => (),
        }
    }
}

pub fn detect_entity_selections(
    entity_action: Res<EntityAction>,
    mut entity_selection: ResMut<EntitySelection>,
    selections: Query<(Entity, &Selection)>,
    game_entities: Query<&GameEntity>,
) {
    if *entity_action != EntityAction::Select {
        return;
    }

    let selected_entities = get_selected_entities(&selections);
    let grouped_entities = selected_entities.iter().fold(HashMap::new(), |mut acc, x| {
        let game_entity = game_entities
            .get(*x)
            .expect("Selected value no longer exists");
        acc.entry(game_entity.entity_type.clone()).or_insert(true);
        acc
    });
    if grouped_entities.len() == 1 {
        let entity_type = grouped_entities
            .iter()
            .nth(0)
            .expect("Hashmap was empty but reported a length of 1")
            .0;
        *entity_selection = match entity_type {
            GameEntityType::Block => EntitySelection::Block,
            GameEntityType::Food => EntitySelection::Food,
            GameEntityType::Snake => EntitySelection::Snake,
            GameEntityType::NotSnake => EntitySelection::NotSnake,
        };
    } else {
        *entity_selection = EntitySelection::None;
    }
}

fn get_selected_entities(items_with_selections: &Query<(Entity, &Selection)>) -> Vec<Entity> {
    items_with_selections
        .iter()
        .filter(|s| s.1.selected())
        .map(|s| s.0)
        .collect()
}

enum Face {
    Up,
    Down,
    Left,
    Right,
    Above,
    Below,
    None,
}

fn convert_normal_to_face(normal: &Vec3) -> Face {
    let is_zero = |n| n < 0.5 && n > -0.5;
    let is_one = |n| n > 0.5;
    let is_negative_one = |n| n < -0.5;

    match (normal.x, normal.y, normal.z) {
        (x, y, z) if is_zero(x) && is_one(y) && is_zero(z) => Face::Above,
        (x, y, z) if is_zero(x) && is_negative_one(y) && is_zero(z) => Face::Below,
        (x, y, z) if is_one(x) && is_zero(y) && is_zero(z) => Face::Up,
        (x, y, z) if is_negative_one(x) && is_zero(y) && is_zero(z) => Face::Down,
        (x, y, z) if is_zero(x) && is_zero(y) && is_negative_one(z) => Face::Left,
        (x, y, z) if is_zero(x) && is_zero(y) && is_one(z) => Face::Right,
        _ => Face::None,
    }
}

pub fn store_selected_values(
    entity_action: Res<EntityAction>,
    entity_selection: Res<EntitySelection>,
    selections: Query<(Entity, &Selection)>,
    blocks: Query<&BlockProperties>,
    not_snakes: Query<&NotSnakeProperties>,
    mut properties: ResMut<Properties>,
    mut last_stored_entity: Local<Option<Entity>>,
) {
    if *entity_action != EntityAction::Select {
        return;
    }

    let selected_entities = get_selected_entities(&selections);

    if selected_entities.is_empty()
        || last_stored_entity.map_or(false, |e| selected_entities.contains(&e))
    {
        // we have nothing selected or we've already "stored" properties for
        // a selection within the set of things currently selected
        return;
    }

    let entity = selected_entities[0];
    *last_stored_entity = Some(entity);

    match *entity_selection {
        EntitySelection::Block => {
            if let Ok(block) = blocks.get(entity) {
                properties.block = block.clone();
            }
        }
        EntitySelection::NotSnake => {
            if let Ok(not_snake) = not_snakes.get(entity) {
                properties.not_snake = not_snake.clone();
            }
        }
        _ => (),
    }
}
