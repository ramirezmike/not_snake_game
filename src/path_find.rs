use bevy::prelude::*;
use crate::{level::Level, environment::DisplayText, Position, dude::Enemy, win_flag::WinFlag, };
use petgraph::{Graph, graph::NodeIndex};
use petgraph::algo::astar;

pub struct PathFinder {
    indices: Vec::<Vec::<NodeIndex<u32>>>,
    graph: Graph::<u32, u32>,
    current_path: Option<(u32, Vec<NodeIndex<u32>>)>,
}

impl PathFinder {
    pub fn new(width: usize, length: usize, height: usize) -> Self {
        let mut indices: Vec::<Vec::<NodeIndex<u32>>> = vec![vec![NodeIndex::new(0); length]; width];
        let mut graph = Graph::<u32, u32>::new();
        for i in 0..width {
            for j in 0..length {
                indices[i][j] = graph.add_node(1); // maybe this should be the position?
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

    pub fn update_path(&mut self, start: &Position, goal: &Position) {
        let start_index = self.indices[start.x as usize][start.z as usize];
        let goal_index = self.indices[goal.x as usize][goal.z as usize];
        let path = astar(&self.graph, start_index, |finish| finish == goal_index, |e| *e.weight(), |_| 0);
        self.current_path = path;
    }

    pub fn get_weight(&self, position: &Position) -> u32 {
        *self.graph.node_weight(self.indices[position.x as usize][position.z as usize])
                   .expect("Node doesn't exist")
    }

    pub fn get_position(&self, index: NodeIndex<u32>) -> Position {
        for i in 0..self.indices.len() {
            for j in 0..self.indices[i as usize].len() {
                if index == self.indices[i as usize][j as usize] {
                    return Position { x: i as i32, y: 0, z: j as i32 };
                }
            }
        }

        Position { x: 0, y: 0, z: 0 }
    }

    pub fn get_path(&self) -> (u32, Vec<NodeIndex<u32>>) {
        self.current_path.clone().unwrap_or((0, vec!()))
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

pub fn update_path(
    mut time: Local<f32>,
    timer: Res<Time>,
    mut path_find: ResMut<PathFinder>,
    dude: Query<(&Enemy, &Position)>,
    win_flag: Query<(&WinFlag, &Position)>,
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


