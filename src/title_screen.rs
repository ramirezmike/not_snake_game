use crate::{
    asset_loading, assets::GameAssets, audio::GameAudio, cleanup, game_controller, menus,
    ui::text_size, AppState, menus::HOVERED_BUTTON, menus::NORMAL_BUTTON, score
};
use bevy::app::AppExit;
use bevy::ecs::event::Events;
use bevy_utils::Instant;
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

pub struct TitlePlugin;
impl Plugin for TitlePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(InputManagerPlugin::<MenuAction>::default())
            .add_system_set(SystemSet::on_enter(AppState::MainMenu).with_system(setup))
            .add_system_set(
                SystemSet::on_update(AppState::MainMenu)
                    .with_system(update_menu_buttons.after("handle_input"))
                    .with_system(
                        handle_controllers
                            .label("handle_input")
                            .after("store_controller_inputs"),
                    ),
            )
            .add_system_set(
                SystemSet::on_exit(AppState::MainMenu)
                    .with_system(release_all_presses)
                    .with_system(cleanup::<CleanupMarker>),
            );
    }
}

#[derive(Component)]
struct CleanupMarker;

#[derive(Component)]
pub struct BylineText;
#[derive(Component)]
pub struct MenuButton;

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug)]
pub enum MenuAction {
    Up,
    Down,
    Left,
    Right,
    Select,
    Other,
}
impl MenuAction {
    pub fn default_input_map() -> InputMap<MenuAction> {
        use MenuAction::*;
        let mut input_map = InputMap::default();

        input_map.insert(Up, KeyCode::Up);
        input_map.insert(Up, KeyCode::W);

        input_map.insert(Down, KeyCode::Down);
        input_map.insert(Down, KeyCode::S);

        input_map.insert(Left, KeyCode::Left);
        input_map.insert(Left, KeyCode::A);

        input_map.insert(Right, KeyCode::Right);
        input_map.insert(Right, KeyCode::D);

        input_map.insert(Select, KeyCode::Space);
        input_map.insert(Select, KeyCode::K);
        input_map.insert(Select, KeyCode::Return);


        input_map.insert(Other, KeyCode::L);

        input_map
    }
}

pub fn load(
    assets_handler: &mut asset_loading::AssetsHandler,
    game_assets: &mut ResMut<GameAssets>,
) {
    assets_handler.add_audio(&mut game_assets.intro_handle, "audio/intro.ogg");
    assets_handler.add_audio(&mut game_assets.blip, "audio/blip.wav");
    assets_handler.add_font(&mut game_assets.font, "fonts/monogram.ttf");
}

fn setup(
    mut commands: Commands,
    game_assets: Res<GameAssets>,
    mut audio: GameAudio,
    text_scaler: text_size::TextScaler,
) {
    commands
        .spawn_bundle(UiCameraBundle::default())
        .insert(CleanupMarker);

    // TODO: move this into main or something, we only want one of these
    commands
        .spawn_bundle(InputManagerBundle {
            input_map: MenuAction::default_input_map(),
            action_state: ActionState::default(),
        });

    commands
        .spawn_bundle(TextBundle {
            style: Style {
                align_self: AlignSelf::FlexEnd,
                position_type: PositionType::Absolute,
                position: Rect {
                    bottom: Val::Px(5.0),
                    left: Val::Px(15.0),
                    ..Default::default()
                },
                ..Default::default()
            },
            text: Text::with_section(
                "by michael ramirez".to_string(),
                TextStyle {
                    font: game_assets.font.clone(),
                    font_size: text_scaler.scale(menus::BY_LINE_FONT_SIZE),
                    color: Color::rgba(0.8, 0.8, 0.8, -0.4),
                },
                TextAlignment::default(),
            ),
            ..Default::default()
        })
        .insert(BylineText)
        .insert(CleanupMarker);

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
        .insert(MenuButton)
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
                    visibility: Visibility {
                        is_visible: false,
                    },
                    color: NORMAL_BUTTON.into(),
                    ..Default::default()
                })
                .with_children(|parent| {
                    parent.spawn_bundle(TextBundle {
                        text: Text::with_section(
                            "Start",
                            TextStyle {
                                font: game_assets.font.clone(),
                                font_size: text_scaler.scale(menus::BUTTON_LABEL_FONT_SIZE),
                                color: Color::rgb(0.9, 0.9, 0.9),
                            },
                            Default::default(),
                        ),
                        visibility: Visibility {
                            is_visible: false,
                        },
                        ..Default::default()
                    })
                    .insert(MenuButton);
                })
                .insert(MenuButton);

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
                    visibility: Visibility {
                        is_visible: false,
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
                        visibility: Visibility {
                            is_visible: false,
                        },
                        ..Default::default()
                    })
                    .insert(MenuButton);
                })
                .insert(MenuButton);
        });

    audio.play_bgm(&game_assets.intro_handle);
}

fn update_menu_buttons(
    mut selected_button: Local<usize>,
    mut exit: ResMut<Events<AppExit>>,
    buttons: Query<Entity, With<Button>>,
    mut button_colors: Query<&mut UiColor, With<Button>>,
    interaction_query: Query<(Entity, &Interaction), (Changed<Interaction>, With<Button>)>,
    action_state: Query<&ActionState<MenuAction>>,
    game_assets: Res<GameAssets>,
    mut audio: GameAudio,
    mut app_state: ResMut<State<AppState>>,
    mut bylines: Query<&mut Text, With<BylineText>>,
    mut menu_buttons: Query<&mut Visibility, With<MenuButton>>,
    time: Res<Time>,
    mut score: ResMut<score::Score>,
) {
    for mut byline in bylines.iter_mut() {
        let a = byline.sections[0].style.color.a();
        if a < 0.95 {
            byline.sections[0].style.color.set_a(a + time.delta_seconds() / 5.0);
        } else {
            byline.sections[0].style.color.set_a(1.0);
        }

        if a > 0.8 {
            // make buttons visible now that byline is visible enough
            for mut visible in menu_buttons.iter_mut() {
                visible.is_visible = true; 
            }
        }
    }

    let action_state = action_state.single();
    let number_of_buttons = buttons.iter().count();
    let mut pressed_button = action_state.just_pressed(MenuAction::Select);

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
            *color = HOVERED_BUTTON.into();
        } else {
            *color = NORMAL_BUTTON.into();
        }
    }

    if pressed_button {
        println!("title screen received pressed");
        if *selected_button == 0 {
            audio.play_sfx(&game_assets.blip);
            *score = score::Score::new();
            app_state.set(AppState::LevelTitle).unwrap();
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

pub fn release_all_presses(
    mut action_states: Query<&mut ActionState<MenuAction>>,
    mut controllers: ResMut<game_controller::GameController>,
) {
    let now = Instant::now();
    for mut action_state in action_states.iter_mut() {
        action_state.tick(now);
    }

    controllers.clear_presses();
}
