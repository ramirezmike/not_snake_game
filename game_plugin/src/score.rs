use bevy::prelude::*;
use crate::{food::FoodEatenEvent, Dude, sounds, level, level_over, game_controller, dude};

#[derive(Component)]
pub struct ContinueText;
pub struct Score {
    pub total: usize,
    pub total_bonus: usize,
    pub current_level: usize,
    pub current_level_bonus: usize,
    pub current_death_count: usize,
}

impl Score {
    pub fn new() -> Self {
        Score {
            total: 0,
            total_bonus: 0,
            current_level: 0,
            current_level_bonus: 0,
            current_death_count: 0,
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
    mut windows: ResMut<Windows>,
    asset_server: Res<AssetServer>,
) {
    commands.spawn_bundle(UiCameraBundle::default())
            .insert(level_over::LevelOverText {});
    let window = windows.get_primary_mut().unwrap();
    let width = window.width(); 
    let height = window.height(); 
    commands
        .spawn_bundle(TextBundle {
            style: Style {
                align_self: AlignSelf::FlexEnd,
                position_type: PositionType::Absolute,
                position: Rect {
                    top: Val::Px(height * 0.35),
                    left: Val::Px(width * 0.25),
                    ..Default::default()
                },
                max_size: Size {
                    width: Val::Px(width / 2.0),
                    height: Val::Undefined,
                    ..Default::default()
                },
                ..Default::default()
            },
            text: Text::with_section(
                "".to_string(),
                TextStyle {
                    font: asset_server.load(crate::FONT),
                    font_size: 80.0,
                    color: Color::WHITE,
                },
                TextAlignment {
                    horizontal: HorizontalAlign::Center,
                    vertical: VerticalAlign::Center,
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
                    "".to_string(),
                    TextStyle {
                        font: asset_server.load(crate::FONT),
                        font_size: 100.0,
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
    level: Res<level::Level>,
    mut text_set: Local<bool>,
    mut continue_text: Query<&mut Text, With<ContinueText>>,
    mut text_blink: Local<bool>,

    time: Res<Time>,
    mut buffer: Local<f32>,
    mut text_counter: Local<usize>,
    mut score_added: Local<bool>,
    keyboard_input: Res<Input<KeyCode>>,
    axes: Res<Axis<GamepadAxis>>,
    buttons: Res<Input<GamepadButton>>,
    gamepad: Option<Res<game_controller::GameController>>,
) {
    if !*score_added {
        score.total += score.current_level;
        *score_added = true;
    }

    let score_texts = level.get_score_text();
    if !*text_set {
        for mut text in query.iter_mut() {
            if let Some(score_text) = &score_texts.get(*text_counter) {
                text.sections[0].value = score_text.print(score.total, score.current_death_count);
            } else {
                text.sections[0].value = "".to_string();
            }
        }
        println!("Score: {} Death: {}",score.total, score.current_death_count);
        *text_set = true;
    }

    *buffer += time.delta_seconds();
    if *buffer > 0.5 {
        let pressed_buttons = game_controller::get_pressed_buttons(&axes, &buttons, gamepad);
        if keyboard_input.just_pressed(KeyCode::Return) || keyboard_input.just_pressed(KeyCode::Space)
        || keyboard_input.just_pressed(KeyCode::J)    
        || pressed_buttons.contains(&game_controller::GameButton::Action){
            *text_counter += 1;
            *text_set = false;
            *buffer = 0.0;
        }
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

    if *text_counter >= score_texts.len() {
        state.set(crate::AppState::LevelTitle).unwrap();
        *text_set = false;
        *text_counter = 0; 
        *score_added = false;
        score.current_death_count = 0;
    }
}

pub fn handle_kill_dude(
    mut score: ResMut<Score>,
    mut kill_dude_event_reader: EventReader<dude::KillDudeEvent>,
) {
    for _ in kill_dude_event_reader.iter() {
        score.current_death_count += 1;
    }
}
