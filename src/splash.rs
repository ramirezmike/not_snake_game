use bevy::prelude::*;
use crate::{AppState, ui::text_size, assets::GameAssets, menus, cleanup};

pub struct SplashPlugin;
impl Plugin for SplashPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_enter(AppState::Splash).with_system(setup))
            .init_resource::<SplashTracker>()
            .add_system_set(
                SystemSet::on_update(AppState::Splash).with_system(tick)
            )
            .add_system_set(
                SystemSet::on_exit(AppState::Splash).with_system(cleanup::<CleanupMarker>)
            );
    }
}

#[derive(Component)]
struct CleanupMarker;

#[derive(Default)]
struct SplashTracker {
    time: f32
}

fn tick(
    time: Res<Time>,
    mut state: ResMut<State<AppState>>,
    mut splash_tracker: ResMut<SplashTracker>,
) {
    splash_tracker.time += time.delta_seconds();

    if splash_tracker.time > 3.0 {
        state.set(AppState::MainMenu).unwrap();
    }
}

fn setup(
    mut commands: Commands,
    game_assets: Res<GameAssets>,
    text_scaler: text_size::TextScaler,
    mut splash_tracker: ResMut<SplashTracker>,
) {
    splash_tracker.time = 0.0;

    commands
        .spawn_bundle(UiCameraBundle::default())
        .insert(CleanupMarker);

   commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                position_type: PositionType::Relative,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::FlexEnd,
                flex_direction: FlexDirection::ColumnReverse,
                margin: Rect {
                    left: Val::Auto,
                    right: Val::Auto,
                    ..Default::default()
                },
                ..Default::default()
            },
            color: UiColor(Color::rgba(1.00, 1.00, 1.00, 0.0)),
            ..Default::default()
        })
        .insert(CleanupMarker)
        .with_children(|parent| {
            parent.spawn_bundle(ImageBundle {
                style: Style {
                    size: Size::new(Val::Auto, Val::Percent(60.0)),
                    margin: Rect {
                        left: Val::Auto,
                        right: Val::Auto,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                image: game_assets.bevy_icon.image.clone().into(),
                ..Default::default()
            });

            parent.spawn_bundle(TextBundle {
                style: Style {
                    position_type: PositionType::Relative,
                    align_items: AlignItems::FlexEnd,
                    justify_content: JustifyContent::Center,
                    margin: Rect {
                        left: Val::Auto,
                        right: Val::Auto,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                text: Text::with_section(
                    "made with Bevy",
                    TextStyle {
                        font: game_assets.font.clone(),
                        font_size: text_scaler.scale(menus::DEFAULT_FONT_SIZE * 1.2),
                        color: Color::WHITE,
                    },
                    TextAlignment {
                        horizontal: HorizontalAlign::Center,
                        ..Default::default()
                    },
                ),
                ..Default::default()
            });
        });
}
