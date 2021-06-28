use bevy::prelude::*;

use crate::environment;

pub struct CreditsEvent {}
pub struct CreditsDelay(pub Timer);

pub fn setup_credits(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    commands.spawn_bundle(UiCameraBundle::default());
    commands.insert_resource(ClearColor(Color::hex(crate::COLOR_BLACK).unwrap()));
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
                "No Snake by Michael Ramirez \n
                 TODO: Actual Credits \n
                 Game design                Michael Ramirez\n
                 Lead Programmer            Michael Ramirez\n
                 Music Composition          Michael Ramirez\n
                 Sound design               Michael Ramirez\n
                 Character Art              Michael Ramirez\n
                 Animation                  Michael Ramirez\n
                 AI Programming             Michael Ramirez\n
                 Catering                   Michael Ramirez\n
                 Scrum Master               Michael Ramirez\n
                 Product Manager            Michael Ramirez\n
                 Junior Programmer          Michael Ramirez\n
                 Spanish Translation        Michael Ramirez\n
                 Human Resources            Michael Ramirez\n
                 Character Controls         Michael Ramirez\n
                 Level Design               Michael Ramirez\n
                 User Experience            Michael Ramirez\n
                 Legal                      Michael Ramirez\n
                 Art Design                 Michael Ramirez\n
                 Creative Management        Michael Ramirez\n
                 Market Research            Michael Ramirez\n
                 Special thanks             Michael Ramirez\n
                ",
                TextStyle {
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    font_size: 100.0,
                    color: Color::WHITE,
                },
                // Note: You can use `Default::default()` in place of the `TextAlignment`
                TextAlignment {
                    ..Default::default()
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
        end_credit.0 = end_credit.0 - time.delta_seconds() * 200.0;
        style.position.top = Val::Percent(end_credit.0);

        if end_credit.0 < -305.0 {
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
