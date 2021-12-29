use bevy::prelude::*;
use bevy_mod_picking::*;
use crate::AppState;

mod editor_camera;
mod help_text;
mod interface; 

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
        .add_system_set(SystemSet::on_update(AppState::Editor)
            .with_system(print_events)
            .with_system(editor_camera::update_camera)
        );
    }
}

fn print_events(
    mut events: EventReader<PickingEvent>, 
    picking_cameras: Query<&PickingCamera>,
) {
    for event in events.iter() {
        match event {
            PickingEvent::Clicked(_) => {
                for picking_camera in picking_cameras.iter() {
                    if let Some((entity, intersection)) = picking_camera.intersect_top() {
                        println!("{:?} {:?}", convert_normal_to_face(&intersection.normal()), intersection.normal());
                    }
                }
            },
            _ => ()
        }
    }
}

#[derive(Debug)]
enum Face {
    Up, Down, Left, Right, Above, Below
}
fn convert_normal_to_face(normal: &Vec3) -> Face {
    match (normal.x, normal.y, normal.z) {
        (0.0, 1.0, 0.0) => Face::Above,
        (0.0, -1.0, 0.0) => Face::Below,
        (1.0, 0.0, 0.0) => Face::Up,
        (-1.0, 0.0, 0.0) => Face::Down,
        (0.0, 0.0, -1.0) => Face::Left,
        (0.0, 0.0, 1.0) => Face::Right,
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

    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        ..Default::default()
    }).insert_bundle(PickableBundle::default());

    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane { size: 5.0 })),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
        ..Default::default()
    });
}
