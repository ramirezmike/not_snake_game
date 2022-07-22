use crate::{AppState, dude, food::FoodEatenEvent, game_controller, level, level_over, audio, Dude, assets::GameAssets,
    title_screen::MenuAction, environment, cleanup, title_screen, ui::text_display, ui::text_size, assets
};
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

pub struct ScorePlugin;
impl Plugin for ScorePlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_enter(AppState::ScoreDisplay)
                .with_system(audio::play_ingame_music)
                .with_system(setup)
                .with_system(title_screen::release_all_presses.after(setup))
        )
        .add_system_set(
            SystemSet::on_update(AppState::ScoreDisplay)
                .with_system(displaying_score.after("handle_input"))
                .with_system(
                    handle_controllers
                        .label("handle_input")
                        .after("store_controller_inputs"),
                ),
        )
        .init_resource::<ControllerBuffer>()
        .add_system_set(
            SystemSet::on_exit(AppState::ScoreDisplay)
                .with_system(title_screen::release_all_presses)
                .with_system(cleanup::<CleanupMarker>)
        );
    }
}

#[derive(Default)]
struct ControllerBuffer {
    cooldown: f32
}

#[derive(Component)]
struct CleanupMarker;

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
    game_assets: Res<GameAssets>,
    mut audio: audio::GameAudio,
) {
    for eater in food_eaten_event_reader.iter() {
        if let Ok(_) = dude.get(eater.0) {
            audio.play_sfx(&game_assets.pickup_handle[score.current_level % 5]);

            if eater.1 {
                score.current_level_bonus += 1;
            } else {
                score.current_level += 1;
            }
        }
    }
}

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

//  commands
//      .spawn_bundle(TextBundle {
//          style: Style {
//              align_self: AlignSelf::FlexEnd,
//              position_type: PositionType::Absolute,
//              position: Rect {
//                  bottom: Val::Px(5.0),
//                  right: Val::Px(15.0),
//                  ..Default::default()
//              },
//              size: Size {
//                  //width: Val::Px(200.0),
//                  ..Default::default()
//              },
//              ..Default::default()
//          },
//          text: Text::with_section(
//              "".to_string(),
//              TextStyle {
//                  font: asset_server.load(crate::FONT),
//                  font_size: 100.0,
//                  color: Color::rgba(0.8, 0.8, 0.8, 1.0),
//              },
//              TextAlignment {
//                  ..Default::default()
//              },
//          ),
//          ..Default::default()
//      })
//      .insert(CleanupMarker)
//      .insert(ContinueText);
    println!("Score text made!");
}

fn displaying_score(
    mut state: ResMut<State<crate::AppState>>,
    mut query: Query<&mut Text, Without<ContinueText>>,
    mut score: ResMut<Score>,
    level: Res<level::Level>,
    mut text_set: Local<bool>,
    mut continue_text: Query<&mut Text, With<ContinueText>>,
    mut text_blink: Local<bool>,
    game_assets: Res<GameAssets>,
    mut audio: audio::GameAudio,

    action_state: Query<&ActionState<MenuAction>>,
    mut text_counter: Local<usize>,
    mut score_added: Local<bool>,
    mut controller_buffer: ResMut<ControllerBuffer>,
    time: Res<Time>,
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
        println!(
            "Score: {} Death: {}",
            score.total, score.current_death_count
        );
        *text_set = true;
        audio.play_sfx(&game_assets.blip);
    }

    controller_buffer.cooldown -= time.delta_seconds();
    controller_buffer.cooldown = controller_buffer.cooldown.clamp(-10.0, 30.0);

    if controller_buffer.cooldown <= 0.0 {
        let action_state = action_state.single();
        if action_state.just_pressed(MenuAction::Select) {
            println!("score screen received");
            *text_counter += 1;
            *text_set = false;
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

pub fn increase_death_count(
    mut score: ResMut<Score>,
) {
    score.current_death_count += 1;
}

fn handle_controllers(
    controllers: Res<game_controller::GameController>,
    mut players: Query<(Entity, &mut ActionState<MenuAction>)>,
) {
    for (_, mut action_state) in players.iter_mut() {
        for (_, just_pressed) in controllers.just_pressed.iter() {
            if just_pressed.contains(&game_controller::GameButton::ActionDown) {
                println!("score screen pressed");
                action_state.release(MenuAction::Select);
                action_state.press(MenuAction::Select);
            }
        }
    }
}
