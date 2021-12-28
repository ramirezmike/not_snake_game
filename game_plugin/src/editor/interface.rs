use bevy::prelude::*;
use crate::{AppState, editor::editor_camera};

// create visual buttons and also handle keybinding
/*
   [(S)elect]                               [Play]
   |___[Single]
       [Multi]
   [(A)dd]                                  [Camera]
   [(R)emove]
   [Copy Settings]
   [Paste Settings]
   [Syste(m)]
   |___[Save]
       [Save As]
       [Load]
       [Save and Quit]
       [Quit without Saving]
*/

pub struct EditorInterfacePlugin;
impl Plugin for EditorInterfacePlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_update(AppState::Editor)
                       .with_system(handle_keyboard_input)
        );
    }
}

fn handle_keyboard_input(
    mut commands: Commands,
    keyboard_input: Res<Input<KeyCode>>,
    mut camera: Query<&mut editor_camera::EditorCamera>,
) {
    if keyboard_input.pressed(KeyCode::C) {
        if let Ok(mut camera) = camera.get_single_mut() {
            camera.is_being_controlled = !camera.is_being_controlled;
        }
    }
}
