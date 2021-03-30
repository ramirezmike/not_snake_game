use bevy::prelude::*;
//  use bevy_mod_picking::*;
//  use bevy_rapier3d::rapier::geometry::ColliderBuilder;
//  use bevy_rapier3d::rapier::dynamics::{RigidBodyBuilder,RigidBodySet};
//  use bevy_rapier3d::physics::RigidBodyHandleComponent;
//  use bevy_rapier3d::rapier::math::Isometry;

mod fly_camera;

use fly_camera::*;

pub struct CameraPlugin;
impl Plugin for CameraPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app//.add_plugin(PickingPlugin)
           .add_plugin(FlyCameraPlugin)
           .add_startup_system(create_camera.system())
//           .add_system(update_camera.system())
           //.add_system(toggle_fly.system())
           //.add_system(update_camera_collisions.system());
           ;
    }
}
    
fn update_camera(cameras: Query<(Entity, &Camera, &Transform)>) {
    for (_e, _camera, transform) in cameras.iter() {
        println!("{:?}", transform);
    }
}

fn create_camera(
    mut commands: Commands,
    mut windows: ResMut<Windows>,
) {
    let mut transform = Transform::default();
    transform.translation = Vec3::new(-6.867214, 5.8081317, 5.4974184);
    transform.rotation = Quat::from_xyzw(-0.14680715, -0.6914177, -0.14668213, 0.692007);

    commands
        .spawn_bundle(PerspectiveCameraBundle {
            transform, 
            ..Default::default()
        })
        .insert(Camera)
//        .insert(FlyCamera::default())
        .with_children(|_parent|  {
//          parent.spawn(Player)
//                .with(ColliderBuilder::cuboid(1.0, 2.0, 1.0))
//                .with(RigidBodyBuilder::new_kinematic());
        })
 //       .with(PickSource::default());
            ;

        let window = windows.get_primary_mut().unwrap();
        window.set_cursor_lock_mode(true);
        window.set_cursor_visibility(false);
}

#[derive(Bundle)]
struct Player { }

pub struct Camera;

/*
fn toggle_fly(commands: &mut Commands, keys: Res<Input<KeyCode>>, 
    mut windows: ResMut<Windows>,
    mut camera: Query<(Entity, &mut Camera, Option<&FlyCamera>)>) {
    if keys.just_pressed(KeyCode::F) {
        let window = windows.get_primary_mut().unwrap();
        for (e, _, f) in camera.iter_mut() {
            match f {
                Some(_) => {
                    commands.remove_one::<FlyCamera>(e);
                    window.set_cursor_lock_mode(false);
                    window.set_cursor_visibility(true);
                },
                None => {
                    commands.insert_one(e, FlyCamera::default());
                    window.set_cursor_lock_mode(true);
                    window.set_cursor_visibility(false);
                }
            }
        }
    }
}

fn update_camera_collisions(
    mut camera: Query<(&mut Camera, &mut Children, &Transform)>,
    mut rigid_body_handles: Query<(Entity, &mut RigidBodyHandleComponent)>,
    mut rigid_bodies: ResMut<RigidBodySet>
) {
    for (_, children, transform) in camera.iter_mut() {
        for entity in children.iter() {
            if let Ok(child) = rigid_body_handles.get_mut(*entity) {
                if let Some(rigid_body_set) = rigid_bodies.get_mut(child.1.handle()) {
                    let (x, y, z) = { 
                        let translation = transform.translation;
                        (translation.x, translation.y, translation.z)
                    };

                    rigid_body_set.set_next_kinematic_position(Isometry::translation(x, y, z));
                }
            }
        }
    }
}
*/
