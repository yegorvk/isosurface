mod samples;
mod wireframe;

use crate::samples::*;
use crate::wireframe::{ExtendedWireframePlugin, Wireframe};
use bevy::asset::RenderAssetUsages;
use bevy::input::common_conditions::input_just_pressed;
use bevy::math::vec3;
use bevy::pbr::wireframe::WireframePlugin;
use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::render::settings::{RenderCreation, WgpuFeatures, WgpuSettings};
use bevy::render::RenderPlugin;
use bevy::window::{CursorGrabMode, PrimaryWindow};
use bevy_flycam::{FlyCam, MovementSettings, NoCameraPlayerPlugin};
use isosurface::extractor::IndexedSeparateNormals;
use isosurface::feature::ParticleBasedMinimisation;
use isosurface::sampler::Sampler;
use isosurface::source::{CentralDifference, ScalarSource};
use isosurface::AdaptiveDualContouring;

#[derive(Resource)]
struct State {
    show_wireframe: bool,
}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Isosurface Demo".to_owned(),
                        ..default()
                    }),
                    ..default()
                })
                .set(RenderPlugin {
                    render_creation: RenderCreation::Automatic(WgpuSettings {
                        features: WgpuFeatures::POLYGON_MODE_LINE,
                        ..default()
                    }),
                    ..default()
                }),
            WireframePlugin,
            ExtendedWireframePlugin,
            NoCameraPlayerPlugin,
        ))
        .insert_resource(MovementSettings {
            sensitivity: 0.00005,
            speed: 4.0,
        })
        .insert_resource(State {
            show_wireframe: false,
        })
        .add_systems(Startup, setup)
        .add_systems(PostStartup, ungrab_cursor)
        .add_systems(
            Update,
            (
                toggle_wireframe.run_if(input_just_pressed(KeyCode::F1)),
                update_wireframe_visibility,
            ),
        )
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    const MAX_DEPTH: usize = 8;

    commands.spawn((
        Mesh3d(meshes.add(create_mesh(&Plane, MAX_DEPTH))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::linear_rgb(0.0, 1.0, 0.0),
            cull_mode: None,
            double_sided: true,
            ..default()
        })),
        Wireframe {
            color: Color::WHITE,
            enabled: false,
        },
        Transform::from_translation(vec3(-2.5, -2.5, -2.5 - 13.0)).with_scale(Vec3::splat(5.0)),
    ));

    commands.spawn((
        Mesh3d(meshes.add(create_mesh(&Sphere, MAX_DEPTH))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::linear_rgb(0.0, 1.0, 0.0),
            cull_mode: None,
            double_sided: true,
            ..default()
        })),
        Wireframe {
            color: Color::WHITE,
            enabled: false,
        },
        Transform::from_translation(Vec3::splat(-2.5)).with_scale(Vec3::splat(5.0)),
    ));

    commands.spawn((
        Mesh3d(meshes.add(create_mesh(&DistortedSphere, MAX_DEPTH))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::linear_rgb(0.0, 1.0, 0.0),
            cull_mode: None,
            double_sided: true,
            ..default()
        })),
        Wireframe {
            color: Color::WHITE,
            enabled: false,
        },
        Transform::from_translation(vec3(-2.5 + 4.0, -2.5, -2.5)).with_scale(Vec3::splat(5.0)),
    ));

    commands.spawn((
        Mesh3d(meshes.add(create_mesh(&PerlinNoise2D::default(), MAX_DEPTH))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::linear_rgb(0.0, 1.0, 0.0),
            cull_mode: None,
            double_sided: true,
            ..default()
        })),
        Wireframe {
            color: Color::WHITE,
            enabled: false,
        },
        Transform::from_translation(vec3(-2.5 - 7.0, -2.5, -2.5)).with_scale(Vec3::splat(5.0)),
    ));

    commands.spawn((
        Mesh3d(meshes.add(create_mesh(&PerlinNoise3D::default(), MAX_DEPTH))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::linear_rgb(0.0, 1.0, 0.0),
            cull_mode: None,
            double_sided: true,
            ..default()
        })),
        Wireframe {
            color: Color::WHITE,
            enabled: false,
        },
        Transform::from_translation(vec3(-2.5, -2.5, -2.5 - 6.0)).with_scale(Vec3::splat(5.0)),
    ));

    commands.spawn((
        DirectionalLight::default(),
        Transform::default().looking_to(Vec3::new(1.0, 0.0, 0.0), Vec3::Y),
    ));

    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 0.0, 6.0).looking_to(-Vec3::Z, Vec3::Y),
        FlyCam,
    ));
}

fn ungrab_cursor(mut windows: Query<&mut Window, With<PrimaryWindow>>) {
    let mut primary_window = windows.single_mut();
    primary_window.cursor_options.grab_mode = CursorGrabMode::None;
    primary_window.cursor_options.visible = true;
}

fn toggle_wireframe(mut state: ResMut<State>) {
    state.show_wireframe = !state.show_wireframe;
}

fn update_wireframe_visibility(mut wireframes: Query<&mut Wireframe>, state: Res<State>) {
    for mut wireframe in &mut wireframes {
        wireframe.enabled = state.show_wireframe;
    }
}

fn create_mesh<S>(source: &S, max_depth: usize) -> Mesh
where
    S: ScalarSource + ?Sized,
{
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut faces = Vec::new();

    let source = CentralDifference::new_with_epsilon(source, 1e-4);

    let mut extractor =
        IndexedSeparateNormals::new(&mut positions, &mut normals, &mut faces, &source, true);

    AdaptiveDualContouring::new(max_depth, ParticleBasedMinimisation {})
        .extract(&Sampler::new(&source), &mut extractor);

    // LinearHashedMarchingCubes::new(max_depth).extract(&Sampler::new(&source), &mut extractor);

    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_indices(Indices::U32(faces.into_flattened()))
}
