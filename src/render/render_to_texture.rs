use bevy::{
    prelude::*,
    render::{
        camera::ActiveCameras,
        pass::{
            LoadOp, Operations, PassDescriptor, RenderPassColorAttachmentDescriptor,
            RenderPassDepthStencilAttachmentDescriptor, TextureAttachment,
        },
        render_graph::{base::node::MAIN_PASS, CameraNode, PassNode, RenderGraph},
        texture::{
            Extent3d, SamplerDescriptor, TextureDescriptor, TextureDimension, TextureFormat,
            TextureUsage,
        },
    },
};

pub use super::{TextureNode, DEPTH_TEXTURE_NODE, RENDER_TEXTURE_HANDLE, TEXTURE_NODE};

pub const POST_PASS: &str = "post_pass";
pub const POST_PASS_CAMERA: &str = "post_pass_camera";

#[derive(Default)]
pub struct PostPass;

pub trait RenderToTextureGraphBuilder {
    fn add_render_to_texture_graph(&mut self, resources: &Resources) -> &mut Self;
}

impl RenderToTextureGraphBuilder for RenderGraph {
    fn add_render_to_texture_graph(&mut self, resources: &Resources) -> &mut Self {
        let mut active_cameras = resources.get_mut::<ActiveCameras>().unwrap();

        let mut pass_node = PassNode::<&PostPass>::new(PassDescriptor {
            color_attachments: vec![RenderPassColorAttachmentDescriptor {
                attachment: TextureAttachment::Input("color_attachment".to_string()),
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(Color::rgb(1.0, 0.0, 1.0)),
                    store: true,
                },
            }],
            depth_stencil_attachment: Some(RenderPassDepthStencilAttachmentDescriptor {
                attachment: TextureAttachment::Input("depth".to_string()),
                depth_ops: Some(Operations {
                    load: LoadOp::Clear(1.0),
                    store: true,
                }),
                stencil_ops: None,
            }),
            sample_count: 1,
        });
        pass_node.add_camera(POST_PASS_CAMERA);

        self.add_node(POST_PASS, pass_node);
        self.add_system_node(POST_PASS_CAMERA, CameraNode::new(POST_PASS_CAMERA));

        active_cameras.add(POST_PASS_CAMERA);
        self.add_node_edge(POST_PASS_CAMERA, POST_PASS).unwrap();

        self.add_node(
            TEXTURE_NODE,
            TextureNode::new(
                TextureDescriptor {
                    size: Extent3d::new(1280, 720, 1),
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: TextureDimension::D2,
                    format: Default::default(),
                    usage: TextureUsage::OUTPUT_ATTACHMENT | TextureUsage::SAMPLED,
                },
                Some(SamplerDescriptor::default()),
                Some(RENDER_TEXTURE_HANDLE),
            ),
        );

        self.add_node(
            DEPTH_TEXTURE_NODE,
            TextureNode::new(
                TextureDescriptor {
                    size: Extent3d::new(1280, 720, 1),
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: TextureDimension::D2,
                    format: TextureFormat::Depth32Float,
                    usage: TextureUsage::OUTPUT_ATTACHMENT | TextureUsage::SAMPLED,
                },
                None,
                None,
            ),
        );

        self.add_node_edge(TEXTURE_NODE, MAIN_PASS).unwrap();
        self.add_slot_edge(
            TEXTURE_NODE,
            TextureNode::TEXTURE,
            MAIN_PASS,
            "color_attachment",
        )
        .unwrap();
        self.add_slot_edge(DEPTH_TEXTURE_NODE, TextureNode::TEXTURE, MAIN_PASS, "depth")
            .unwrap();

        self.add_node_edge(MAIN_PASS, POST_PASS).unwrap();
        self.add_slot_edge(
            bevy::render::render_graph::base::node::PRIMARY_SWAP_CHAIN,
            bevy::render::render_graph::WindowSwapChainNode::OUT_TEXTURE,
            POST_PASS,
            "color_attachment",
        )
        .unwrap();
        self.add_slot_edge(
            bevy::render::render_graph::base::node::MAIN_DEPTH_TEXTURE,
            bevy::render::render_graph::WindowTextureNode::OUT_TEXTURE,
            POST_PASS,
            "depth",
        )
        .unwrap();
        // self.add_node_edge("transform", POST_PASS).unwrap();

        self.add_node_edge(POST_PASS, "ui_pass").unwrap();
        self
    }
}
