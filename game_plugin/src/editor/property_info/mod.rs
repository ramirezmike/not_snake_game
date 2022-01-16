use crate::editor::property_editor::{
    get_common_color, get_selected_entities, EntitySelection, PropertyWrapper,
};
use crate::editor::{GameEntity, GameEntityType};
use crate::AppState;
use bevy::ecs::schedule::ShouldRun;
use bevy::ecs::system::Resource;
use bevy::prelude::*;
use bevy_inspector_egui::plugin::InspectorWindows;
use bevy_inspector_egui::{Inspectable, InspectorPlugin};
use bevy_mod_picking::*;

mod block;
mod common;
mod not_snake;
mod snake;

pub struct PropertyInfoPlugin;
impl Plugin for PropertyInfoPlugin {
    fn build(&self, app: &mut App) {
        common::CommonPropertyHandler::add_systems_to_app(app);
        block::BlockPropertyHandler::add_systems_to_app(app);
        not_snake::NotSnakePropertyHandler::add_systems_to_app(app);
        snake::SnakePropertyHandler::add_systems_to_app(app);
    }
}

trait PropertyInfoHandler {
    type T: 'static + Sync + Resource + FromWorld + Inspectable;

    fn get_entity_selection() -> EntitySelection;

    fn apply_properties_to_selection(
        prop: Res<Self::T>,
        materials: ResMut<Assets<StandardMaterial>>,
        selected: Query<(&Selection, &mut PickableButton<StandardMaterial>)>,
    );

    fn handle_item_selection(
        entity: Entity,
        prop: &mut ResMut<Self::T>,
        selections: &Query<(Entity, &Selection)>,
        materials: &Res<Assets<StandardMaterial>>,
        pickables: &Query<&PickableButton<StandardMaterial>>,
    );

    fn handle_item_deselection(
        entity: Entity,
        prop: &mut ResMut<Self::T>,
        selections: &Query<(Entity, &Selection)>,
        materials: &Res<Assets<StandardMaterial>>,
        pickables: &Query<&PickableButton<StandardMaterial>>,
    );

    fn run_block_handlers(entity_selection: Res<EntitySelection>) -> ShouldRun {
        if *entity_selection == Self::get_entity_selection() {
            ShouldRun::Yes
        } else {
            ShouldRun::No
        }
    }

    fn add_systems_to_app(app: &mut App)
    where
        Self: 'static,
    {
        app.add_system_set(
            SystemSet::on_update(AppState::Editor)
                .with_system(Self::toggle_inspector_visibility.after("detect")),
        )
        .add_system_set(
            SystemSet::on_update(AppState::Editor)
                .after("detect")
                .label("selection")
                .with_run_criteria(Self::run_block_handlers)
                .with_system(Self::handle_selection_events),
        )
        .add_system_set(
            SystemSet::on_update(AppState::Editor)
                .after("selection")
                .with_run_criteria(Self::run_block_handlers)
                .with_system(Self::apply_properties_to_selection),
        )
        .add_plugin(InspectorPlugin::<Self::T>::new());
    }

    fn toggle_inspector_visibility(
        entity_selection: Res<EntitySelection>,
        mut inspector_windows: ResMut<InspectorWindows>,
    ) {
        let mut window = inspector_windows.window_data_mut::<Self::T>();
        window.visible = *entity_selection == Self::get_entity_selection();
    }

    fn handle_selection_events(
        mut prop: ResMut<Self::T>,
        mut events: EventReader<PickingEvent>,
        materials: Res<Assets<StandardMaterial>>,
        selections: Query<(Entity, &Selection)>,
        pickables: Query<&PickableButton<StandardMaterial>>,
    ) {
        for event in events.iter() {
            match event {
                PickingEvent::Selection(selection) => match selection {
                    SelectionEvent::JustSelected(entity) => {
                        Self::handle_item_selection(
                            *entity,
                            &mut prop,
                            &selections,
                            &materials,
                            &pickables,
                        );
                    }
                    SelectionEvent::JustDeselected(entity) => {
                        Self::handle_item_selection(
                            *entity,
                            &mut prop,
                            &selections,
                            &materials,
                            &pickables,
                        );
                    }
                },
                _ => (),
            }
        }
    }
}
