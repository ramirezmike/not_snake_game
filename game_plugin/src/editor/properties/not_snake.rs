use bevy::prelude::*;
use bevy_inspector_egui::bevy_egui::egui;

#[derive(Component, Copy, Clone)]
pub struct NotSnakeProperties {
    pub color: [f32; 3],
}

impl Default for NotSnakeProperties {
    fn default() -> Self {
        let color = Color::hex("f3a787").unwrap();

        NotSnakeProperties {
            color: [color.r(), color.g(), color.b()], 
        }
    }
}

impl NotSnakeProperties {
    pub fn render(&mut self, ui: &mut egui::Ui) {
        ui.label("Color: ");
        ui.color_edit_button_rgb(&mut self.color);
        ui.end_row();
    }
}
