use bevy::prelude::*;
use crate::AppState;
use bevy_inspector_egui::plugin::InspectorWindows;
use bevy_inspector_egui::widgets::{InspectorQuery, InspectorQuerySingle};
use bevy_inspector_egui::{Inspectable, InspectorPlugin};

pub struct PropertyEditorPlugin;
impl Plugin for PropertyEditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_update(AppState::Editor)
                      .with_system(display_inspectors)
        )
        .add_plugin(InspectorPlugin::<Data>::new());
//        .add_plugin(InspectorPlugin::<InspectorQuerySingle<Entity, With<GameEntity>>>::new());
    }
}

#[derive(Default, Inspectable)]
struct Data {
    field: f32,
}

fn display_inspectors(
    mut input: ResMut<Input<KeyCode>>,
    mut inspector_windows: ResMut<InspectorWindows>,
) {
    if input.just_pressed(KeyCode::Space) {
        let mut inspector_window_data = inspector_windows.window_data_mut::<Data>();
        inspector_window_data.visible = !inspector_window_data.visible;
    }
}
