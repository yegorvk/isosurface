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
    distance::{Directed, Distance, Signed},
    source::{HermiteSource, ScalarSource, VectorSource},
};
use glam::Vec3;

/// Sample a distance field defined in terms of a specific [Distance] metric.
pub trait Sample<D: Distance>: Sized {
    fn sample(&self, p: Vec3) -> D;
}

/// Samplers abstract sampling across multiple different [Distance] metrics
pub struct Sampler<'a, S> {
    pub source: &'a S,
}

impl<'a, S> Sampler<'a, S> {
    /// Create a new sampler from a source.
    pub fn new(source: &'a S) -> Self {
        Self { source }
    }
}

impl<S: ScalarSource> Sample<Signed> for Sampler<'_, S> {
    fn sample(&self, p: Vec3) -> Signed {
        self.source.sample_scalar(p)
    }
}

impl<S: VectorSource> Sample<Directed> for Sampler<'_, S> {
    fn sample(&self, p: Vec3) -> Directed {
        self.source.sample_vector(p)
    }
}

impl<S: ScalarSource> ScalarSource for Sampler<'_, S> {
    fn sample_scalar(&self, p: Vec3) -> Signed {
        self.source.sample_scalar(p)
    }
}

impl<S: VectorSource + ScalarSource> VectorSource for Sampler<'_, S> {
    fn sample_vector(&self, p: Vec3) -> Directed {
        self.source.sample_vector(p)
    }
}

impl<S: HermiteSource> HermiteSource for Sampler<'_, S> {
    fn sample_normal(&self, p: Vec3) -> Vec3 {
        self.source.sample_normal(p)
    }
}
