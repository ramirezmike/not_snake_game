use crate::editor::{
    editor_camera, help_text::HelpTextBoxEvent, play, EditorTrashMarker, GameEntity,
};
use crate::AppState;
use bevy::prelude::*;
use bevy_inspector_egui::bevy_egui::{egui, EguiContext, EguiPlugin};

pub struct EditorInterfacePlugin;
impl Plugin for EditorInterfacePlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
               SystemSet::on_update(AppState::Editor)
                   .with_system(paint_ui)
           )
           .add_plugin(EguiPlugin)
           .insert_resource(EntityAction::Select)
           .insert_resource(EntityType::Block);
    }
}

#[derive(PartialEq, Clone, Copy)]
pub enum EntityAction {
    Select,
    Add,
    Delete,
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum EntityType {
    Block,
    NotSnake,
    Snake,
    Food
}

fn paint_ui(
    mut ctx: ResMut<EguiContext>,
    mut entity_action: ResMut<EntityAction>,
    mut entity_type: ResMut<EntityType>,
    mut state: ResMut<State<AppState>>,
) {
    let ctx = ctx.ctx();

    let mut style: egui::Style = (*ctx.style()).clone();
    style.body_text_style = egui::TextStyle::Heading;
    style.override_text_style = Some(egui::TextStyle::Heading);
    ctx.set_style(style);

    egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
        // The top panel is often a good place for a menu bar:
        egui::menu::bar(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("Quit").clicked() {
                    println!("Quit hit");
                }
            });
        });
    });

    egui::SidePanel::left("side_panel").show(ctx, |ui| {

        ui.separator();

        ui.heading("Entities");

        ui.vertical(|ui| {
            ui.selectable_value(&mut *entity_action, EntityAction::Select, "Select");
            ui.horizontal(|ui| {
                ui.selectable_value(&mut *entity_action, EntityAction::Add, "Add");
                if *entity_action == EntityAction::Add {
                    egui::ComboBox::from_label("")
                                   .selected_text(format!("{:?}", *entity_type))
                                   .show_ui(ui, |ui| {
                                       ui.selectable_value(&mut *entity_type, EntityType::Block, "Block");
                                       ui.selectable_value(&mut *entity_type, EntityType::NotSnake, "Not Snake");
                                       ui.selectable_value(&mut *entity_type, EntityType::Snake, "Snake");
                                       ui.selectable_value(&mut *entity_type, EntityType::Food, "Food");
                                   });
                }
            });
            ui.selectable_value(&mut *entity_action, EntityAction::Delete, "Delete");
        });

        ui.separator();

        ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
            if ui.button("Play").clicked() {
                state.set(AppState::EditorPlay).unwrap();
            }
        });

        ui.separator();
    });
}
