use crate::{assets::GameAssets, AppState, title_screen, ui::text_size, menus, cleanup};
use bevy::{asset::Asset, ecs::system::SystemParam, gltf::Gltf, prelude::*};
use bevy_kira_audio::AudioSource;
use std::marker::PhantomData;

pub struct AssetLoadingPlugin;
impl Plugin for AssetLoadingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<NextState>()
            .init_resource::<AssetsLoading>()
            .add_system_set(SystemSet::on_enter(AppState::Loading).with_system(setup))
            .add_system_set(SystemSet::on_exit(AppState::Loading).with_system(cleanup::<CleanupMarker>))
            .add_system_set(
                SystemSet::on_update(AppState::Loading).with_system(check_assets_ready),
            );
    }
}

#[derive(Default)]
pub struct GameTexture {
    pub material: Handle<StandardMaterial>,
    pub image: Handle<Image>,
}

#[derive(Component)]
struct CleanupMarker;

pub struct NextState {
    state: AppState,
}
impl Default for NextState {
    fn default() -> Self {
        NextState {
            state: AppState::Splash,
        }
    }
}

#[derive(Default)]
pub struct AssetsLoading {
    pub asset_handles: Vec<HandleUntyped>,
}

#[derive(SystemParam)]
pub struct AssetsHandler<'w, 's> {
    asset_server: Res<'w, AssetServer>,
    assets_loading: ResMut<'w, AssetsLoading>,
    materials: ResMut<'w, Assets<StandardMaterial>>,
    state: ResMut<'w, State<AppState>>,
    next_state: ResMut<'w, NextState>,

    #[system_param(ignore)]
    phantom: PhantomData<&'s ()>,
}

impl<'w, 's> AssetsHandler<'w, 's> {
    pub fn add_asset<T: Asset>(&mut self, asset: &mut Handle<T>, path: &str) {
        *asset = self.asset_server.load(path);
        self.asset_server.watch_for_changes().unwrap();
        self.assets_loading
            .asset_handles
            .push(asset.clone_untyped());
    }

    pub fn load(&mut self, next_state: AppState, game_assets: &mut ResMut<GameAssets>) {
        self.queue_assets_for_state(&next_state, game_assets);
        self.next_state.state = next_state;
        println!("watching for changes");
        self.asset_server.watch_for_changes().unwrap();
        self.state.set(AppState::Loading).unwrap();
    }

    pub fn add_mesh(&mut self, mesh: &mut Handle<Mesh>, path: &str) {
        self.add_asset(mesh, path);
    }

    pub fn add_font(&mut self, font: &mut Handle<Font>, path: &str) {
        self.add_asset(font, path);
    }

    pub fn add_audio(&mut self, audio: &mut Handle<AudioSource>, path: &str) {
        self.add_asset(audio, path);
    }

    pub fn add_glb(&mut self, glb: &mut Handle<Gltf>, path: &str) {
        self.add_asset(glb, path);
    }

    pub fn add_material(&mut self, game_texture: &mut GameTexture, path: &str, transparent: bool) {
        self.add_asset(&mut game_texture.image, path);
        game_texture.material = self.materials.add(StandardMaterial {
            base_color_texture: Some(game_texture.image.clone()),
            alpha_mode: if transparent {
                AlphaMode::Blend
            } else {
                AlphaMode::Opaque
            },
            ..Default::default()
        });
    }

    fn queue_assets_for_state(&mut self, state: &AppState, game_assets: &mut ResMut<GameAssets>) {
        match state {
            AppState::MainMenu => title_screen::load(self, game_assets),
            _ => (),
        }
    }
}

fn check_assets_ready(mut assets_handler: AssetsHandler) {
    use bevy::asset::LoadState;

    let mut ready = true;
    for handle in assets_handler.assets_loading.asset_handles.iter() {
        match assets_handler.asset_server.get_load_state(handle) {
            LoadState::Failed => {
                println!("An asset had an error: {:?}", handle);
            }
            LoadState::Loaded => {}
            _ => {
                ready = false;
            }
        }
    }

    if ready {
        assets_handler.assets_loading.asset_handles = vec![]; // clear list since we've loaded everything
        assets_handler
            .state
            .set(assets_handler.next_state.state)
            .unwrap(); // move to next state
    }
}

fn setup(mut commands: Commands, game_assets: Res<GameAssets>, text_scaler: text_size::TextScaler) {
    commands
        .spawn_bundle(UiCameraBundle::default())
        .insert(CleanupMarker);

    commands
        .spawn_bundle(TextBundle {
            style: Style {
                align_self: AlignSelf::FlexEnd,
                position_type: PositionType::Absolute,
                position: Rect {
                    bottom: Val::Px(5.0),
                    left: Val::Px(15.0),
                    ..Default::default()
                },
                ..Default::default()
            },
            text: Text::with_section(
                "Loading...".to_string(),
                TextStyle {
                    font: game_assets.font.clone(),
                    font_size: text_scaler.scale(menus::DEFAULT_FONT_SIZE),
                    color: Color::WHITE,
                },
                TextAlignment::default(),
            ),
            ..Default::default()
        })
        .insert(CleanupMarker);
}

