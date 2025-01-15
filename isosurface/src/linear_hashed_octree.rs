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

use crate::morton::Morton;
use fxhash::FxHashMap;
use std::collections::VecDeque;

pub enum OctreeNode<Node> {
    Internal,
    Leaf(Node),
    None,
}

impl<Node> OctreeNode<Node> {
    pub fn into_leaf(self) -> Option<Node> {
        match self {
            OctreeNode::Leaf(leaf) => Some(leaf),
            _ => None,
        }
    }
}

pub struct LinearHashedOctree<Node> {
    nodes: FxHashMap<Morton, Node>,
}

impl<Node> LinearHashedOctree<Node> {
    pub fn new() -> Self {
        Self {
            nodes: FxHashMap::default(),
        }
    }

    pub fn build<C>(&mut self, mut create_node: C)
    where
        C: FnMut(Morton) -> OctreeNode<Node>,
    {
        let mut queue = VecDeque::new();
        queue.push_back(Morton::new());

        while let Some(key) = queue.pop_front() {
            match create_node(key) {
                OctreeNode::Internal => {
                    for i in 0..8 {
                        queue.push_back(key.child(i));
                    }
                }
                OctreeNode::Leaf(node) => {
                    self.nodes.insert(key, node);
                }
                OctreeNode::None => {}
            };
        }
    }

    /// Returns an octree node given its coordinates.
    #[inline]
    pub fn get_node(&self, key: Morton) -> OctreeNode<&Node> {
        if !key.is_none() {
            return OctreeNode::None;
        }

        match self.nodes.get(&key) {
            Some(node) => OctreeNode::Leaf(node),
            None => OctreeNode::Internal,
        }
    }
}
