use bevy::{prelude::*,};
use crate::{credits, level, dude, moveable};

pub struct LevelOverEvent {}
pub struct LevelOverText {} // TODO: change this to like "BetweenLevelEntity" or something marker or something

pub fn setup_level_over_screen(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    commands.spawn_bundle(UiCameraBundle::default())
            .insert(LevelOverText {});
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
    println!("Level over text made!");
}

pub fn displaying_title (
    mut state: ResMut<State<crate::AppState>>,
    time: Res<Time>,
    mut query: Query<&mut Text>,
    mut level: ResMut<level::Level>,
    mut timer: Local<f32>,
    mut text_set: Local<bool>,
) {
    if !*text_set {
        for mut text in query.iter_mut() {
            println!("showing level text !");
            text.sections[0].value = level.get_next_level_title();
        }
        *text_set = true;
    }

    *timer += time.delta_seconds();

    println!("displaying title...");
    if *timer > 1.0 {
        state.set(crate::AppState::ChangingLevel).unwrap();
        *text_set = false;
        *timer = 0.0; 
    }
}

pub fn level_over_check(
    mut state: ResMut<State<crate::AppState>>,
    mut level_over_events: EventReader<LevelOverEvent>,
    mut query: Query<&mut Text>,
    mut game_is_over: Local<bool>,
    level: ResMut<level::Level>,
    time: Res<Time>, 
    mut commands: Commands,
    mut dudes: Query<Entity, With<dude::Dude>>,
    mut credits_delay: ResMut<credits::CreditsDelay>,
    mut credits_event_writer: EventWriter<crate::credits::CreditsEvent>,
) {
    if level_over_events.iter().count() > 0 {
        println!("LEVEL IS OVER!");
        if level.is_last_level() {
            for mut text in query.iter_mut() {
                for entity in dudes.iter_mut() {
                    commands.entity(entity).remove::<moveable::Moveable>();
                }
                text.sections[0].value = "YOU WIN!".to_string();
                *game_is_over = true;
                credits_delay.0.reset();
            }
        } else {
            state.set(crate::AppState::ScoreDisplay).unwrap();
        }
    }

    if *game_is_over && credits_delay.0.tick(time.delta()).finished() {
        credits_event_writer.send(crate::credits::CreditsEvent {});
    }
}
