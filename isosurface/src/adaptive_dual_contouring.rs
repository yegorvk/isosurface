use crate::distance::Signed;
use crate::extractor::Extractor;
use crate::feature::PlaceFeatureInCell;
use crate::linear_hashed_octree::LinearHashedOctree;
use crate::morton::Morton;
use crate::sampler::Sample;
use crate::source::HermiteSource;
use glam::Vec3;

pub struct AdaptiveDualContouring<P: PlaceFeatureInCell> {
    max_depth: usize,
    place_feature: P,
}

impl<P: PlaceFeatureInCell> AdaptiveDualContouring<P> {
    pub fn new(max_depth: usize, place_feature: P) -> Self {
        Self {
            max_depth,
            place_feature,
        }
    }

    pub fn extract<S, E>(&mut self, source: &S, extractor: &mut E)
    where
        S: Sample<Signed> + HermiteSource,
        E: Extractor,
    {
        // let mut octree = LinearHashedOctree::new();
        // 
        // octree.build(
        //     |key, cell: &Cell| key.level() < 2 || cell.is_flat(0.01),
        //     |key| Cell::new(source, key, &self.place_feature),
        // );
        // 
        // octree.walk_leaves(|key| {
        //     let vertex = octree.get_leaf_node(key).unwrap().vertex;
        //     
        // })
    }
}

struct Cell {
    normals: [Vec3; 8],
    vertex: Vec3,
}

impl Cell {
    fn new<S, P>(source: &S, key: Morton, place_feature: &P) -> Cell
    where
        S: Sample<Signed> + HermiteSource,
        P: PlaceFeatureInCell,
    {
        let mut corners = [Vec3::ZERO; 8];

        for (i, corner) in corners.iter_mut().enumerate() {
            *corner = key.primal_vertex(key.level(), i).center();
        }

        let normals = corners.map(|corner| source.sample_normal(corner).normalize());
        let vertex = place_feature.place_feature_in_cell(&corners, &normals);

        Self { normals, vertex }
    }

    fn is_flat(&self, tol: f32) -> bool {
        todo!()
        // for (n_a, n_b) in self.normals.iter().tuple_combinations() {
        //     if (n_a - n_b).length_squared() > tol * tol {
        //         return false;
        //     }
        // }
        // 
        // true
    }
}
