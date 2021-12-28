use bevy::prelude::*;
use bevy::input::mouse::{MouseWheel,MouseMotion};
use bevy::render::camera::PerspectiveProjection;
use crate::camera::MainCamera;
use bevy_mod_picking::*;

#[derive(Component)]
pub struct EditorCamera {
    pub is_being_controlled: bool
}

#[derive(Component, Debug)]
pub struct PanOrbitCamera {
    pub focus: Vec3,
    pub radius: f32,
    pub upside_down: bool,
}

impl Default for PanOrbitCamera {
    fn default() -> Self {
        PanOrbitCamera {
            focus: Vec3::new(0.0, 0.0, 0.0),
            radius: 5.0,
            upside_down: false,
        }
    }
}

pub fn update_camera(
    windows: Res<Windows>,
    mut ev_motion: EventReader<MouseMotion>,
    mut ev_scroll: EventReader<MouseWheel>,
    input_mouse: Res<Input<MouseButton>>,
    mut query: Query<(&mut PanOrbitCamera, &mut Transform, &PerspectiveProjection, &EditorCamera)>,
) {
    for (mut pan_orbit, mut transform, projection, editor_camera) in query.iter_mut() {
        if !editor_camera.is_being_controlled { continue; }

        // change input mapping for orbit and panning here
        let orbit_button = MouseButton::Right;
        let pan_button = MouseButton::Middle;

        let mut pan = Vec2::ZERO;
        let mut rotation_move = Vec2::ZERO;
        let mut scroll = 0.0;
        let mut orbit_button_changed = false;

        for ev in ev_motion.iter() {
            rotation_move += ev.delta;
        }

        for ev in ev_scroll.iter() {
            scroll += ev.y;
        }

        // if the camera is "upside" down, panning horizontally would be inverted, so invert the input to make it correct
        let up = transform.rotation * Vec3::Y;
        pan_orbit.upside_down = up.y <= 0.0;

        let mut any = false;
        if rotation_move.length_squared() > 0.0 {
            any = true;
            let window = get_primary_window_size(&windows);
            let delta_x = {
                let delta = rotation_move.x / window.x * std::f32::consts::PI * 2.0;
                if pan_orbit.upside_down { -delta } else { delta }
            };
            let delta_y = rotation_move.y / window.y * std::f32::consts::PI;
            let yaw = Quat::from_rotation_y(-delta_x);
            let pitch = Quat::from_rotation_x(-delta_y);
            transform.rotation = yaw * transform.rotation; // rotate around global y axis
            transform.rotation = transform.rotation * pitch; // rotate around local x axis
        } else if scroll.abs() > 0.0 {
            any = true;
            pan_orbit.radius -= scroll * pan_orbit.radius * 0.2;
            // dont allow zoom to reach zero or you get stuck
            pan_orbit.radius = f32::max(pan_orbit.radius, 0.05);
        }

        if any {
            // emulating parent/child to make the yaw/y-axis rotation behave like a turntable
            // parent = x and y rotation
            // child = z-offset
            let rot_matrix = Mat3::from_quat(transform.rotation);
            transform.translation = pan_orbit.focus + rot_matrix.mul_vec3(Vec3::new(0.0, 0.0, pan_orbit.radius));
        }
    }
}

fn get_primary_window_size(windows: &Res<Windows>) -> Vec2 {
    let window = windows.get_primary().unwrap();
    let window = Vec2::new(window.width() as f32, window.height() as f32);
    window
}

pub fn spawn_camera(
    mut commands: Commands,
    mut cameras: Query<Entity, With<MainCamera>>,
) {
    if let Ok(camera_entity) = cameras.get_single_mut() {
        commands.entity(camera_entity).despawn_recursive();
    }

    let translation = Vec3::new(-2.0, 2.5, 0.0);
    let radius = translation.length();

    commands.spawn_bundle(PerspectiveCameraBundle {
        transform: Transform::from_translation(translation)
            .looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
        ..Default::default()
    })
    .insert_bundle(PickingCameraBundle::default())
    .insert(EditorCamera { is_being_controlled: false })
    .insert(PanOrbitCamera {
        radius,
        ..Default::default()
    });
}
