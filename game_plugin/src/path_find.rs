use bevy::prelude::*;
use crate::{level::Level, level::PositionChangeEvent, EntityType, dude::Dude, camera::MainCamera,
            environment::LevelReady, Position, snake, food::Food,};
use petgraph::{Graph, graph::NodeIndex, graph::EdgeIndex};
use petgraph::algo::astar;
use bevy_prototype_debug_lines::*; 

/*
    everything should start with edges pointing down
    then, every block should modify the edges of the space above it
    work from down to up. Each block should ensure the edges going
    into it are removed and then that the space above it can move 
    into each cardinal direction space around it. That is, each
    block prevents moving into but enables moving "out of" above
    it
*/
pub struct PathFinder {
    indices: Vec::<Vec::<Vec::<NodeIndex<u32>>>>,
    graph: Graph::<(i32, i32, i32), u32>,
}

impl PathFinder {
    pub fn load_level(&mut self, level: &Level) {
        let length = level.length();
        let width = level.width();
        let height = level.height();
        let mut indices: Vec::<Vec::<Vec::<NodeIndex<u32>>>> = vec![vec![vec![NodeIndex::new(0); length]; height]; width];
        let mut graph = Graph::<(i32, i32, i32), u32>::new();
        for x in 0..width {
            for y in 0..height {
                for z in 0..length {
                    indices[x][y][z] = graph.add_node((x as i32, y as i32, z as i32));
                }
            }
        }

        // set everything to connect to the node underneath it
        for x in 0..width {
            for y in 0..height {
                for z in 0..length {
                    if y > 0 {
                        graph.add_edge(indices[x][y][z], indices[x][y - 1][z], 1);
                    }
                }
            }
        }
        self.indices = indices;
        self.graph = graph;
    }

    pub fn new() -> Self {
        PathFinder {
            indices: vec!(),
            graph: Graph::<(i32, i32, i32), u32>::new(),
        }
    }

    fn get_edge(&self, position_a: &Position, position_b: &Position, level: &Res<Level>) -> Option::<EdgeIndex<u32>> {
        if level.is_inbounds(position_a.x, position_a.y, position_a.z) 
        && level.is_inbounds(position_b.x, position_b.y, position_b.z) {
            self.graph.find_edge(self.indices[position_a.x as usize][position_a.y as usize][position_a.z as usize], 
                                 self.indices[position_b.x as usize][position_b.y as usize][position_b.z as usize])

        } else {
            None
        }
    }

    // this should just get called for everything 
    fn update_position_in_graph(&mut self, position: &Position, level: &Res<Level>) {
        if !level.is_inbounds(position.x, position.y, position.z) {
            return;
        }

        let (x, y, z) = (position.x as usize, position.y as usize, position.z as usize);

        // remove everything entering into this spot?
        // above
        if let Some(edge) = self.get_edge(&Position { x: position.x, y: position.y + 1, z: position.z }, &position, &level) {
            self.graph.remove_edge(edge);
        }
        // below
        if let Some(edge) = self.get_edge(&Position { x: position.x, y: position.y - 1, z: position.z }, &position, &level) {
            self.graph.remove_edge(edge);
        }
        // up 
        if let Some(edge) = self.get_edge(&Position { x: position.x + 1, y: position.y, z: position.z }, &position, &level) {
            self.graph.remove_edge(edge);
        }
        // down
        if let Some(edge) = self.get_edge(&Position { x: position.x - 1, y: position.y, z: position.z }, &position, &level) {
            self.graph.remove_edge(edge);
        }
        // left
        if let Some(edge) = self.get_edge(&Position { x: position.x, y: position.y, z: position.z - 1 }, &position, &level) {
            self.graph.remove_edge(edge);
        }
        // right
        if let Some(edge) = self.get_edge(&Position { x: position.x, y: position.y, z: position.z + 1 }, &position, &level) {
            self.graph.remove_edge(edge);
        }

        let weight = match level.get(x as i32, y as i32 - 1, z as i32) {
                        Some(game_object) => {
                            match game_object.entity_type {
                                EntityType::EnemyHead => 99,
                                EntityType::Enemy => 3, // try to prevent snakes from climbing on themselves
                                _ => 1
                            }
                        },
                        _ => {
                            if level.is_inbounds(x as i32, y as i32 - 1, z as i32) {
                                2 // try to prevent snakes from floating over empty spaces
                            } else {
                                1 // the position must be the bottom of the map so just return 1
                            }
                        }
                    };

        let mut handle_general_case = || {
            if level.is_position_enterable(*position) || level.is_position_entity(position) {
                if level.is_position_standable(*position) {
                    // up
                    if level.is_inbounds(position.x + 1, position.y, position.z)
                    && (level.is_enterable(position.x + 1, position.y, position.z) 
                        || level.is_entity(position.x + 1, position.y, position.z)) {
                        self.graph.update_edge(self.indices[x + 1][y][z], self.indices[x][y][z], weight);
                    }
                    // down
                    if level.is_inbounds(position.x - 1, position.y, position.z) 
                    && (level.is_enterable(position.x - 1, position.y, position.z) 
                        || level.is_entity(position.x - 1, position.y, position.z)) {
                        self.graph.update_edge(self.indices[x - 1][y][z], self.indices[x][y][z], weight);
                    }
                    // left  
                    if level.is_inbounds(position.x, position.y, position.z - 1) 
                    && (level.is_enterable(position.x, position.y, position.z - 1) 
                        || level.is_entity(position.x, position.y, position.z - 1)) {
                        self.graph.update_edge(self.indices[x][y][z - 1], self.indices[x][y][z], weight);
                    }
                    // right
                    if level.is_inbounds(position.x, position.y, position.z + 1) 
                    && (level.is_enterable(position.x, position.y, position.z + 1) 
                        || level.is_entity(position.x, position.y, position.z + 1)) {
                        self.graph.update_edge(self.indices[x][y][z + 1], self.indices[x][y][z], weight);
                    }

                    // need to add this if enemy or something
                    // Below
                    if level.is_inbounds(position.x, position.y - 1, position.z) 
                    && (level.is_type(position.x, position.y - 1, position.z, Some(EntityType::Enemy)) 
                     || level.is_type(position.x, position.y - 1, position.z, Some(EntityType::EnemyHead))) { 
                        self.graph.update_edge(self.indices[x][y - 1][z], self.indices[x][y][z], weight);
                    }
                } else {
                    let up_is_standable = level.is_standable(position.x + 1, position.y, position.z);
                    let down_is_standable = level.is_standable(position.x - 1, position.y, position.z);
                    let left_is_standable = level.is_standable(position.x, position.y, position.z - 1);
                    let right_is_standable = level.is_standable(position.x, position.y, position.z + 1);

                    // Below
                    if level.is_inbounds(position.x, position.y - 1, position.z) 
                    && (up_is_standable || down_is_standable || right_is_standable || left_is_standable) { 
                        self.graph.update_edge(self.indices[x][y - 1][z], self.indices[x][y][z], weight);
                    }

                    // need to make connections to up/down/left/right if any of those are standable

                    // up
                    if level.is_inbounds(position.x + 1, position.y, position.z)
                    && up_is_standable 
                    && (level.is_enterable(position.x + 1, position.y, position.z) 
                        || level.is_entity(position.x + 1, position.y, position.z)) {
                        self.graph.update_edge(self.indices[x + 1][y][z], self.indices[x][y][z], weight);
                    }
                    // down
                    if level.is_inbounds(position.x - 1, position.y, position.z) 
                    && down_is_standable 
                    && (level.is_enterable(position.x - 1, position.y, position.z) 
                        || level.is_entity(position.x - 1, position.y, position.z)) {
                        self.graph.update_edge(self.indices[x - 1][y][z], self.indices[x][y][z], weight);
                    }
                    // left  
                    if level.is_inbounds(position.x, position.y, position.z - 1) 
                    && left_is_standable 
                    && (level.is_enterable(position.x, position.y, position.z - 1) 
                        || level.is_entity(position.x, position.y, position.z - 1)) {
                        self.graph.update_edge(self.indices[x][y][z - 1], self.indices[x][y][z], weight);
                    }
                    // right
                    if level.is_inbounds(position.x, position.y, position.z + 1) 
                    && right_is_standable 
                    && (level.is_enterable(position.x, position.y, position.z + 1) 
                        || level.is_entity(position.x, position.y, position.z + 1)) {
                        self.graph.update_edge(self.indices[x][y][z + 1], self.indices[x][y][z], weight);
                    }
                }

                // Above
                if level.is_enterable(position.x, position.y + 1, position.z) 
                || level.is_entity(position.x, position.y + 1, position.z) {
                    self.graph.update_edge(self.indices[x][y + 1][z], self.indices[x][y][z], weight);
                }
            }
        };
        
        if let Some(object) = level.get_with_position(*position) {
            match object.entity_type {
                EntityType::Dude => {
                    // up
                    if level.is_inbounds(position.x + 1, position.y, position.z) {
                        self.graph.update_edge(self.indices[x + 1][y][z], self.indices[x][y][z], weight);
                    }
                    // down
                    if level.is_inbounds(position.x - 1, position.y, position.z) {
                        self.graph.update_edge(self.indices[x - 1][y][z], self.indices[x][y][z], weight);
                    }
                    // left 
                    if level.is_inbounds(position.x, position.y, position.z - 1) {
                        self.graph.update_edge(self.indices[x][y][z - 1], self.indices[x][y][z], weight);
                    }
                    // right
                    if level.is_inbounds(position.x, position.y, position.z + 1) {
                        self.graph.update_edge(self.indices[x][y][z + 1], self.indices[x][y][z], weight);
                    }
                    // above
                    if level.is_inbounds(position.x, position.y + 1, position.z) {
                        self.graph.update_edge(self.indices[x][y + 1][z], self.indices[x][y][z], weight);
                    }
                    // below
                    if level.is_inbounds(position.x, position.y - 1, position.z) {
                        self.graph.update_edge(self.indices[x][y - 1][z], self.indices[x][y][z], weight);
                    }
                },
                EntityType::Block | EntityType::Enemy | EntityType::EnemyHead => (),
                _ => handle_general_case()
            }
        } else {
            handle_general_case()
        }
    }

    pub fn update_path(&mut self,       
        claimed_nodes: &Vec::<NodeIndex<u32>>,
        requesting_entity: Entity, // probably a snake
        level: &Res<Level>, 
        start: &Position, 
        goal: &Position,
        kill_snake_event_writer: &mut EventWriter::<snake::KillSnakeEvent>,
    ) -> Option<(u32, Vec<NodeIndex<u32>>)> {
        let start_index = self.indices[start.x as usize][start.y as usize][start.z as usize];
        let goal_index = self.indices[goal.x as usize][goal.y as usize][goal.z as usize];
        let mut path = astar(&self.graph, start_index, 
                             |finish| finish == goal_index, 
                             |e| *e.weight(), 
                             |n| if claimed_nodes.iter().any(|claimed| *claimed == n) { 99 } else { 0 });

        let mut attempts = 0;
        let max_attempts = 10;
        // just pick somewhere randomly
        while path.is_none() && attempts < max_attempts {
            println!("Trying to find new random spot... {}", attempts);
            let random_goal = level.get_random_standable();
            let goal_index = self.indices[random_goal.x as usize][random_goal.y as usize][random_goal.z as usize];
            path = astar(&self.graph, start_index, 
                         |finish| finish == goal_index, 
                         |e| *e.weight(), 
                         |n| if claimed_nodes.iter().any(|claimed| *claimed == n) { 99 } else { 0 });
            attempts += 1; 

            if attempts >= max_attempts {
                println!("Sending kill event");
                // killing the snake since it's probably stuck
                //kill_snake_event_writer.send(snake::KillSnakeEvent(requesting_entity));
            }
        }

        path
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
    keyboard_input: Res<Input<KeyCode>>,
    path_find: Res<PathFinder>,
    snakes: Query<&snake::Enemy>,
    level: Res<Level>,
    mut should_draw: Local<bool>,
    mut b: Local<bool>,
    mut lines: ResMut<DebugLines>,
) {
    if keyboard_input.just_pressed(KeyCode::I) {
        *should_draw = !*should_draw;
    }
    
    if *should_draw {
        *b = !*b;
        for snake in snakes.iter() {
            if let Some(path) = &snake.current_path {
                for x in 0..level.width() {
                    for y in 0..level.height() {
                        for z in 0..level.length() {
                            let current = path_find.indices[x][y][z];
                            if path.1.contains(&current) {
                                let start = Vec3::new(0.1 + x as f32, y as f32, z as f32);
                                let end = Vec3::new(x as f32, 1.0 + y as f32, z as f32);
                                let thickness = 0.01;
                                if *b {
                                    lines.line_gradient(start, end, thickness, Color::WHITE, Color::RED);
                                } else {
                                    lines.line_gradient(start, end, thickness, Color::WHITE, Color::YELLOW);
                                }
                            } 
                        }
                    }
                }
            }
        }
    }
}

pub fn update_graph(
    mut changes: EventReader<PositionChangeEvent>,
    mut path_finder: ResMut<PathFinder>,
    mut initial_iteration_completed: Local<bool>,
    level: Res<Level>,
) {
/*
    if !*initial_iteration_completed || changes.iter().count() > 0 {
        *initial_iteration_completed = true; 
        for x in 0..level.width {
            for y in 0..level.height{
                for z in 0..level.length {
                    let p = Position { x: x as i32, y: y as i32, z: z as i32 };
                    path_finder.update_position_in_graph(&p, &level);
                }
            }
        }
    }
*/
}

pub fn draw_edges(
    mut time: Local<f32>,
    keyboard_input: Res<Input<KeyCode>>,
    mut should_draw: Local<bool>,
    timer: Res<Time>,
    path_find: Res<PathFinder>,
    mut lines: ResMut<DebugLines>,
) {

    if keyboard_input.just_pressed(KeyCode::O) {
        *should_draw = !*should_draw;
        *time = 0.0;
    }
    
    if *should_draw {
        *time += timer.delta_seconds();
        if *time > 0.2 {
            for (p1, p2) in path_find.get_edges().iter() {
                    let start = Vec3::new(0.1 + p1.x as f32, 0.8 + p1.y as f32, p1.z as f32);
                    let end = Vec3::new(p2.x as f32, 0.2 + p2.y as f32, p2.z as f32);

//                    let end = Vec3::new(p2.x as f32, 0.2 + p2.y as f32, p2.z as f32);
//                    let start = Vec3::new(end.x, 0.8 + end.y as f32, end.z);
                    let thickness = 0.01;
                    
                    //if !(p1.x == p2.x && p1.z == p2.z && p1.y > p2.y) {
                        let mut blue = Color::BLUE;
                        blue.set_r(p1.y as f32 * 0.1);
                        lines.line_gradient(start, end, thickness, Color::GREEN, blue);
                   // }
            }

            *time = 0.0;
        }
    }
}

pub fn update_path(
    mut time: Local<f32>,
    level: Res<Level>,
    timer: Res<Time>,
    mut path_find: ResMut<PathFinder>,
    mut snake: Query<(Entity, &mut snake::Enemy, &Position, &Transform), Without<Dude>>,
    dude: Query<(&Transform, &Position), (With<Dude>, Without<snake::Enemy>)>,
    food: Query<(&Position, &Transform), With<Food>>,
    mut kill_snake_event_writer: EventWriter::<snake::KillSnakeEvent>,
    level_ready: Res<LevelReady>,
) {
    if !level_ready.0 {
        return; 
    }
    *time += timer.delta_seconds();

    if *time > 0.0 {
        for x in 0..level.width() {
            for y in 0..level.height() {
                for z in 0..level.length() {
                    let p = Position { x: x as i32, y: y as i32, z: z as i32 };
                    path_find.update_position_in_graph(&p, &level);
                }
            }
        }

        let snake_speed = level.snake_speed();
        let (seek_food, seek_dude, mut seek_random) 
            = match level.snake_target() {
                  snake::SnakeTarget::Normal => (true, true, true),
                  snake::SnakeTarget::OnlyFood => (true, false, false),
                  snake::SnakeTarget::OnlyRandom => (false, false, true),
                  snake::SnakeTarget::OnlyDude => (false, true, false),
              };

        let mut claimed_nodes = vec!();
        let mut claimed_targets = vec!();
        for (entity, mut snake, snake_position, snake_transform) in snake.iter_mut() {
            // try to stop snakes from moving backward
            let first_body = snake.get_first_body();
            claimed_nodes.push(path_find.indices[first_body.x as usize][first_body.y as usize][first_body.z as usize]); 

            if !snake.is_dead {
                if let Some((_, current_path)) = &snake.current_path {
                    let is_still_valid = current_path
                                           .iter().skip(1)
                                           .zip(current_path.iter().skip(2))
                                           .all(|(current, next)| path_find.graph.contains_edge(*current, *next));
                    let path_contains_snake_position =
                                         current_path
                                           .iter()
                                           .any(|p| path_find.get_position(*p) == *snake_position);

                    if is_still_valid && current_path.len() > 2 && path_contains_snake_position {
//                      println!("Path valid {} {}", is_still_valid, current_path.len());
//                      println!("First: {:?} Snake: {:?} Pos{:?}", 
//                                  path_find.get_position(current_path[0]), 
//                                  snake_position,
//                                  path_contains_snake_position);
                        continue;
                    } 
//                  println!("Path no longer valid {} {} {}", is_still_valid, current_path.len(), path_contains_snake_position);
                } 
                snake.current_path = None;
                snake.speed = snake_speed;

                if seek_dude {
                    if let Ok((dude_transform, dude_position)) = dude.single() {
                        if claimed_targets.iter().any(|x| path_find.get_position(*x) == *dude_position) {
                            // dude is claimed by another snake, so just seek random for a step
                            seek_random = true;
                        }

                        if (!seek_food && !seek_random)
                        || dude_transform.translation.distance(snake_transform.translation) <= 1.5 {
                            snake.speed -= 0.1;
                            snake.current_path = path_find.update_path(&claimed_nodes, entity, &level, snake_position, 
                                                                       dude_position, &mut kill_snake_event_writer);
                        }
                    }
                }

                // TODO: this needs to be rewritten because snakes can get stuck 
                //       between two foods that are the same distance apart
                if seek_food && snake.current_path.is_none() {
                    let mut closest_food: Option::<(&Position, &Transform)> = None;
                    for (food_position, food_transform) in food.iter() {
                        if !claimed_targets.iter().any(|x| path_find.get_position(*x) == *food_position) {
                            if closest_food.is_none() 
                            || closest_food.unwrap().1.translation.distance(snake_transform.translation)
                               > food_transform.translation.distance(snake_transform.translation) {
                                closest_food = Some((food_position, food_transform));
                            } 
                        } else {
                            // go to next food but potentially seek random to fallback on if no food
                            // is available
                            seek_random = true;
                        }
                    }

                    if let Some((food_position, _food_transform)) = closest_food {
                        snake.current_path = path_find.update_path(&claimed_nodes, entity, &level, snake_position, 
                                                                   food_position, &mut kill_snake_event_writer);
                    }
                }

                if seek_random && snake.current_path.is_none() {
                    let random_goal = level.get_random_standable();

                    println!("looking for random");
                    snake.current_path = path_find.update_path(&claimed_nodes, entity, &level, 
                                                               snake_position, &random_goal, 
                                                               &mut kill_snake_event_writer);
                }

                // TODO: might be better to just check if the snake can move in at least
                //       one unit in a "forward" direction?
                if snake.current_path.is_none() {
                    snake.is_dead = true;
                } else {
                    let current_path = snake.current_path.as_ref().unwrap().1.iter().cloned();
                    if current_path.len() == 3 {
                        println!("Something got claimed!");
                        claimed_targets.push(current_path.clone().last().unwrap()); 
                    }
                    claimed_nodes.extend(current_path);
                }
            }
        }

        *time = 0.0;
    }
}
