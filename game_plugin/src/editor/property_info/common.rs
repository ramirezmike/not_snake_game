use super::PropertyInfoHandler;
use crate::editor::property_editor::{
    get_common_color, get_selected_entities, EntitySelection, PropertyWrapper,
};
use bevy::prelude::*;
use bevy_inspector_egui::Inspectable;
use bevy_mod_picking::*;

#[derive(Default, Inspectable)]
pub struct CommonPropertyInfo {
    color: PropertyWrapper<Color>,
}

pub struct CommonPropertyHandler;
impl PropertyInfoHandler for CommonPropertyHandler {
    type T = CommonPropertyInfo;

    fn get_entity_selection() -> EntitySelection {
        EntitySelection::Common
    }

    fn apply_properties_to_selection(
        prop: Res<CommonPropertyInfo>,
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
        mut prop: &mut ResMut<CommonPropertyInfo>,
        selections: &Query<(Entity, &Selection)>,
        materials: &Res<Assets<StandardMaterial>>,
        pickables: &Query<&PickableButton<StandardMaterial>>,
    ) {
        let selected_entities = get_selected_entities(selections);
        let selected_entity_color = pickables
            .get(entity)
            .ok()
            .and_then(|p| p.initial.as_ref())
            .and_then(|m| materials.get(m))
            .and_then(|m| Some(m.base_color));

        prop.color = get_common_color(
            &selected_entities,
            &selected_entity_color,
            pickables,
            materials,
        );
    }

    fn handle_item_deselection(
        _: Entity,
        mut prop: &mut ResMut<CommonPropertyInfo>,
        selections: &Query<(Entity, &Selection)>,
        materials: &Res<Assets<StandardMaterial>>,
        pickables: &Query<&PickableButton<StandardMaterial>>,
    ) {
        let selected_entities = get_selected_entities(&selections);
        prop.color = if let Some(entity) = selected_entities.first() {
            let first_color = pickables
                .get(*entity)
                .ok()
                .and_then(|p| p.initial.as_ref())
                .and_then(|m| materials.get(m))
                .and_then(|m| Some(m.base_color));
            get_common_color(&selected_entities, &first_color, &pickables, &materials)
        } else {
            PropertyWrapper::<Color>::MultipleValues
        };
    }
}
