use crate::editor::interface::{EntityAction, EntitySelection};
use bevy::prelude::*;
use bevy_inspector_egui::bevy_egui::egui;
use bevy_mod_picking::*;
use serde::{Serialize, Deserialize};

pub mod block;
pub mod food;
pub mod not_snake;
pub mod snake;

#[derive(Serialize, Deserialize, Clone)]
pub struct Properties {
    pub level_title: String,
    pub is_food_random: bool,
    pub minimum_food: usize,
    pub background_color: [f32; 3],
    pub block: block::BlockProperties,
    pub not_snake: not_snake::NotSnakeProperties,
    pub snake: snake::SnakeProperties,
    pub food: food::FoodProperties,
}
impl Properties {
    pub fn new() -> Self {
        Properties {
            level_title: "Untitled".to_string(),
            is_food_random: false,
            minimum_food: 1,
            background_color: [0.0, 0.0, 0.0],
            block: block::BlockProperties::default(),
            not_snake: not_snake::NotSnakeProperties::default(),
            snake: snake::SnakeProperties::default(),
            food: food::FoodProperties::default(),
        }
    }

    pub fn draw_property_ui(
        &mut self,
        entity_selection: &ResMut<EntitySelection>,
        ui: &mut egui::Ui,
    ) {
        match **entity_selection {
            EntitySelection::Block => self.block.render(ui),
            EntitySelection::NotSnake => self.not_snake.render(ui),
            EntitySelection::Snake => self.snake.render(ui),
            EntitySelection::Food => self.food.render(ui),
            _ => (),
        }
    }
}

pub fn apply_properties_to_selected_block(
    entity_action: Res<EntityAction>,
    entity_selection: Res<EntitySelection>,
    properties: Res<Properties>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut selected: Query<(
        &Selection,
        &mut PickableButton<StandardMaterial>,
        &mut block::BlockProperties,
    )>,
) {
    if *entity_action != EntityAction::Select || *entity_selection != EntitySelection::Block {
        return;
    }

    for (selection, mut button, mut block_properties) in selected.iter_mut() {
        if selection.selected() {
            let color = properties.block.color;
            button.initial = Some(materials.add(Color::rgb(color[0], color[1], color[2]).into()));
            *block_properties = properties.block.clone();
        }
    }
}

pub fn apply_properties_to_selected_not_snake(
    entity_action: Res<EntityAction>,
    entity_selection: Res<EntitySelection>,
    properties: Res<Properties>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut selected: Query<(
        &Selection,
        &mut PickableButton<StandardMaterial>,
        &mut not_snake::NotSnakeProperties,
    )>,
) {
    if *entity_action != EntityAction::Select || *entity_selection != EntitySelection::NotSnake {
        return;
    }

    for (selection, mut button, mut not_snake_properties) in selected.iter_mut() {
        if selection.selected() {
            let color = properties.not_snake.color;
            button.initial = Some(materials.add(Color::rgb(color[0], color[1], color[2]).into()));
            *not_snake_properties = properties.not_snake.clone();
        }
    }
}

pub fn apply_properties_to_selected_snake(
    entity_action: Res<EntityAction>,
    entity_selection: Res<EntitySelection>,
    properties: Res<Properties>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut selected: Query<(
        &Selection,
        &mut PickableButton<StandardMaterial>,
        &mut snake::SnakeProperties,
    )>,
) {
    if *entity_action != EntityAction::Select || *entity_selection != EntitySelection::Snake {
        return;
    }

    for (selection, mut button, mut snake_properties) in selected.iter_mut() {
        if selection.selected() {
            let color = properties.snake.color;
            button.initial = Some(materials.add(Color::rgb(color[0], color[1], color[2]).into()));
            *snake_properties = properties.snake.clone();
        }
    }
}
