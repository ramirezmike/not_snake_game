use crate::game_controller;
use bevy::app::AppExit;
use bevy::app::Events;
use bevy::prelude::*;
use std::collections::HashMap;

pub fn setup_menu(mut commands: Commands, asset_server: Res<AssetServer>) {
    // ui camera
    commands.spawn_bundle(UiCameraBundle::default());

    let width = 350.0;
    let resume_button_entity = commands
        .spawn_bundle(ButtonBundle {
            style: Style {
                size: Size::new(Val::Px(width), Val::Px(65.0)),
                // center button
                margin: Rect::all(Val::Auto),
                // horizontally center child text
                justify_content: JustifyContent::Center,
                // vertically center child text
                align_items: AlignItems::Center,
                position_type: PositionType::Absolute,
                position: Rect {
                    bottom: Val::Percent(20.0),
                    left: Val::Percent(45.0),
                    ..Default::default()
                },
                ..Default::default()
            },
            color: NORMAL_BUTTON.into(),
            ..Default::default()
        })
        .with_children(|parent| {
            parent.spawn_bundle(TextBundle {
                text: Text::with_section(
                    "Resume",
                    TextStyle {
                        font: asset_server.load(crate::FONT),
                        font_size: 40.0,
                        color: Color::rgb(0.9, 0.9, 0.9),
                    },
                    Default::default(),
                ),
                ..Default::default()
            });
        })
        .id();

    let restart_button_entity = commands
        .spawn_bundle(ButtonBundle {
            style: Style {
                size: Size::new(Val::Px(width), Val::Px(65.0)),
                // center button
                margin: Rect::all(Val::Auto),
                // horizontally center child text
                justify_content: JustifyContent::Center,
                // vertically center child text
                align_items: AlignItems::Center,
                position_type: PositionType::Absolute,
                position: Rect {
                    bottom: Val::Percent(15.0),
                    left: Val::Percent(45.0),
                    ..Default::default()
                },
                ..Default::default()
            },
            color: NORMAL_BUTTON.into(),
            ..Default::default()
        })
        .with_children(|parent| {
            parent.spawn_bundle(TextBundle {
                text: Text::with_section(
                    "Restart Level",
                    TextStyle {
                        font: asset_server.load(crate::FONT),
                        font_size: 40.0,
                        color: Color::rgb(0.9, 0.9, 0.9),
                    },
                    Default::default(),
                ),
                ..Default::default()
            });
        })
        .id();

    let main_menu_button_entity = commands
        .spawn_bundle(ButtonBundle {
            style: Style {
                size: Size::new(Val::Px(width), Val::Px(65.0)),
                // center button
                margin: Rect::all(Val::Auto),
                // horizontally center child text
                justify_content: JustifyContent::Center,
                // vertically center child text
                align_items: AlignItems::Center,
                position_type: PositionType::Absolute,
                position: Rect {
                    bottom: Val::Percent(10.0),
                    left: Val::Percent(45.0),
                    ..Default::default()
                },
                ..Default::default()
            },
            color: NORMAL_BUTTON.into(),
            ..Default::default()
        })
        .with_children(|parent| {
            parent.spawn_bundle(TextBundle {
                text: Text::with_section(
                    "Quit to Main Menu",
                    TextStyle {
                        font: asset_server.load(crate::FONT),
                        font_size: 40.0,
                        color: Color::rgb(0.9, 0.9, 0.9),
                    },
                    Default::default(),
                ),
                ..Default::default()
            });
        })
        .id();

    let quit_button_entity = commands
        .spawn_bundle(ButtonBundle {
            style: Style {
                size: Size::new(Val::Px(width), Val::Px(65.0)),
                // center button
                margin: Rect::all(Val::Auto),
                // horizontally center child text
                justify_content: JustifyContent::Center,
                // vertically center child text
                align_items: AlignItems::Center,
                position_type: PositionType::Absolute,
                position: Rect {
                    bottom: Val::Percent(5.0),
                    left: Val::Percent(45.0),
                    ..Default::default()
                },
                ..Default::default()
            },
            color: NORMAL_BUTTON.into(),
            ..Default::default()
        })
        .with_children(|parent| {
            parent.spawn_bundle(TextBundle {
                text: Text::with_section(
                    "Exit Game",
                    TextStyle {
                        font: asset_server.load(crate::FONT),
                        font_size: 40.0,
                        color: Color::rgb(0.9, 0.9, 0.9),
                    },
                    Default::default(),
                ),
                ..Default::default()
            });
        })
        .id();

    commands.insert_resource(PauseMenuData {
        resume_button_entity,
        main_menu_button_entity,
        quit_button_entity,
        restart_button_entity,
        selected: resume_button_entity,
    });
}

pub struct PauseMenuData {
    resume_button_entity: Entity,
    main_menu_button_entity: Entity,
    quit_button_entity: Entity,
    restart_button_entity: Entity,
    selected: Entity,
}

pub fn cleanup_pause_menu(mut commands: Commands, menu_data: Res<PauseMenuData>) {
    commands
        .entity(menu_data.resume_button_entity)
        .despawn_recursive();
    commands
        .entity(menu_data.main_menu_button_entity)
        .despawn_recursive();
    commands
        .entity(menu_data.restart_button_entity)
        .despawn_recursive();
    commands
        .entity(menu_data.quit_button_entity)
        .despawn_recursive();
}

pub fn pause_menu(
    mut state: ResMut<State<crate::AppState>>,
    mut exit: ResMut<Events<AppExit>>,
    mut level: ResMut<crate::level::Level>,
    mut menu_data: ResMut<PauseMenuData>,
    keyboard_input: Res<Input<KeyCode>>,
    mut button_colors: Query<(Entity, &mut UiColor), With<Button>>,
    interaction_query: Query<(Entity, &Interaction), (Changed<Interaction>, With<Button>)>,
    axes: Res<Axis<GamepadAxis>>,
    buttons: Res<Input<GamepadButton>>,
    gamepad: Option<Res<game_controller::GameController>>,
    mut gamepad_buffer: Local<f32>,
    time: Res<Time>,
) {
    *gamepad_buffer += time.delta_seconds();
    let mut selected_button = None;

    let mut next_button = HashMap::new();
    next_button.insert(
        menu_data.resume_button_entity,
        menu_data.restart_button_entity,
    );
    next_button.insert(
        menu_data.restart_button_entity,
        menu_data.main_menu_button_entity,
    );
    next_button.insert(
        menu_data.main_menu_button_entity,
        menu_data.quit_button_entity,
    );
    next_button.insert(menu_data.quit_button_entity, menu_data.resume_button_entity);

    let mut prev_button = HashMap::new();
    prev_button.insert(menu_data.resume_button_entity, menu_data.quit_button_entity);
    prev_button.insert(
        menu_data.restart_button_entity,
        menu_data.resume_button_entity,
    );
    prev_button.insert(
        menu_data.main_menu_button_entity,
        menu_data.restart_button_entity,
    );
    prev_button.insert(
        menu_data.quit_button_entity,
        menu_data.main_menu_button_entity,
    );

    let mut pressed_buttons = game_controller::get_pressed_buttons(&axes, &buttons, gamepad);
    if *gamepad_buffer < 0.25 {
        pressed_buttons = vec![];
    }

    if !pressed_buttons.is_empty() {
        *gamepad_buffer = 0.0;
    }

    // keyboard and gamepad
    if keyboard_input.just_pressed(KeyCode::Return)
        || keyboard_input.just_pressed(KeyCode::Space)
        || pressed_buttons.contains(&game_controller::GameButton::Action)
    {
        selected_button = Some(menu_data.selected);
    }

    if keyboard_input.just_pressed(KeyCode::W)
        || keyboard_input.just_pressed(KeyCode::Up)
        || pressed_buttons.contains(&game_controller::GameButton::Up)
    {
        menu_data.selected = *prev_button.get(&menu_data.selected).unwrap();
    }

    if keyboard_input.just_pressed(KeyCode::S)
        || keyboard_input.just_pressed(KeyCode::Down)
        || pressed_buttons.contains(&game_controller::GameButton::Down)
    {
        menu_data.selected = *next_button.get(&menu_data.selected).unwrap();
    }

    // mouse
    for (button_entity, interaction) in interaction_query.iter() {
        match *interaction {
            Interaction::Clicked => selected_button = Some(button_entity),
            Interaction::Hovered => menu_data.selected = button_entity,
            _ => (),
        }
    }

    if let Some(selected_button) = selected_button {
        if selected_button == menu_data.resume_button_entity {
            state.pop().unwrap();
        }
        if selected_button == menu_data.restart_button_entity {
            state.set(crate::AppState::RestartLevel).unwrap();
        }
        if selected_button == menu_data.main_menu_button_entity {
            level.current_level = 0;
            state.set(crate::AppState::Loading).unwrap();
        }
        if selected_button == menu_data.quit_button_entity {
            exit.send(AppExit);
        }
    }

    for (entity, mut color) in button_colors.iter_mut() {
        if entity == menu_data.selected {
            *color = HOVERED_BUTTON.into();
        } else {
            *color = NORMAL_BUTTON.into();
        }
    }
}

const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
