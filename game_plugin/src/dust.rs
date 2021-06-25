use bevy::prelude::*;
use crate::{level, level::Level, Position, EntityType, GameObject, environment::HudFoodMesh};
use rand::{thread_rng, Rng};

pub struct Dust;
pub struct CreateDustEvent {
    pub position: Position
}

pub fn animate_dust(
    mut commands: Commands,
    mut dusts: Query<(&mut Transform, &Handle<StandardMaterial>, &Parent), With<Dust>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    time: Res<Time>,
) {
    for (mut transform, material, parent) in dusts.iter_mut() {
        transform.rotate(Quat::from_rotation_x(time.delta_seconds()));
        transform.rotate(Quat::from_rotation_y(time.delta_seconds()));
        transform.scale *= 1.0 + (time.delta_seconds() * 0.9);
        transform.translation.y += time.delta_seconds() * 0.5;

        let mut despawn_entity = true; // if the material doesn't exist, just despawn
        if let Some(material) = materials.get_mut(material) {
            let a = material.base_color.a();
            if a > 0.0 {
                despawn_entity = false;
                material.base_color.set_a(a - (time.delta_seconds() * 2.0));
            }
        } 

        if despawn_entity {
            commands.entity(**parent).despawn_recursive();
        }
    }
}

pub fn handle_create_dust_event(
    mut commands: Commands,
    mut create_dust_event_reader: EventReader<CreateDustEvent>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    level: Res<Level>,
) {
    for event in create_dust_event_reader.iter() {
        let position =  event.position;
        let color = Color::hex(level.get_palette().dude.clone()).unwrap();
        let transform = Transform::from_xyz(position.x as f32, position.y as f32, position.z as f32);

        for _ in 0..3 {
            let inner_mesh_x = thread_rng().gen_range(-50..50) as f32 / 100.0;
            let inner_mesh_z = thread_rng().gen_range(-50..50) as f32 / 100.0;

            let color = Color::rgba(color.r(), color.g(), color.b(), 0.9 + inner_mesh_x.abs());

            commands.spawn_bundle(PbrBundle {
              transform,
              ..Default::default()
            })
            .with_children(|parent| {
                parent.spawn_bundle(PbrBundle {
                    mesh: meshes.add(Mesh::from(shape::Cube { size: 0.1 })),
                    material: materials.add(color.into()),
                    transform: {
                        let mut t = Transform::from_xyz(inner_mesh_x, 0.1, inner_mesh_z);
                        t.rotate(Quat::from_rotation_x(inner_mesh_z));
                        t.rotate(Quat::from_rotation_y(inner_mesh_x));
                        t
                    },
                    visible: Visible {
                        is_visible: true,
                        is_transparent: false
                    },
                    ..Default::default()
                })
                .insert(Dust);
            });
        }
    }
}


