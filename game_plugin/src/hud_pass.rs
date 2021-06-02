// Taken from https://github.com/superdump/bevy-hud-pass

use bevy::{
    prelude::*,
    render::{
        camera::{ActiveCameras, Camera, PerspectiveProjection, VisibleEntities},
        pass::{
            LoadOp, Operations, PassDescriptor, RenderPassDepthStencilAttachmentDescriptor,
            TextureAttachment,
        },
        render_graph::{
            base, CameraNode, PassNode, RenderGraph, RenderResourcesNode, WindowSwapChainNode,
            WindowTextureNode,
        },
        renderer::RenderResources,
    },
    ui,
};

pub mod camera {
    /// The name of the camera used in the HUD pass
    pub const CAMERA_HUD: &str = "camera_hud";
}

pub mod node {
    pub const CAMERA_HUD: &str = "camera_hud";
    pub const HUD_PASS: &str = "hud_pass";
    pub const HUD_MESH: &str = "hud_mesh";
}

pub const HUD_SETUP_SYSTEM: &str = "hud_setup";

/// Add a HUDPass component to an entity to have it render in the HUD pass
#[derive(Debug, Clone, Default, RenderResources)]
pub struct HUDPass;

/// Just a PerspectiveCameraBundle with `Default` providing the correct name for the
/// HUD pass
#[derive(Bundle, Debug)]
pub struct HUDCameraBundle {
    pub camera: Camera,
    pub perspective_projection: PerspectiveProjection,
    pub visible_entities: VisibleEntities,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
}

impl Default for HUDCameraBundle {
    fn default() -> Self {
        let PerspectiveCameraBundle {
            camera,
            perspective_projection,
            visible_entities,
            transform,
            global_transform,
        } = PerspectiveCameraBundle::with_name(&camera::CAMERA_HUD.to_string());
        Self {
            camera,
            perspective_projection,
            visible_entities,
            transform,
            global_transform,
        }
    }
}

/// Just a PbrBundle but with a HUDPass component instead of a MainPass component
/// so that the mesh is rendered by the HUD pass
#[derive(Bundle)]
pub struct HUDPbrBundle {
    pub mesh: Handle<Mesh>,
    pub material: Handle<StandardMaterial>,
    pub hud_pass: HUDPass,
    pub draw: Draw,
    pub visible: Visible,
    pub render_pipelines: RenderPipelines,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
}

impl Default for HUDPbrBundle {
    fn default() -> Self {
        let PbrBundle {
            mesh,
            material,
            draw,
            visible,
            render_pipelines,
            transform,
            global_transform,
            ..
        } = PbrBundle::default();
        Self {
            mesh,
            material,
            hud_pass: HUDPass,
            draw,
            visible,
            render_pipelines,
            transform,
            global_transform,
        }
    }
}

pub struct HUDPassPlugin;

impl Plugin for HUDPassPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_startup_system(hud_setup.system().label(HUD_SETUP_SYSTEM));
    }
}

fn hud_setup(
    mut graph: ResMut<RenderGraph>,
    mut active_cameras: ResMut<ActiveCameras>,
    msaa: Res<Msaa>,
) {
    let mut hud_pass_node = PassNode::<&HUDPass>::new(PassDescriptor {
        color_attachments: vec![msaa.color_attachment_descriptor(
            TextureAttachment::Input("color_attachment".to_string()),
            TextureAttachment::Input("color_resolve_target".to_string()),
            Operations {
                load: LoadOp::Load,
                store: true,
            },
        )],
        depth_stencil_attachment: Some(RenderPassDepthStencilAttachmentDescriptor {
            attachment: TextureAttachment::Input("depth".to_string()),
            depth_ops: Some(Operations {
                // NOTE: Clearing here makes everything in this pass be drawn on top of things in the main pass
                load: LoadOp::Clear(1.0),
                store: true,
            }),
            stencil_ops: None,
        }),
        sample_count: msaa.samples,
    });

    hud_pass_node.add_camera(node::CAMERA_HUD);
    graph.add_node(node::HUD_PASS, hud_pass_node);

    graph
        .add_slot_edge(
            base::node::PRIMARY_SWAP_CHAIN,
            WindowSwapChainNode::OUT_TEXTURE,
            node::HUD_PASS,
            if msaa.samples > 1 {
                "color_resolve_target"
            } else {
                "color_attachment"
            },
        )
        .unwrap();

    graph
        .add_slot_edge(
            base::node::MAIN_DEPTH_TEXTURE,
            WindowTextureNode::OUT_TEXTURE,
            node::HUD_PASS,
            "depth",
        )
        .unwrap();

    if msaa.samples > 1 {
        graph
            .add_slot_edge(
                base::node::MAIN_SAMPLED_COLOR_ATTACHMENT,
                WindowSwapChainNode::OUT_TEXTURE,
                node::HUD_PASS,
                "color_attachment",
            )
            .unwrap();
    }

    graph
        .add_node_edge(base::node::MAIN_PASS, node::HUD_PASS)
        .unwrap();
    graph
        .add_node_edge(node::HUD_PASS, ui::node::UI_PASS)
        .unwrap();

    graph.add_system_node(node::CAMERA_HUD, CameraNode::new(camera::CAMERA_HUD));
    graph
        .add_node_edge(node::CAMERA_HUD, node::HUD_PASS)
        .unwrap();
    graph.add_system_node(node::HUD_MESH, RenderResourcesNode::<HUDPass>::new(true));
    graph.add_node_edge(node::HUD_MESH, node::HUD_PASS).unwrap();
    active_cameras.add(camera::CAMERA_HUD);
}
