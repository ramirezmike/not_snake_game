use bevy::prelude::*;
use bevy_inspector_egui::bevy_egui::egui;

#[derive(Default, Component, Copy, Clone)]
pub struct FoodProperties {

}
impl FoodProperties {
    pub fn render(&mut self, ui: &mut egui::Ui) {
        ui.label("Food has no properties");
    }
}
