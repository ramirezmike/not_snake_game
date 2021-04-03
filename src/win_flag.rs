use bevy::prelude::*;

pub struct WinFlag { }
pub struct WinFlagInnerMesh { }

pub fn update_flag(
    mut flags: Query<(&WinFlagInnerMesh, &mut Transform)>,
    time: Res<Time>,
) {
    for (_flag, mut transform) in flags.iter_mut() {
        transform.translation.y = 0.5 + (0.2 * time.seconds_since_startup().sin() as f32);
        transform.rotate(Quat::from_rotation_y(time.delta_seconds()));
        transform.rotate(Quat::from_rotation_z(time.delta_seconds()));
        transform.rotate(Quat::from_rotation_x(time.delta_seconds()));
    }
}
