use bevy::prelude::*;
use bevy::window::WindowResized;
use crate::AppState;
use crate::editor::{select_entity, properties};
use bevy_inspector_egui::bevy_egui::{egui, EguiContext, EguiPlugin};

pub struct EditorInterfacePlugin;
impl Plugin for EditorInterfacePlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
               SystemSet::on_update(AppState::Editor)
                   .with_system(store_current_window_size)
                   .with_system(select_entity::handle_entity_click_events.before("detect"))
                   .with_system(select_entity::detect_entity_selections.label("detect"))
                   .with_system(select_entity::store_selected_values.label("select_store").after("detect"))
                   .with_system(paint_ui.label("paint").after("select_store"))
                   .with_system(properties::apply_properties_to_selected_block.after("paint"))
                   .with_system(properties::apply_properties_to_selected_not_snake.after("paint"))
                   .with_system(properties::apply_properties_to_selected_snake.after("paint"))
           )
           .add_plugin(EguiPlugin)
           .insert_resource(properties::Properties::new())
           .insert_resource(Interface {
               show_level_properties: false,
           })
           .insert_resource(WindowSize {
               width: 0.0,
               height: 0.0,
           })
           .insert_resource(EntitySelection::None)
           .insert_resource(EntityAction::Select);
    }
}

// TODO: Rename this to InterfaceAction?
#[derive(PartialEq, Clone, Copy)]
pub enum EntityAction {
    Select,
    Add,
    Delete,
}

// TODO: Rename this to SelectedEntityType?
#[derive(PartialEq, Debug)]
pub enum EntitySelection {
    Block,
    Snake,
    NotSnake,
    Food,
    None,
}

struct Interface {
    show_level_properties: bool
}

struct WindowSize {
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

fn paint_ui(
    ctx: Res<EguiContext>,
    win_size: Res<WindowSize>,
    mut entity_action: ResMut<EntityAction>,
    mut entity_selection: ResMut<EntitySelection>,
    mut state: ResMut<State<AppState>>,
    mut properties: ResMut<properties::Properties>,
    mut interface: ResMut<Interface>,
) {
    let ctx = ctx.ctx();

    let mut fonts = egui::FontDefinitions::default();

    let base_font_size = 20.0;

    fonts.family_and_size.insert(
        egui::TextStyle::Button,
        (egui::FontFamily::Proportional, base_font_size)
    );
    fonts.family_and_size.insert(
        egui::TextStyle::Small,
        (egui::FontFamily::Proportional, base_font_size)
    );
    fonts.family_and_size.insert(
        egui::TextStyle::Body,
        (egui::FontFamily::Proportional, base_font_size * 1.5)
    );
    fonts.family_and_size.insert(
        egui::TextStyle::Heading,
        (egui::FontFamily::Proportional, base_font_size * 1.2)
    );

    // use custom font
    fonts.font_data.insert("custom".to_owned(),
       egui::FontData::from_static(include_bytes!("../../../assets/fonts/monogram.ttf"))); // .ttf and .otf supported

    // put custom font first
    fonts.fonts_for_family.get_mut(&egui::FontFamily::Proportional).unwrap()
        .insert(0, "custom".to_owned());

    ctx.set_fonts(fonts);

    egui::Window::new("")
        .anchor(egui::Align2::LEFT_TOP, [0.0, 0.0])
        .resizable(false)
        .title_bar(false)
        .fixed_size([win_size.width / 7.0, win_size.height])
        .collapsible(false)
        .show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.menu_button("File", |ui| {
                        if ui.button("Quit").clicked() {
                            panic!("Quit hit");
                        }
                    });
                    if ui.button("Level").clicked() {
                        interface.show_level_properties = true;
                    }
                    if ui.button("Play").clicked() {
                        state.set(AppState::EditorPlay).unwrap();
                    }
                });
            });

            ui.separator();

            ui.heading("Action");

            ui.vertical(|ui| {
              ui.selectable_value(&mut *entity_action, EntityAction::Select, "Select");
              ui.horizontal(|ui| {
                  ui.selectable_value(&mut *entity_action, EntityAction::Add, "Create");
              });
              ui.selectable_value(&mut *entity_action, EntityAction::Delete, "Delete");
            });

            ui.separator();

            ui.heading("Properties");
            if *entity_selection != EntitySelection::None || *entity_action == EntityAction::Add {
              egui::ComboBox::from_label("")
                             .selected_text(format!("{:?}", *entity_selection))
                             .show_ui(ui, |ui| {
                                 ui.selectable_value(&mut *entity_selection, EntitySelection::Block, "Block");
                                 ui.selectable_value(&mut *entity_selection, EntitySelection::NotSnake, "Not Snake");
                                 ui.selectable_value(&mut *entity_selection, EntitySelection::Snake, "Snake");
                                 ui.selectable_value(&mut *entity_selection, EntitySelection::Food, "Food");
                             });

              properties.draw_property_ui(&entity_selection, ui);
            } else {
              ui.label("-----");
            }

            ui.separator();

            // put something at the bottom to make it pane like
            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                ui.separator();
            });
        });

    egui::Window::new("Level Properties")
        .open(&mut interface.show_level_properties)
        .resizable(false)
        .collapsible(false)
        .default_width(300.0)
        .show(ctx, |ui| {
            ui.label("Level Title:");
            ui.add(egui::TextEdit::singleline(&mut properties.level_title));
            ui.end_row();
            ui.checkbox(&mut properties.is_food_random, "Foods Spawn Randomly");
            ui.end_row();
            ui.label("Goal Points:");
            ui.add(egui::DragValue::new(&mut properties.minimum_food).speed(1.0));
            ui.end_row();
            ui.label("Background: ");
            ui.color_edit_button_rgb(&mut properties.background_color);
            ui.end_row();
        });
}
