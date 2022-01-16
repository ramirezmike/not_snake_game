use bevy::prelude::*;
use crate::AppState;
use bevy_mod_picking::*;
use bevy_inspector_egui::plugin::InspectorWindows;
use bevy_inspector_egui::widgets::{InspectorQuery, InspectorQuerySingle};
use bevy_inspector_egui::{Inspectable, InspectorPlugin};
use crate::editor::{GameEntity, GameEntityType};
use std::collections::HashMap;
use bevy::ecs::schedule::ShouldRun;

// detect current selection and set Entity Selection
// conditionally run systems that update values in PropInfos based on entity selection
// show/hide displays based on entity selection
// update PropInfo based on what user selected?

pub struct PropertyEditorPlugin;
impl Plugin for PropertyEditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_update(AppState::Editor)
                      .with_system(detect_entity_selections.label("detect"))
                      .with_system(toggle_inspectors_display.after("detect"))
        )


        .add_system_set(
            SystemSet::on_update(AppState::Editor)
                      .after("detect")
                      .label("selection")
                      .with_run_criteria(run_common_handlers)
                      .with_system(handle_common_item_selected)
                      .with_system(handle_common_item_deselected)
        )
        .add_system_set(
            SystemSet::on_update(AppState::Editor)
                      .after("detect")
                      .label("selection")
                      .with_run_criteria(run_block_handlers)
                      .with_system(handle_block_item_selected)
                      .with_system(handle_block_item_deselected)
        )


        .add_system_set(
            SystemSet::on_update(AppState::Editor)
                      .after("selection")
                      .with_run_criteria(run_common_handlers)
                      .with_system(apply_common)
        )
        .add_system_set(
            SystemSet::on_update(AppState::Editor)
                      .after("selection")
                      .with_run_criteria(run_block_handlers)
                      .with_system(apply_block)
        )

        .insert_resource(EntitySelection::default())
        .add_plugin(InspectorPlugin::<CommonPropertyInfo>::new())
        .add_plugin(InspectorPlugin::<BlockPropertyInfo>::new());
//        .add_plugin(InspectorPlugin::<InspectorQuerySingle<Entity, With<GameEntity>>>::new());
    }
}

#[derive(Default, Inspectable)]
struct CommonPropertyInfo {
    color: Color,
}

#[derive(Default, Inspectable)]
struct BlockPropertyInfo {
    moveable: bool,
    visible: bool,
    color: Color,
}

#[derive(PartialEq, Debug)]
enum EntitySelection {
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

fn toggle_inspectors_display(
    entity_selection: Res<EntitySelection>, 
    mut inspector_windows: ResMut<InspectorWindows>,
) {
    let mut common_window = inspector_windows.window_data_mut::<CommonPropertyInfo>();
    common_window.visible = *entity_selection == EntitySelection::Common;
    println!("Common visible {}", common_window.visible);

    let mut block_window = inspector_windows.window_data_mut::<BlockPropertyInfo>();
    block_window.visible = *entity_selection == EntitySelection::Block;
    println!("Block visible {}", block_window.visible);
}

fn get_selected_entities(items_with_selections: &Query<(Entity, &Selection)>) -> Vec<Entity> {
    items_with_selections.iter()
                         .filter(|s| s.1.selected())  
                         .map(|s| s.0)
                         .collect()
}

fn run_common_handlers(
    entity_selection: Res<EntitySelection>, 
) -> ShouldRun {
    if *entity_selection == EntitySelection::Common {
        ShouldRun::Yes
    } else {
        ShouldRun::No
    }
}

fn run_block_handlers(
    entity_selection: Res<EntitySelection>, 
) -> ShouldRun {
    if *entity_selection == EntitySelection::Block {
        ShouldRun::Yes
    } else {
        ShouldRun::No
    }
}

fn detect_entity_selections(
    mut entity_selection: ResMut<EntitySelection>, 
    selections: Query<(Entity, &Selection)>,
    game_entities: Query<&GameEntity>,
) {
    let selected_entities = get_selected_entities(&selections);
    let grouped_entities = selected_entities.iter()
                                            .fold(HashMap::new(), |mut acc, x| { 
                                                let game_entity = game_entities.get(*x)
                                                                               .expect("Selected value no longer exists");
                                                acc.entry(game_entity.entity_type.clone()).or_insert(true);
                                                acc
                                            });
    if grouped_entities.len() == 1 {
        println!("One entity type selected");
        let entity_type = grouped_entities.iter()
                                          .nth(0)
                                          .expect("Hashmap was empty but reported a length of 1")
                                          .0;
        println!("Type: {:?}", entity_type);
        *entity_selection = match entity_type {
                                GameEntityType::Block => EntitySelection::Block,
                                GameEntityType::Snake => EntitySelection::Snake,
                                GameEntityType::NotSnake => EntitySelection::NotSnake,
                            };
        println!("selection: {:?}", entity_selection);
    } else if grouped_entities.len() == 0 {
        println!("Zero entity type selected");
        *entity_selection = EntitySelection::None;
    } else {
        println!("Common entity type selected");
        *entity_selection = EntitySelection::Common;
    }
}

fn get_common_color(
    entities: &Vec::<Entity>,
    color: &Option::<Color>,
    pickables: &Query<&PickableButton<StandardMaterial>>,
    materials: &Res<Assets<StandardMaterial>>,
) -> Color {
    if let Some(color) = color {
        let all_colors_equal = entities.iter()
                                       .map(|e| 
                                           pickables.get(*e)
                                                    .ok()
                                                    .and_then(|m| m.initial.as_ref())
                                                    .and_then(|h| materials.get(h))
                                                    .and_then(|m| Some(m.base_color)) 
                                       )
                                       .filter(|c| c.is_some())
                                       .map(|c| c.unwrap())
                                       .all(|c| c.eq(color));
        if all_colors_equal {
            *color
        } else {
            Color::default()
        }
    } else {
        Color::default()
    }
}

fn handle_common_item_selected(
    mut prop: ResMut<CommonPropertyInfo>,
    mut events: EventReader<PickingEvent>,
    materials: Res<Assets<StandardMaterial>>,
    selections: Query<(Entity, &Selection)>,
    pickables: Query<&PickableButton<StandardMaterial>>,
) {
    for event in events.iter() {
        match event {
            PickingEvent::Selection(selection) => {
                match selection {
                    SelectionEvent::JustSelected(entity) => {
                        let selected_entities = get_selected_entities(&selections);
                        let selected_entity_color = pickables.get(*entity)
                                                             .ok()
                                                             .and_then(|p| p.initial.as_ref())
                                                             .and_then(|m| materials.get(m))
                                                             .and_then(|m| Some(m.base_color));

                        prop.color = get_common_color(&selected_entities,
                                                      &selected_entity_color,
                                                      &pickables,
                                                      &materials);
                    },
                    _ => ()
                }
            }
            _ => ()
        }
    }
}

fn handle_common_item_deselected(
    mut prop: ResMut<CommonPropertyInfo>,
    mut events: EventReader<PickingEvent>,
    materials: Res<Assets<StandardMaterial>>,
    selections: Query<(Entity, &Selection)>,
    pickables: Query<&PickableButton<StandardMaterial>>,
) {
    for event in events.iter() {
        match event {
            PickingEvent::Selection(selection) => {
                match selection {
                    SelectionEvent::JustDeselected(_) => {
                        let selected_entities = get_selected_entities(&selections);
                        prop.color = 
                            if let Some(entity) = selected_entities.first() {
                                let first_color = pickables.get(*entity)
                                                           .ok()
                                                           .and_then(|p| p.initial.as_ref())
                                                           .and_then(|m| materials.get(m))
                                                           .and_then(|m| Some(m.base_color));
                                get_common_color(&selected_entities,
                                                 &first_color,
                                                 &pickables,
                                                 &materials)
                            } else {
                                Color::default()
                            };
                    },
                    _ => ()
                }
            }
            _ => ()
        }
    }
}


fn handle_block_item_selected(
    mut prop: ResMut<BlockPropertyInfo>,
    mut events: EventReader<PickingEvent>,
    materials: Res<Assets<StandardMaterial>>,
    selections: Query<(Entity, &Selection)>,
    pickables: Query<&PickableButton<StandardMaterial>>,
) {
    for event in events.iter() {
        match event {
            PickingEvent::Selection(selection) => {
                match selection {
                    SelectionEvent::JustSelected(entity) => {
                        let selected_entities = get_selected_entities(&selections);
                        let selected_entity_color = pickables.get(*entity)
                                                             .ok()
                                                             .and_then(|p| p.initial.as_ref())
                                                             .and_then(|m| materials.get(m))
                                                             .and_then(|m| Some(m.base_color));

                        prop.color = get_common_color(&selected_entities,
                                                      &selected_entity_color,
                                                      &pickables,
                                                      &materials);
                    },
                    _ => ()
                }
            }
            _ => ()
        }
    }
}

fn handle_block_item_deselected(
    mut prop: ResMut<BlockPropertyInfo>,
    mut events: EventReader<PickingEvent>,
    materials: Res<Assets<StandardMaterial>>,
    selections: Query<(Entity, &Selection)>,
    pickables: Query<&PickableButton<StandardMaterial>>,
) {
    for event in events.iter() {
        match event {
            PickingEvent::Selection(selection) => {
                match selection {
                    SelectionEvent::JustDeselected(_) => {
                        let selected_entities = get_selected_entities(&selections);
                        prop.color = 
                            if let Some(entity) = selected_entities.first() {
                                let first_color = pickables.get(*entity)
                                                           .ok()
                                                           .and_then(|p| p.initial.as_ref())
                                                           .and_then(|m| materials.get(m))
                                                           .and_then(|m| Some(m.base_color));
                                get_common_color(&selected_entities,
                                                 &first_color,
                                                 &pickables,
                                                 &materials)
                            } else {
                                Color::default()
                            };
                    },
                    _ => ()
                }
            }
            _ => ()
        }
    }
}



fn apply_common(
    prop: Res<CommonPropertyInfo>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut selected: Query<(&Selection, &mut PickableButton<StandardMaterial>)>,
) {
    if prop.color != Color::default() {
        for (selection, mut button) in selected.iter_mut() {
            if selection.selected() {
                button.initial = Some(materials.add(prop.color.into()));
            }
        }
    }
}

fn apply_block(
    prop: Res<BlockPropertyInfo>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut selected: Query<(&Selection, &mut PickableButton<StandardMaterial>)>,
) {
    if prop.color != Color::default() {
        for (selection, mut button) in selected.iter_mut() {
            if selection.selected() {
                button.initial = Some(materials.add(prop.color.into()));
            }
        }
    }
}
