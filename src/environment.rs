use bevy::prelude::*;

pub struct EnvironmentPlugin;
impl Plugin for EnvironmentPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.insert_resource(Level {
               width: 6,
               length: 12,
               game_objects: vec![vec![vec![None; 12]; 1]; 6],
           })
           .add_startup_system(create_environment.system())
           .add_system(update_box.system());
    }
}

pub struct Level {
    pub width: i32,
    pub length: i32,
    pub game_objects: Vec::<Vec::<Vec::<Option::<GameObject>>>>
}

impl Level {
    pub fn set_with_vec(&mut self, position: Vec3, game_object: Option::<GameObject>) {
        self.set(position.x as i32, position.y as i32, position.z as i32, game_object);
    }

    pub fn set(&mut self, x: i32, y: i32, z: i32, game_object: Option::<GameObject>) {
        if x < 0 || y < 0 || z < 0 { return; }

        let (x, y, z) = (x as usize, y as usize, z as usize);
        if x < self.game_objects.len()
        && y < self.game_objects[x].len()
        && z < self.game_objects[x][y].len() { 
            self.game_objects[x][y][z] = game_object;
        }
    }

    pub fn get(&self, x: i32, y: i32, z: i32) -> Option::<GameObject> {
        self.game_objects[x as usize][y as usize][z as usize]
    }

    pub fn is_type_with_vec(&self, position: Vec3, entity_type: Option::<EntityType>) -> bool {
        self.is_type(position.x as i32, position.y as i32, position.z as i32, entity_type)
    }

    pub fn is_with_vec(&self, position: Vec3, game_object: Option::<GameObject>) -> bool {
        self.is(position.x as i32, position.y as i32, position.z as i32, game_object)
    }

    pub fn is(&self, x: i32, y: i32, z: i32, game_object: Option::<GameObject>) -> bool {
        if x < 0 || y < 0 || z < 0 { return false; }

        let (x, y, z) = (x as usize, y as usize, z as usize);

        if let Some(x_objects) = self.game_objects.get(x) {
            if let Some(y_objects) = x_objects.get(y) {
                if let Some(stored_game_object) = y_objects.get(z) {
                    return *stored_game_object == game_object;
                }
            }
        }

        false
    }

    pub fn is_type(&self, x: i32, y: i32, z: i32, entity_type: Option::<EntityType>) -> bool {
        if x < 0 || y < 0 || z < 0 { return false; }

        let (x, y, z) = (x as usize, y as usize, z as usize);

        if let Some(x_objects) = self.game_objects.get(x) {
            if let Some(y_objects) = x_objects.get(y) {
                if let Some(game_object) = y_objects.get(z) {
                    return match (game_object, entity_type) {
                        (None, None) => true,
                        (None, _) | (_, None) => false,
                        _ => game_object.unwrap().entity_type == entity_type.unwrap()
                    }
                }
            }
        }

        false
    }
}

#[derive(Copy, Clone, PartialEq)]
pub struct GameObject {
    pub entity: Entity,
    pub entity_type: EntityType
}

impl GameObject {
    pub fn new(entity: Entity, entity_type: EntityType) -> Self {
        GameObject { entity, entity_type }
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum EntityType {
    Block,
    Dude,
}

#[derive(Copy, Clone, Debug)]
pub enum Direction {
    Up, Down, Left, Right 
}

#[derive(Copy, Clone)]
pub struct Position { pub x: i32, pub y: i32, pub z: i32 }
impl Position {
    pub fn matches(&self, v: Vec3) -> bool {
        v.x as i32 == self.x && v.y as i32 == self.y && v.z as i32 == self.z
    }
}

pub fn create_environment(
    mut commands: Commands,
    mut level: ResMut<Level>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mesh = meshes.add(Mesh::from(shape::Plane { size: 1.0 }));

    for i in 0..level.width {
        for j in 0..level.length {
            commands.spawn_bundle(PbrBundle {
                mesh: mesh.clone(),
                material: if (i + j + 1) % 2 == 0 { 
                              materials.add(Color::hex(crate::COLOR_GROUND_1).unwrap().into())
                          } else {
                              materials.add(Color::hex(crate::COLOR_GROUND_2).unwrap().into())
                          },
                transform: Transform::from_translation(Vec3::new(i as f32, 0.0, j as f32)),
                ..Default::default()
            });
        }
    }

    commands.spawn_bundle(LightBundle {
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..Default::default()
    });

    let block_entity =
        commands.spawn_bundle(PbrBundle {
          mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
          material: materials.add(Color::hex(crate::COLOR_BOX).unwrap().into()),
          transform: Transform::from_xyz(2.0, 0.5, 5.0),
          ..Default::default()
        })
        .insert(Position { x: 2, y: 0, z: 5 })
        .insert(BoxObject { target: None })
        .id();
    level.set(2, 0, 5, Some(GameObject::new(block_entity, EntityType::Block)));
}

pub struct BoxObject { 
    pub target: Option::<Direction>,
}

fn update_box(
    mut boxes: Query<(Entity, &mut BoxObject, &mut Position, &mut Transform)>, 
    mut level: ResMut<Level>,
    time: Res<Time>, 
) {
    for (entity, mut box_object, mut position, mut transform) in boxes.iter_mut() {
        if !box_object.target.is_some() { continue; }

        let current = transform.translation;
        let target_translation = match box_object.target.unwrap() {
                                     Direction::Up 
                                         => Transform::from_xyz(current.x + 1.0, 
                                                                current.y, 
                                                                current.z),
                                     Direction::Down 
                                         => Transform::from_xyz(current.x - 1.0, 
                                                                current.y, 
                                                                current.z),
                                     Direction::Right 
                                         => Transform::from_xyz(current.x, 
                                                                current.y, 
                                                                current.z + 1.0),
                                     Direction::Left 
                                         => Transform::from_xyz(current.x, 
                                                                current.y, 
                                                                current.z - 1.0),
                                 }.translation;

        if level.is_type_with_vec(target_translation, None) 
            || level.is_with_vec(target_translation, Some(GameObject::new(entity, EntityType::Block))) {
            let target_position = Vec3::new(target_translation.x - transform.translation.x,
                                            0.0,
                                            target_translation.z - transform.translation.z).normalize();
             
            level.set(position.x, position.y, position.z, None);
            level.set_with_vec(target_position, Some(GameObject::new(entity, EntityType::Block)));
            transform.translation += target_position * 0.01 * time.delta().subsec_millis() as f32;
        } else {
            println!("Can't move!");
            box_object.target = None;
            position.x = transform.translation.x as i32;
            position.y = transform.translation.y as i32;
            position.z = transform.translation.z as i32;
            transform.translation = Vec3::new(position.x as f32, position.y as f32 + 0.5, position.z as f32);
            level.set(position.x, position.y, position.z, Some(GameObject::new(entity, EntityType::Block)));
        }
    }
}
