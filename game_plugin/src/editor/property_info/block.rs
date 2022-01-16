use bevy::prelude::*;
use bevy_mod_picking::*;
use bevy_inspector_egui::Inspectable;
use crate::editor::property_editor::{EntitySelection, PropertyWrapper, get_common_color, get_selected_entities};
use super::PropertyInfoHandler;

#[derive(Default, Inspectable)]
pub struct BlockPropertyInfo {
    moveable: bool,
    visible: bool,
    color: PropertyWrapper::<Color>,
}

pub struct BlockPropertyHandler;
impl PropertyInfoHandler for BlockPropertyHandler {
    type T = BlockPropertyInfo;

    fn get_entity_selection() -> EntitySelection {
        EntitySelection::Block 
    }

    fn apply_properties_to_selection(
        prop: Res<BlockPropertyInfo>,
        mut materials: ResMut<Assets<StandardMaterial>>,
        mut selected: Query<(&Selection, &mut PickableButton<StandardMaterial>)>,
    ) {
        if prop.color.is_single_value() {
            for (selection, mut button) in selected.iter_mut() {
                if selection.selected() {
                    button.initial = Some(materials.add(prop.color.get().into()));
                }
            }
        }
    }

    fn handle_item_selection(
        entity: Entity,
        mut prop: &mut ResMut<BlockPropertyInfo>,
        selections: &Query<(Entity, &Selection)>,
        materials: &Res<Assets<StandardMaterial>>,
        pickables: &Query<&PickableButton<StandardMaterial>>,
    ) {
        let selected_entities = get_selected_entities(selections);
        let selected_entity_color = pickables.get(entity)
                                             .ok()
                                             .and_then(|p| p.initial.as_ref())
                                             .and_then(|m| materials.get(m))
                                             .and_then(|m| Some(m.base_color));

        prop.color = get_common_color(&selected_entities,
                                      &selected_entity_color,
                                      pickables,
                                      materials);
    }

    fn handle_item_deselection(
        _: Entity,
        mut prop: &mut ResMut<BlockPropertyInfo>,
        selections: &Query<(Entity, &Selection)>,
        materials: &Res<Assets<StandardMaterial>>,
        pickables: &Query<&PickableButton<StandardMaterial>>,
    ) {
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
                PropertyWrapper::<Color>::MultipleValues
            };
    }
}
