use crate::morton::Morton;
use const_array_map::{const_array_map, ConstArrayMap, PrimitiveEnum};
use std::mem;
use std::mem::MaybeUninit;

/// An octree node/cell.
#[repr(transparent)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Cell(Morton);

impl Cell {
    /// Creates a new `Cell` from its morton code.
    ///
    /// If `key` doesn't represent a valid octree node, returns None.
    pub fn new(key: Morton) -> Option<Cell> {
        if !key.is_none() {
            Some(Cell(key))
        } else {
            None
        }
    }

    /// Retrieves the morton code ("key") corresponding this cell.
    pub fn key(&self) -> Morton {
        self.0
    }

    /// Retrieves the sub-cell of this cell adjacent to `corner`.
    ///
    /// This method does not distinguish between interior and leaf cells,
    /// so the caller must ensure that `self` is not a leaf to preserve
    /// the expected behavior.
    fn sub_cell(&self, corner: Corner) -> Cell {
        Cell(self.0.child(corner.0))
    }

    /// Returns an iterator over this cell's sub-cells, in any order.
    ///
    /// This method does not distinguish between interior and leaf cells, so
    /// the returned iterator will always yield 8 elements.
    pub fn subdivide(&self) -> impl Iterator<Item = Cell> + use<'_> {
        (0..8).map(|i| Cell(self.0.child(i)))
    }

    /// Returns an iterator over this cell's interior faces, i.e., those
    /// between the face-adjacent pairs of its sub-cells, in any order.
    ///
    /// This method does not distinguish between interior and leaf cells, so
    /// the returned iterator will always yield 12 elements.
    pub fn interior_faces(&self) -> impl Iterator<Item = Face> + use<'_> {
        CELL_EDGES.iter().map(|edge| Face::from_edge(self, *edge))
    }

    /// Returns an iterator over this cell's interior edges, i.e., those
    /// adjacent to 4 sub-cells of this cell at a time, in any order.
    ///
    /// This method does not distinguish between interior and leaf cells, so
    /// the returned iterator will always yield 6 elements.
    pub fn interior_edges(&self) -> impl Iterator<Item = Edge> + use<'_> {
        CELL_FACES.iter().map(|face| Edge::from_face(self, *face))
    }

    /// Retrieves the sub-cells of this cell adjacent to the given face.
    fn face_sub_cells(&self, face: FaceKind) -> [Cell; 4] {
        CELL_FACE_CORNERS[face].map(|corner| self.sub_cell(corner))
    }

    /// Retrieves the sub-cells of this cell adjacent to the given edge.
    fn edge_sub_cells(&self, edge: EdgeKind) -> [Cell; 2] {
        let (start, dir) = edge.into_parts();
        [self.sub_cell(start), self.sub_cell(start.step(dir))]
    }
}

const STEP_X: u8 = 0x1;
const STEP_Y: u8 = 0x2;
const STEP_Z: u8 = 0x4;

#[repr(transparent)]
#[derive(Debug, Copy, Clone)]
struct Dir(u8);

impl Dir {
    const X: Dir = Dir(STEP_X);
    const Y: Dir = Dir(STEP_Y);
    const Z: Dir = Dir(STEP_Z);

    fn axis(&self) -> Axis {
        match self.0 {
            STEP_X => Axis::X,
            STEP_Y => Axis::Y,
            STEP_Z => Axis::Z,
            _ => unreachable!(),
        }
    }
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, PrimitiveEnum)]
enum Axis {
    X, // 0
    Y, // 1
    Z, // 2
}

#[repr(transparent)]
#[derive(Debug, Default, Copy, Clone)]
struct Corner(u8);

impl Corner {
    const LEFT_BOTTOM_BACK: Corner = Corner(0);
    const RIGHT_BOTTOM_BACK: Corner = Corner(STEP_X);
    const LEFT_TOP_BACK: Corner = Corner(STEP_Y);
    const LEFT_BOTTOM_FRONT: Corner = Corner(STEP_Z);
    const RIGHT_TOP_BACK: Corner = Corner(STEP_X | STEP_Y);
    const LEFT_TOP_FRONT: Corner = Corner(STEP_Y | STEP_Z);
    const RIGHT_BOTTOM_FRONT: Corner = Corner(STEP_X | STEP_Z);
    const RIGHT_TOP_FRONT: Corner = Corner(STEP_X | STEP_Y | STEP_Z);

    fn step(&self, dir: Dir) -> Corner {
        Corner(self.0 | dir.0)
    }
}

#[repr(transparent)]
#[derive(Debug, Copy, Clone)]
struct EdgeKind(u8);

impl EdgeKind {
    const fn new(start: Corner, dir: Dir) -> EdgeKind {
        EdgeKind(start.0 << 3 | dir.0)
    }

    const fn dir(&self) -> Dir {
        Dir(self.0 & 0x7)
    }

    const fn into_parts(self) -> (Corner, Dir) {
        (Corner(self.0 >> 3), self.dir())
    }
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, PrimitiveEnum)]
enum FaceKind {
    Left,   // 0
    Right,  // 1
    Bottom, // 2
    Top,    // 3
    Back,   // 4
    Front,  // 5
}

impl FaceKind {
    fn normal(&self) -> Axis {
        match self {
            FaceKind::Left | FaceKind::Right => Axis::X,
            FaceKind::Bottom | FaceKind::Top => Axis::Y,
            FaceKind::Back | FaceKind::Front => Axis::Z,
        }
    }
}

const CELL_EDGES: [EdgeKind; 12] = [
    EdgeKind::new(Corner::LEFT_BOTTOM_BACK, Dir::X),
    EdgeKind::new(Corner::LEFT_BOTTOM_BACK, Dir::Y),
    EdgeKind::new(Corner::LEFT_BOTTOM_BACK, Dir::Z),
    EdgeKind::new(Corner::RIGHT_BOTTOM_BACK, Dir::Y),
    EdgeKind::new(Corner::RIGHT_BOTTOM_BACK, Dir::Z),
    EdgeKind::new(Corner::LEFT_TOP_BACK, Dir::X),
    EdgeKind::new(Corner::LEFT_TOP_BACK, Dir::Z),
    EdgeKind::new(Corner::LEFT_BOTTOM_FRONT, Dir::X),
    EdgeKind::new(Corner::LEFT_BOTTOM_FRONT, Dir::Y),
    EdgeKind::new(Corner::LEFT_TOP_FRONT, Dir::X),
    EdgeKind::new(Corner::RIGHT_BOTTOM_FRONT, Dir::Y),
    EdgeKind::new(Corner::RIGHT_TOP_BACK, Dir::Z),
];

const CELL_FACES: [FaceKind; 6] = [
    FaceKind::Left,
    FaceKind::Right,
    FaceKind::Bottom,
    FaceKind::Top,
    FaceKind::Back,
    FaceKind::Front,
];

const CELL_FACE_CORNERS: ConstArrayMap<FaceKind, [Corner; 4]> = const_array_map! {
    FaceKind::Left => [
        Corner::LEFT_BOTTOM_BACK,
        Corner::LEFT_TOP_BACK,
        Corner::LEFT_TOP_FRONT,
        Corner::LEFT_BOTTOM_FRONT,
    ],
    FaceKind::Right => [
        Corner::RIGHT_BOTTOM_BACK,
        Corner::RIGHT_TOP_BACK,
        Corner::RIGHT_TOP_FRONT,
        Corner::RIGHT_BOTTOM_FRONT,
    ],
    FaceKind::Bottom => [
        Corner::LEFT_BOTTOM_BACK,
        Corner::RIGHT_BOTTOM_BACK,
        Corner::RIGHT_BOTTOM_FRONT,
        Corner::LEFT_BOTTOM_FRONT,
    ],
    FaceKind::Top => [
        Corner::LEFT_TOP_BACK,
        Corner::RIGHT_TOP_BACK,
        Corner::RIGHT_TOP_FRONT,
        Corner::LEFT_TOP_FRONT,
    ],
    FaceKind::Back => [
        Corner::LEFT_BOTTOM_BACK,
        Corner::LEFT_TOP_BACK,
        Corner::RIGHT_TOP_BACK,
        Corner::RIGHT_BOTTOM_BACK,
    ],
    FaceKind::Front => [
        Corner::LEFT_BOTTOM_FRONT,
        Corner::LEFT_TOP_FRONT,
        Corner::RIGHT_TOP_FRONT,
        Corner::RIGHT_BOTTOM_FRONT,
    ],
};

const EDGE_NEIGHBORS: ConstArrayMap<Axis, [EdgeKind; 4]> = const_array_map! {
    Axis::X => [
        EdgeKind::new(Corner::LEFT_BOTTOM_BACK, Dir::X),
        EdgeKind::new(Corner::LEFT_TOP_BACK, Dir::X),
        EdgeKind::new(Corner::LEFT_TOP_FRONT, Dir::X),
        EdgeKind::new(Corner::LEFT_BOTTOM_FRONT, Dir::X),
    ],
    Axis::Y => [
        EdgeKind::new(Corner::LEFT_BOTTOM_BACK, Dir::Y),
        EdgeKind::new(Corner::RIGHT_BOTTOM_BACK, Dir::Y),
        EdgeKind::new(Corner::RIGHT_BOTTOM_FRONT, Dir::Y),
        EdgeKind::new(Corner::LEFT_BOTTOM_FRONT, Dir::Y),
    ],
    Axis::Z => [
        EdgeKind::new(Corner::LEFT_BOTTOM_BACK, Dir::Z),
        EdgeKind::new(Corner::RIGHT_BOTTOM_BACK, Dir::Z),
        EdgeKind::new(Corner::RIGHT_TOP_BACK, Dir::Z),
        EdgeKind::new(Corner::LEFT_TOP_BACK, Dir::Z),
    ],
};

const FACE_NEIGHBORS: ConstArrayMap<Axis, [FaceKind; 2]> = const_array_map! {
    Axis::X => [FaceKind::Left, FaceKind::Right],
    Axis::Y => [FaceKind::Bottom, FaceKind::Top],
    Axis::Z => [FaceKind::Back, FaceKind::Front],
};

#[allow(clippy::type_complexity)]
const FACE_EDGES: ConstArrayMap<Axis, [(Axis, [[(usize, Corner); 4]; 2]); 2]> = const_array_map! {
    Axis::X => [
        (
            Axis::Y,
            [
                [
                    (0, Corner::RIGHT_BOTTOM_BACK),
                    (1, Corner::LEFT_BOTTOM_BACK),
                    (1, Corner::LEFT_BOTTOM_FRONT),
                    (0, Corner::RIGHT_BOTTOM_FRONT),
                ],
                [
                    (0, Corner::RIGHT_TOP_BACK),
                    (1, Corner::LEFT_TOP_BACK),
                    (1, Corner::LEFT_TOP_FRONT),
                    (0, Corner::RIGHT_TOP_FRONT),
                ],
            ]
        ),
        (
            Axis::Z,
            [
                [
                    (0, Corner::RIGHT_BOTTOM_BACK),
                    (1, Corner::LEFT_BOTTOM_BACK),
                    (1, Corner::LEFT_TOP_BACK),
                    (0, Corner::RIGHT_TOP_BACK),
                ],
                [
                    (0, Corner::RIGHT_BOTTOM_FRONT),
                    (1, Corner::LEFT_BOTTOM_FRONT),
                    (1, Corner::LEFT_TOP_FRONT),
                    (0, Corner::RIGHT_TOP_FRONT),
                ],
            ]
        ),
    ],
    Axis::Y => [
        (
            Axis::X,
            [
                [
                    (0, Corner::LEFT_TOP_BACK),
                    (1, Corner::LEFT_BOTTOM_BACK),
                    (1, Corner::LEFT_BOTTOM_FRONT),
                    (0, Corner::LEFT_TOP_FRONT),
                ],
                [
                    (0, Corner::RIGHT_TOP_BACK),
                    (1, Corner::RIGHT_BOTTOM_BACK),
                    (1, Corner::RIGHT_BOTTOM_FRONT),
                    (0, Corner::RIGHT_TOP_FRONT),
                ],
            ]
        ),
        (
            Axis::Z,
            [
                [
                    (0, Corner::LEFT_TOP_BACK),
                    (0, Corner::RIGHT_TOP_BACK),
                    (1, Corner::RIGHT_BOTTOM_BACK),
                    (1, Corner::LEFT_BOTTOM_BACK),
                ],
                [
                    (0, Corner::LEFT_TOP_FRONT),
                    (0, Corner::RIGHT_TOP_FRONT),
                    (1, Corner::RIGHT_BOTTOM_FRONT),
                    (1, Corner::LEFT_BOTTOM_FRONT),
                ],
            ]
        ),
    ],
    Axis::Z => [
        (
            Axis::X,
            [
                [
                    (0, Corner::LEFT_BOTTOM_FRONT),
                    (0, Corner::LEFT_TOP_FRONT),
                    (1, Corner::LEFT_TOP_BACK),
                    (1, Corner::LEFT_BOTTOM_BACK),
                ],
                [
                    (0, Corner::RIGHT_BOTTOM_FRONT),
                    (0, Corner::RIGHT_TOP_FRONT),
                    (1, Corner::RIGHT_TOP_BACK),
                    (1, Corner::RIGHT_BOTTOM_BACK),
                ],
            ]
        ),
        (
            Axis::Y,
            [
                [
                    (0, Corner::LEFT_BOTTOM_FRONT),
                    (0, Corner::RIGHT_BOTTOM_FRONT),
                    (1, Corner::RIGHT_BOTTOM_BACK),
                    (1, Corner::LEFT_BOTTOM_BACK),
                ],
                [
                    (0, Corner::LEFT_TOP_FRONT),
                    (0, Corner::RIGHT_TOP_FRONT),
                    (1, Corner::RIGHT_TOP_BACK),
                    (1, Corner::LEFT_TOP_BACK),
                ],
            ]
        ),
    ],
};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Face {
    normal: Axis,
    neighbors: [Cell; 2],
}

impl Face {
    fn from_edge(cell: &Cell, edge: EdgeKind) -> Face {
        let neighbors = cell.edge_sub_cells(edge);
        Self {
            neighbors,
            normal: edge.dir().axis(),
        }
    }

    // TODO: rewrite using safe abstractions.
    pub fn subdivide_by<F>(&self, mut is_leaf: F) -> Option<[Face; 4]>
    where
        F: FnMut(&Cell) -> bool,
    {
        let first = self.neighbors[0];
        let second = self.neighbors[1];

        let subdivide_first = is_leaf(&first);
        let subdivide_second = is_leaf(&second);

        if !subdivide_first && !subdivide_second {
            return None;
        }

        let sub_cells_first = if !is_leaf(&first) {
            first.face_sub_cells(FACE_NEIGHBORS[self.normal][0])
        } else {
            [first; 4]
        };

        let sub_cells_second = if !is_leaf(&second) {
            second.face_sub_cells(FACE_NEIGHBORS[self.normal][1])
        } else {
            [second; 4]
        };

        // SAFETY: reinterpreting uninitialized memory as an array of
        // uninitialized values is always safe.
        let mut sub_faces: [MaybeUninit<Face>; 4] = unsafe { MaybeUninit::uninit().assume_init() };

        for i in 0..4 {
            sub_faces[i] = MaybeUninit::new(Face {
                normal: self.normal,
                neighbors: [sub_cells_first[i], sub_cells_second[i]],
            });
        }

        // SAFETY: we have just fully initialized `sub_faces`.
        let sub_face: [Face; 4] = unsafe { mem::transmute(sub_faces) };

        Some(sub_face)
    }
    
    // pub fn interior_edges(&self, sub_faces: &[Face; 4]) -> [Edge; 4] {
    //     // SAFETY: reinterpreting uninitialized memory as an array of
    //     // uninitialized values is always safe.
    //     let mut interior_edges: [MaybeUninit<Edge>; 4] =
    //         unsafe { MaybeUninit::uninit().assume_init() };
    // 
    //     let mut i = 0;
    // 
    //     for (axis, edges) in &FACE_EDGES[self.normal] {
    //         for neighbors in edges {
    //             let neighbors =
    //                 neighbors.map(|(which, corner)| self.neighbors[which].sub_cell(corner));
    // 
    //             interior_edges[i].write(Edge::new(*axis, neighbors));
    //             i += 1;
    //         }
    //     }
    // 
    //     // SAFETY: we have just fully initialized `interior_edges`.
    //     unsafe { mem::transmute(interior_edges) }
    // }
    
    pub fn interior_edges(&self) -> impl Iterator<Item = Edge> + use<'_> {
        FACE_EDGES[self.normal].iter().flat_map(|(axis, edges)| {
            edges.map(|neighbors| {
                let neighbors = neighbors.map(|(which, corner)| {
                    self.neighbors[which].sub_cell(corner)
                });

                Edge::new(*axis, neighbors)
            })
        })
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Edge {
    axis: Axis,
    neighbors: [Cell; 4],
}

impl Edge {
    fn new(axis: Axis, neighbors: [Cell; 4]) -> Edge {
        Self { axis, neighbors }
    }

    fn from_face(cell: &Cell, face: FaceKind) -> Edge {
        let neighbors = cell.face_sub_cells(face);
        let axis = face.normal();
        Self { axis, neighbors }
    }
    
    pub fn neighbors(&self) -> &[Cell; 4] {
        &self.neighbors
    }

    // TODO: rewrite using safe abstractions.
    fn subdivide(&self, mask: EdgeNeighborsMask) -> Option<[Edge; 2]> {
        if mask.is_empty() {
            return None;
        }

        // SAFETY: reinterpreting uninitialized memory as an array
        // of uninitialized values is always safe.
        let mut halves: [[MaybeUninit<Cell>; 4]; 2] =
            unsafe { MaybeUninit::uninit().assume_init() };

        for i in 0..4usize {
            let [a, b] = if mask.contains(i as u8) {
                self.neighbors[i].edge_sub_cells(EDGE_NEIGHBORS[self.axis][i])
            } else {
                [self.neighbors[i], self.neighbors[i]]
            };

            (halves[0][i], halves[1][i]) = (MaybeUninit::new(a), MaybeUninit::new(b));
        }

        // SAFETY: we have just fully initialized `halves`.
        let halves: [[Cell; 4]; 2] = unsafe { mem::transmute(halves) };

        let sub_edges = halves.map(|neighbors| Edge {
            axis: self.axis,
            neighbors,
        });

        Some(sub_edges)
    }

    pub fn subdivide_by<F>(&self, is_leaf: F) -> Option<[Edge; 2]>
    where
        F: FnMut(&Cell) -> bool,
    {
        let mask = EdgeNeighborsMask::from_fn(self, is_leaf);
        self.subdivide(mask)
    }
}

#[repr(transparent)]
#[derive(Debug, Default, Copy, Clone)]
struct EdgeNeighborsMask(u8);

impl EdgeNeighborsMask {
    fn from_fn<F>(edge: &Edge, mut f: F) -> Self
    where
        F: FnMut(&Cell) -> bool,
    {
        let mask = edge
            .neighbors
            .iter()
            .enumerate()
            .map(|(i, neighbor)| (!f(neighbor) as u8) << i as u8)
            .sum();

        EdgeNeighborsMask(mask)
    }

    fn is_empty(&self) -> bool {
        self.0 == 0
    }

    fn contains(&self, i: u8) -> bool {
        (self.0 & i) != 0
    }
}
