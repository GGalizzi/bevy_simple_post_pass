use bevy::{
    prelude::*,
    render::{
        camera::{Camera, CameraProjection},
        pipeline::{
            BlendFactor, BlendOperation, BlendState, ColorTargetState, ColorWrite, CompareFunction,
            CullMode, DepthBiasState, DepthStencilState, FrontFace, PipelineDescriptor,
            PolygonMode, PrimitiveState, PrimitiveTopology, RenderPipeline, StencilFaceState,
            StencilState,
        },
        render_graph::base::MainPass,
        render_graph::RenderGraph,
        shader::{ShaderStage, ShaderStages},
        texture::TextureFormat,
    },
    window::WindowId,
};

mod render_to_texture;
mod texture_node;

pub use render_to_texture::*;
pub use texture_node::*;

pub fn setup_render_graph(app: &mut AppBuilder) {
    let resources = app.resources_mut();
    let mut render_graph = resources.get_mut::<RenderGraph>().unwrap();
    render_graph.add_render_to_texture_graph(resources);
}

pub fn setup_final_pass(
    commands: &mut Commands,
    pipelines: &mut ResMut<Assets<PipelineDescriptor>>,
    shaders: &mut ResMut<Assets<Shader>>,
    color_materials: &mut ResMut<Assets<ColorMaterial>>,
) {
    let texture_handle = RENDER_TEXTURE_HANDLE.typed();

    let mut cam_trans = Transform::default();
    cam_trans.rotate(Quat::from_rotation_x(std::f32::consts::PI));
    // camera
    let mut post_pass_camera = OrthographicCameraBundle {
        camera: Camera {
            name: Some(POST_PASS_CAMERA.to_string()),
            window: WindowId::new(),
            ..Default::default()
        },
        transform: cam_trans,
        ..OrthographicCameraBundle::new_2d()
    };
    let camera_projection = &mut post_pass_camera.orthographic_projection;
    camera_projection.update(1280.0, 720.0);
    post_pass_camera.camera.projection_matrix = camera_projection.get_projection_matrix();
    post_pass_camera.camera.depth_calculation = camera_projection.depth_calculation();

    commands.spawn(post_pass_camera);

    let pipeline_handle = pipelines.add(build_sprite_pipeline(ShaderStages {
        vertex: shaders.add(Shader::from_glsl(
            ShaderStage::Vertex,
            include_str!("final_pass.vert"),
        )),
        fragment: Some(shaders.add(Shader::from_glsl(
            ShaderStage::Fragment,
            include_str!("final_pass.frag"),
        ))),
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
        .with(PostPass);
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
