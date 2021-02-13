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
        render_graph::{
            base::{node::MAIN_PASS, MainPass},
            CameraNode, Node, PassNode, RenderGraph, ResourceSlotInfo,
        },
        renderer::{RenderResourceId, RenderResourceType},
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

fn main() {
    let mut app = App::build();
    app
        // .add_resource(Msaa { samples: 1 })
        .add_plugins(DefaultPlugins)
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
                    load: LoadOp::Clear(Color::rgb(0.1, 0.2, 0.3)),
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
                    size: Extent3d::new(512, 512, 1),
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
                    size: Extent3d::new(512, 512, 1),
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

        self.add_node_edge(TEXTURE_NODE, FIRST_PASS).unwrap();
        self.add_slot_edge(
            TEXTURE_NODE,
            TextureNode::TEXTURE,
            FIRST_PASS,
            "color_attachment",
        )
        .unwrap();
        self.add_slot_edge(
            DEPTH_TEXTURE_NODE,
            TextureNode::TEXTURE,
            FIRST_PASS,
            "depth",
        )
        .unwrap();
        self.add_node_edge(FIRST_PASS, MAIN_PASS).unwrap();
        self.add_node_edge("transform", FIRST_PASS).unwrap();
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
        transform.rotation *= Quat::from_rotation_x(1.0 * time.delta_seconds());
        transform.rotation *= Quat::from_rotation_y(0.7 * time.delta_seconds());
    }
}

fn setup(
    commands: &mut Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let cube_handle = meshes.add(Mesh::from(shape::Cube { size: 56.0 }));
    let cube_material_handle = materials.add(StandardMaterial {
        albedo: Color::rgb(0.8, 0.7, 0.6),
        ..Default::default()
    });

    commands
        .spawn(PbrBundle {
            mesh: cube_handle.clone(),
            material: cube_material_handle.clone(),
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 1.0)),
            ..Default::default()
        })
        .with(Rotator)
        .with(FirstPass);
    commands.remove_one::<MainPass>(commands.current_entity().unwrap());

    let mut first_pass_camera = OrthographicCameraBundle {
        camera: Camera {
            name: Some(FIRST_PASS_CAMERA.to_string()),
            window: WindowId::new(), // otherwise it will use main window size / aspect for calculation of projection matrix
            ..Default::default()
        },
        ..OrthographicCameraBundle::new_2d()
    };
    first_pass_camera.camera.window = WindowId::new();

    commands.spawn(first_pass_camera);

    let texture_handle = RENDER_TEXTURE_HANDLE.typed();

    let cube_size = 56.0;
    let cube_handle = meshes.add(Mesh::from(shape::Box::new(cube_size, cube_size, cube_size)));

    let material_handle = materials.add(StandardMaterial {
        albedo_texture: Some(texture_handle.clone()),
        ..Default::default()
    });

    // add entities to the world
    commands
        .spawn(PbrBundle {
            mesh: cube_handle.clone(),
            material: material_handle,
            transform: Transform {
                translation: Vec3::new(0.0, 0.0, 1.5),
                rotation: Quat::from_rotation_x(-std::f32::consts::PI / 5.0),
                ..Default::default()
            },
            visible: Visible {
                is_transparent: true,
                ..Default::default()
            },
            ..Default::default()
        })
        .with(Cube)
        .spawn(OrthographicCameraBundle::new_2d());
}

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
