use bevy::{prelude::*,};
use crate::{credits, level, dude, moveable, environment};

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
    mut clear_color: ResMut<ClearColor>,
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

    // change background color gradually to next level's color
    let current_palette = &level.get_palette();
    let target_palette = &level.get_next_level_palette();
    let current_color = Color::hex(current_palette.base.clone()).unwrap();
    let target_color = Color::hex(target_palette.base.clone()).unwrap();
    let target = Vec3::new(target_color.r(), target_color.g(), target_color.b());
    let start = Vec3::new(current_color.r(), current_color.g(), current_color.b());
    let new_color = start.lerp(target, *timer);

    clear_color.0 = Color::rgb(new_color.x, new_color.y, new_color.z); 

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
    mut game_is_over: ResMut<environment::GameOver>,
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
                game_is_over.0 = true;
                credits_delay.0.reset();
            }
        } else {
            state.set(crate::AppState::ScoreDisplay).unwrap();
        }
    }

    if game_is_over.0 && credits_delay.0.tick(time.delta()).finished() {
        credits_event_writer.send(crate::credits::CreditsEvent {});
    }
}
