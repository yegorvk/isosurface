use bevy::pbr::wireframe::{WireframePlugin, WIREFRAME_SHADER_HANDLE};
use bevy::pbr::{MaterialPipeline, MaterialPipelineKey};
use bevy::prelude::*;
use bevy::render::mesh::MeshVertexBufferLayoutRef;
use bevy::render::render_resource::{
    AsBindGroup, Face, PolygonMode, RenderPipelineDescriptor, ShaderRef,
    SpecializedMeshPipelineError,
};

pub struct ExtendedWireframePlugin;

impl Plugin for ExtendedWireframePlugin {
    fn build(&self, app: &mut App) {
        // `WireframePlugin` loads WIREFRAME_SHADER_HANDLE which is used by `DoubleSidedWireframeMaterial`.
        if !app.is_plugin_added::<WireframePlugin>() {
            panic!("`DoubleSidedWireframePlugin` requires `WireframePlugin` in order to work.");
        }

        app.add_plugins(MaterialPlugin::<WireframeMaterial>::default())
            .add_systems(
                Update,
                (enumerate_wireframes, apply_wireframe_material).chain(),
            );
    }
}

#[derive(Debug, Component)]
pub struct Wireframe {
    pub color: Color,
    pub enabled: bool,
}

#[derive(Debug, Component)]
struct HasEnabledWireframe;

fn enumerate_wireframes(mut commands: Commands, wireframes: Query<(Entity, &Wireframe)>) {
    for (entity, wireframe) in &wireframes {
        if wireframe.enabled {
            commands.entity(entity).insert(HasEnabledWireframe);
        } else {
            commands.entity(entity).remove::<HasEnabledWireframe>();
        }
    }
}

fn apply_wireframe_material(
    mut commands: Commands,
    mut materials: ResMut<Assets<WireframeMaterial>>,
    wireframes: Query<(Entity, &Wireframe), With<HasEnabledWireframe>>,
    mut removed_wireframes: RemovedComponents<HasEnabledWireframe>,
) {
    for entity in removed_wireframes.read() {
        if let Some(mut commands) = commands.get_entity(entity) {
            commands.remove::<MeshMaterial3d<WireframeMaterial>>();
        }
    }

    let mut materials_to_spawn = vec![];

    for (entity, wireframe) in &wireframes {
        let material = materials.add(WireframeMaterial {
            color: wireframe.color.into(),
        });

        materials_to_spawn.push((entity, MeshMaterial3d(material)));
    }

    commands.insert_or_spawn_batch(materials_to_spawn);
}

#[derive(Debug, Default, Clone, Asset, TypePath, AsBindGroup)]
struct WireframeMaterial {
    #[uniform(0)]
    color: LinearRgba,
}

impl Material for WireframeMaterial {
    fn fragment_shader() -> ShaderRef {
        WIREFRAME_SHADER_HANDLE.into()
    }

    fn specialize(
        _pipeline: &MaterialPipeline<Self>,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayoutRef,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        descriptor.primitive.polygon_mode = PolygonMode::Line;
        descriptor.primitive.cull_mode = None;

        if let Some(depth_stencil) = descriptor.depth_stencil.as_mut() {
            depth_stencil.bias.slope_scale = 1.0;
        }

        Ok(())
    }
}
