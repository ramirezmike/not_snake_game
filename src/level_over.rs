use bevy::{prelude::*,};
use crate::{credits};

pub struct LevelOverEvent {}
pub struct LevelIsOver(pub bool);
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
    mut level_over_events: EventReader<LevelOverEvent>,
    mut query: Query<&mut Text>,
    mut level_is_over: ResMut<LevelIsOver>,
    time: Res<Time>, 
    mut credits_delay: ResMut<credits::CreditsDelay>,
    mut credits_event_writer: EventWriter<crate::credits::CreditsEvent>
) {
    for _level_over in level_over_events.iter() {
        for mut text in query.iter_mut() {
            println!("Changing text!");
            text.sections[0].value = "YOU WIN!".to_string();
            level_is_over.0 = true;
            credits_delay.0.reset();
        }
    }

    if level_is_over.0 && credits_delay.0.tick(time.delta()).finished() {
        credits_event_writer.send(crate::credits::CreditsEvent {});
    }
}
