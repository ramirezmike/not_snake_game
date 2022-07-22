use bevy::{ecs::system::SystemParam, prelude::*, window::WindowResized};
use std::marker::PhantomData;

pub struct TextSizePlugin;
impl Plugin for TextSizePlugin {
    fn build(&self, app: &mut App) {
        app.add_system(store_current_window_size)
            .insert_resource(WindowSize {
                width: 0.0,
                height: 0.0,
            });
    }
}

#[derive(SystemParam)]
pub struct TextScaler<'w, 's> {
    window_size: Res<'w, WindowSize>,

    #[system_param(ignore)]
    phantom: PhantomData<&'s ()>,
}

impl<'w, 's> TextScaler<'w, 's> {
    pub fn scale(&self, font_size: f32) -> f32 {
        let dev_window_width_in_px = 1276.0;
        (font_size / dev_window_width_in_px) * self.window_size.width
    }

    pub fn width_percent_to_px(&self, percent: f32) -> f32 {
        self.window_size.width * (percent / 100.0)
    }

    pub fn height_percent_to_px(&self, percent: f32) -> f32 {
        self.window_size.width * (percent / 100.0)
    }
}

pub struct WindowSize {
    width: f32,
    height: f32,
}

fn store_current_window_size(
    windows: Res<Windows>,
    mut win_size: ResMut<WindowSize>,
    mut resize_event: EventReader<WindowResized>,
) {
    if win_size.width == 0.0 && win_size.height == 0.0 {
        if let Some(window) = windows.get_primary() {
            win_size.width = window.width();
            win_size.height = window.height();
        }
    }

    for e in resize_event.iter() {
        win_size.width = e.width;
        win_size.height = e.height;
    }
}
