use bevy::prelude::*;
use crate::{Direction, EntityType, GameObject, level::Level, 
            Position, holdable, block, moveable, facing::Facing};

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
                   .with_system(load_dude.system())
           )
           .add_system_set(
               SystemSet::on_update(crate::AppState::InGame)
                   .with_system(check_assets_ready.system())
                   .with_system(spawn_dude.system())
                   .with_system(player_input.system())
                   .with_system(push_block.system())
           );
    }
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
                action_cooldown: Timer::from_seconds(0.15, false),
            })
            .insert(Position { x: 0, y: 0, z: 0 })
            .insert(EntityType::Dude)
            .insert(holdable::Holder { holding: None })
            .insert(moveable::Moveable::new(true, true, 0.1))
            .insert(Facing::new(Direction::Right))
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

fn player_input(
    keyboard_input: Res<Input<KeyCode>>,
    time: Res<Time>, 
    mut lift_holdable_event_writer: EventWriter<holdable::LiftHoldableEvent>,
    mut dudes: Query<(Entity, &mut Dude, &mut moveable::Moveable, &Facing)>, 
) {
    for (entity, mut dude, mut moveable, facing) in dudes.iter_mut() {
        dude.action_cooldown.tick(time.delta());
        if !dude.action_cooldown.finished() {
            continue;
        }

        if keyboard_input.just_pressed(KeyCode::J) && !moveable.is_moving() {
            lift_holdable_event_writer.send(holdable::LiftHoldableEvent(entity, facing.direction));
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
            if !moveable.is_moving()   {
                moveable.set_movement(move_dir, moveable::MovementType::Step);
            }
            dude.action_cooldown.reset();
        }
    }
}

fn push_block(
    keyboard_input: Res<Input<KeyCode>>,
    level: Res<Level>,
    dudes: Query<(&Transform, &Position, &Facing)>, 
    mut blocks: Query<(&block::BlockObject, &mut moveable::Moveable)>,
) { 
    for (_transform, position, facing) in dudes.iter() {
        if keyboard_input.just_pressed(KeyCode::K) {
            let (x, y, z) = match facing.direction {
                                Direction::Up => (position.x + 1, position.y, position.z),
                                Direction::Down => (position.x - 1, position.y, position.z),
                                Direction::Right => (position.x, position.y, position.z + 1),
                                Direction::Left => (position.x, position.y, position.z - 1),
                                _ => (position.x, position.y, position.z),
                            };

            if level.is_type(x, y, z, Some(EntityType::Block)) {
                if let Some(block) = level.get(x, y, z) {
                    if let Ok((_block, mut moveable)) = blocks.get_mut(block.entity) {
                        moveable.set_movement(facing.direction, moveable::MovementType::Slide);
                        println!("Pushed block {:?}", moveable);
                    }
                }
            }
        }
    }
}

pub struct Dude {
    action_cooldown: Timer,
}

fn load_dude(
    mut _commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<DudeMeshes>,
    mut loading: ResMut<AssetsLoading>,
) {
    meshes.step1 = asset_server.load("models/dude.glb#Mesh0/Primitive0");
    meshes.material = materials.add(Color::hex(crate::COLOR_DUDE).unwrap().into());

    loading.0.push(meshes.step1.clone_untyped());
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
