use bevy::prelude::*;
use std::collections::HashMap;

pub struct GameControllerPlugin;
impl Plugin for GameControllerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GameController::default())
           .add_system(gamepad_connections)
           .add_system(store_controller_inputs.label("store_controller_inputs"));
    }
}

#[derive(Default)]
pub struct GameController {
    pub players: Vec<Gamepad>,
    pub pressed: HashMap<usize, Vec<GameButton>>,
    pub just_pressed: HashMap<usize, Vec<GameButton>>,
}

impl GameController {
    pub fn clear_presses(&mut self) {
//      self.pressed = HashMap::<usize, Vec<GameButton>>::new();
//      self.just_pressed = HashMap::<usize, Vec<GameButton>>::new();
    }
}

fn store_controller_inputs(
    axes: Res<Axis<GamepadAxis>>,
    buttons: Res<Input<GamepadButton>>,
    mut controllers: ResMut<GameController>,
) {
    let mut pressed: HashMap<usize, Vec<GameButton>> = HashMap::new();
    let mut just_pressed: HashMap<usize, Vec<GameButton>> = HashMap::new();
    for gamepad in controllers.players.iter() {
        let mut pressed_buttons = vec![];
        let gamepad = *gamepad;

        // The joysticks are represented using a separate axis for X and Y
        let axis_lx = GamepadAxis(gamepad, GamepadAxisType::LeftStickX);
        let axis_ly = GamepadAxis(gamepad, GamepadAxisType::LeftStickY);

        if let (Some(x), Some(y)) = (axes.get(axis_lx), axes.get(axis_ly)) {
            // combine X and Y into one vector
            let left_stick_pos = Vec2::new(x, y);

            // implement a dead-zone to ignore small inputs
            if left_stick_pos.length() > 0.2 {
                // do something with the position of the left stick
                if x > 0.0 {
                    pressed_buttons.push(GameButton::Right);
                }
                if x < 0.0 {
                    pressed_buttons.push(GameButton::Left);
                }
                if y > 0.0 {
                    pressed_buttons.push(GameButton::Up);
                }
                if y < 0.0 {
                    pressed_buttons.push(GameButton::Down);
                }
            }
        }

        // The joysticks are represented using a separate axis for X and Y
        //      let axis_rx = GamepadAxis(gamepad, GamepadAxisType::RightStickX);
        //      let axis_ry = GamepadAxis(gamepad, GamepadAxisType::RightStickY);

        //      if let (Some(x), Some(y)) = (axes.get(axis_rx), axes.get(axis_ry)) {
        //          // combine X and Y into one vector
        //          let left_stick_pos = Vec2::new(x, y);

        //          // implement a dead-zone to ignore small inputs
        //          if left_stick_pos.length() > 0.1 {
        //              // do something with the position of the left stick
        //              if x > 0.0 {
        //                  pressed_buttons.push(GameButton::ActionRight);
        //              }
        //              if x < 0.0 {
        //                  pressed_buttons.push(GameButton::ActionLeft);
        //              }
        //              if y > 0.0 {
        //                  pressed_buttons.push(GameButton::ActionUp);
        //              }
        //              if y < 0.0 {
        //                  pressed_buttons.push(GameButton::ActionDown);
        //              }
        //          }
        //      }

        let axis_dx = GamepadAxis(gamepad, GamepadAxisType::DPadX);
        let axis_dy = GamepadAxis(gamepad, GamepadAxisType::DPadY);

        if let (Some(x), Some(y)) = (axes.get(axis_dx), axes.get(axis_dy)) {
            // combine X and Y into one vector
            let left_stick_pos = Vec2::new(x, y);

            // implement a dead-zone to ignore small inputs
            if left_stick_pos.length() > 0.2 {
                // do something with the position of the left stick
                if x > 0.0 {
                    pressed_buttons.push(GameButton::Right);
                }
                if x < 0.0 {
                    pressed_buttons.push(GameButton::Left);
                }
                if y > 0.0 {
                    pressed_buttons.push(GameButton::Up);
                }
                if y < 0.0 {
                    pressed_buttons.push(GameButton::Down);
                }
            }
        }

        let dpad_up = GamepadButton(gamepad, GamepadButtonType::DPadUp);
        let dpad_down = GamepadButton(gamepad, GamepadButtonType::DPadDown);
        let dpad_left = GamepadButton(gamepad, GamepadButtonType::DPadLeft);
        let dpad_right = GamepadButton(gamepad, GamepadButtonType::DPadRight);

        if buttons.pressed(dpad_up) {
            pressed_buttons.push(GameButton::Up);
        }

        if buttons.pressed(dpad_down) {
            pressed_buttons.push(GameButton::Down);
        }

        if buttons.pressed(dpad_left) {
            pressed_buttons.push(GameButton::Left);
        }

        if buttons.pressed(dpad_right) {
            pressed_buttons.push(GameButton::Right);
        }

        let south = GamepadButton(gamepad, GamepadButtonType::South);
        let east = GamepadButton(gamepad, GamepadButtonType::East);
        let west = GamepadButton(gamepad, GamepadButtonType::West);
        let north = GamepadButton(gamepad, GamepadButtonType::North);

        if buttons.pressed(south) {
            pressed_buttons.push(GameButton::ActionDown);
        }
        if buttons.pressed(north) {
            pressed_buttons.push(GameButton::ActionUp);
        }
        if buttons.pressed(west) {
            pressed_buttons.push(GameButton::ActionLeft);
        }
        if buttons.pressed(east) {
            pressed_buttons.push(GameButton::ActionRight);
        }

        let start_button = GamepadButton(gamepad, GamepadButtonType::Start);
        if buttons.pressed(start_button) {
            pressed_buttons.push(GameButton::Start);
        }

        let game_id = gamepad.0;
        let mut just_pressed_buttons = pressed_buttons.clone();
        just_pressed_buttons.retain(|button| {
            !controllers.pressed.contains_key(&game_id)
                || !controllers.pressed[&game_id].contains(button)
        });

        pressed.insert(game_id, pressed_buttons);
        just_pressed.insert(game_id, just_pressed_buttons);
    }

    controllers.pressed = pressed;
    controllers.just_pressed = just_pressed;
}

pub fn gamepad_connections(
    mut gamepad_evr: EventReader<GamepadEvent>,
    mut controllers: ResMut<GameController>,
) {
    for GamepadEvent(id, kind) in gamepad_evr.iter() {
        if *kind == GamepadEventType::Connected {
            println!("New gamepad connected with ID: {:?}", id);
            controllers.players.push(*id);
        }
    }
}

pub fn clear_presses(mut controllers: ResMut<GameController>) {
//    controllers.clear_presses();
}

#[derive(PartialEq, Debug, Copy, Clone, Eq, PartialOrd, Ord)]
pub enum GameButton {
    Up,
    Down,
    Left,
    Right,
    ActionUp,
    ActionLeft,
    ActionRight,
    ActionDown,
    Start,
    Other,
}
