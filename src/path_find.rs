use bevy::prelude::*;
use crate::{level::Level, level::PositionChangeEvent, EntityType, dude::Dude, 
            environment::DisplayText, Position, dude::Enemy, win_flag::WinFlag, GameObject};
use petgraph::{Graph, graph::NodeIndex};
use petgraph::algo::astar;
use bevy_prototype_debug_lines::*; 

pub struct PathFinder {
    indices: Vec::<Vec::<NodeIndex<u32>>>,
    graph: Graph::<(i32, i32, i32), u32>,
    current_path: Option<(u32, Vec<NodeIndex<u32>>)>,
}

impl PathFinder {
    pub fn new(width: usize, length: usize, height: usize) -> Self {
        let mut indices: Vec::<Vec::<NodeIndex<u32>>> = vec![vec![NodeIndex::new(0); length]; width];
        let mut graph = Graph::<(i32, i32, i32), u32>::new();
        for i in 0..width {
            for j in 0..length {
                indices[i][j] = graph.add_node((i as i32, 0, j as i32));
            }
        }

        for i in 0..width {
            for j in 0..length {
                if i > 0 {
                    graph.add_edge(indices[i][j], indices[i - 1][j], 1);
                }
                if i < width - 1 {
                    graph.add_edge(indices[i][j], indices[i + 1][j], 1);
                }
                if j > 0 {
                    graph.add_edge(indices[i][j], indices[i][j - 1], 1);
                }
                if j < length - 1 {
                    graph.add_edge(indices[i][j], indices[i][j + 1], 1);
                }
            }
        }

        PathFinder {
            indices,
            graph,
            current_path: None,
        }
    }

    pub fn update_graph(&mut self, position: &Position, object: &Option::<GameObject>) {
        let (x, y, z) = (position.x as usize, position.y as usize, position.z as usize);
        println!("x: {} y: {} z: {} ", x, y, z);
        if x > 0 {
            self.graph.update_edge(self.indices[x - 1][z], self.indices[x][z], 1);

            if let Some(object) = object {
                match object.entity_type {
                    EntityType::Block => {
                        if let Some(edge) = self.graph.find_edge(self.indices[x - 1][z], self.indices[x][z]) {
                            self.graph.remove_edge(edge);
                        }
                    },
                    _ => ()
                }
            } 
        }
        if x < self.indices.len()- 1 {
            self.graph.update_edge(self.indices[x + 1][z], self.indices[x][z], 1);

            if let Some(object) = object {
                match object.entity_type {
                    EntityType::Block => {
                        if let Some(edge) = self.graph.find_edge(self.indices[x + 1][z], self.indices[x][z]) {
                            self.graph.remove_edge(edge);
                        }
                    },
                    _ => ()
                }
            } 
        }
        if z > 0 {
            self.graph.update_edge(self.indices[x][z - 1], self.indices[x][z], 1);

            if let Some(object) = object {
                match object.entity_type {
                    EntityType::Block => {
                        if let Some(edge) = self.graph.find_edge(self.indices[x][z - 1], self.indices[x][z]) {
                            self.graph.remove_edge(edge);
                        }
                    },
                    _ => ()
                }
            } 
        }
        if z < self.indices[0].len()- 1 {
            self.graph.update_edge(self.indices[x][z + 1], self.indices[x][z], 1);

            if let Some(object) = object {
                match object.entity_type {
                    EntityType::Block => {
                        if let Some(edge) = self.graph.find_edge(self.indices[x][z + 1], self.indices[x][z]) {
                            self.graph.remove_edge(edge);
                        }
                    },
                    _ => ()
                }
            } 
        }
    }

    pub fn update_path(&mut self, start: &Position, goal: &Position) {
        let start_index = self.indices[start.x as usize][start.z as usize];
        let goal_index = self.indices[goal.x as usize][goal.z as usize];
        let path = astar(&self.graph, start_index, |finish| finish == goal_index, |e| *e.weight(), |_| 0);
        if path.is_none() {
//          println!("path: {:?}", path);
//          println!("start: {:?}", start);
//          println!("end: {:?}", goal);
//          println!();
        }
        self.current_path = path;
    }

//  pub fn get_weight(&self, position: &Position) -> Position {
//      let weight = 
//          *self.graph.node_weight(self.indices[position.x as usize][position.z as usize])
//                     .expect("Node doesn't exist");
//      Position { x: weight.0, y: weight.1, z: weight.2 }
//  }

    pub fn get_position(&self, index: NodeIndex<u32>) -> Position {
        let weight = *self.graph.node_weight(index)
                                .expect("Node doesn't exist");
        Position { x: weight.0, y: weight.1, z: weight.2 }
    }

    pub fn get_path(&self) -> (u32, Vec<NodeIndex<u32>>) {
        self.current_path.clone().unwrap_or((0, vec!()))
    }

    pub fn get_edges(&self) -> Vec::<(Position, Position)> {
        self.graph
            .node_indices()
            .flat_map(|n| self.graph
                              .neighbors_directed(n, petgraph::Direction::Outgoing)
                              .map(move |neighbor| (n, neighbor)))
            .map(|(node, neighbor)| (self.graph.node_weight(node).unwrap(), self.graph.node_weight(neighbor).unwrap()))
            .map(|(node, neighbor)| (Position { x: node.0, y: node.1, z: node.2 }, 
                                     Position { x: neighbor.0, y: neighbor.1, z: neighbor.2 }))
            .collect()
    }
}

pub fn show_path(
    path_find: Res<PathFinder>,
    mut platforms: Query<(&mut DisplayText, &Transform)>
) {
    if let Some(path) = &path_find.current_path {
        for (mut display_text, transform) in platforms.iter_mut() {
            let current = path_find.indices[transform.translation.x as usize][transform.translation.z as usize];
            if path.1.contains(&current) {
                display_text.0 = "*".to_string();
                    //format!("{}", path_find.get_weight(&Position::from_vec(transform.translation))).to_string();
            } else {
                display_text.0 = "".to_string();
            }
        }
    }
}

pub fn update_graph(
    mut changes: EventReader<PositionChangeEvent>,
    mut path_finder: ResMut<PathFinder>
) {
    for change in changes.iter() {
        println!("updating graph..{:?}", change);
        path_finder.update_graph(&change.0, &change.1);
    }
}

pub fn draw_edges(
    mut time: Local<f32>,
    timer: Res<Time>,
    path_find: Res<PathFinder>,
    mut lines: ResMut<DebugLines>,
) {
    *time += timer.delta_seconds();
    if *time > 0.2 {
        for (p1, p2) in path_find.get_edges().iter() {
                let start = Vec3::new(0.1 + p1.x as f32, 0.8, p1.z as f32);
                let end = Vec3::new(p2.x as f32, 0.2, p2.z as f32);
                let thickness = 0.01;
                
                lines.line_gradient(start, end, thickness, Color::BLUE, Color::GREEN);
        }

        *time = 0.0;
    }
}

pub fn update_path(
    mut time: Local<f32>,
    timer: Res<Time>,
    mut path_find: ResMut<PathFinder>,
    dude: Query<(&Enemy, &Position)>,
    win_flag: Query<(&Dude, &Position)>,
) {
    *time += timer.delta_seconds();

    if *time > 0.2 {
        if let Ok((_dude, dude_position)) = dude.single() {
            if let Ok((_win_flag, win_flag_position)) = win_flag.single() {
                path_find.update_path(dude_position, win_flag_position);
            }
        }

        *time = 0.0;
    }
}
