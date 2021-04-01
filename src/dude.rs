use bevy::prelude::*;
use crate::{environment, Direction, EntityType, GameObject, level::Level, Position};

#[derive(Default)]
struct Loaded(bool);
#[derive(Default)]
struct Spawned(bool);
pub struct DudePlugin;
impl Plugin for DudePlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.init_resource::<DudeMeshes>()
           .init_resource::<AssetsLoading>()
           .init_resource::<Loaded>()
           .init_resource::<Spawned>()
           .add_system(check_assets_ready.system())
           .add_startup_system(setup_dude.system())
           .add_system(spawn_dude.system())
           .add_system(move_dude.system())
           .add_system(lift_box.system())
           .add_system(push_box.system())
           .add_system(update_dude.system());
    }
}

fn setup_dude(
    mut _commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<DudeMeshes>,
    mut loading: ResMut<AssetsLoading>,
) {
    meshes.step1 = asset_server.load("models/dude.glb#Mesh0/Primitive0");
    meshes.material = materials.add(Color::hex(crate::COLOR_BOX).unwrap().into());

    loading.0.push(meshes.step1.clone_untyped());
}

fn spawn_dude( 
    mut commands: Commands, 
    meshes: Res<DudeMeshes>, 
    loaded: Res<Loaded>,
    mut spawned: ResMut<Spawned>,
    mut level: ResMut<Level>,
) {
    if !loaded.0 || spawned.0 { return; }

    let mut transform = Transform::from_translation(Vec3::new(0.0, 0.0, 0.0));
    transform.apply_non_uniform_scale(Vec3::new(0.25, 0.25, 0.25)); 
    transform.rotate(Quat::from_axis_angle(Vec3::new(0.0, 1.0, 0.0), 0.0));
    let player_entity = 
    commands.spawn_bundle(PbrBundle {
                transform,
                ..Default::default()
            })
            .insert(Dude {
                facing: Direction::Left,
                target: None,
                queued_movement: None,
                holding: None,
                lift_cooldown: Timer::from_seconds(0.2, false),
            })
            .insert(Position { x: 0, y: 0, z: 0 })
            .insert(EntityType::Dude)
            .with_children(|parent|  {
                parent.spawn_bundle(PbrBundle {
                    mesh: meshes.step1.clone(),
                    material: meshes.material.clone(),
                    transform: {
                        let mut transform = Transform::from_rotation(Quat::from_axis_angle(Vec3::new(0.0, 1.0, 0.0), 1.57079632679));
                        transform.translation = Vec3::new(0.0, 0.5, 0.0);
                        transform
                    },
                    ..Default::default()
                });
            }).id();
    spawned.0 = true;
    level.set(0, 0, 0, Some(GameObject::new(player_entity, EntityType::Dude)));
}

fn update_dude(
    mut dudes: Query<(Entity, &mut Dude, &mut Transform, &mut Position)>, 
    mut level: ResMut<Level>,
    time: Res<Time>, 
) {
    for (entity, mut dude, mut dude_transform, mut dude_position) in dudes.iter_mut() {
        dude.lift_cooldown.tick(time.delta());
//        println!("{} {} {}", dude.x, dude.y, dude.z);
        if !dude.target.is_some() && dude.queued_movement.is_some() {
            let queued_movement = dude.queued_movement.take().unwrap();
            dude.target = move_direction(&mut dude_position, level.width, level.length, queued_movement);
        }

        if !dude.target.is_some() { continue; }

        let (target_translation, target_rotation) = dude.target.unwrap();

        if target_translation == dude_transform.translation || target_translation.distance(dude_transform.translation) < 0.1 {
            dude_transform.translation = target_translation;
            dude.target = None;

            level.set(dude_position.x, dude_position.y, dude_position.z, None);
            dude_position.x = dude_transform.translation.x as i32;
            dude_position.y = dude_transform.translation.y as i32;
            dude_position.z = dude_transform.translation.z as i32;
            level.set(dude_position.x, dude_position.y, dude_position.z, Some(GameObject::new(entity, EntityType::Dude)));

            if level.is_type(dude_position.x, dude_position.y - 1, dude_position.z, None) {
                let target = Vec3::new(dude_position.x as f32, dude_position.y as f32 - 1.0, dude_position.z as f32);
                dude.target = Some((target, target_rotation));
            }

            continue;
        }

        let target_position = Vec3::new(target_translation.x - dude_transform.translation.x,
                                        target_translation.y - dude_transform.translation.y,
                                        target_translation.z - dude_transform.translation.z).normalize();

        dude_transform.rotation = match target_rotation {
                                      Direction::Up => Quat::from_axis_angle(Vec3::Y, -std::f32::consts::FRAC_PI_2),
                                      Direction::Down => Quat::from_axis_angle(Vec3::Y, std::f32::consts::FRAC_PI_2),
                                      Direction::Right => Quat::from_axis_angle(Vec3::Y, std::f32::consts::PI),
                                      Direction::Left => Quat::from_axis_angle(Vec3::Y, 0.0),
                                  };
        dude.facing = target_rotation;

        if !level.is_type_with_vec(target_translation, None) {
            // can't move here
            println!("NO! {:?} is here", level.get_with_vec(target_translation));
            dude.target = None;
            dude_position.x = dude_transform.translation.x as i32;
            dude_position.y = dude_transform.translation.y as i32;
            dude_position.z = dude_transform.translation.z as i32;
            continue;
        } else {
            dude_transform.translation += target_position * 0.01 * time.delta().subsec_millis() as f32;
        }
    }
}

fn lift_box(
    mut commands: Commands, 
    keyboard_input: Res<Input<KeyCode>>,
    mut level: ResMut<Level>,
    mut dudes: Query<(Entity, &mut Dude, &mut Transform, &mut Position), Without<environment::BoxObject>>, 
    mut blocks: QuerySet<(Query<(&mut environment::BoxObject, &mut Transform, &mut Position)>, 
                          Query<(&environment::BoxObject, &Transform, &mut Position)>)>
) {
    if keyboard_input.just_pressed(KeyCode::J) {
        for (dude_entity, mut dude, mut dude_transform, mut dude_position) in dudes.iter_mut() {
            if !dude.lift_cooldown.finished() {
                continue;
            }
            dude.lift_cooldown.reset();
            match dude.holding {
                Some(held_entity) => {
                    println!("Dropping box {} {} {}", dude_position.x, dude_position.y, dude_position.z);
                    commands.entity(held_entity)
                            .remove::<environment::BeingHeld>();

                    if let Ok((_block, mut transform, mut position)) = blocks.q0_mut().get_mut(held_entity) {
                        let drop_point = dude_transform.translation.as_i32();
                        transform.translation = drop_point.as_f32();
                        level.set_with_vec(transform.translation, Some(GameObject::new(held_entity, EntityType::Block)));
                        position.x = drop_point.x;
                        position.y = drop_point.y;
                        position.z = drop_point.z;
                    }

                    dude_transform.translation.y += 1.0;
                    dude_position.x = dude_transform.translation.x as i32;
                    dude_position.y = dude_transform.translation.y as i32;
                    dude_position.z = dude_transform.translation.z as i32;
                    level.set_with_vec(dude_transform.translation, Some(GameObject::new(dude_entity, EntityType::Dude)));
                    dude.holding = None;
                },
                None => {
                    let (x, y, z) = match dude.facing {
                                        Direction::Up => (dude_position.x + 1, dude_position.y, dude_position.z),
                                        Direction::Down => (dude_position.x - 1, dude_position.y, dude_position.z),
                                        Direction::Right => (dude_position.x, dude_position.y, dude_position.z + 1),
                                        Direction::Left => (dude_position.x, dude_position.y, dude_position.z - 1),
                                    };
                    if level.is_type(x, y, z, Some(EntityType::Block)) {
                        if let Some(block) = level.get(x, y, z) {
                            println!("Picking up box {} {} {}", dude_position.x, dude_position.y, dude_position.z);
                            commands.entity(block.entity)
                                    .insert(environment::BeingHeld { held_by: dude_entity });
                            if let Ok((_block, transform, mut position)) = blocks.q1_mut().get_mut(block.entity) {
                                level.set_with_vec(transform.translation, None);
                                *position = Position { x: -1, y: -1, z: -1 };
                            }
                            dude.holding = Some(block.entity);
                        }
                    }
                }
            }
        }
    }
}

fn push_box(
    keyboard_input: Res<Input<KeyCode>>,
    level: Res<Level>,
    dudes: Query<(&Dude, &Transform, &Position)>, 
    mut blocks: Query<&mut environment::BoxObject>,
) { 
    for (dude, _transform, position) in dudes.iter() {
        if keyboard_input.just_pressed(KeyCode::H) {
            let (x, y, z) = match dude.facing {
                                Direction::Up => (position.x + 1, position.y, position.z),
                                Direction::Down => (position.x - 1, position.y, position.z),
                                Direction::Right => (position.x, position.y, position.z + 1),
                                Direction::Left => (position.x, position.y, position.z - 1),
                            };

            if level.is_type(x, y, z, Some(EntityType::Block)) {
                if let Some(block) = level.get(x, y, z) {
                    if let Ok(mut block) = blocks.get_mut(block.entity) {
                        block.target = Some(dude.facing);
                        println!("Pushed box {:?}", block.target);
                    }
                }
            }
        }
    }
}

fn move_dude(
    keyboard_input: Res<Input<KeyCode>>,
    level: Res<Level>,
    mut dudes: Query<(Entity, &mut Dude, &mut Position)>, 
) {
    for (_entity, mut dude, mut position) in dudes.iter_mut() {
        let mut move_dir = None;
        if keyboard_input.just_pressed(KeyCode::W) {
            move_dir = Some(Direction::Up); 
        }
        if keyboard_input.just_pressed(KeyCode::S) {
            move_dir = Some(Direction::Down); 
        }
        if keyboard_input.just_pressed(KeyCode::A) {
            move_dir = Some(Direction::Left); 
        }
        if keyboard_input.just_pressed(KeyCode::D) {
            move_dir = Some(Direction::Right); 
        }

        if let Some(move_dir) = move_dir {
            if !dude.target.is_some() {
                dude.target = move_direction(&mut position, level.width, level.length, move_dir);
            } else {
                dude.queued_movement = Some(move_dir);
            }
        }
    }
}

struct Dude {
    facing: Direction,
    target: Option::<(Vec3, Direction)>,
    queued_movement: Option::<Direction>,
    holding: Option::<Entity>,
    lift_cooldown: Timer,
}

fn move_direction(position: &mut Position, width: i32, length: i32, direction: Direction) -> Option::<(Vec3, Direction)> {
    let mut result = None;
    match direction {
        Direction::Up => {
            if position.x < (width - 1) {
                let target = Vec3::new(position.x as f32 + 1.0, position.y as f32, position.z as f32);
                result = Some((target, direction))
            }
        },
        Direction::Down => {
            if position.x > 0 {
                let target = Vec3::new(position.x as f32 - 1.0, position.y as f32, position.z as f32);
                result = Some((target, direction))
            }
        }
        Direction::Right => {
            if position.z < length - 1 {
                let target = Vec3::new(position.x as f32, position.y as f32, position.z as f32 + 1.0);
                result = Some((target, direction))
            }
        }
        Direction::Left => {
            if position.z > 0 {
                let target = Vec3::new(position.x as f32, position.y as f32, position.z as f32 - 1.0);
                result = Some((target, direction))
            }
        }
    }

    result
}

#[derive(Default)]
struct DudeMeshes {
    step1: Handle<Mesh>,
    material: Handle<StandardMaterial>,
}

#[derive(Default)]
struct AssetsLoading(Vec<HandleUntyped>);
fn check_assets_ready(
    server: Res<AssetServer>,
    loading: Res<AssetsLoading>,
    mut loaded: ResMut<Loaded>,
) {
    if loaded.0 { return; }

    use bevy::asset::LoadState;

    let mut ready = true;

    for handle in loading.0.iter() {
        match server.get_load_state(handle) {
            LoadState::Failed => {
                // one of our assets had an error
            }
            LoadState::Loaded => {
            }
            _ => {
                ready = false;
            }
        }
    }

    loaded.0 = ready;
}
