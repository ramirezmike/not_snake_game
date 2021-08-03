use bevy::{prelude::*,};
use crate::{credits, level, dude, moveable, environment, game_controller, snake::Enemy};

pub struct LevelOverEvent {}
pub struct LevelOverText {} // TODO: change this to like "BetweenLevelEntity" or something marker or something

pub fn setup_level_over_screen(
    mut commands: Commands,
    mut windows: ResMut<Windows>,
    asset_server: Res<AssetServer>,
) {
    let window = windows.get_primary_mut().unwrap();
    let width = window.width(); 
    let height = window.height(); 

    commands.spawn_bundle(UiCameraBundle::default())
            .insert(LevelOverText {});

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
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    font_size: 100.0,
                    color: Color::WHITE,
                },
                TextAlignment {
                    horizontal: HorizontalAlign::Center,
                    vertical: VerticalAlign::Center,
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
    level: Res<level::Level>,
    mut timer: Local<f32>,
    mut text_set: Local<bool>,
    mut color_set: Local<bool>,

    mut buffer: Local<f32>,
    mut text_counter: Local<usize>,
    keyboard_input: Res<Input<KeyCode>>,
    axes: Res<Axis<GamepadAxis>>,
    buttons: Res<Input<GamepadButton>>,
    gamepad: Option<Res<game_controller::GameController>>,
) {
    let level_texts = level.get_level_text();
    if !*text_set {
        for mut text in query.iter_mut() {
            if let Some(level_text) = &level_texts.get(*text_counter) {
                text.sections[0].value = level_text.print(0, 0);
            } else {
                text.sections[0].value = "".to_string();
            }
        }
        *text_set = true;
    }

    // change background color gradually to next level's color
    let target_palette = &level.get_next_level_palette();
    
    let current_color = clear_color.0.clone();//Color::hex(current_palette.base.clone()).unwrap();
    let target_color = Color::hex(target_palette.base.clone()).unwrap();
    let target = Vec3::new(target_color.r(), target_color.g(), target_color.b());
    let start = Vec3::new(current_color.r(), current_color.g(), current_color.b());
    let new_color = start.lerp(target, *timer);
    clear_color.0 = Color::rgb(new_color.x, new_color.y, new_color.z); 

    *timer += time.delta_seconds();

    *buffer += time.delta_seconds();
    if *buffer > 0.2 {
        let pressed_buttons = game_controller::get_pressed_buttons(&axes, &buttons, gamepad);
        if keyboard_input.just_pressed(KeyCode::Return) || keyboard_input.just_pressed(KeyCode::Space)
        || pressed_buttons.contains(&game_controller::GameButton::Action){
            *text_counter += 1;
            *text_set = false;
            *buffer = 0.0;
        }
    }

    if *text_counter >= level_texts.len() {
        state.set(crate::AppState::ChangingLevel).unwrap();
        *text_set = false;
        *color_set = false;
        *text_counter = 0; 
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
    mut snakes: Query<&mut Enemy>,
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
                for mut snake in snakes.iter_mut() {
                    snake.is_dead = true;
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
