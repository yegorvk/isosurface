use bevy::prelude::*;
use isosurface::distance::Signed;
use isosurface::source::ScalarSource;
use noise::{NoiseFn, Perlin};

pub struct Plane;

impl ScalarSource for Plane {
    fn sample_scalar(&self, p: Vec3) -> Signed {
        let p = Vec3::new(p.x, p.y, p.z);
        let v = p - 0.5;
        Signed(v.y)
    }
}

pub struct Sphere;

impl ScalarSource for Sphere {
    fn sample_scalar(&self, p: Vec3) -> Signed {
        let p = Vec3::new(p.x, p.y, p.z);
        let v = p - 0.5;
        //Signed(0.4 * 0.4 - v.length_squared())
        Signed(v.length_squared() - 0.4 * 0.4)
    }
}

pub struct DistortedSphere;

impl ScalarSource for DistortedSphere {
    fn sample_scalar(&self, p: Vec3) -> Signed {
        let p = Vec3::new(p.x, p.y, p.z);
        let v = p - 0.5;
        Signed(v.length() + (v.element_sum() * 32.0).sin() * 0.02 - 0.25)
    }
}

#[derive(Default)]
pub struct PerlinNoise2D {
    perlin: Perlin,
}

impl ScalarSource for PerlinNoise2D {
    fn sample_scalar(&self, p: Vec3) -> Signed {
        let p = Vec3::new(p.x, p.y, p.z);
        let v = p - 0.5;
        Signed(v.y - 0.1 * self.perlin.get((v.xz() * 8.0).as_dvec2().to_array()) as f32)
    }
}

#[derive(Default)]
pub struct PerlinNoise3D {
    perlin: Perlin,
}

impl ScalarSource for PerlinNoise3D {
    fn sample_scalar(&self, p: Vec3) -> Signed {
        let p = Vec3::new(p.x, p.y, p.z);
        let v = p - 0.5;
        Signed(1.0 * self.perlin.get((v * 8.0).as_dvec3().to_array()) as f32)
    }
}
