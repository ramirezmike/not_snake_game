use bevy::prelude::*;
use crate::{score::Score, level};

pub struct WinFlag { }
pub struct WinFlagInnerMesh { }
pub struct WinFlagOuterMesh { }

pub fn update_flag(
    mut flags: Query<(&WinFlagInnerMesh, &mut Visible, &mut Transform), Without<WinFlagOuterMesh>>,
    mut outer_flags: Query<(&WinFlagOuterMesh, &mut Visible, &mut Transform), Without<WinFlagInnerMesh>>,
    score: Res<Score>,
    level: ResMut<level::Level>,
    time: Res<Time>,
) {
    if score.current_level < level.get_current_minimum_food() { return; }

    let mut show_flag = false;
    for (_flag, mut visible, mut transform) in outer_flags.iter_mut() {
        if transform.scale.x == 1.0 {
            transform.scale = Vec3::new(50.0, 50.0, 50.0);
        }
        if transform.scale.x < 1.0 {
            transform.scale = Vec3::new(0.0, 0.0, 0.0);
            visible.is_visible = true;
            visible.is_transparent = true;
            show_flag = true; 
            continue;
        } else {
            visible.is_visible = true;
            visible.is_transparent = true;
        }

        transform.scale -= Vec3::splat(time.delta_seconds() * 75.0);
        transform.rotate(Quat::from_rotation_y(time.delta_seconds()));
        transform.rotate(Quat::from_rotation_z(time.delta_seconds()));
        transform.rotate(Quat::from_rotation_x(time.delta_seconds()));
    }

    if show_flag {
        for (_flag, mut visible, mut transform) in flags.iter_mut() {
            //transform.translation.y = 0.5 + (0.2 * time.seconds_since_startup().sin() as f32);
            visible.is_visible = true;
            transform.rotate(Quat::from_rotation_y(time.delta_seconds()));
            transform.rotate(Quat::from_rotation_z(time.delta_seconds()));
            transform.rotate(Quat::from_rotation_x(time.delta_seconds()));
        }
    }
}
