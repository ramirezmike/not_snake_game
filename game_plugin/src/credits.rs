use bevy::prelude::*;

use crate::{environment, score::Score};

pub struct CreditsEvent {}
pub struct CreditsDelay(pub Timer);

pub fn setup_credits(
    mut commands: Commands,
    score: Res<Score>,
    mut windows: ResMut<Windows>,
    asset_server: Res<AssetServer>,
) {
    commands.spawn_bundle(UiCameraBundle::default());
    commands.insert_resource(ClearColor(Color::hex(crate::COLOR_BLACK).unwrap()));
    let window = windows.get_primary_mut().unwrap();
    let width = window.width(); 
    let height = window.height(); 
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
            // Use the `Text::with_section` constructor
            text: Text::with_section(
                format!(
                "Congratulations!\n
                 Your final score was: {}\n\n\n
                 Thank you for playing!\n\n
                 Not Snake by Michael Ramirez \n\n
                 Special Thanks To...\n

                 Krystal, mi media naranja, for being a source of endless encouragment and also for not rolling your eyes too much at me\n\n
                 My cat, Meow. All your suggestions were terrible. Have you ever even played a video game before?\n\n
                 Dana for telling me everything wrong with the game\n\n
                 Hedzer for refusing to play my game and suggesting I add a \"Quit\" option \n\n
                 Logan and Erik both for playing my game but not telling me what you thought\n\n
                 Norman for playing my game and saying it's \"actually fun\" like, in a really surprised way?\n\n

                 Not Snake was made with the Bevy game engine\n
                 Special Bevy Thanks\n
                 cart - for being a cool dude\n
                 gin - for making really crazy suggestions\n
                 TheRawMeatball and alice - for being really knowledgable and super helpful\n
                 NiklasEi - the Bevy game template is amazing\n
                 OptimisticPeach and StarToaster - for answering my shader questions that was neat\n
                 robswain - your bevy-hud-pass made my hud all cool\n
                 aevyrie - I think I managed to use all of your plugins\n
                 Toqoz - your line crate helped me fix a ton of bugs\n
                 Joy - for helping me learn rotations\n
                 jamadazi - your cheatbook was super super helpful!\n
                 and thanks to everyone else in the Bevy discord!\n 
                 You all are so nice and helpful\n
                 \n\n
                 and special thanks to you, the player\n
                 unless you were one of the people listed above\n
                 bye!
                ", (score.total + score.current_level)),
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
        .insert(EndCredits(30.0));
}

pub struct EndCredits(f32);

pub fn update_credits(
    mut commands: Commands,
    mut end_credits: Query<(Entity, &mut EndCredits, &mut Style)>,
    mut level: ResMut<crate::level::Level>,
    time: Res<Time>,
    mut state: ResMut<State<crate::AppState>>,
    mut game_is_over: ResMut<environment::GameOver>,
) {
    let mut end_credits_have_ended = false;
    for (_, mut end_credit, mut style) in end_credits.iter_mut() {
        end_credit.0 = end_credit.0 - time.delta_seconds() * 16.0;
        style.position.top = Val::Percent(end_credit.0);

        if end_credit.0 < -405.0 {
            end_credits_have_ended = true; 
        }
    }

    if end_credits_have_ended {
        for (entity, _, _) in end_credits.iter_mut() {
            commands.entity(entity).despawn();
        }
        level.current_level = 0;
        game_is_over.0 = false;
        state.set(crate::AppState::MainMenu).unwrap();
    }
}

pub fn show_credits(
    mut credit_event: EventReader<CreditsEvent>,
    mut app_state: ResMut<State<crate::AppState>>
) { 
    if credit_event.iter().count() > 0 {
        app_state.set(crate::AppState::Credits).unwrap();
    }
}
