use crate::snake::SnakeTarget;
use bevy::prelude::*;
use bevy_inspector_egui::bevy_egui::egui;
use serde::{Serialize, Deserialize};

#[derive(Component, Copy, Clone, Serialize, Deserialize)]
pub struct SnakeProperties {
    pub color: [f32; 3],
    pub speed: f32,
    pub target: SnakeTarget,
    pub min_length: usize,
}

impl Default for SnakeProperties {
    fn default() -> Self {
        let color = Color::hex("ff4f69").unwrap();

        SnakeProperties {
            color: [color.r(), color.g(), color.b()],
            speed: 0.5,
            target: SnakeTarget::Normal,
            min_length: 5,
        }
    }
}

impl SnakeProperties {
    pub fn render(&mut self, ui: &mut egui::Ui) {
        ui.label("Speed: ");
        ui.add(egui::Slider::new(&mut self.speed, 0.1..=2.0));
        ui.end_row();

        ui.label("Length: ");
        ui.add(egui::Slider::new(&mut self.min_length, 2..=20));
        ui.end_row();

        ui.label("Target: ");
        egui::ComboBox::from_id_source(123)
            .selected_text(format!("{:?}", SnakeTarget::Normal))
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut self.target, SnakeTarget::Normal, "Normal");
                ui.selectable_value(&mut self.target, SnakeTarget::OnlyFood, "Only Food");
                ui.selectable_value(&mut self.target, SnakeTarget::OnlyDude, "Not Snake");
                ui.selectable_value(&mut self.target, SnakeTarget::OnlyRandom, "Random");
            });
        ui.end_row();

        ui.label("Color: ");
        ui.color_edit_button_rgb(&mut self.color);
        ui.end_row();
    }
}
