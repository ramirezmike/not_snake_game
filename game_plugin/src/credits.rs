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
"Congratulations!
Your final score was: {}
Thank you for playing!

Not Snake by Michael Ramirez

Special Thanks To...

Krystal
mi media naranja, for being a source of endless encouragment and also for not rolling your eyes too much at me

My cat, Meow
All your suggestions were terrible. Have you ever even played a video game before?

Dana
thanks for telling me everything wrong with the game

Hedzer 
thanks for refusing to play my game and suggesting I add a \"Quit\" option

Logan and Erik 
thanks for playing my game but not telling me what you thought

Norman 
thanks for playing my game and saying it's \"actually fun\" like, in a really surprised way?


Not Snake was made with the Bevy game engine

Special Bevy Thanks

cart - for being a cool dude

alice - for being really knowledgable and super helpful

TheRawMeatball - for being really helpful a lot

NiklasEi - the Bevy game template is amazing

OptimisticPeach - for answering my shader questions that was neat

StarToaster - also for answering my other shader questions 

robswain - your bevy-hud-pass made my hud all cool

aevyrie - I think I managed to use all of your plugins

gin - for making really out-of-the-box suggestions

Toqoz - your line crate helped me fix a ton of bugs

Joy - for helping me learn rotations

jamadazi - your cheatbook was super super helpful!

and thanks to everyone else in the Bevy discord!

You all are so nice and helpful

ok bye!










uhh the game should restart soon


any second now...

", (score.total + score.current_level)),
                TextStyle {
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    font_size: 60.0,
                    color: Color::WHITE,
                },
                TextAlignment {
                    horizontal: HorizontalAlign::Center,
                    vertical: VerticalAlign::Center,
                },
            ),
            ..Default::default()
        })
        .insert(EndCredits(60.0));
}

#[derive(Component)]
pub struct EndCredits(f32);

pub fn update_credits(
    mut commands: Commands,
    mut end_credits: Query<(Entity, &mut EndCredits, &mut Style)>,
    mut level: ResMut<crate::level::Level>,
    time: Res<Time>,
    mut state: ResMut<State<crate::AppState>>,
    mut game_is_over: ResMut<environment::GameOver>,
    mut windows: ResMut<Windows>,
) {
    let window = windows.get_primary_mut().unwrap();
    let height = window.height(); 

    let mut end_credits_have_ended = false;
    for (_, mut end_credit, mut style) in end_credits.iter_mut() {
        end_credit.0 = end_credit.0 - (time.delta_seconds() * (8592.0 / height));
        style.position.top = Val::Percent(end_credit.0);

        println!("End Credit: {}",end_credit.0);
        if end_credit.0 < -60.0 * (8592.0 / height) {
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
