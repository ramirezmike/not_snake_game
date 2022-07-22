use crate::{credits, dude, environment, game_controller, cleanup, level, moveable, snake::Enemy, AppState, title_screen::MenuAction, title_screen, ui::text_size, ui::text_display, menus, assets, audio};
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

pub struct LevelOverEvent {}

pub struct LevelOverPlugin;
impl Plugin for LevelOverPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
                SystemSet::on_enter(AppState::LevelTitle)
                    .with_system(audio::play_ingame_music)
                    .with_system(setup)
                    .with_system(title_screen::release_all_presses.after(setup))
            )
            .add_system_set(
                SystemSet::on_update(AppState::LevelTitle)
                    .with_system(displaying_title.after("handle_input"))
                    .with_system(
                        handle_controllers
                            .label("handle_input")
                            .after("store_controller_inputs"),
                    ),
            )
            .init_resource::<ControllerBuffer>()
            .add_system_set(
                SystemSet::on_exit(AppState::LevelTitle)
                    .with_system(title_screen::release_all_presses)
                    .with_system(cleanup::<CleanupMarker>),
            );
    }
}

#[derive(Default)]
struct ControllerBuffer {
    cooldown: f32
}

#[derive(Component)]
struct CleanupMarker;

fn setup(
    mut commands: Commands,
    game_assets: Res<assets::GameAssets>,
    text_scaler: text_size::TextScaler,
    mut controller_buffer: ResMut<ControllerBuffer>,
) {
    commands
        .spawn_bundle(UiCameraBundle::default())
        .insert(CleanupMarker);

    text_display::add_text(&mut commands,
                          game_assets.font.clone(),
                          &text_scaler,
                          vec!(CleanupMarker));
    controller_buffer.cooldown = 0.1;

    println!("Level over text made!");
}

fn displaying_title(
    mut state: ResMut<State<crate::AppState>>,
    time: Res<Time>,
    mut query: Query<&mut Text>,
    mut clear_color: ResMut<ClearColor>,
    level: Res<level::Level>,
    mut timer: Local<f32>,
    mut text_set: Local<bool>,
    mut color_set: Local<bool>,
    game_assets: Res<assets::GameAssets>,
    mut audio: audio::GameAudio,

    action_state: Query<&ActionState<MenuAction>>,
    mut text_counter: Local<usize>,
    mut controller_buffer: ResMut<ControllerBuffer>,
) {
    let level_texts = level.get_level_text();
    if !*text_set {
        for mut text in query.iter_mut() {
            if let Some(level_text) = &level_texts.get(*text_counter) {
                text.sections[0].value = level_text.print(0, 0);
            } else {
                text.sections[0].value = "".to_string();
            }

            println!("Text set {}", text.sections[0].value);
        }
        *text_set = true;
        audio.play_sfx(&game_assets.blip);
    }

    // change background color gradually to next level's color
    let target_palette = &level.get_next_level_palette();

    let current_color = clear_color.0.clone(); //Color::hex(current_palette.base.clone()).unwrap();
    let target_color = Color::hex(target_palette.base.clone()).unwrap();
    let target = Vec3::new(target_color.r(), target_color.g(), target_color.b());
    let start = Vec3::new(current_color.r(), current_color.g(), current_color.b());
    let new_color = start.lerp(target, *timer);
    clear_color.0 = Color::rgb(new_color.x, new_color.y, new_color.z);

    *timer += time.delta_seconds();

    controller_buffer.cooldown -= time.delta_seconds();
    controller_buffer.cooldown = controller_buffer.cooldown.clamp(-10.0, 30.0);

    if controller_buffer.cooldown <= 0.0 {
        let action_state = action_state.single();
        if action_state.just_pressed(MenuAction::Select) {
            println!("level over screen received");
            *text_counter += 1;
            *text_set = false;
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

fn handle_controllers(
    controllers: Res<game_controller::GameController>,
    mut players: Query<(Entity, &mut ActionState<MenuAction>)>,
) {
    for (_, mut action_state) in players.iter_mut() {
        for (_, just_pressed) in controllers.just_pressed.iter() {
            if just_pressed.contains(&game_controller::GameButton::ActionDown)
            {
                println!("level over screen pressed");
                action_state.release(MenuAction::Select);
                action_state.press(MenuAction::Select);
            }

            if just_pressed.contains(&game_controller::GameButton::Other)
                || just_pressed.contains(&game_controller::GameButton::Start) {
                action_state.release(MenuAction::Other);
                action_state.press(MenuAction::Other);
            }
        }
    }
}
