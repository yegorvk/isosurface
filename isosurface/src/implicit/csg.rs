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

use crate::{
    distance::{Directed, Signed},
    source::{HermiteSource, ScalarSource, VectorSource},
};
use glam::Vec3;

/// The CSG union operation. An implicit function that is solid where either of
/// the provided implicit functions is solid.
pub struct Union<A, B> {
    /// The first implicit function.
    pub a: A,
    /// The second implicit function.
    pub b: B,
}

impl<A, B> Union<A, B> {
    pub fn new(a: A, b: B) -> Self {
        Self { a, b }
    }
}

impl<A: ScalarSource, B: ScalarSource> ScalarSource for Union<A, B> {
    fn sample_scalar(&self, p: Vec3) -> Signed {
        Signed(self.a.sample_scalar(p).0.min(self.b.sample_scalar(p).0))
    }
}

impl<A: VectorSource, B: VectorSource> VectorSource for Union<A, B> {
    fn sample_vector(&self, p: Vec3) -> Directed {
        Directed(self.a.sample_vector(p).0.min(self.b.sample_vector(p).0))
    }
}

impl<A: HermiteSource, B: HermiteSource> HermiteSource for Union<A, B> {
    fn sample_normal(&self, p: Vec3) -> Vec3 {
        self.a.sample_normal(p).min(self.b.sample_normal(p))
    }
}

/// The CSG intersection operation. An implicit function that is solid only
/// where both of the provided implicit functions are solid.
pub struct Intersection<A, B> {
    /// The first implicit function.
    pub a: A,
    /// The second implicit function.
    pub b: B,
}

impl<A, B> Intersection<A, B> {
    pub fn new(a: A, b: B) -> Self {
        Self { a, b }
    }
}

impl<A: ScalarSource, B: ScalarSource> ScalarSource for Intersection<A, B> {
    fn sample_scalar(&self, p: Vec3) -> Signed {
        Signed(self.a.sample_scalar(p).0.max(self.b.sample_scalar(p).0))
    }
}

impl<A: VectorSource, B: VectorSource> VectorSource for Intersection<A, B> {
    fn sample_vector(&self, p: Vec3) -> Directed {
        Directed(self.a.sample_vector(p).0.max(self.b.sample_vector(p).0))
    }
}

/// The CSG difference operation. Subtracts the first provided implicit function
/// from the second, i.e. the result is solid where the second
/// function is solid, except where the first is solid.
pub struct Difference<A, B> {
    /// The first implicit function.
    pub a: A,
    /// The second implicit function.
    pub b: B,
}

impl<A, B> Difference<A, B> {
    pub fn new(a: A, b: B) -> Self {
        Self { a, b }
    }
}

impl<A: ScalarSource, B: ScalarSource> ScalarSource for Difference<A, B> {
    fn sample_scalar(&self, p: Vec3) -> Signed {
        Signed(self.b.sample_scalar(p).0.max(-self.a.sample_scalar(p).0))
    }
}

impl<A: VectorSource, B: VectorSource> VectorSource for Difference<A, B> {
    fn sample_vector(&self, p: Vec3) -> Directed {
        Directed(self.b.sample_vector(p).0.max(-self.a.sample_vector(p).0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::implicit::RectangularPrism;
    use glam::vec3;

    #[test]
    fn test_csg() {
        let a = RectangularPrism::new(vec3(4.0, 4.0, 1.0));
        let b = RectangularPrism::new(vec3(2.0, 2.0, 4.0));

        let u = Union::new(a, b);

        assert_eq!(u.sample_scalar(Vec3::ZERO).0, -2.0);
        assert_eq!(u.sample_scalar(vec3(4.0, 4.0, 1.0)).0, 0.0);
        assert_eq!(u.sample_scalar(vec3(0.0, 0.0, 8.0)).0, 4.0);
        assert_eq!(u.sample_scalar(vec3(8.0, 0.0, 0.0)).0, 4.0);

        assert_eq!(u.sample_vector(Vec3::ZERO).0, vec3(-4.0, -4.0, -4.0));
        assert_eq!(u.sample_vector(vec3(4.0, 4.0, 1.0)).0, Vec3::ZERO);
        assert_eq!(
            u.sample_vector(vec3(0.0, 0.0, 8.0)).0,
            vec3(f32::MAX, f32::MAX, 4.0)
        );
        assert_eq!(
            u.sample_vector(vec3(8.0, 0.0, 0.0)).0,
            vec3(4.0, f32::MAX, f32::MAX)
        );

        assert_eq!(
            u.sample_normal(vec3(0.0, 0.0, 8.0))
                .try_normalize()
                .unwrap(),
            vec3(0.0, 0.0, 1.0)
        );
        assert_eq!(
            u.sample_normal(vec3(8.0, 0.0, 0.0))
                .try_normalize()
                .unwrap(),
            vec3(1.0, 0.0, 0.0)
        );

        let i = Intersection::new(a, b);

        assert_eq!(i.sample_scalar(Vec3::ZERO).0, -1.0);
        assert_eq!(i.sample_scalar(vec3(2.0, 2.0, 1.0)).0, 0.0);
        assert_eq!(i.sample_scalar(vec3(0.0, 0.0, 8.0)).0, 7.0);
        assert_eq!(i.sample_scalar(vec3(8.0, 0.0, 0.0)).0, 6.0);

        assert_eq!(i.sample_vector(Vec3::ZERO).0, vec3(-2.0, -2.0, -1.0));
        assert_eq!(i.sample_vector(vec3(2.0, 2.0, 1.0)).0, Vec3::ZERO);
        assert_eq!(
            i.sample_vector(vec3(0.0, 0.0, 8.0)).0,
            vec3(f32::MAX, f32::MAX, 7.0)
        );
        assert_eq!(
            i.sample_vector(vec3(8.0, 0.0, 0.0)).0,
            vec3(6.0, f32::MAX, f32::MAX)
        );
    }
}
