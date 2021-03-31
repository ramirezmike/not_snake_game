use bevy::prelude::*;

pub struct EnvironmentPlugin;
impl Plugin for EnvironmentPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.insert_resource(Level {
               width: 6,
               length: 12,
           })
           .add_startup_system(create_environment.system())
           .add_system(update_box.system());
    }
}

pub struct Level {
    pub width: i32,
    pub length: i32,
}

pub struct Position { pub x: i32, pub y: i32, pub z: i32 }
#[derive(Copy, Clone, Debug)]
pub enum Direction {
    Up, Down, Left, Right 
}


impl Position {
    pub fn matches(&self, v: Vec3) -> bool {
        v.x as i32 == self.x && v.y as i32 == self.y && v.z as i32 == self.z
    }
}

impl Level {
}

pub fn create_environment(
    mut commands: Commands,
    level: Res<Level>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>
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

    commands.spawn_bundle(PbrBundle {
      mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
      material: materials.add(Color::hex(crate::COLOR_BOX).unwrap().into()),
      transform: Transform::from_xyz(2.0, 0.5, 5.0),
      ..Default::default()
    })
    .insert(Position { x: 2, y: 0, z: 5 })
    .insert(BoxObject { target: None });
}

pub struct BoxObject { 
    pub target: Option::<Direction>,
}

fn update_box(
    mut boxes: QuerySet<(Query<(Entity, &mut BoxObject, &mut Position, &mut Transform)>, 
                         Query<(Entity, &BoxObject, &Position)>)>, 
    time: Res<Time>, 
) {
    let mut boxes_to_update: Vec::<(Entity, Vec3)> = Vec::new();
    for (entity, mut box_object, mut position, mut transform) in boxes.q0_mut().iter_mut() {
        if !box_object.target.is_some() { continue; }

        let target_translation = match box_object.target.unwrap() {
                                     Direction::Up 
                                         => Transform::from_xyz((position.x + 1) as f32, 
                                                                position.y as f32, 
                                                                position.z as f32),
                                     Direction::Down 
                                         => Transform::from_xyz((position.x - 1) as f32, 
                                                                position.y as f32, 
                                                                position.z as f32),
                                     Direction::Right 
                                         => Transform::from_xyz(position.x as f32, 
                                                                position.y as f32, 
                                                                (position.z + 1) as f32),
                                     Direction::Left 
                                         => Transform::from_xyz(position.x as f32, 
                                                                position.y as f32, 
                                                                (position.z - 1) as f32),
                                 }.translation;

        if target_translation == transform.translation || target_translation.distance(transform.translation) < 0.1 {
            transform.translation = target_translation;
            box_object.target = None;
            position.x = transform.translation.x as i32;
            position.y = transform.translation.y as i32;
            position.z = transform.translation.z as i32;
            continue;
        }

        boxes_to_update.push((entity, target_translation));
    }

    let mut boxes_that_cant_move = Vec::new();
    for (entity, _box_object, position) in boxes.q1_mut().iter_mut() {
        let (ok_boxes, mut blocked_boxes): (Vec::<(Entity, Vec3)>, Vec::<(Entity, Vec3)>) 
            = boxes_to_update.into_iter().partition(|x| entity == x.0 || !position.matches((*x).1));
        boxes_that_cant_move.append(&mut blocked_boxes); 
        boxes_to_update = ok_boxes;
    }

    for blocked_box in boxes_that_cant_move {
        // need to get the box entity and set its target to None
        // also need to populate boxes_that_cant_move with boxes that are
        // out of bounds based on the level structure
    }

    for (entity, target_translation) in boxes_to_update {
        if let Ok((_entity, _box_object, mut position, mut transform)) = boxes.q0_mut().get_mut(entity) {
            let target_position = Vec3::new(target_translation.x - transform.translation.x,
                                            0.0,
                                            target_translation.z - transform.translation.z).normalize();
             
            transform.translation += target_position * 0.01 * time.delta().subsec_millis() as f32;
            position.x = transform.translation.x as i32;
            position.y = transform.translation.y as i32;
            position.z = transform.translation.z as i32;
        }
    }
}

//  fn spawn_box(
//      mut commands: Commands,
//      mut meshes: ResMut<Assets<Mesh>>,
//      mut materials: ResMut<Assets<StandardMaterial>>,
//  ) {
//      commands.spawn_bundle(PbrBundle {
//        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
//        material: materials.add(Color::hex(crate::COLOR_BOX).unwrap().into()),
//        transform: Transform::from_xyz(0.0, 0.5, 0.0),
//        ..Default::default()
//      });
//  }
