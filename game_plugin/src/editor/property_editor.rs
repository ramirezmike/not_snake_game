use crate::editor::{GameEntity, GameEntityType};
use crate::AppState;
use bevy::ecs::schedule::ShouldRun;
use bevy::prelude::*;
use bevy_inspector_egui::plugin::InspectorWindows;
use bevy_inspector_egui::widgets::{InspectorQuery, InspectorQuerySingle};
use bevy_inspector_egui::{Inspectable, InspectorPlugin};
use bevy_mod_picking::*;
use std::collections::HashMap;

// detect current selection and set Entity Selection
// conditionally run systems that update values in PropInfos based on entity selection
// show/hide displays based on entity selection
// update PropInfo based on what user selected?

pub struct PropertyEditorPlugin;
impl Plugin for PropertyEditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_update(AppState::Editor)
                .with_system(detect_entity_selections.label("detect")),
        )
        .insert_resource(EntitySelection::default())
        .add_plugin(super::property_info::PropertyInfoPlugin);
        //        .add_plugin(InspectorPlugin::<InspectorQuerySingle<Entity, With<GameEntity>>>::new());
    }
}

#[derive(PartialEq, Inspectable)]
pub enum PropertyWrapper<T: Default> {
    MultipleValues,
    Value(T),
}
impl<T: Default + PartialEq + Copy + Clone> PropertyWrapper<T> {
    pub fn is_single_value(&self) -> bool {
        *self != PropertyWrapper::<T>::MultipleValues
    }

    pub fn get(&self) -> T {
        match self {
            PropertyWrapper::<T>::Value(x) => *x,
            _ => panic!("Wrapping multiple properties; missing check prior to calling get"),
        }
    }
}
impl<T: Default> Default for PropertyWrapper<T> {
    fn default() -> Self {
        PropertyWrapper::MultipleValues
    }
}

#[derive(PartialEq, Debug)]
pub enum EntitySelection {
    Block,
    Snake,
    NotSnake,
    Common,
    None,
}
impl Default for EntitySelection {
    fn default() -> Self {
        EntitySelection::None
    }
}

pub fn get_selected_entities(items_with_selections: &Query<(Entity, &Selection)>) -> Vec<Entity> {
    items_with_selections
        .iter()
        .filter(|s| s.1.selected())
        .map(|s| s.0)
        .collect()
}

fn detect_entity_selections(
    mut entity_selection: ResMut<EntitySelection>,
    selections: Query<(Entity, &Selection)>,
    game_entities: Query<&GameEntity>,
) {
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
            GameEntityType::Snake => EntitySelection::Snake,
            GameEntityType::NotSnake => EntitySelection::NotSnake,
        };
    } else if grouped_entities.len() == 0 {
        *entity_selection = EntitySelection::None;
    } else {
        *entity_selection = EntitySelection::Common;
    }
}

pub fn get_common_color(
    entities: &Vec<Entity>,
    color: &Option<Color>,
    pickables: &Query<&PickableButton<StandardMaterial>>,
    materials: &Res<Assets<StandardMaterial>>,
) -> PropertyWrapper<Color> {
    if let Some(color) = color {
        let all_colors_equal = entities
            .iter()
            .map(|e| {
                pickables
                    .get(*e)
                    .ok()
                    .and_then(|m| m.initial.as_ref())
                    .and_then(|h| materials.get(h))
                    .and_then(|m| Some(m.base_color))
            })
            .filter(|c| c.is_some())
            .map(|c| c.unwrap())
            .all(|c| c.eq(color));
        if all_colors_equal {
            PropertyWrapper::<Color>::Value(*color)
        } else {
            PropertyWrapper::<Color>::MultipleValues
        }
    } else {
        PropertyWrapper::<Color>::MultipleValues
    }
}
