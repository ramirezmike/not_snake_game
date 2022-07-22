use crate::{
    assets::GameAssets, audio::GameAudio, cleanup, game_controller, menus,
    title_screen::MenuAction, ui::text_size, AppState, title_screen
};
use bevy::app::AppExit;
use bevy::ecs::event::Events;
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

pub struct PausePlugin;
impl Plugin for PausePlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_enter(AppState::Pause)
                .with_system(setup)
        )
        .add_system_set(
            SystemSet::on_update(AppState::Pause)
                .with_system(update_menu_buttons.after("handle_input"))
                .with_system(
                    handle_controllers
                        .label("handle_input")
                        .after("store_controller_inputs"),
                ),
        )
        .add_system_set(
            SystemSet::on_exit(AppState::Pause)
                .with_system(title_screen::release_all_presses)
                .with_system(cleanup::<CleanupMarker>)
        );
    }
}

#[derive(Component)]
struct CleanupMarker;

const NORMAL_BUTTON: Color = Color::rgba(1.00, 1.00, 1.00, 0.0);
const HOVERED_BUTTON: Color = Color::rgb(1.00, 1.00, 0.75);

fn setup(mut commands: Commands, game_assets: Res<GameAssets>, text_scaler: text_size::TextScaler) {
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(20.0), Val::Percent(15.0)),
                position_type: PositionType::Relative,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::ColumnReverse,
                margin: Rect {
                    left: Val::Auto,
                    right: Val::Auto,
                    top: Val::Percent(60.0),
                    ..Default::default()
                },
                align_items: AlignItems::FlexStart,
                ..Default::default()
            },
            color: Color::NONE.into(),
            ..Default::default()
        })
        .insert(CleanupMarker)
        .with_children(|parent| {
            parent
                .spawn_bundle(ButtonBundle {
                    style: Style {
                        position_type: PositionType::Relative,
                        margin: Rect::all(Val::Auto),
                        size: Size::new(Val::Percent(100.0), Val::Percent(40.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..Default::default()
                    },
                    color: NORMAL_BUTTON.into(),
                    ..Default::default()
                })
                .with_children(|parent| {
                    parent.spawn_bundle(TextBundle {
                        text: Text::with_section(
                            "Resume",
                            TextStyle {
                                font: game_assets.font.clone(),
                                font_size: text_scaler.scale(menus::BUTTON_LABEL_FONT_SIZE),
                                color: Color::rgb(0.9, 0.9, 0.9),
                            },
                            Default::default(),
                        ),
                        ..Default::default()
                    });
                })
                .insert(CleanupMarker);

            parent
                .spawn_bundle(ButtonBundle {
                    style: Style {
                        size: Size::new(Val::Percent(100.0), Val::Percent(40.0)),
                        margin: Rect::all(Val::Auto),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        position_type: PositionType::Relative,
                        ..Default::default()
                    },
                    color: NORMAL_BUTTON.into(),
                    ..Default::default()
                })
                .with_children(|parent| {
                    parent.spawn_bundle(TextBundle {
                        text: Text::with_section(
                            "Quit",
                            TextStyle {
                                font: game_assets.font.clone(),
                                font_size: text_scaler.scale(menus::BUTTON_LABEL_FONT_SIZE),
                                color: Color::rgb(0.9, 0.9, 0.9),
                            },
                            Default::default(),
                        ),
                        ..Default::default()
                    });
                })
                .insert(CleanupMarker);
        });
}

fn update_menu_buttons(
    mut selected_button: Local<usize>,
    mut exit: ResMut<Events<AppExit>>,
    buttons: Query<Entity, With<Button>>,
    mut button_colors: Query<&mut UiColor, With<Button>>,
    interaction_query: Query<(Entity, &Interaction), (Changed<Interaction>, With<Button>)>,
    action_state: Query<&ActionState<MenuAction>>,
    game_assets: Res<GameAssets>,
    mut app_state: ResMut<State<AppState>>,
    mut audio: GameAudio,
) {
    let action_state = action_state.single();
    let number_of_buttons = buttons.iter().count();
    let mut pressed_button = action_state.pressed(MenuAction::Select);

    if action_state.just_pressed(MenuAction::Up) {
        audio.play_sfx(&game_assets.blip);
        *selected_button = selected_button
            .checked_sub(1)
            .unwrap_or(number_of_buttons - 1);
    }
    if action_state.just_pressed(MenuAction::Down) {
        audio.play_sfx(&game_assets.blip);
        let new_selected_button = selected_button.checked_add(1).unwrap_or(0);
        *selected_button = if new_selected_button > number_of_buttons - 1 {
            0
        } else {
            new_selected_button
        };
    }

    // mouse
    for (button_entity, interaction) in interaction_query.iter() {
        match *interaction {
            Interaction::Clicked => pressed_button = true,
            Interaction::Hovered => {
                *selected_button = buttons
                    .iter()
                    .enumerate()
                    .filter(|(_, x)| *x == button_entity)
                    .map(|(i, _)| i)
                    .last()
                    .unwrap_or(*selected_button)
            }
            _ => (),
        }
    }

    for (i, mut color) in button_colors.iter_mut().enumerate() {
        if i == *selected_button {
            *color = menus::HOVERED_BUTTON.into();
        } else {
            *color = menus::NORMAL_BUTTON.into();
        }
    }

    if pressed_button {
        if *selected_button == 0 {
            app_state.pop().unwrap();
        }
        if *selected_button == 1 {
            exit.send(AppExit);
        }
    }
}

fn handle_controllers(
    controllers: Res<game_controller::GameController>,
    mut players: Query<(Entity, &mut ActionState<MenuAction>)>,
) {
    for (_, mut action_state) in players.iter_mut() {
        for (_, just_pressed) in controllers.just_pressed.iter() {
            if just_pressed.contains(&game_controller::GameButton::Up) {
                action_state.release(MenuAction::Up);
                action_state.press(MenuAction::Up);
            }
            if just_pressed.contains(&game_controller::GameButton::Down) {
                action_state.release(MenuAction::Down);
                action_state.press(MenuAction::Down);
            }
            if just_pressed.contains(&game_controller::GameButton::ActionDown)
                || just_pressed.contains(&game_controller::GameButton::Start)
            {
                action_state.release(MenuAction::Select);
                action_state.press(MenuAction::Select);
            }
        }
    }
}
