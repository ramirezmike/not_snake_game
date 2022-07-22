use bevy::prelude::*;
use crate::{ui, menus};

pub fn add_text(
    commands: &mut Commands,
    font: Handle<Font>,
    text_scaler: &ui::text_size::TextScaler,
    mut components: Vec<impl Component>,
) {
    let mut text_bundle = 
        commands
            .spawn_bundle(TextBundle {
                style: Style {
                    align_self: AlignSelf::FlexEnd,
                    position_type: PositionType::Relative,
                    margin: Rect {
                        left: Val::Auto,
                        right: Val::Auto,
                        top: Val::Auto,
                        bottom: Val::Auto,
                        ..Default::default()
                    },
                    max_size: Size {
                        width: Val::Px(text_scaler.width_percent_to_px(75.0)),
                        height: Val::Undefined,
                    },
                    ..default()
                },
                text: Text::with_section(
                    "".to_string(),
                    TextStyle {
                        font,
                        font_size: text_scaler.scale(menus::DEFAULT_FONT_SIZE * 1.2),
                        color: Color::WHITE,
                    },
                    TextAlignment {
                        horizontal: HorizontalAlign::Center,
                        vertical: VerticalAlign::Center,
                    },
                ),
                ..Default::default()
            });

    components.drain(..).for_each(|c| {
        text_bundle.insert(c);
    });
}

