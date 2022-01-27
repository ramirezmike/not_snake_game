// disable console on windows for release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(target_arch = "wasm32")]
use bevy_webgl2;

use bevy::prelude::{App, ClearColor, Color, Msaa};
use game_plugin::GamePlugin;

pub static COLOR_BLACK: &str = "000000";

fn main() {
    let mut app = App::new();
    app
        .insert_resource(Msaa { samples: 1 })
        .insert_resource(ClearColor(Color::hex(COLOR_BLACK).unwrap()))
//      .insert_resource(WindowDescriptor {
//          width: 800.,
//          height: 600.,
//          title: "Bevy game".to_string(), // ToDo
//          ..Default::default()
//      })
        .add_plugin(GamePlugin);

    #[cfg(target_arch = "wasm32")]
    app.add_plugin(bevy_webgl2::WebGL2Plugin);

    app.run();
}
