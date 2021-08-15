use bevy::prelude::*;
use bevy::app::Events;
use bevy::app::AppExit;
use std::collections::HashMap;
use crate::{game_controller, level, score};

pub struct BylineText;
pub struct MenuButton;
pub fn setup_menu(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    button_materials: Res<ButtonMaterials>,
) {
    // ui camera
    commands.spawn_bundle(UiCameraBundle::default());

    let byline_text_entity = 
    commands
        .spawn_bundle(TextBundle {
            style: Style {
                align_self: AlignSelf::FlexEnd,
                position_type: PositionType::Absolute,
                position: Rect {
                    bottom: Val::Px(5.0),
                    left: Val::Px(15.0),
                    ..Default::default()
                },
                size: Size {
                    //width: Val::Px(200.0),
                    ..Default::default()
                },
                ..Default::default()
            },
            text: Text::with_section(
                "".to_string(),
                TextStyle {
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    font_size: 40.0,
                    color: Color::rgba(0.8, 0.8, 0.8, -0.4),
                },
                TextAlignment {
                    ..Default::default()
                }
            ),
            ..Default::default()
        })
        .insert(BylineText)
        .id();
    
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
            visible: Visible {
                is_visible: false,
                is_transparent: false,
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
                visible: Visible {
                    is_visible: false,
                    is_transparent: false,
                },
                ..Default::default()
            }).insert(MenuButton);
        })
        .insert(MenuButton)
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
            visible: Visible {
                is_visible: false,
                is_transparent: false,
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
                visible: Visible {
                    is_visible: false,
                    is_transparent: false,
                },
                ..Default::default()
            }).insert(MenuButton);
        })
        .insert(MenuButton)
        .id();

    commands.insert_resource(
        MenuData { 
            start_button_entity, 
            quit_button_entity, 
            byline_text_entity,
            selected: start_button_entity
        });
}

pub struct MenuData {
    start_button_entity: Entity,
    quit_button_entity: Entity,
    byline_text_entity: Entity, 
    selected: Entity,
}

pub fn cleanup_menu(mut commands: Commands, menu_data: Res<MenuData>) {
    commands.entity(menu_data.start_button_entity).despawn_recursive();
    commands.entity(menu_data.quit_button_entity).despawn_recursive();
    commands.entity(menu_data.byline_text_entity).despawn_recursive();
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
    mut bylines: Query<&mut Text, With<BylineText>>,
    mut menu_buttons: Query<&mut Visible, With<MenuButton>>,
    time: Res<Time>,
    mut score: ResMut<score::Score>,
) {
    *gamepad_buffer += time.delta_seconds();
    let mut selected_button = None;

    for mut byline in bylines.iter_mut() {
        let a = byline.sections[0].style.color.a();
        if a < 1.0 {
            byline.sections[0].style.color.set_a(a + time.delta_seconds() / 5.0);
        } 

        if a > 0.8 {
            // make buttons visible now that byline is visible enough
            for mut visible in menu_buttons.iter_mut() {
                visible.is_visible = true; 
            }
        }
    }

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
            *score = score::Score::new();
            state.set(crate::AppState::LevelTitle).unwrap();
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
