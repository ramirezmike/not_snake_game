use bevy::prelude::*;
use crate::environment;
use crate::environment::{Direction, EntityType, GameObject, Position, Level};

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
           .add_system(move_dude.system())
           .add_system(push_box.system());
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
                x: 0,
                y: 0,
                z: 0,
                facing: Direction::Left,
                target: None,
                queued_movement: None,
            })
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
    mut dudes: Query<(Entity, &mut Dude, &mut Transform)>, 
    mut level: ResMut<Level>,
    time: Res<Time>, 
) {
    for (entity, mut dude, mut dude_transform) in dudes.iter_mut() {
        if !dude.target.is_some() && dude.queued_movement.is_some() {
            let queued_movement = dude.queued_movement.take().unwrap();
            dude.move_direction(level.width, level.length, queued_movement);
        }

        if !dude.target.is_some() { continue; }

        let (target_translation, target_rotation) = dude.target.unwrap();

        if target_translation == dude_transform.translation || target_translation.distance(dude_transform.translation) < 0.1 {
            dude_transform.translation = target_translation;
            dude.target = None;

            level.set(dude.x, dude.y, dude.z, None);
            dude.x = dude_transform.translation.x as i32;
            dude.y = dude_transform.translation.y as i32;
            dude.z = dude_transform.translation.z as i32;
            level.set(dude.x, dude.y, dude.z, Some(GameObject::new(entity, EntityType::Dude)));
            continue;
        }

        let target_position = Vec3::new(target_translation.x - dude_transform.translation.x,
                                        0.0,
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
            dude.target = None;
            dude.x = dude_transform.translation.x as i32;
            dude.y = dude_transform.translation.y as i32;
            dude.z = dude_transform.translation.z as i32;
            println!("NO!");
            continue;
        } else {
            dude_transform.translation += target_position * 0.01 * time.delta().subsec_millis() as f32;
        }
    }
}

fn push_box(
    keyboard_input: Res<Input<KeyCode>>,
    level: Res<Level>,
    dudes: Query<(&Dude, &Transform)>, 
    mut blocks: Query<&mut environment::BoxObject>,
) { 
    for (dude, _transform) in dudes.iter() {
        if keyboard_input.just_pressed(KeyCode::E) {
            let (x, y, z) = match dude.facing {
                                Direction::Up => (dude.x + 1, dude.y, dude.z),
                                Direction::Down => (dude.x - 1, dude.y, dude.z),
                                Direction::Right => (dude.x, dude.y, dude.z + 1),
                                Direction::Left => (dude.x, dude.y, dude.z - 1),
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
    mut dudes: Query<(Entity, &mut Dude)>, 
) {
    for (_entity, mut dude) in dudes.iter_mut() {
        let mut move_direction = None;
        if keyboard_input.just_pressed(KeyCode::W) {
            move_direction = Some(Direction::Up); 
        }
        if keyboard_input.just_pressed(KeyCode::S) {
            move_direction = Some(Direction::Down); 
        }
        if keyboard_input.just_pressed(KeyCode::A) {
            move_direction = Some(Direction::Left); 
        }
        if keyboard_input.just_pressed(KeyCode::D) {
            move_direction = Some(Direction::Right); 
        }

        if let Some(move_direction) = move_direction {
            if !dude.target.is_some() {
                dude.move_direction(level.width, level.length, move_direction);
            } else {
                dude.queued_movement = Some(move_direction);
            }
        }
    }
}


struct Dude {
    x: i32,
    y: i32,
    z: i32,
    facing: Direction,
    target: Option::<(Vec3, Direction)>,
    queued_movement: Option::<Direction>,
}

impl Dude {
    fn move_direction(&mut self, width: i32, length: i32, direction: Direction) {
        match direction {
            Direction::Up => {
                if self.x < (width - 1) {
                    let target = Vec3::new(self.x as f32 + 1.0, self.y as f32, self.z as f32);
                    self.target = Some((target, direction));
                }
            },
            Direction::Down => {
                if self.x > 0 {
                    let target = Vec3::new(self.x as f32 - 1.0, self.y as f32, self.z as f32);
                    self.target = Some((target, direction));
                }
            }
            Direction::Right => {
                if self.z < length - 1 {
                    let target = Vec3::new(self.x as f32, self.y as f32, self.z as f32 + 1.0);
                    self.target = Some((target, direction));
                }
            }
            Direction::Left => {
                if self.z > 0 {
                    let target = Vec3::new(self.x as f32, self.y as f32, self.z as f32 - 1.0);
                    self.target = Some((target, direction));
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
