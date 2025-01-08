use glam::Vec3;
use crate::distance::Signed;
use crate::extractor::Extractor;
use crate::feature::PlaceFeatureInCell;
use crate::index_cache::GridKey;
use crate::marching_cubes_impl::{
    classify_corners, find_edge_crossings, march_cube, sample_normals_at_corners,
};
use crate::mesh::MeshTopologyBuilder;
use crate::sampler::Sample;
use crate::source::HermiteSource;
use crate::traversal::DualGrid;

pub struct AdaptiveDualContouring<P: PlaceFeatureInCell> {
    dual_grid: DualGrid<Signed>,
    place_feature: P,
}

impl<P: PlaceFeatureInCell> AdaptiveDualContouring<P> {
    pub fn new(max_depth: usize, place_feature: P) -> Self {
        Self {
            dual_grid: DualGrid::new(1 << max_depth),
            place_feature,
        }
    }

    pub fn extract<S, E>(&mut self, source: &S, extractor: &mut E)
    where
        S: Sample<Signed> + HermiteSource,
        E: Extractor,
    {
        let mut mesh_builder = MeshTopologyBuilder::new(extractor);
        let mut normals = [Vec3::ZERO; 8];

        let dual_grid = &mut self.dual_grid;
        let place_feature = &mut self.place_feature;

        dual_grid.traverse(
            source,
            Some(|corners: &[Vec3; 8], values: &[Signed; 8]| {
                let cube_index = classify_corners(values);
                if cube_index == 0 || cube_index == 255 {
                    return None;
                }

                sample_normals_at_corners(source, corners, &mut normals);

                Some(place_feature.place_feature_in_cell(corners, &normals))
            }),
            |keys, corners, values| {
                let cube_index = classify_corners(values);

                let mut vertices = [Vec3::ZERO; 12];
                find_edge_crossings(cube_index, corners, values, &mut vertices);

                march_cube(cube_index, |a, b, c| {
                    let a = mesh_builder.add_vertex(Some(GridKey::new(keys, a)), vertices[a]);
                    let b = mesh_builder.add_vertex(Some(GridKey::new(keys, b)), vertices[b]);
                    let c = mesh_builder.add_vertex(Some(GridKey::new(keys, c)), vertices[c]);

                    mesh_builder.add_face(a, b, c);
                });
            },
        );

        mesh_builder.build().extract_indices(extractor);
    }
}
