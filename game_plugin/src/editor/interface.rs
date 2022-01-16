use crate::editor::{
    editor_camera, help_text::HelpTextBoxEvent, play, EditorTrashMarker, GameEntity,
};
use crate::AppState;
use bevy::prelude::*;

// create visual buttons and also handle keybinding
/*
   [(S)elect]                               [Play]
   |___[Single]
       [Multi]
   [(A)dd]                                  [Camera]
   [(R)emove]
   [Copy Properties]
   [Paste Properties]
   [Level Properties]
   [Syste(m)]
   |___[Save]
       [Save As]
       [Load]
       [Save and Quit]
       [Quit without Saving]
*/

pub struct EditorInterfacePlugin;
impl Plugin for EditorInterfacePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<InterfaceActionEvent>()
            .insert_resource(Interface {
                selected_button: None,
                hovered_button: None,
                current_entity: EntityButton::Block,
            })
            .add_system_set(
                SystemSet::on_update(AppState::Editor)
                    .with_system(update_button_states)
                    .with_system(handle_interface_events)
                    .with_system(handle_button_interaction)
                    .with_system(handle_keyboard_input),
            )
            .add_system_set(SystemSet::on_exit(AppState::Editor).with_system(cleanup_editor))
            .add_system_set(SystemSet::on_enter(AppState::Editor).with_system(create_editor_ui));
    }
}

struct InterfaceActionEvent {
    action: InterfaceAction,
}

pub struct Interface {
    selected_button: Option<InterfaceButton>,
    hovered_button: Option<InterfaceButton>,
    pub current_entity: EntityButton,
}

impl Interface {
    pub fn current_state(&self) -> InterfaceMode {
        match self.selected_button {
            Some(button) => match button {
                InterfaceButton::Add => InterfaceMode::Add,
                InterfaceButton::Camera => InterfaceMode::Camera,
                _ => InterfaceMode::Normal,
            },
            None => InterfaceMode::Normal,
        }
    }
}

#[derive(Component)]
struct InterfaceButtonComponent {
    interface_button: InterfaceButton,
}

#[derive(Component)]
struct EntityButtonComponent {
    pub entity_button: EntityButton,
}

#[derive(PartialEq, Clone, Copy)]
pub enum InterfaceMode {
    Normal,
    Camera,
    Add,
}

#[derive(PartialEq, Clone, Copy)]
pub enum EntityButton {
    Block,
    NotSnakeSpawn,
    SnakeSpawn,
}

#[derive(PartialEq, Clone, Copy)]
enum InterfaceButton {
    Camera,
    Play,
    Select,
    Add,
    Remove,
    CopyProps,
    PasteProps,
    LevelProps,
    System,
}

#[derive(PartialEq)]
enum InterfaceAction {
    ToggleCamera,
    Add,
    Play,
}

const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);

fn create_editor_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn_bundle(UiCameraBundle::default())
        .insert(EditorTrashMarker);

    create_button(
        &mut commands,
        &asset_server,
        1,
        "Select",
        InterfaceButton::Select,
        HorizontalPosition::Left,
    );
    create_button(
        &mut commands,
        &asset_server,
        2,
        "Add",
        InterfaceButton::Add,
        HorizontalPosition::Left,
    );
    create_button(
        &mut commands,
        &asset_server,
        3,
        "Remove",
        InterfaceButton::Remove,
        HorizontalPosition::Left,
    );
    create_button(
        &mut commands,
        &asset_server,
        4,
        "Copy Props",
        InterfaceButton::CopyProps,
        HorizontalPosition::Left,
    );
    create_button(
        &mut commands,
        &asset_server,
        5,
        "Paste Props",
        InterfaceButton::PasteProps,
        HorizontalPosition::Left,
    );
    create_button(
        &mut commands,
        &asset_server,
        6,
        "Level Props",
        InterfaceButton::LevelProps,
        HorizontalPosition::Left,
    );
    create_button(
        &mut commands,
        &asset_server,
        7,
        "System",
        InterfaceButton::System,
        HorizontalPosition::Left,
    );

    create_button(
        &mut commands,
        &asset_server,
        1,
        "Play",
        InterfaceButton::Play,
        HorizontalPosition::Right,
    );
    create_button(
        &mut commands,
        &asset_server,
        2,
        "Camera",
        InterfaceButton::Camera,
        HorizontalPosition::Right,
    );

    create_entity_button(
        &mut commands,
        &asset_server,
        1,
        "Block",
        EntityButton::Block,
    );
    create_entity_button(
        &mut commands,
        &asset_server,
        2,
        "Not Snake",
        EntityButton::NotSnakeSpawn,
    );
    create_entity_button(
        &mut commands,
        &asset_server,
        3,
        "Snake",
        EntityButton::SnakeSpawn,
    );
}

#[derive(PartialEq)]
enum HorizontalPosition {
    Left,
    Right,
}
#[derive(PartialEq)]
enum VerticalPosition {
    Top,
    Bottom,
}

fn create_entity_button(
    mut commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    position: usize,
    title: &str,
    entity_button: EntityButton,
) {
    commands
        .spawn_bundle(ButtonBundle {
            style: Style {
                size: Size::new(Val::Px(250.0), Val::Px(65.0)),
                // center button
                margin: Rect::all(Val::Auto),
                // horizontally center child text
                justify_content: JustifyContent::Center,
                display: Display::None,
                // vertically center child text
                align_items: AlignItems::Center,
                position_type: PositionType::Absolute,
                position: Rect {
                    top: Val::Auto,
                    bottom: Val::Percent(5.0 * (position as f32)),
                    left: Val::Auto,
                    right: Val::Percent(0.2),
                    ..Default::default()
                },
                ..Default::default()
            },
            color: NORMAL_BUTTON.into(),
            ..Default::default()
        })
        .insert(EntityButtonComponent { entity_button })
        .insert(EditorTrashMarker)
        .with_children(|parent| {
            parent.spawn_bundle(TextBundle {
                text: Text::with_section(
                    title.clone(),
                    TextStyle {
                        font: asset_server.load(crate::FONT),
                        font_size: 40.0,
                        color: Color::rgb(0.9, 0.9, 0.9),
                    },
                    Default::default(),
                ),
                ..Default::default()
            });
        });
}

fn create_button(
    mut commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    position: usize,
    title: &str,
    interface_button: InterfaceButton,
    horizontal_position: HorizontalPosition,
) {
    commands
        .spawn_bundle(ButtonBundle {
            style: Style {
                size: Size::new(Val::Px(250.0), Val::Px(65.0)),
                // center button
                margin: Rect::all(Val::Auto),
                // horizontally center child text
                justify_content: JustifyContent::Center,
                // vertically center child text
                align_items: AlignItems::Center,
                position_type: PositionType::Absolute,
                position: Rect {
                    top: Val::Percent(5.0 * (position as f32)),
                    left: if horizontal_position == HorizontalPosition::Left {
                        Val::Percent(0.2)
                    } else {
                        Val::Auto
                    },
                    right: if horizontal_position == HorizontalPosition::Right {
                        Val::Percent(0.2)
                    } else {
                        Val::Auto
                    },
                    ..Default::default()
                },
                ..Default::default()
            },
            color: NORMAL_BUTTON.into(),
            ..Default::default()
        })
        .insert(InterfaceButtonComponent { interface_button })
        .insert(EditorTrashMarker)
        .with_children(|parent| {
            parent.spawn_bundle(TextBundle {
                text: Text::with_section(
                    title.clone(),
                    TextStyle {
                        font: asset_server.load(crate::FONT),
                        font_size: 40.0,
                        color: Color::rgb(0.9, 0.9, 0.9),
                    },
                    Default::default(),
                ),
                ..Default::default()
            });
        });
}

fn update_button_states(
    mut buttons: Query<(&mut UiColor, &InterfaceButtonComponent), Without<EntityButtonComponent>>,
    mut entity_buttons: Query<(&mut Style, &mut UiColor, &EntityButtonComponent)>,
    interface: Res<Interface>,
) {
    for (mut color, button) in buttons.iter_mut() {
        if !interface.selected_button.is_none()
            && &button.interface_button == interface.selected_button.as_ref().unwrap()
        {
            *color = PRESSED_BUTTON.into();
        } else if !interface.hovered_button.is_none()
            && &button.interface_button == interface.hovered_button.as_ref().unwrap()
        {
            *color = HOVERED_BUTTON.into();
        } else {
            *color = NORMAL_BUTTON.into();
        }
    }

    for (mut style, mut color, button) in entity_buttons.iter_mut() {
        style.display = match interface.selected_button {
            Some(b) => match b {
                InterfaceButton::Add => Display::Flex,
                _ => Display::None,
            },
            None => Display::None,
        };

        if interface.current_entity == button.entity_button {
            *color = PRESSED_BUTTON.into();
        } else {
            *color = NORMAL_BUTTON.into();
        }
    }
}

fn handle_button_interaction(
    interface_button_query: Query<
        (&Interaction, &InterfaceButtonComponent),
        (Changed<Interaction>, With<Button>),
    >,
    entity_button_query: Query<
        (&Interaction, &EntityButtonComponent),
        (Changed<Interaction>, With<Button>),
    >,
    mut interface: ResMut<Interface>,
    mut interface_event_writer: EventWriter<InterfaceActionEvent>,
) {
    for (interaction, button) in interface_button_query.iter() {
        match *interaction {
            Interaction::Clicked => {
                if !interface.selected_button.is_none()
                    && interface.selected_button.as_ref().unwrap() == &button.interface_button
                {
                    // Deselect button
                    interface.selected_button = None;
                } else {
                    match button.interface_button {
                        InterfaceButton::Camera => {
                            interface_event_writer.send(InterfaceActionEvent {
                                action: InterfaceAction::ToggleCamera,
                            })
                        }
                        InterfaceButton::Play => {
                            interface_event_writer.send(InterfaceActionEvent {
                                action: InterfaceAction::Play,
                            })
                        }
                        InterfaceButton::Select => (),
                        InterfaceButton::Add => interface_event_writer.send(InterfaceActionEvent {
                            action: InterfaceAction::Add,
                        }),
                        InterfaceButton::Remove => (),
                        InterfaceButton::CopyProps => (),
                        InterfaceButton::PasteProps => (),
                        InterfaceButton::LevelProps => (),
                        InterfaceButton::System => (),
                    }
                }
            }
            Interaction::Hovered => {
                interface.hovered_button = Some(button.interface_button);
            }
            Interaction::None => {
                // Hovered button is no longer hovered, so reset its color to normal
                if !interface.hovered_button.is_none()
                    && interface.hovered_button.as_ref().unwrap() == &button.interface_button
                {
                    interface.hovered_button = None;
                }
            }
            _ => (),
        }
    }

    for (interaction, button) in entity_button_query.iter() {
        match *interaction {
            Interaction::Clicked => {
                interface.current_entity = button.entity_button;
            }
            _ => (),
        }
    }
}

fn handle_interface_events(
    mut interface_event_reader: EventReader<InterfaceActionEvent>,
    mut help_text_box_event_writer: EventWriter<HelpTextBoxEvent>,
    mut interface: ResMut<Interface>,
    mut state: ResMut<State<crate::AppState>>,
    game_entities: Query<(&Transform, &GameEntity)>,
) {
    for event in interface_event_reader.iter() {
        match event.action {
            InterfaceAction::ToggleCamera => {
                interface.selected_button = match interface.current_state() {
                    InterfaceMode::Camera => {
                        help_text_box_event_writer.send(HelpTextBoxEvent { text: None });

                        None
                    }
                    _ => {
                        help_text_box_event_writer.send(HelpTextBoxEvent {
                            text: Some("Press 'C' to exit Camera mode".to_string()),
                        });

                        Some(InterfaceButton::Camera)
                    }
                };
            }
            InterfaceAction::Play => {
                println!("Pushing editor play state");
                state.push(crate::AppState::EditorPlay).unwrap();
            }
            InterfaceAction::Add => {
                help_text_box_event_writer.send(HelpTextBoxEvent { text: None });
                interface.selected_button = match interface.current_state() {
                    InterfaceMode::Add => None,
                    _ => Some(InterfaceButton::Add),
                };
            }
            _ => (),
        }
    }
}

fn cleanup_editor(editor_elements: Query<Entity, With<EditorTrashMarker>>) {}

fn handle_keyboard_input(
    mut commands: Commands,
    mut keyboard_input: ResMut<Input<KeyCode>>,
    mut interface_event_writer: EventWriter<InterfaceActionEvent>,
    mut cooldown: Local<usize>,
) {
    if *cooldown != 0 {
        *cooldown -= 1;
        return;
    }

    if keyboard_input.pressed(KeyCode::C) {
        println!("C pressed");
        interface_event_writer.send(InterfaceActionEvent {
            action: InterfaceAction::ToggleCamera,
        });
    }

    if keyboard_input.pressed(KeyCode::A) {
        interface_event_writer.send(InterfaceActionEvent {
            action: InterfaceAction::Add,
        });
    }

    if keyboard_input.get_pressed().len() != 0 {
        *cooldown = 10;
    }

    keyboard_input.clear();
}
