// Copyright 2021 Tristam MacDonald
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::source::HermiteSource;
use glam::Vec3;

/// Trait for outputting mesh vertices and indices.
pub trait Extractor {
    /// Extracts a single vertex from the mesh generated from an isosurface.
    fn extract_vertex(&mut self, position: Vec3);

    /// Extracts a single triangular face provided the indices of
    /// its constituent vertices in any order.
    ///
    /// The vertices referred by the provided indices
    /// must have been already added to the mesh
    /// via `extract_vertex`.
    fn extract_face(&mut self, indices: [u32; 3]);
}

/// Output vertices as a tightly packed array of floats representing
/// vertex positions without the corresponding face data.
pub struct OnlyVertexPositions<'a> {
    positions: &'a mut Vec<[f32; 3]>,
}

impl<'a> OnlyVertexPositions<'a> {
    pub fn new(positions: &'a mut Vec<[f32; 3]>) -> Self {
        Self { positions }
    }
}

impl Extractor for OnlyVertexPositions<'_> {
    fn extract_vertex(&mut self, v: Vec3) {
        self.positions.push(v.to_array());
    }

    fn extract_face(&mut self, _indices: [u32; 3]) {}
}

/// Output vertices as a tightly packed array of floats representing vertex positions
/// interleaved with vertex normals, without the corresponding face data.
///
/// The layout will be: `position_1`, `normal_1`, `position_2`, `normal_2`, ...
pub struct VertexPositionsInterleavedNormals<'a, S: HermiteSource> {
    vertices: &'a mut Vec<[f32; 6]>,
    source: &'a S,
}

impl<'a, S: HermiteSource> VertexPositionsInterleavedNormals<'a, S> {
    pub fn new(vertices: &'a mut Vec<[f32; 6]>, source: &'a S) -> Self {
        Self { vertices, source }
    }
}

impl<S: HermiteSource> Extractor for VertexPositionsInterleavedNormals<'_, S> {
    fn extract_vertex(&mut self, v: Vec3) {
        let n = self.source.sample_normal(v);
        self.vertices.push([v.x, v.y, v.z, n.x, n.y, n.z]);
    }

    fn extract_face(&mut self, _indices: [u32; 3]) {}
}

/// Outputs vertices as two tightly packed arrays of floats, one for vertex positions
/// and one for vertex normals, without the corresponding face data.
pub struct VertexPositionsSeparateNormals<'a, S: HermiteSource> {
    positions: &'a mut Vec<[f32; 3]>,
    normals: &'a mut Vec<[f32; 3]>,
    source: &'a S,
}

impl<'a, S: HermiteSource> VertexPositionsSeparateNormals<'a, S> {
    pub fn new(
        positions: &'a mut Vec<[f32; 3]>,
        normals: &'a mut Vec<[f32; 3]>,
        source: &'a S,
    ) -> Self {
        Self {
            positions,
            normals,
            source,
        }
    }
}

impl<S: HermiteSource> Extractor for VertexPositionsSeparateNormals<'_, S> {
    fn extract_vertex(&mut self, v: Vec3) {
        let n = self.source.sample_normal(v);
        self.positions.push(v.to_array());
        self.normals.push(n.to_array());
    }

    fn extract_face(&mut self, _indices: [u32; 3]) {}
}

/// Output vertices as a tightly packed array of floats
/// alongside the indices array representing individual faces.
pub struct IndexedVertices<'a> {
    vertices: &'a mut Vec<[f32; 3]>,
    faces: &'a mut Vec<[u32; 3]>,
}

impl<'a> IndexedVertices<'a> {
    pub fn new(vertices: &'a mut Vec<[f32; 3]>, faces: &'a mut Vec<[u32; 3]>) -> Self {
        Self { vertices, faces }
    }
}

impl Extractor for IndexedVertices<'_> {
    fn extract_vertex(&mut self, v: Vec3) {
        self.vertices.push(v.to_array());
    }

    fn extract_face(&mut self, indices: [u32; 3]) {
        self.faces.push(indices);
    }
}

/// Output vertices as a tightly packed array of floats representing vertex positions
/// interleaved with vertex normals alongside the corresponding face data.
///
/// The layout will be: `position_1`, `normal_1`, `position_2`, `normal_2`, ...
pub struct IndexedInterleavedNormals<'a, S: HermiteSource + ?Sized> {
    vertices: &'a mut Vec<[f32; 6]>,
    faces: &'a mut Vec<[u32; 3]>,
    source: &'a S,
}

impl<'a, S: HermiteSource + ?Sized> IndexedInterleavedNormals<'a, S> {
    pub fn new(
        vertices: &'a mut Vec<[f32; 6]>,
        indices: &'a mut Vec<[u32; 3]>,
        source: &'a S,
    ) -> Self {
        Self {
            vertices,
            faces: indices,
            source,
        }
    }
}

impl<S: HermiteSource> Extractor for IndexedInterleavedNormals<'_, S> {
    fn extract_vertex(&mut self, v: Vec3) {
        let n = self.source.sample_normal(v);
        self.vertices.push([v.x, v.y, v.z, n.x, n.y, n.z]);
    }

    fn extract_face(&mut self, indices: [u32; 3]) {
        self.faces.push(indices);
    }
}

/// Outputs vertices as two tightly packed arrays of floats, one for vertex positions
/// and one for vertex normals alongside the corresponding face data.
pub struct IndexedSeparateNormals<'a, S: HermiteSource + ?Sized> {
    positions: &'a mut Vec<[f32; 3]>,
    normals: &'a mut Vec<[f32; 3]>,
    faces: &'a mut Vec<[u32; 3]>,
    source: &'a S,
    adjust_normals: bool,
}

impl<'a, S: HermiteSource + ?Sized> IndexedSeparateNormals<'a, S> {
    pub fn new(
        positions: &'a mut Vec<[f32; 3]>,
        normals: &'a mut Vec<[f32; 3]>,
        faces: &'a mut Vec<[u32; 3]>,
        source: &'a S,
        adjust_normals: bool,
    ) -> Self {
        Self {
            positions,
            normals,
            faces,
            source,
            adjust_normals,
        }
    }
}

impl<S: HermiteSource> Extractor for IndexedSeparateNormals<'_, S> {
    fn extract_vertex(&mut self, v: Vec3) {
        let n = self.source.sample_normal(v);
        self.positions.push(v.to_array());
        self.normals.push(n.to_array());
    }

    fn extract_face(&mut self, mut indices: [u32; 3]) {
        if self.adjust_normals {
            let v = indices.map(|i| Vec3::from(self.positions[i as usize]));
            let v_n = indices.map(|i| Vec3::from(self.normals[i as usize]));

            let n_face = face_normal(v[0], v[1], v[2]);
            let n_actual = (v_n[0] + v_n[1] + v_n[2]) / 3.0;

            if n_face.dot(n_actual) < 0.0 {
                indices.reverse();
            }
        }

        self.faces.push(indices);
    }
}

fn face_normal(a: Vec3, b: Vec3, c: Vec3) -> Vec3 {
    (b - a).cross(c - b).normalize()
}
