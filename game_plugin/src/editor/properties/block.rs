use bevy::prelude::*;
use bevy_inspector_egui::bevy_egui::egui;
use serde::{Serialize, Deserialize};

#[derive(Default, Component, Copy, Clone, Serialize, Deserialize)]
pub struct BlockProperties {
    pub moveable: bool,
    pub visible: bool,
    pub color: [f32; 3],
}
impl BlockProperties {
    pub fn render(&mut self, ui: &mut egui::Ui) {
        ui.checkbox(&mut self.visible, "Visible");
        ui.end_row();

        ui.checkbox(&mut self.moveable, "Moveable");
        ui.end_row();

        ui.label("Color: ");
        ui.color_edit_button_rgb(&mut self.color);
        ui.end_row();
    }
}
