use std::borrow::Cow;

use bevy::{
    prelude::*,
    reflect::TypeUuid,
    render::{
        camera::{ActiveCameras, Camera, CameraProjection},
        pass::{
            LoadOp, Operations, PassDescriptor, RenderPassColorAttachmentDescriptor,
            RenderPassDepthStencilAttachmentDescriptor, TextureAttachment,
        },
        pipeline::{
            BlendFactor, BlendOperation, BlendState, ColorTargetState, ColorWrite, CompareFunction,
            CullMode, DepthBiasState, DepthStencilState, FrontFace, PipelineDescriptor,
            PolygonMode, PrimitiveState, PrimitiveTopology, RenderPipeline, StencilFaceState,
            StencilState,
        },
        render_graph::{
            base::{node::MAIN_PASS, BaseRenderGraphConfig, MainPass},
            CameraNode, Node, PassNode, RenderGraph, ResourceSlotInfo,
        },
        renderer::{RenderResourceId, RenderResourceType},
        shader::{ShaderStage, ShaderStages},
        texture::{
            Extent3d, SamplerDescriptor, TextureDescriptor, TextureDimension, TextureFormat,
            TextureUsage, SAMPLER_ASSET_INDEX, TEXTURE_ASSET_INDEX,
        },
    },
    window::WindowId,
};

pub const RENDER_TEXTURE_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Texture::TYPE_UUID, 13378939762009864029);

pub const TEXTURE_NODE: &str = "texure_node";
pub const DEPTH_TEXTURE_NODE: &str = "depth_texure_node";
pub const FIRST_PASS: &str = "first_pass";
pub const FIRST_PASS_CAMERA: &str = "first_pass_camera";

const FRAGMENT_SHADER: &str = r#"
#version 450

layout(location = 0) in vec2 v_Uv;
layout(location = 1) in vec4 gl_FragCoord;

layout(location = 0) out vec4 o_Target;

layout(set = 1, binding = 0) uniform ColorMaterial_color {
    vec4 Color;
};

layout(set = 1, binding = 1) uniform texture2D ColorMaterial_texture;
layout(set = 1, binding = 2) uniform sampler ColorMaterial_texture_sampler;

void main() {
    vec4 color = Color;

    vec4 sampled_color = texture(
        sampler2D(ColorMaterial_texture, ColorMaterial_texture_sampler),
        v_Uv);

    float rem = mod(gl_FragCoord.y, 5.0);
    if (rem >= 0.0 && rem <= 1.1) {
        sampled_color += vec4(0.005, 0.005, 0.005, 0.0);
    }

    color *= sampled_color;
    o_Target = color;
}
"#;

const VERTEX_SHADER: &str = r#"
#version 450

layout(location = 0) in vec3 Vertex_Position;
layout(location = 1) in vec3 Vertex_Normal;
layout(location = 2) in vec2 Vertex_Uv;

layout(location = 0) out vec2 v_Uv;

layout(set = 0, binding = 0) uniform Camera {
    mat4 ViewProj;
};

layout(set = 2, binding = 0) uniform Transform {
    mat4 Model;
};
layout(set = 2, binding = 1) uniform Sprite_size {
    vec2 size;
};

void main() {
    v_Uv = Vertex_Uv;
    vec3 position = Vertex_Position * vec3(size, 1.0);
    gl_Position = ViewProj * Model * vec4(position, 1.0);
}
"#;

fn main() {
    let mut app = App::build();
    app
        // .add_resource(Msaa { samples: 1 })
        .insert_resource(ClearColor(Color::rgb(0.025, 0.0, 0.035)))
        .add_plugin(bevy::log::LogPlugin::default())
        .add_plugin(bevy::reflect::ReflectPlugin::default())
        .add_plugin(bevy::core::CorePlugin::default())
        .add_plugin(TransformPlugin::default())
        .add_plugin(bevy::diagnostic::DiagnosticsPlugin::default())
        .add_plugin(bevy::input::InputPlugin::default())
        .add_plugin(bevy::window::WindowPlugin::default())
        .add_plugin(bevy::asset::AssetPlugin::default())
        .add_plugin(bevy::scene::ScenePlugin::default())
        .add_plugin(bevy::render::RenderPlugin {
            base_render_graph_config: Some(BaseRenderGraphConfig {
                connect_main_pass_to_swapchain: false,
                connect_main_pass_to_main_depth_texture: false,
                ..Default::default()
            }),
        })
        .add_plugin(bevy::sprite::SpritePlugin::default())
        .add_plugin(bevy::pbr::PbrPlugin::default())
        .add_plugin(bevy::ui::UiPlugin::default())
        .add_plugin(bevy::text::TextPlugin::default())
        .add_plugin(bevy::audio::AudioPlugin::default())
        .add_plugin(GilrsPlugin::default())
        .add_plugin(bevy::gltf::GltfPlugin::default())
        .add_plugin(bevy::winit::WinitPlugin::default())
        .add_plugin(bevy::wgpu::WgpuPlugin::default())
        .add_startup_system(setup.system())
        .add_system(cube_rotator_system.system())
        .add_system(rotator_system.system());
    {
        let resources = app.resources_mut();
        let mut render_graph = resources.get_mut::<RenderGraph>().unwrap();
        render_graph.add_render_to_texture_graph(resources);
    }

    app.run();
}

pub trait RenderToTextureGraphBuilder {
    fn add_render_to_texture_graph(&mut self, resources: &Resources) -> &mut Self;
}

impl RenderToTextureGraphBuilder for RenderGraph {
    fn add_render_to_texture_graph(&mut self, resources: &Resources) -> &mut Self {
        let mut active_cameras = resources.get_mut::<ActiveCameras>().unwrap();

        let mut pass_node = PassNode::<&FirstPass>::new(PassDescriptor {
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
        pass_node.add_camera(FIRST_PASS_CAMERA);

        self.add_node(FIRST_PASS, pass_node);
        self.add_system_node(FIRST_PASS_CAMERA, CameraNode::new(FIRST_PASS_CAMERA));

        active_cameras.add(FIRST_PASS_CAMERA);
        self.add_node_edge(FIRST_PASS_CAMERA, FIRST_PASS).unwrap();

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

        self.add_node_edge(MAIN_PASS, FIRST_PASS).unwrap();
        self.add_slot_edge(
            bevy::render::render_graph::base::node::PRIMARY_SWAP_CHAIN,
            bevy::render::render_graph::WindowSwapChainNode::OUT_TEXTURE,
            FIRST_PASS,
            "color_attachment",
        )
        .unwrap();
        self.add_slot_edge(
            bevy::render::render_graph::base::node::MAIN_DEPTH_TEXTURE,
            bevy::render::render_graph::WindowTextureNode::OUT_TEXTURE,
            FIRST_PASS,
            "depth",
        )
        .unwrap();
        // self.add_node_edge("transform", FIRST_PASS).unwrap();
        self
    }
}

/// this component indicates what entities should rotate
struct Rotator;
struct Cube;

#[derive(Default)]
pub struct FirstPass;

/// rotates the inner cube (first pass)
fn rotator_system(time: Res<Time>, mut query: Query<&mut Transform, With<Rotator>>) {
    for mut transform in query.iter_mut() {
        transform.rotation *= Quat::from_rotation_x(1.5 * time.delta_seconds());
        transform.rotation *= Quat::from_rotation_z(1.3 * time.delta_seconds());
    }
}
/// rotates the outer cube (main pass)
fn cube_rotator_system(time: Res<Time>, mut query: Query<&mut Transform, With<Cube>>) {
    for mut transform in query.iter_mut() {
        transform.translation.x += 10.0 * time.delta_seconds();
    }
}

fn setup(
    commands: &mut Commands,
    mut color_materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut pipelines: ResMut<Assets<PipelineDescriptor>>,
    mut shaders: ResMut<Assets<Shader>>,
) {
    let atlas_texture_handle: Handle<Texture> = asset_server.load("textures/atlas.png");
    let texture_atlas =
        TextureAtlas::from_grid(atlas_texture_handle.clone(), Vec2::new(24.0, 24.0), 19, 19);
    let atlas_handle = texture_atlases.add(texture_atlas);

    /*/ light
    commands.spawn(LightBundle {
        transform: Transform::from_translation(Vec3::new(4.0, 10.0, 10.0)),
        ..Default::default()
    });*/

    let mut cam_trans = Transform::default();
    cam_trans.rotate(Quat::from_rotation_y(std::f32::consts::PI));
    // camera
    let mut first_pass_camera = OrthographicCameraBundle {
        camera: Camera {
            name: Some(FIRST_PASS_CAMERA.to_string()),
            window: WindowId::new(),
            ..Default::default()
        },
        transform: cam_trans,
        ..OrthographicCameraBundle::new_2d()
    };
    let camera_projection = &mut first_pass_camera.orthographic_projection;
    camera_projection.update(1280.0, 720.0);
    first_pass_camera.camera.projection_matrix = camera_projection.get_projection_matrix();
    first_pass_camera.camera.depth_calculation = camera_projection.depth_calculation();

    commands.spawn(first_pass_camera);

    let texture_handle = RENDER_TEXTURE_HANDLE.typed();

    // add entities to the world
    commands
        .spawn(SpriteSheetBundle {
            texture_atlas: atlas_handle,
            sprite: TextureAtlasSprite {
                index: 19 * 2 + 5,
                ..Default::default()
            },
            transform: Transform::from_scale(Vec3::splat(3.0)),
            ..Default::default()
        })
        .with(Cube)
        .spawn(OrthographicCameraBundle {
            ..OrthographicCameraBundle::new_2d()
        });

    // Final pass

    let pipeline_handle = pipelines.add(build_sprite_pipeline(ShaderStages {
        vertex: shaders.add(Shader::from_glsl(ShaderStage::Vertex, VERTEX_SHADER)),
        fragment: Some(shaders.add(Shader::from_glsl(ShaderStage::Fragment, FRAGMENT_SHADER))),
    }));

    commands
        .spawn(SpriteBundle {
            sprite: Sprite {
                size: Vec2::new(1280., 720.),
                ..Default::default()
            },
            material: color_materials.add(texture_handle.clone().into()),
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 1.0)),
            render_pipelines: RenderPipelines::from_pipelines(vec![RenderPipeline::new(
                pipeline_handle,
            )]),
            ..Default::default()
        })
        .with(FirstPass);
    commands.remove_one::<MainPass>(commands.current_entity().unwrap());
}

pub fn build_sprite_pipeline(shader_stages: ShaderStages) -> PipelineDescriptor {
    PipelineDescriptor {
        depth_stencil: Some(DepthStencilState {
            format: TextureFormat::Depth32Float,
            depth_write_enabled: true,
            depth_compare: CompareFunction::LessEqual,
            stencil: StencilState {
                front: StencilFaceState::IGNORE,
                back: StencilFaceState::IGNORE,
                read_mask: 0,
                write_mask: 0,
            },
            bias: DepthBiasState {
                constant: 0,
                slope_scale: 0.0,
                clamp: 0.0,
            },
            clamp_depth: false,
        }),
        color_target_states: vec![ColorTargetState {
            format: TextureFormat::default(),
            color_blend: BlendState {
                src_factor: BlendFactor::SrcAlpha,
                dst_factor: BlendFactor::OneMinusSrcAlpha,
                operation: BlendOperation::Add,
            },
            alpha_blend: BlendState {
                src_factor: BlendFactor::One,
                dst_factor: BlendFactor::One,
                operation: BlendOperation::Add,
            },
            write_mask: ColorWrite::ALL,
        }],
        primitive: PrimitiveState {
            topology: PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: FrontFace::Ccw,
            cull_mode: CullMode::None,
            polygon_mode: PolygonMode::Fill,
        },
        ..PipelineDescriptor::new(shader_stages)
    }
}

pub struct TextureNode {
    pub texture_descriptor: TextureDescriptor,
    pub sampler_descriptor: Option<SamplerDescriptor>,
    pub handle: Option<HandleUntyped>,
}

impl TextureNode {
    pub const TEXTURE: &'static str = "btexture";

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
