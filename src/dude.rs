use bevy::prelude::*;
use crate::environment;

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
           .add_system(update_dude.system())
           .add_system(move_dude.system());
//         .add_system(animate_ant.system())
//         .add_system(update_ant.system())
//         .add_system(update_raycast_with_cursor.system())
//          .add_system_to_stage(
//              stage::POST_UPDATE, // We want this system to run after we've updated our ray casting source
//              update_raycast::<crate::MyRaycastSet>.system(), // This provided system does the ray casting
//          )
//          .add_system_to_stage(
//              stage::POST_UPDATE, // We want this system to run after ray casting has been computed
//              update_debug_cursor::<crate::MyRaycastSet>.system(), // Update the debug cursor location
//          );
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
) {
    if !loaded.0 || spawned.0 { return; }

    let mut transform = Transform::from_translation(Vec3::new(0.0, 0.0, 0.0));
    transform.apply_non_uniform_scale(Vec3::new(0.25, 0.25, 0.25)); 
    transform.rotate(Quat::from_axis_angle(Vec3::new(0.0, 1.0, 0.0), 0.0));
    commands.spawn_bundle(PbrBundle {
                transform,
                ..Default::default()
            })
            .insert(Dude {
                x: 0,
                y: 0,
                z: 0,
                target: None,
                queued_movement: None,
            })
            .with_children(|parent|  {
                parent.spawn_bundle(PbrBundle {
                    mesh: meshes.step1.clone(),
                    material: meshes.material.clone(),
                    transform: Transform::from_rotation(Quat::from_axis_angle(Vec3::new(0.0, 1.0, 0.0), 1.57079632679)),
                    ..Default::default()
                });
            });
    spawned.0 = true;
}

fn update_dude(
    mut dudes: Query<(Entity, &mut Dude, &mut Transform)>, 
    level: Res<environment::Level>,
    time: Res<Time>, 
) {
    for (_entity, mut dude, mut dude_transform) in dudes.iter_mut() {
        if !dude.target.is_some() && dude.queued_movement.is_some() {
            let queued_movement = dude.queued_movement.take().unwrap();
            dude.move_direction(level.width, level.length, queued_movement);
        }

        if !dude.target.is_some() { continue; }

        let (target_translation, target_rotation) = dude.target.unwrap();

        if target_translation == dude_transform.translation || target_translation.distance(dude_transform.translation) < 0.1 {
            dude_transform.translation = target_translation;
            dude.target = None;
            dude.x = dude_transform.translation.x as i32;
            dude.y = dude_transform.translation.y as i32;
            dude.z = dude_transform.translation.z as i32;
            continue;
        }

        let target_position = Vec3::new(target_translation.x - dude_transform.translation.x,
                                        0.0,
                                        target_translation.z - dude_transform.translation.z).normalize();
         
        dude_transform.translation += target_position * 0.01 * time.delta().subsec_millis() as f32;
        dude_transform.rotation = target_rotation;
    }
}

enum MoveDirection {
    Up, Down, Left, Right 
}

fn move_dude(
    keyboard_input: Res<Input<KeyCode>>,
    level: Res<environment::Level>,
    mut dudes: Query<(Entity, &mut Dude)>, 
) {
    for (_entity, mut dude) in dudes.iter_mut() {
        if keyboard_input.just_pressed(KeyCode::W) {
            if !dude.target.is_some() {
                dude.move_direction(level.width, level.length, MoveDirection::Up);
            } else {
                dude.queued_movement = Some(MoveDirection::Up);
            }
        }
        if keyboard_input.just_pressed(KeyCode::S) {
            if !dude.target.is_some() {
                dude.move_direction(level.width, level.length, MoveDirection::Down);
            } else {
                dude.queued_movement = Some(MoveDirection::Down);
            }
        }
        if keyboard_input.just_pressed(KeyCode::A) {
            if !dude.target.is_some() {
                dude.move_direction(level.width, level.length, MoveDirection::Left);
            } else {
                dude.queued_movement = Some(MoveDirection::Left);
            }
        }
        if keyboard_input.just_pressed(KeyCode::D) {
            if !dude.target.is_some() {
                dude.move_direction(level.width, level.length, MoveDirection::Right);
            } else {
                dude.queued_movement = Some(MoveDirection::Right);
            }
        }
    }
}


struct Dude {
    x: i32,
    y: i32,
    z: i32,
    target: Option::<(Vec3, Quat)>,
    queued_movement: Option::<MoveDirection>,
}

impl Dude {
    fn move_direction(&mut self, width: i32, length: i32, direction: MoveDirection) {
        match direction {
            MoveDirection::Up => {
                if self.x < (width - 1) {
                    let target = Vec3::new(self.x as f32 + 1.0, self.y as f32, self.z as f32);
                    let rotation = Quat::from_axis_angle(Vec3::Y, -std::f32::consts::FRAC_PI_2);
                    self.target = Some((target, rotation));
                }
            },
            MoveDirection::Down => {
                if self.x > 0 {
                    let target = Vec3::new(self.x as f32 - 1.0, self.y as f32, self.z as f32);
                    let rotation = Quat::from_axis_angle(Vec3::Y, std::f32::consts::FRAC_PI_2);
                    self.target = Some((target, rotation));
                }
            }
            MoveDirection::Right => {
                if self.z < length - 1 {
                    let target = Vec3::new(self.x as f32, self.y as f32, self.z as f32 + 1.0);
                    let rotation = Quat::from_axis_angle(Vec3::Y, std::f32::consts::PI);
                    self.target = Some((target, rotation));
                }
            }
            MoveDirection::Left => {
                if self.z > 0 {
                    let target = Vec3::new(self.x as f32, self.y as f32, self.z as f32 - 1.0);
                    let rotation = Quat::from_axis_angle(Vec3::Y, 0.0);
                    self.target = Some((target, rotation));
                }
            }
        }
    }
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
