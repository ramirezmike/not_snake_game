use bevy::prelude::*;
use crate::{Direction, EntityType, GameObject, level::Level, Position, holdable, block};

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
           .add_system_set(
               SystemSet::on_enter(crate::AppState::InGame)
                   .with_system(setup_dude.system())
           )
           .add_system_set(
               SystemSet::on_update(crate::AppState::InGame)
                   .with_system(check_assets_ready.system())
                   .with_system(spawn_dude.system())
                   .with_system(player_input.system())
                   .with_system(push_block.system())
                   .with_system(update_dude.system())
           );
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
    meshes.material = materials.add(Color::hex(crate::COLOR_BLOCK).unwrap().into());

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
    transform.rotate(Quat::from_axis_angle(Vec3::new(0.0, 1.0, 0.0), std::f32::consts::PI));
    let player_entity = 
    commands.spawn_bundle(PbrBundle {
                transform,
                ..Default::default()
            })
            .insert(Dude {
                facing: Direction::Left,
                target: None,
                is_jumping: false,
                queued_movement: None,
                action_cooldown: Timer::from_seconds(0.1, false),
                current_movement_time: 0.0
            })
            .insert(Position { x: 0, y: 0, z: 0 })
            .insert(EntityType::Dude)
            .insert(holdable::Holder { holding: None })
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
    let dude_movement_time = 0.10;
    for (entity, mut dude, mut dude_transform, mut dude_position) in dudes.iter_mut() {
        if !dude.target.is_some() && dude.queued_movement.is_some() {
            let queued_movement = dude.queued_movement.take().unwrap();
            dude.target = move_direction(&mut dude_position, level.width, level.length, queued_movement);
            dude.current_movement_time = 0.0;
        }

        if !dude.target.is_some() { continue; }

        let (mut target_translation, target_rotation) = dude.target.unwrap();

        let i32_target_translation = target_translation.as_i32();

        // move on top of block
        if level.is_type_with_vec(target_translation, Some(EntityType::Block)) 
           && level.is_type(dude_position.x, dude_position.y + 1, dude_position.z, None) 
           && level.is_type(i32_target_translation.x, i32_target_translation.y + 1, i32_target_translation.z, None) 
           && dude.facing == target_rotation {
            let above_dude = Vec3::new(dude_transform.translation.x, dude_transform.translation.y + 1.0, dude_transform.translation.z);
            target_translation.y += 1.0;
            dude.target = Some((above_dude, target_rotation));
            dude.queued_movement = Some(target_rotation);
            dude.is_jumping = true;
        }

        if dude.current_movement_time > dude_movement_time {
            dude_transform.translation = target_translation;
            dude.target = None;

            level.set(dude_position.x, dude_position.y, dude_position.z, None);
            dude_position.x = dude_transform.translation.x as i32;
            dude_position.y = dude_transform.translation.y as i32;
            dude_position.z = dude_transform.translation.z as i32;
            level.set(dude_position.x, dude_position.y, dude_position.z, Some(GameObject::new(entity, EntityType::Dude)));

            if level.is_type(dude_position.x, dude_position.y - 1, dude_position.z, None) && !dude.is_jumping {
                // dude is falling
                let target = Vec3::new(dude_position.x as f32, dude_position.y as f32 - 1.0, dude_position.z as f32);
                dude.target = Some((target, target_rotation));
                dude.current_movement_time = 0.0;
            }

            dude.is_jumping = false;

            continue;
        }

        dude_transform.rotation = match target_rotation {
                                      Direction::Up => Quat::from_axis_angle(Vec3::Y, -std::f32::consts::FRAC_PI_2),
                                      Direction::Down => Quat::from_axis_angle(Vec3::Y, std::f32::consts::FRAC_PI_2),
                                      Direction::Right => Quat::from_axis_angle(Vec3::Y, std::f32::consts::PI),
                                      Direction::Left => Quat::from_axis_angle(Vec3::Y, 0.0),
                                      _ => dude_transform.rotation
                                  };

        let is_target_cliff_player_isnt_facing =
                level.is_type_with_vec(target_translation, None)
             && level.is_type(i32_target_translation.x, i32_target_translation.y - 1, i32_target_translation.z, None) 
             && dude.facing != target_rotation;

        dude.facing = target_rotation;
        
        if !(level.is_type_with_vec(target_translation, None) || level.is_collectable_with_vec(target_translation))
            || is_target_cliff_player_isnt_facing {
            // can't move here
            println!("NO! {:?} is here", level.get_with_vec(target_translation));
            dude.target = None;
            dude_position.x = dude_transform.translation.x as i32;
            dude_position.y = dude_transform.translation.y as i32;
            dude_position.z = dude_transform.translation.z as i32;
            continue;
        } 

        dude.current_movement_time += time.delta_seconds();
        let new_translation = dude_transform.translation.lerp(target_translation, dude.current_movement_time / dude_movement_time);
        if !new_translation.is_nan() {
            dude_transform.translation = new_translation;
        }
    }
}

fn player_input(
    keyboard_input: Res<Input<KeyCode>>,
    level: Res<Level>,
    time: Res<Time>, 
    mut lift_holdable_event_writer: EventWriter<holdable::LiftHoldableEvent>,
    mut dudes: Query<(Entity, &mut Dude, &mut Position)>, 
) {
    for (entity, mut dude, mut position) in dudes.iter_mut() {
        dude.action_cooldown.tick(time.delta());
        if !dude.action_cooldown.finished() {
            continue;
        }

        if keyboard_input.just_pressed(KeyCode::J) && !dude.target.is_some() {
            dude.target = None;
            dude.queued_movement = None;
            lift_holdable_event_writer.send(holdable::LiftHoldableEvent(entity, dude.facing));
            dude.action_cooldown.reset();
            continue;
        }

        let mut move_dir = None;
        if keyboard_input.pressed(KeyCode::W) {
            move_dir = Some(Direction::Up); 
        }
        if keyboard_input.pressed(KeyCode::S) {
            move_dir = Some(Direction::Down); 
        }
        if keyboard_input.pressed(KeyCode::A) {
            move_dir = Some(Direction::Left); 
        }
        if keyboard_input.pressed(KeyCode::D) {
            move_dir = Some(Direction::Right); 
        }

        if let Some(move_dir) = move_dir {
            if !dude.target.is_some()   {
                dude.target = move_direction(&mut position, level.width, level.length, move_dir);
                dude.current_movement_time = 0.0;
                dude.action_cooldown.reset();
            }
        }
    }
}

fn push_block(
    keyboard_input: Res<Input<KeyCode>>,
    level: Res<Level>,
    dudes: Query<(&Dude, &Transform, &Position)>, 
    mut blocks: Query<&mut block::BlockObject>,
) { 
    for (dude, _transform, position) in dudes.iter() {
        if keyboard_input.just_pressed(KeyCode::K) {
            let (x, y, z) = match dude.facing {
                                Direction::Up => (position.x + 1, position.y, position.z),
                                Direction::Down => (position.x - 1, position.y, position.z),
                                Direction::Right => (position.x, position.y, position.z + 1),
                                Direction::Left => (position.x, position.y, position.z - 1),
                                _ => (position.x, position.y, position.z),
                            };

            if level.is_type(x, y, z, Some(EntityType::Block)) {
                if let Some(block) = level.get(x, y, z) {
                    if let Ok(mut block) = blocks.get_mut(block.entity) {
                        block.target = Some(dude.facing);
                        println!("Pushed block {:?}", block.target);
                    }
                }
            }
        }
    }
}

pub struct Dude {
    pub facing: Direction,
    pub target: Option::<(Vec3, Direction)>,
    queued_movement: Option::<Direction>,
    is_jumping: bool,
    action_cooldown: Timer,
    current_movement_time: f32
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
        _ => ()
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
