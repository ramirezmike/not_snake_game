use crate::{level, score::Score, sounds};
use bevy::prelude::*;

#[derive(Component)]
pub struct WinFlag {}
#[derive(Component)]
pub struct WinFlagInnerMesh {}
#[derive(Component)]
pub struct WinFlagOuterMesh {}

#[derive(Default)]
pub struct WinFlagMeshes {
    pub flag: Handle<Mesh>,
}

pub fn update_flag(
    mut flags: Query<
        (&WinFlagInnerMesh, &mut Visibility, &mut Transform),
        Without<WinFlagOuterMesh>,
    >,
    mut outer_flags: Query<
        (&WinFlagOuterMesh, &mut Visibility, &mut Transform),
        Without<WinFlagInnerMesh>,
    >,
    score: Res<Score>,
    level: ResMut<level::Level>,
    time: Res<Time>,
    mut is_flag_scaling_up: Local<bool>,
    mut sound_writer: EventWriter<sounds::SoundEvent>,
) {
    if score.current_level < level.get_current_minimum_food() {
        return;
    }

    for (_flag, mut visible, mut transform) in outer_flags.iter_mut() {
        if transform.scale.x == 1.0 {
            transform.scale = Vec3::new(50.0, 50.0, 50.0);
        }
        if transform.scale.x < 1.0 {
            transform.scale = Vec3::new(0.0, 0.0, 0.0);
            visible.is_visible = true;
            continue;
        } else {
            visible.is_visible = true;
        }

        transform.scale -= Vec3::splat(time.delta_seconds() * 75.0);
        transform.rotate(Quat::from_rotation_y(time.delta_seconds()));
        transform.rotate(Quat::from_rotation_z(time.delta_seconds()));
        transform.rotate(Quat::from_rotation_x(time.delta_seconds()));
    }

    for (_flag, mut visible, mut transform) in flags.iter_mut() {
        //transform.translation.y = 0.5 + (0.2 * time.seconds_since_startup().sin() as f32);

        if !visible.is_visible {
            print!("Making flag visible and sending sound!");
            // flag is becoming visible so send spawn sound
            sound_writer.send(sounds::SoundEvent(sounds::Sounds::FlagSpawn));
        }
        visible.is_visible = true;
        transform.rotate(Quat::from_rotation_y(time.delta_seconds()));

        if transform.scale.y < 1.0 {
            transform.scale += Vec3::splat(time.delta_seconds());
        } else if *is_flag_scaling_up {
            transform.scale.y += time.delta_seconds() * 0.2;
            *is_flag_scaling_up = transform.scale.y < 1.45;
        } else {
            transform.scale.y -= time.delta_seconds() * 0.2;
            *is_flag_scaling_up = transform.scale.y < 1.0;
        }
    }
}
