use bevy::prelude::*;
use crate::{AppState, editor::EditorTrashMarker};

#[derive(Component)]
pub struct HelpTextBox;
#[derive(Component)]
pub struct HelpText;
pub struct HelpTextBoxEvent {
    pub text: Option::<String> // hide if empty
}

pub struct HelpTextPlugin;
impl Plugin for HelpTextPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_enter(AppState::Editor)
                       .with_system(create_help_text_box)
        )
        .add_system_set(
            SystemSet::on_update(AppState::Editor)
                       .with_system(handle_help_text_box_event)
        )
       .add_event::<HelpTextBoxEvent>();
    }
}

fn create_help_text_box(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(30.0)),
                ..Default::default()
            },
            color: UiColor(Color::NONE),
            visibility: Visibility {
                is_visible: false,
            },
            ..Default::default()
        })
        .insert(HelpTextBox)
        .insert(EditorTrashMarker)
        .with_children(|parent| {
            parent
                .spawn_bundle(NodeBundle {
                    style: Style {
                        size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                        border: Rect::all(Val::Px(2.0)),
                        ..Default::default()
                    },
                    visibility: Visibility {
                        is_visible: false,
                    },
                    color: UiColor(Color::rgb(0.65, 0.65, 0.65)),
                    ..Default::default()
                })
                .insert(HelpTextBox)
                .with_children(|parent| {
                    parent
                        .spawn_bundle(NodeBundle {
                            style: Style {
                                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                                align_items: AlignItems::FlexEnd,
                                ..Default::default()
                            },
                            color: UiColor(Color::rgb(0.15, 0.15, 0.15)),
                            visibility: Visibility {
                                is_visible: false,
                            },
                            ..Default::default()
                        })
                        .insert(HelpTextBox)
                        .with_children(|parent| {
                            parent.spawn_bundle(TextBundle {
                                style: Style {
                                    margin: Rect::all(Val::Px(5.0)),
                                    max_size: Size {
                                        width: Val::Px(1280.0),
                                        height: Val::Undefined,
                                    },
                                    ..Default::default()
                                },
                                text: Text::with_section(
                                    "",
                                    TextStyle {
                                        font: asset_server.load(crate::FONT),
                                        font_size: 50.0,
                                        color: Color::WHITE,

                                    },
                                    Default::default(),
                                ),
                                ..Default::default()
                            })
                            .insert(HelpText);
                        });
                });
        });
}

fn handle_help_text_box_event(
    mut help_text_box_event_reader: EventReader<HelpTextBoxEvent>, 
    mut textbox_visibility: Query<&mut Visibility, With<HelpTextBox>>,
    mut textbox_text: Query<&mut Text, With<HelpText>>,
) {
    for event in help_text_box_event_reader.iter() {
        if let Some(text_to_display) = &event.text {
            for mut textbox_text in textbox_text.iter_mut() {
                textbox_text.sections[0].value = text_to_display.to_string();
            }
            for mut visibility in textbox_visibility.iter_mut() {
                visibility.is_visible = true;
            }
        } else {
            for mut textbox_text in textbox_text.iter_mut() {
                textbox_text.sections[0].value = "".to_string();
            }
            for mut visibility in textbox_visibility.iter_mut() {
                visibility.is_visible = false;
            }
        }
    }
}
