use std::borrow::Cow;

use bevy::{
    prelude::*,
    reflect::TypeUuid,
    render::{
        render_graph::{Node, ResourceSlotInfo},
        renderer::{RenderResourceId, RenderResourceType},
        texture::{SamplerDescriptor, TextureDescriptor, SAMPLER_ASSET_INDEX, TEXTURE_ASSET_INDEX},
    },
};

pub const RENDER_TEXTURE_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Texture::TYPE_UUID, 13378939762009864029);

pub const TEXTURE_NODE: &str = "texure_node";
pub const DEPTH_TEXTURE_NODE: &str = "depth_texure_node";

pub struct TextureNode {
    pub texture_descriptor: TextureDescriptor,
    pub sampler_descriptor: Option<SamplerDescriptor>,
    pub handle: Option<HandleUntyped>,
}

impl TextureNode {
    pub const TEXTURE: &'static str = "texture";

    pub fn new(
        texture_descriptor: TextureDescriptor,
        sampler_descriptor: Option<SamplerDescriptor>,
        handle: Option<HandleUntyped>,
    ) -> Self {
        Self {
            texture_descriptor,
            sampler_descriptor,
            handle,
        }
    }
}

impl Node for TextureNode {
    fn output(&self) -> &[ResourceSlotInfo] {
        static OUTPUT: &[ResourceSlotInfo] = &[ResourceSlotInfo {
            name: Cow::Borrowed(TextureNode::TEXTURE),
            resource_type: RenderResourceType::Texture,
        }];
        OUTPUT
    }

    fn update(
        &mut self,
        _world: &World,
        _resources: &Resources,
        render_context: &mut dyn bevy::render::renderer::RenderContext,
        _input: &bevy::render::render_graph::ResourceSlots,
        output: &mut bevy::render::render_graph::ResourceSlots,
    ) {
        if output.get(0).is_none() {
            let render_resource_context = render_context.resources_mut();
            let texture_id = render_resource_context.create_texture(self.texture_descriptor);
            if let Some(handle) = &self.handle {
                println!("handle yes");
                render_resource_context.set_asset_resource_untyped(
                    handle.clone(),
                    RenderResourceId::Texture(texture_id),
                    TEXTURE_ASSET_INDEX,
                );
                if let Some(sampler_descriptor) = self.sampler_descriptor {
                    let sampler_id = render_resource_context.create_sampler(&sampler_descriptor);
                    render_resource_context.set_asset_resource_untyped(
                        handle.clone(),
                        RenderResourceId::Sampler(sampler_id),
                        SAMPLER_ASSET_INDEX,
                    );
                }
            }
            output.set(0, RenderResourceId::Texture(texture_id));
        }
    }
}
