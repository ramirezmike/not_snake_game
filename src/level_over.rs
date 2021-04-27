use bevy::{prelude::*,};
use crate::{credits, level};

pub struct LevelOverEvent {}
pub struct LevelOverText {}

pub fn setup_level_over_screen(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    commands.spawn_bundle(UiCameraBundle::default());
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
    .insert(LevelOverText {});
}

pub fn level_over_check(
    mut state: ResMut<State<crate::AppState>>,
    mut level_over_events: EventReader<LevelOverEvent>,
    mut query: Query<&mut Text>,
    mut game_is_over: Local<bool>,
    mut level: ResMut<level::Level>,
    time: Res<Time>, 
    mut credits_delay: ResMut<credits::CreditsDelay>,
    mut credits_event_writer: EventWriter<crate::credits::CreditsEvent>,
    mut next_level_event_writer: EventWriter<level::NextLevelEvent>,
) {
    if level_over_events.iter().count() > 0 {
        if level.is_last_level() {
            for mut text in query.iter_mut() {
                text.sections[0].value = "YOU WIN!".to_string();
                *game_is_over = true;
                credits_delay.0.reset();
            }
        } else {
            //next_level_event_writer.send(level::NextLevelEvent);
            for mut text in query.iter_mut() {
                text.sections[0].value = format!("LEVEL {}", level.current_level + 2).to_string();
            }

            state.set(crate::AppState::ChangingLevel).unwrap();
        }
    }

    if *game_is_over && credits_delay.0.tick(time.delta()).finished() {
        credits_event_writer.send(crate::credits::CreditsEvent {});
    }
}
