use bevy::prelude::*;
use bevy::app::Events;
use bevy::app::AppExit;
use std::collections::HashMap;
use crate::game_controller;

pub fn setup_menu(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    button_materials: Res<ButtonMaterials>,
) {
    // ui camera
    commands.spawn_bundle(UiCameraBundle::default());

    let title_text_entity = 
    commands
        .spawn_bundle(TextBundle {
            style: Style {
                // center button
                margin: Rect::all(Val::Auto),
                // horizontally center child text
                justify_content: JustifyContent::Center,
                // vertically center child text
                align_items: AlignItems::Center,
                position_type: PositionType::Absolute,
                position: Rect {
                    top: Val::Percent(0.0),
                    ..Default::default()
                },
                ..Default::default()
            },
            // Use the `Text::with_section` constructor
            text: Text::with_section(
                // Accepts a `String` or any type that converts into a `String`, such as `&str`
                "A Game \nby Michael Ramirez",
                TextStyle {
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    font_size: 150.0,
                    color: Color::WHITE,
                },
                // Note: You can use `Default::default()` in place of the `TextAlignment`
                TextAlignment {
                    ..Default::default()
                },
            ),
            ..Default::default()
        }).id();
    let start_button_entity = commands
        .spawn_bundle(ButtonBundle {
            style: Style {
                size: Size::new(Val::Px(150.0), Val::Px(65.0)),
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
            material: button_materials.normal.clone(),
            ..Default::default()
        })
        .with_children(|parent| {
            parent.spawn_bundle(TextBundle {
                text: Text::with_section(
                    "Start",
                    TextStyle {
                        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
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
                size: Size::new(Val::Px(150.0), Val::Px(65.0)),
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
            material: button_materials.normal.clone(),
            ..Default::default()
        })
        .with_children(|parent| {
            parent.spawn_bundle(TextBundle {
                text: Text::with_section(
                    "Quit",
                    TextStyle {
                        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                        font_size: 40.0,
                        color: Color::rgb(0.9, 0.9, 0.9),
                    },
                    Default::default(),
                ),
                ..Default::default()
            });
        })
        .id();

    commands.insert_resource(MenuData { start_button_entity , title_text_entity, quit_button_entity, selected: start_button_entity });
}

pub struct MenuData {
    start_button_entity: Entity,
    title_text_entity: Entity,
    quit_button_entity: Entity,
    selected: Entity,
}

pub fn cleanup_menu(mut commands: Commands, menu_data: Res<MenuData>) {
    commands.entity(menu_data.start_button_entity).despawn_recursive();
    commands.entity(menu_data.title_text_entity).despawn_recursive();
    commands.entity(menu_data.quit_button_entity).despawn_recursive();
}

pub fn menu(
    mut state: ResMut<State<crate::AppState>>,
    mut exit: ResMut<Events<AppExit>>,
    mut menu_data: ResMut<MenuData>,
    keyboard_input: Res<Input<KeyCode>>,
    button_materials: Res<ButtonMaterials>,
    mut button_colors: Query<(Entity, &mut Handle<ColorMaterial>), With<Button>>,
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
    next_button.insert(menu_data.start_button_entity, menu_data.quit_button_entity);
    next_button.insert(menu_data.quit_button_entity, menu_data.start_button_entity);

    let mut prev_button = HashMap::new();
    prev_button.insert(menu_data.start_button_entity, menu_data.quit_button_entity);
    prev_button.insert(menu_data.quit_button_entity, menu_data.start_button_entity);

    let mut pressed_buttons = game_controller::get_pressed_buttons(&axes, &buttons, gamepad);
    if *gamepad_buffer < 0.25 {
        pressed_buttons = vec!(); 
    }

    if !pressed_buttons.is_empty() {
        *gamepad_buffer = 0.0;
    }


    // keyboard and gamepad
    if keyboard_input.just_pressed(KeyCode::Return) || keyboard_input.just_pressed(KeyCode::Space)
    || pressed_buttons.contains(&game_controller::GameButton::Action){
        selected_button = Some(menu_data.selected); 
    }

    if keyboard_input.just_pressed(KeyCode::W) || keyboard_input.just_pressed(KeyCode::Up) 
    || pressed_buttons.contains(&game_controller::GameButton::Up) {
        menu_data.selected = *prev_button.get(&menu_data.selected).unwrap();
    }

    if keyboard_input.just_pressed(KeyCode::S) || keyboard_input.just_pressed(KeyCode::Down) 
    || pressed_buttons.contains(&game_controller::GameButton::Down) {
        menu_data.selected = *next_button.get(&menu_data.selected).unwrap();
    }

    // mouse
    for (button_entity, interaction) in interaction_query.iter() {
        match *interaction {
            Interaction::Clicked => selected_button = Some(button_entity),
            Interaction::Hovered => menu_data.selected = button_entity,
            _ => ()
        }
    }

    if let Some(selected_button) = selected_button {
        if selected_button == menu_data.start_button_entity {
            state.set(crate::AppState::Loading).unwrap();
        }
        if selected_button == menu_data.quit_button_entity {
            exit.send(AppExit);
        }
    }

    for (entity, mut color) in button_colors.iter_mut() {
        if entity == menu_data.selected {
            *color = button_materials.hovered.clone();
        } else {
            *color = button_materials.normal.clone();
        }
    }
}

pub struct ButtonMaterials {
    normal: Handle<ColorMaterial>,
    hovered: Handle<ColorMaterial>,
    pressed: Handle<ColorMaterial>,
}

impl FromWorld for ButtonMaterials {
    fn from_world(world: &mut World) -> Self {
        let mut materials = world.get_resource_mut::<Assets<ColorMaterial>>().unwrap();
        ButtonMaterials {
            normal: materials.add(Color::rgb(0.15, 0.15, 0.15).into()),
            hovered: materials.add(Color::rgb(0.25, 0.25, 0.25).into()),
            pressed: materials.add(Color::rgb(0.35, 0.75, 0.35).into()),
        }
    }
}
