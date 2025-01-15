use std::io::empty;
use crate::morton::Morton;
use crate::octree_topology::{Cell, Edge, Face};

pub struct MinimalEdges;

impl MinimalEdges {
    pub fn traverse<L, E>(&self, mut is_leaf: L, mut edge_callback: E)
    where
        L: FnMut(Morton) -> bool,
        E: FnMut(&[Morton; 4]),
    {
        let mut stack = Vec::new();
        stack.push(Primitive::Cell(Cell::new(Morton::ROOT).unwrap()));
        
        let mut is_leaf = |cell: &Cell| is_leaf(cell.key());

        while let Some(primitive) = stack.pop() {
            match primitive {
                Primitive::Cell(cell) => {
                    if is_leaf(&cell) {
                        continue;
                    }
                    
                    for sub_cell in cell.subdivide(){
                        stack.push(Primitive::Cell(sub_cell));
                    }
                    
                    for interior_face in cell.interior_faces() {
                        stack.push(Primitive::Face(interior_face));
                    }
                    
                    for interior_edge in cell.interior_edges() {
                        stack.push(Primitive::Edge(interior_edge));
                    }
                }
                Primitive::Face(face) => {
                    if let Some(sub_faces) = face.subdivide_by(&mut is_leaf) {
                        for sub_face in sub_faces {
                            stack.push(Primitive::Face(sub_face))
                        }
                    }
                    
                    
                }
                Primitive::Edge(edge) => {}
            }
        }
    }
}

enum Primitive {
    Cell(Cell),
    Face(Face),
    Edge(Edge),
}
