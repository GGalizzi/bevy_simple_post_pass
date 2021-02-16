use bevy::{
    prelude::*,
    render::{pipeline::PipelineDescriptor, render_graph::base::BaseRenderGraphConfig},
};

mod render;

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
        .add_system(cube_rotator_system.system());
    {
        render::setup_render_graph(&mut app);
    }

    app.run();
}

struct Cube;

/// rotates the outer cube (main pass)
fn cube_rotator_system(time: Res<Time>, mut query: Query<&mut Transform, With<Cube>>) {
    for mut transform in query.iter_mut() {
        transform.translation.x += 10.0 * time.delta_seconds();
        transform.translation.y += 10.0 * time.delta_seconds();
    }
}

fn setup(
    mut commands: &mut Commands,
    mut color_materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut pipelines: ResMut<Assets<PipelineDescriptor>>,
    mut shaders: ResMut<Assets<Shader>>,
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

    render::setup_final_pass(
        &mut commands,
        &mut pipelines,
        &mut shaders,
        &mut color_materials,
    );
}
