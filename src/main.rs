use bevy::{
    prelude::*,
    render::camera::Camera,
    render::{
        camera::{CameraProjection, OrthographicProjection},
        pipeline::PipelineDescriptor,
        render_graph::base::BaseRenderGraphConfig,
    },
};

mod render;

struct GameCam;

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
        .add_system(camera_zoom_system.system())
        .add_system(cube_rotator_system.system());
    {
        render::setup_render_graph(&mut app);
    }

    app.run();
}

struct Cube;

/// rotates the outer cube (main pass)
fn cube_rotator_system(
    time: Res<Time>,
    mut char_inputs: EventReader<ReceivedCharacter>,
    mut query: Query<&mut Transform, With<Cube>>,
) {
    for mut transform in query.iter_mut() {
        for event in char_inputs.iter() {
            transform.translation.x += if event.char == '6' {
                1.
            } else if event.char == '4' {
                -1.
            } else {
                0.0
            } * 1.0;

            transform.translation.z += if event.char == '2' {
                1.
            } else if event.char == '8' {
                -1.
            } else {
                0.0
            } * 1.0;
        }
    }
}

fn camera_zoom_system(
    time: Res<Time>,
    mut char_inputs: EventReader<ReceivedCharacter>,
    mut query: Query<&mut Transform, With<GameCam>>,
    player_query: Query<&Transform, With<Cube>>,
) {
    for mut ortho in query.iter_mut() {
        if let Some(player_transform) = player_query.iter().next() {
            ortho.translation.x = player_transform.translation.x + 12.0;
            ortho.translation.z = player_transform.translation.z + 12.0;
        }
        for event in char_inputs.iter() {
            ortho.scale += if event.char == 'k' {
                1.
            } else if event.char == 'j' {
                -1.
            } else {
                0.0
            } * 2.0
                * Vec3::one();
                

            ortho.translation.x += if event.char == 'd' {
                1.
            } else if event.char == 'a' {
                -1.
            } else {
                0.0
            } * 1.0;

            ortho.translation.z += if event.char == 's' {
                1.
            } else if event.char == 'w' {
                -1.
            } else {
                0.0
            } * 1.0;
        }
    }
}

fn setup(
    mut commands: &mut Commands,
    mut color_materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut pipelines: ResMut<Assets<PipelineDescriptor>>,
    mut shaders: ResMut<Assets<Shader>>,

    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn(UiCameraBundle::default()).spawn(TextBundle {
        style: Style {
            align_self: AlignSelf::FlexEnd,
            position_type: PositionType::Absolute,
            position: Rect {
                bottom: Val::Px(5.0),
                right: Val::Px(15.0),
                ..Default::default()
            },
            ..Default::default()
        },
        text: Text::with_section(
            "ui test hello there",
            TextStyle {
                font: asset_server.load("fonts/monogram.ttf"),
                font_size: 100.0,
                color: Color::WHITE,
            },
            Default::default(),
        ),
        ..Default::default()
    });

    let atlas_texture_handle: Handle<Texture> = asset_server.load("textures/atlas.png");
    let texture_atlas =
        TextureAtlas::from_grid(atlas_texture_handle, Vec2::new(24.0, 24.0), 19, 19);
    let atlas_handle = texture_atlases.add(texture_atlas);

    let grass_texture_handle = asset_server.load("textures/grass.png");

    let mut ortho_cam = OrthographicCameraBundle::new_3d();
    ortho_cam.orthographic_projection.scale = 7.;
    ortho_cam.transform =
        Transform::from_xyz(12.0, 12.0, 12.0).looking_at(Vec3::zero(), Vec3::unit_y());

    let cube_size = 1.0;

    for x in 0..10 {
        for y in 0..10 {
            commands.spawn(PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Cube { size: cube_size })),
                material: materials.add(
                    StandardMaterial {
                        albedo_texture: Some(grass_texture_handle.clone()),
                        ..Default::default()
                    }
                ),
                transform: Transform::from_xyz(x as f32, 0., y as f32),
                ..Default::default()
            });
        }
    }
    // add entities to the world
    commands
        .spawn(SpriteSheetBundle {
            texture_atlas: atlas_handle,
            mesh: meshes.add(Mesh::from(shape::Quad::new(Vec2::new(0.1, 0.1)))),
            sprite: TextureAtlasSprite {
                index: 19 * 2 + 5,
                ..Default::default()
            },
            transform: {
                let mut base = Transform::from_scale(Vec3::splat(1.0));
                /*base.translation.x += 32.0 / 2.0;
                base.translation.z += 32.0 / 2.0;*/
                base.translation.y += 1.5;
                base.rotate(Quat::from_rotation_y(std::f32::consts::PI / 4.0));
                base
            },
            ..Default::default()
        })
        .with(Cube)
        .spawn(ortho_cam)
        .with(GameCam);

    render::setup_final_pass(
        &mut commands,
        &mut pipelines,
        &mut shaders,
        &mut color_materials,
    );
}
