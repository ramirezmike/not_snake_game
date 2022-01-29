use crate::AppState;
use bevy::prelude::*;
use bevy_mod_picking::*;

mod add_entity;
mod editor_camera;
mod help_text;
mod interface;
mod play;
pub mod properties;
mod select_entity;
mod file;

#[derive(Component)]
pub struct EditorTrashMarker;

pub struct EditorPlugin;
impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_enter(AppState::Editor)
                .with_system(editor_camera::spawn_camera)
                .with_system(load_editor_world),
        )
        .add_plugin(PickingPlugin)
        .add_plugin(InteractablePickingPlugin)
        .add_plugin(HighlightablePickingPlugin)
        .add_plugin(DebugCursorPickingPlugin)
        .add_plugin(interface::EditorInterfacePlugin)
        .add_plugin(help_text::HelpTextPlugin)
        .add_plugin(play::EditorPlayPlugin)
        .add_plugin(file::EditorFilePlugin) 
        .add_event::<editor_camera::PositionCameraEvent>()
        .add_system_set(
            SystemSet::on_update(AppState::Editor)
                .with_system(editor_camera::handle_position_camera_event)
                .with_system(editor_camera::update_camera),
        );
    }
}

#[derive(Component, Debug)]
pub struct GameEntity {
    pub entity_type: GameEntityType,
}

#[derive(PartialEq, Eq, Clone, Copy, Hash, Debug)]
pub enum GameEntityType {
    Block,
    Snake,
    NotSnake,
    Food,
}

fn load_editor_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    properties: Res<properties::Properties>,
    mut new_level_event_writer: EventWriter<file::NewLevelEvent>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.50,
    });

    new_level_event_writer.send(file::NewLevelEvent);

    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Plane { size: 5.0 })),
            material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
            ..Default::default()
        })
        .insert(EditorTrashMarker);
}

fn cleanup_editor(mut commands: Commands, editor_trash: Query<Entity, With<EditorTrashMarker>>) {
    for entity in editor_trash.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
