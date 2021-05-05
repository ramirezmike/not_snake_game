use bevy::prelude::*;
use crate::{score::Score, level};

pub struct WinFlag { }
pub struct WinFlagInnerMesh { }

pub fn update_flag(
    mut flags: Query<(&WinFlagInnerMesh, &mut Visible, &mut Transform)>,
    score: Res<Score>,
    level: ResMut<level::Level>,
    time: Res<Time>,
) {
    for (_flag, mut visible, mut transform) in flags.iter_mut() {
        //transform.translation.y = 0.5 + (0.2 * time.seconds_since_startup().sin() as f32);

        if score.current_level >= level.get_current_minimum_food() {
            visible.is_visible = true;
            transform.rotate(Quat::from_rotation_y(time.delta_seconds()));
            transform.rotate(Quat::from_rotation_z(time.delta_seconds()));
            transform.rotate(Quat::from_rotation_x(time.delta_seconds()));
        }
    }
}
