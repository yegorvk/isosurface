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

/// A capped cylinder.
pub struct Cylinder {
    /// The radius of the cylinder.
    pub radius: f32,
    /// Half the length of the cylinder, the distance from the center point
    /// to each capped end.
    pub half_length: f32,
}

impl Cylinder {
    /// Create a capped cylinder from the desired radius and half of the desired
    /// length.
    pub fn new(radius: f32, half_length: f32) -> Self {
        Self {
            radius,
            half_length,
        }
    }
}

impl ScalarSource for Cylinder {
    fn sample_scalar(&self, p: Vec3) -> Signed {
        let q_x = p.xy().length().abs() - self.radius;
        let q_z = p.z.abs() - self.half_length;
        let d = Vec3::new(q_x.max(0.0), q_z.max(0.0), 0.0);
        Signed(q_x.max(q_z).min(0.0) + d.length())
    }
}

impl VectorSource for Cylinder {
    fn sample_vector(&self, p: Vec3) -> Directed {
        // Flip the point into the positive quadrant
        let a = p.abs();
        Directed(Vec3::new(
            if a.z > self.half_length || a.y > self.radius {
                f32::MAX
            } else {
                a.x - (self.radius * self.radius - a.y * a.y).sqrt()
            },
            if a.z > self.half_length || a.x > self.radius {
                f32::MAX
            } else {
                a.y - (self.radius * self.radius - a.x * a.x).sqrt()
            },
            if a.xy().length_squared() > self.radius * self.radius {
                f32::MAX
            } else {
                a.z - self.half_length
            },
        ))
    }
}

impl HermiteSource for Cylinder {
    fn sample_normal(&self, p: Vec3) -> Vec3 {
        let z = p.z.abs() / self.half_length;
        let r = (p.x * p.x + p.y * p.y).sqrt() / self.radius;

        if z > r {
            Vec3::new(0.0, 0.0, p.z)
        } else {
            Vec3::new(p.x, p.y, 0.0)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec2;

    #[test]
    fn test_cylinder() {
        let cylinder = Cylinder::new(2.0, 4.0);

        assert_eq!(cylinder.sample_scalar(Vec3::ZERO).0, -2.0);
        assert_eq!(cylinder.sample_scalar(Vec3::new(2.0, 0.0, 4.0)).0, 0.0);
        assert_eq!(cylinder.sample_scalar(Vec3::new(0.0, 0.0, 8.0)).0, 4.0);
        assert_eq!(cylinder.sample_scalar(Vec3::new(8.0, 0.0, 0.0)).0, 6.0);

        assert_eq!(
            cylinder.sample_vector(Vec3::ZERO).0,
            Vec3::new(-2.0, -2.0, -4.0)
        );
        assert_eq!(
            cylinder.sample_vector(Vec3::new(0.0, 0.0, 1.0)).0,
            Vec3::new(-2.0, -2.0, -3.0)
        );
        assert_eq!(
            cylinder.sample_vector(Vec3::new(1.0, 1.0, 1.0)).0,
            Vec2::splat(1.0 - (4.0f32 - 1.0f32).sqrt()).extend(-3.0)
        );
        assert_eq!(
            cylinder.sample_vector(Vec3::new(2.0, 0.0, 4.0)).0,
            Vec3::ZERO
        );
        assert_eq!(
            cylinder.sample_vector(Vec3::new(0.0, 0.0, 8.0)).0,
            Vec3::new(f32::MAX, f32::MAX, 4.0)
        );
        assert_eq!(
            cylinder.sample_vector(Vec3::new(8.0, 0.0, 0.0)).0,
            Vec3::new(6.0, f32::MAX, f32::MAX)
        );

        assert_eq!(
            cylinder
                .sample_normal(Vec3::new(0.0, 0.0, 8.0))
                .try_normalize()
                .unwrap(),
            Vec3::new(0.0, 0.0, 1.0)
        );
        assert_eq!(
            cylinder
                .sample_normal(Vec3::new(8.0, 0.0, 0.0))
                .try_normalize()
                .unwrap(),
            Vec3::new(1.0, 0.0, 0.0)
        );
    }
}
