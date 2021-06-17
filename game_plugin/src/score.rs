use bevy::prelude::*;
use crate::{food::FoodEatenEvent, Dude, sounds, level, level_over};

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
    println!("Level over text made!");
}

pub fn displaying_score(
    mut state: ResMut<State<crate::AppState>>,
    time: Res<Time>,
    mut query: Query<&mut Text>,
    mut score: ResMut<Score>,
    mut level: ResMut<level::Level>,
    mut timer: Local<f32>,
    mut text_set: Local<bool>,
) {
    if !*text_set {
        score.total += score.current_level;
        for mut text in query.iter_mut() {
            println!("showing score text !");
            text.sections[0].value = format!("Score {}", score.total).to_string();
        }
        *text_set = true;
    }

    *timer += time.delta_seconds();

    println!("displaying score...");
    if *timer > 1.0 {
        state.set(crate::AppState::LevelTitle).unwrap();
        *text_set = false;
        *timer = 0.0; 
    }
}
