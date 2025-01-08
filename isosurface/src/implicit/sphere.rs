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
use glam::{Vec3, Vec3Swizzles};

/// A sphere.
#[derive(Copy, Clone)]
pub struct Sphere {
    /// The radius of the sphere.
    pub radius: f32,
}

impl Sphere {
    /// Create a new sphere from the desired radius.
    pub fn new(radius: f32) -> Self {
        Self { radius }
    }
}

impl ScalarSource for Sphere {
    fn sample_scalar(&self, p: Vec3) -> Signed {
        Signed(p.length() - self.radius)
    }
}

impl VectorSource for Sphere {
    fn sample_vector(&self, p: Vec3) -> Directed {
        // Flip the point into the positive quadrant
        let a = p.abs();

        let r2 = self.radius * self.radius;
        let l_yz = r2 - a.yz().length_squared();
        let l_xz = r2 - a.xz().length_squared();
        let l_xy = r2 - a.xy().length_squared();

        Directed(Vec3::new(
            if l_yz < 0.0 {
                f32::MAX
            } else {
                a.x - l_yz.sqrt()
            },
            if l_xz < 0.0 {
                f32::MAX
            } else {
                a.y - l_xz.sqrt()
            },
            if l_xy < 0.0 {
                f32::MAX
            } else {
                a.z - l_xy.sqrt()
            },
        ))
    }
}

impl HermiteSource for Sphere {
    fn sample_normal(&self, p: Vec3) -> Vec3 {
        p
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sphere() {
        let sphere = Sphere::new(2.0);

        assert_eq!(sphere.sample_scalar(Vec3::ZERO).0, -2.0);
        assert_eq!(sphere.sample_scalar(Vec3::new(2.0, 0.0, 0.0)).0, 0.0);
        assert_eq!(sphere.sample_scalar(Vec3::new(0.0, 0.0, 8.0)).0, 6.0);
        assert_eq!(sphere.sample_scalar(Vec3::new(8.0, 0.0, 0.0)).0, 6.0);

        assert_eq!(sphere.sample_vector(Vec3::ZERO).0, Vec3::splat(-2.0));
        assert_eq!(sphere.sample_vector(Vec3::new(2.0, 0.0, 0.0)).0, Vec3::ZERO);
        assert_eq!(
            sphere.sample_vector(Vec3::new(0.0, 0.0, 8.0)).0,
            Vec3::new(f32::MAX, f32::MAX, 6.0)
        );
        assert_eq!(
            sphere.sample_vector(Vec3::new(8.0, 0.0, 0.0)).0,
            Vec3::new(6.0, f32::MAX, f32::MAX)
        );

        assert_eq!(
            sphere
                .sample_normal(Vec3::new(0.0, 0.0, 8.0))
                .try_normalize()
                .unwrap(),
            Vec3::new(0.0, 0.0, 1.0)
        );
        assert_eq!(
            sphere
                .sample_normal(Vec3::new(8.0, 0.0, 0.0))
                .try_normalize()
                .unwrap(),
            Vec3::new(1.0, 0.0, 0.0)
        );
    }
}
