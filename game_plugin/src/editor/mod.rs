use bevy::prelude::*;
use bevy_mod_picking::*;
use crate::{AppState, dude, snake};

mod editor_camera;
mod help_text;
mod play;
mod property_editor;
mod interface; 
mod add_entity;

#[derive(Component)]
pub struct EditorTrashMarker;

pub struct EditorPlugin;
impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_enter(AppState::Editor)
                      .with_system(editor_camera::spawn_camera)
                      .with_system(load_editor)

        )
        .add_plugin(PickingPlugin)
        .add_plugin(InteractablePickingPlugin)
        .add_plugin(HighlightablePickingPlugin)
        .add_plugin(DebugCursorPickingPlugin)
        .add_plugin(interface::EditorInterfacePlugin)
        .add_plugin(help_text::HelpTextPlugin)
        .add_plugin(play::EditorPlayPlugin)
        .add_plugin(property_editor::PropertyEditorPlugin)
        .add_event::<editor_camera::PositionCameraEvent>()
        .add_system_set(SystemSet::on_update(AppState::Editor)
            .with_system(handle_entity_click_events)
            .with_system(editor_camera::handle_position_camera_event)
            .with_system(editor_camera::update_camera)
        );
    }
}

fn handle_entity_click_events(
    mut commands: Commands,
    mut events: EventReader<PickingEvent>, 
    mut meshes: ResMut<Assets<Mesh>>,
    mut dude_meshes: ResMut<dude::DudeMeshes>,
    mut enemy_meshes: ResMut<snake::EnemyMeshes>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut position_camera_event_writer: EventWriter<editor_camera::PositionCameraEvent>,
    picking_cameras: Query<&PickingCamera>,
    interface: Res<interface::Interface>,
    entities: Query<&Transform>,
) {
    for event in events.iter() {
        match event {
            PickingEvent::Clicked(_) => {
                for picking_camera in picking_cameras.iter() {
                    if let Some((entity, intersection)) = picking_camera.intersect_top() {
                        println!("{:?} {:?}", convert_normal_to_face(&intersection.normal()), intersection.normal());
                        println!("{:?}", intersection.position().round());

                        match interface.current_state() {
                            interface::InterfaceMode::Normal => {
                                if let Ok(transform) = entities.get(entity) {
                                    position_camera_event_writer.send(editor_camera::PositionCameraEvent {
                                        translation: Vec3::ZERO,
                                        look_at: transform.translation
                                    });
                                }
                            },
                            interface::InterfaceMode::Add => {
                                if let Ok(transform) = entities.get(entity) {
                                    let mut selected_position = transform.translation;
                                    match convert_normal_to_face(&intersection.normal()) {
                                        Face::Above => selected_position.y += 1.0,
                                        Face::Below => selected_position.y -= 1.0,
                                        Face::Up => selected_position.x += 1.0,
                                        Face::Down => selected_position.x -= 1.0, 
                                        Face::Left => selected_position.z -= 1.0,
                                        Face::Right => selected_position.z += 1.0,
                                    }

                                    match interface.current_entity {
                                        interface::EntityButton::SnakeSpawn =>
                                            add_entity::add_snake(&mut commands, &mut enemy_meshes, &selected_position),
                                        interface::EntityButton::NotSnakeSpawn =>
                                            add_entity::add_not_snake(&mut commands, &mut dude_meshes, &selected_position),
                                        _ => add_entity::add_block(&mut commands, &mut meshes, &mut materials, &selected_position)
                                    }
                                    
                                    position_camera_event_writer.send(editor_camera::PositionCameraEvent {
                                        translation: intersection.normal(),
                                        look_at: selected_position
                                    });
                                }
                            },
                            _ => ()
                        }
                    }
                }
            },
            _ => ()
        }
    }
}

#[derive(Component)]
pub struct GameEntity {
    pub entity_type: GameEntityType 
}

pub enum GameEntityType {
    Block,
    Snake,
    NotSnake
}

#[derive(Debug)]
enum Face {
    Up, Down, Left, Right, Above, Below
}
fn convert_normal_to_face(normal: &Vec3) -> Face {
    let is_zero = |n| n < 0.5 && n > -0.5;
    let is_one = |n| n > 0.5;
    let is_negative_one = |n| n < -0.5;

    match (normal.x, normal.y, normal.z) {
        (x, y, z) if is_zero(x) && is_one(y) && is_zero(z)          => Face::Above,
        (x, y, z) if is_zero(x) && is_negative_one(y) && is_zero(z) => Face::Below,
        (x, y, z) if is_one(x) && is_zero(y) && is_zero(z)          => Face::Up,
        (x, y, z) if is_negative_one(x) && is_zero(y) && is_zero(z) => Face::Down,
        (x, y, z) if is_zero(x) && is_zero(y) && is_negative_one(z) => Face::Left,
        (x, y, z) if is_zero(x) && is_zero(y) && is_one(z)          => Face::Right,
        _ => panic!("Normal {:?} was not a unit normal", normal)
    }
}

fn load_editor(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.50,
    });

    add_entity::add_block(&mut commands, &mut meshes, &mut materials, &Vec3::new(0.0, 0.5, 0.0));

    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane { size: 5.0 })),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
        ..Default::default()
    }).insert(EditorTrashMarker);
}

fn cleanup_editor(
    mut commands: Commands,
    editor_trash: Query<Entity, With<EditorTrashMarker>>,
) {
    for entity in editor_trash.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
