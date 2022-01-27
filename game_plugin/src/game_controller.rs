use bevy::prelude::*;

pub struct GameController(Gamepad);

pub fn gamepad_connections(
    mut commands: Commands,
    my_gamepad: Option<Res<GameController>>,
    mut gamepad_evr: EventReader<GamepadEvent>,
) {
    for GamepadEvent(id, kind) in gamepad_evr.iter() {
        match kind {
            GamepadEventType::Connected => {
                println!("New gamepad connected with ID: {:?}", id);

                // if we don't have any gamepad yet, use this one
                if my_gamepad.is_none() {
                    commands.insert_resource(GameController(*id));
                }
            }
            GamepadEventType::Disconnected => {
                println!("Lost gamepad connection with ID: {:?}", id);

                // if it's the one we previously associated with the player,
                // disassociate it:
                if let Some(GameController(old_id)) = my_gamepad.as_deref() {
                    if old_id == id {
                        commands.remove_resource::<GameController>();
                    }
                }
            }
            // other events are irrelevant
            _ => {}
        }
    }
}

#[derive(PartialEq)]
pub enum GameButton {
    Up,
    Down,
    Left,
    Right,
    Action,
    Nothing,
    Start,
}

pub fn get_pressed_buttons(
    axes: &Res<Axis<GamepadAxis>>,
    buttons: &Res<Input<GamepadButton>>,
    gamepad: Option<Res<GameController>>,
) -> Vec<GameButton> {
    let mut pressed_buttons = vec![];
    let gamepad = if let Some(gp) = gamepad {
        // a gamepad is connected, we have the id
        gp.0
    } else {
        // no gamepad is connected
        return pressed_buttons;
    };

    // The joysticks are represented using a separate axis for X and Y
    let axis_lx = GamepadAxis(gamepad, GamepadAxisType::LeftStickX);
    let axis_ly = GamepadAxis(gamepad, GamepadAxisType::LeftStickY);

    if let (Some(x), Some(y)) = (axes.get(axis_lx), axes.get(axis_ly)) {
        // combine X and Y into one vector
        let left_stick_pos = Vec2::new(x, y);

        // implement a dead-zone to ignore small inputs
        if left_stick_pos.length() > 0.1 {
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

    let axis_dx = GamepadAxis(gamepad, GamepadAxisType::DPadX);
    let axis_dy = GamepadAxis(gamepad, GamepadAxisType::DPadY);

    if let (Some(x), Some(y)) = (axes.get(axis_dx), axes.get(axis_dy)) {
        // combine X and Y into one vector
        let left_stick_pos = Vec2::new(x, y);

        // implement a dead-zone to ignore small inputs
        if left_stick_pos.length() > 0.1 {
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

    let action_1 = GamepadButton(gamepad, GamepadButtonType::South);
    let action_2 = GamepadButton(gamepad, GamepadButtonType::East);

    if buttons.just_pressed(action_1) || buttons.just_pressed(action_2) {
        pressed_buttons.push(GameButton::Action);
    }

    let start_button = GamepadButton(gamepad, GamepadButtonType::Start);
    if buttons.just_pressed(start_button) {
        pressed_buttons.push(GameButton::Start);
    }

    pressed_buttons
}
