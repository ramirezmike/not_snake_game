use bevy::prelude::*;
use crate::{food::FoodEatenEvent, Dude, sounds, level, level_over, game_controller};

pub struct ContinueText;
pub struct Score {
    pub total: usize,
    pub total_bonus: usize,
    pub current_level: usize,
    pub current_level_bonus: usize,
}

impl Score {
    pub fn new() -> Self {
        Score {
            total: 0,
            total_bonus: 0,
            current_level: 0,
            current_level_bonus: 0,
        }
    }
}

pub fn handle_food_eaten(
    mut score: ResMut<Score>,
    mut food_eaten_event_reader: EventReader<FoodEatenEvent>,
    dude: Query<Entity, With<Dude>>,
    mut sound_writer: EventWriter<sounds::SoundEvent>,
) {
    for eater in food_eaten_event_reader.iter() {
        if let Ok(_) = dude.get(eater.0) {
            if eater.1 {
                score.current_level_bonus += 1;
                sound_writer.send(sounds::SoundEvent(sounds::Sounds::Pickup));
            } else {
                score.current_level += 1;
                sound_writer.send(sounds::SoundEvent(sounds::Sounds::Pickup));
            }
        }
    }
}

pub fn setup_score_screen(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    commands.spawn_bundle(UiCameraBundle::default())
            .insert(level_over::LevelOverText {});
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
                    top: Val::Percent(30.0),
                    left: Val::Percent(10.0),
                    ..Default::default()
                },
                ..Default::default()
            },
            // Use the `Text::with_section` constructor
            text: Text::with_section(
                // Accepts a `String` or any type that converts into a `String`, such as `&str`
                "",
                TextStyle {
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    font_size: 500.0,
                    color: Color::WHITE,
                },
                // Note: You can use `Default::default()` in place of the `TextAlignment`
                TextAlignment {
                    ..Default::default()
                },
            ),
            ..Default::default()
        })
        .insert(level_over::LevelOverText {});

        commands
            .spawn_bundle(TextBundle {
                style: Style {
                    align_self: AlignSelf::FlexEnd,
                    position_type: PositionType::Absolute,
                    position: Rect {
                        bottom: Val::Px(5.0),
                        right: Val::Px(15.0),
                        ..Default::default()
                    },
                    size: Size {
                        //width: Val::Px(200.0),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                text: Text::with_section(
                    "continue".to_string(),
                    TextStyle {
                        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                        font_size: 200.0,
                        color: Color::rgba(0.8, 0.8, 0.8, 1.0),
                    },
                    TextAlignment {
                        ..Default::default()
                    }
                ),
                ..Default::default()
            })
            .insert(level_over::LevelOverText {})
            .insert(ContinueText);
    println!("Score text made!");
}

pub fn displaying_score(
    mut state: ResMut<State<crate::AppState>>,
    mut query: Query<&mut Text, Without<ContinueText>>,
    mut score: ResMut<Score>,
    mut level: ResMut<level::Level>,
    mut text_set: Local<bool>,
    mut continue_text: Query<&mut Text, With<ContinueText>>,
    mut text_blink: Local<bool>,

    keyboard_input: Res<Input<KeyCode>>,
    axes: Res<Axis<GamepadAxis>>,
    buttons: Res<Input<GamepadButton>>,
    gamepad: Option<Res<game_controller::GameController>>,
) {
    if !*text_set {
        score.total += score.current_level;
        for mut text in query.iter_mut() {
            println!("showing score text !");
            text.sections[0].value = format!("Score {}", score.total).to_string();
        }
        *text_set = true;
    }

    let pressed_buttons = game_controller::get_pressed_buttons(&axes, &buttons, gamepad);
    if keyboard_input.just_pressed(KeyCode::Return) || keyboard_input.just_pressed(KeyCode::Space)
    || pressed_buttons.contains(&game_controller::GameButton::Action){
        state.set(crate::AppState::LevelTitle).unwrap();
        *text_set = false;
    }

    for mut text in continue_text.iter_mut() {
        let a = text.sections[0].style.color.a();
        if a < 0.5 { 
            *text_blink = false;
        }
        if a > 1.0 {
            *text_blink = true;
        }

        if *text_blink {
            text.sections[0].style.color.set_a(a - 0.015);
        } else {
            text.sections[0].style.color.set_a(a + 0.015);
        }
    }
}
