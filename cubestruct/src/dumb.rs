use crate::cubestate::{CubeState, CubeStateConstructionError};
use crate::cubicle_indexed::{CornerCubicleIndexed, EdgeCubicleIndexed};
use crate::cubiestate::{
    CornerCubicle, CornerOrientation, CornerState, EdgeCubicle, EdgeOrientation, EdgeState,
};

/// A simpler cube representation than [`CubeState`]. A `DumbCube` is just an array of
/// 6 faces where each face is an array of 9 colors.
#[derive(Debug, Eq, PartialEq)]
pub struct DumbCube {
    /// See [`Self::get_face()`] for the layout of this array
    faces: [[Color; 9]; 6],
}

impl DumbCube {
    pub fn builder() -> DumbCubeBuilder {
        DumbCubeBuilder {
            initialized: [[false; 9]; 6],
            faces: [[Color::Blue; 9]; 6],
        }
    }

    pub fn to_cubestate(&self) -> Result<CubeState, DumbConversionError> {
        let corner_map = {
            use Color::*;
            use CornerCubicle::*;
            CornerCubicleIndexed::new([
                (C0, [White, Orange, Blue], [0, 0, 2]),
                (C1, [White, Blue, Red], [2, 0, 2]),
                (C2, [White, Green, Orange], [6, 0, 2]),
                (C3, [White, Red, Green], [8, 0, 2]),
                (C4, [Yellow, Blue, Orange], [6, 8, 6]),
                (C5, [Yellow, Red, Blue], [8, 8, 6]),
                (C6, [Yellow, Orange, Green], [0, 8, 6]),
                (C7, [Yellow, Green, Red], [2, 8, 6]),
            ])
        };

        // another yucky hack to avoid MaybeUninit (because logic error is easier to debug than UB)
        let mut corners = CornerCubicleIndexed::new(
            [CornerState::new(CornerCubicle::C0, CornerOrientation::O0); 8],
        );
        let mut edges =
            EdgeCubicleIndexed::new([EdgeState::new(EdgeCubicle::C0, EdgeOrientation::O0); 12]);

        for (home, home_colors, _) in corner_map {
            'orientations: for orientation in CornerOrientation::all() {
                // list of `(cubicle, oriented colors of the cubie in that cubicle)`
                let colored_corner_cubies =
                    corner_map.into_iter().map(|(cubicle, faces, indices)| {
                        let [fa, fb, fc] = faces;
                        let [ia, ib, ic] = indices;
                        (
                            cubicle,
                            [
                                self.get_face(fa)[ia],
                                self.get_face(fb)[ib],
                                self.get_face(fc)[ic],
                            ],
                        )
                    });

                for (cubicle, colors_in_cubicle) in colored_corner_cubies {
                    if corner_shift(colors_in_cubicle, orientation) == home_colors {
                        corners[home] = CornerState::new(cubicle, orientation);
                        break 'orientations;
                    }
                }
                // The `home` cubie doesn't exist in the DumbCube
                return Err(DumbConversionError::CornerCubieNotFound { cubicle: home });
            }
        }

        let edge_map = {
            use Color::*;
            use EdgeCubicle::*;
            EdgeCubicleIndexed::new([
                (C0, [White, Blue], [1, 1]),
                (C1, [White, Orange], [3, 1]),
                (C2, [White, Red], [5, 1]),
                (C3, [White, Green], [7, 1]),
                (C4, [Blue, Orange], [5, 3]),
                (C5, [Blue, Red], [3, 5]),
                (C6, [Green, Orange], [3, 5]),
                (C7, [Green, Red], [5, 3]),
                (C8, [Yellow, Blue], [7, 7]),
                (C9, [Yellow, Orange], [3, 7]),
                (C10, [Yellow, Red], [5, 7]),
                (C11, [Yellow, Green], [1, 7]),
            ])
        };

        for (home, home_colors, _) in edge_map {
            'orientations: for orientation in EdgeOrientation::all() {
                // list of `(cubicle, oriented colors of the cubie in that cubicle)`
                let colored_edge_cubies = edge_map.into_iter().map(|(cubicle, faces, indices)| {
                    let [fa, fb] = faces;
                    let [ia, ib] = indices;
                    (cubicle, [self.get_face(fa)[ia], self.get_face(fb)[ib]])
                });

                for (cubicle, colors_in_cubicle) in colored_edge_cubies {
                    if edge_shift(colors_in_cubicle, orientation) == home_colors {
                        edges[home] = EdgeState::new(cubicle, orientation);
                        break 'orientations;
                    }
                }
                // The `home` cubie doesn't exist in the DumbCube
                return Err(DumbConversionError::EdgeCubieNotFound { cubicle: home });
            }
        }

        Ok(CubeState::try_new(corners, edges)?)
    }

    pub(crate) fn from_cubestate(state: &CubeState) -> Self {
        let corner_cubie_colors = {
            use Color::*;
            use CornerCubicle::*;
            // (homecubicle, [clockwise_colors])
            CornerCubicleIndexed::new([
                // (C0, [White, Orange, Blue]) => the cubicle that lives in C0 has colors [W, O, B],
                // starting on the numbered face then going around clockwise
                (C0, [White, Orange, Blue]),
                (C1, [White, Blue, Red]),
                (C2, [White, Green, Orange]),
                (C3, [White, Red, Green]),
                (C4, [Yellow, Blue, Orange]),
                (C5, [Yellow, Red, Blue]),
                (C6, [Yellow, Orange, Green]),
                (C7, [Yellow, Green, Red]),
            ])
        };

        let corner_face_index_map = CornerCubicleIndexed::new([
            // `corner_cubie_colors` tells us that the cubie who lives at C0 has colors [W, O, B];
            // elements in the [0, 0, 2] array here correspond to the order of colors in that [W, O, B] array.
            // here, corner_face_index_map[C0] = [0, 0, 2] tells us that the C0 cubie has its white facelet
            // at index 0 of the white face (see `get_face()` for these indices), its orange facelet
            // at index 0 of the orange face, and its blue facelet at index 2 of the blue face.
            [0, 0, 2], // C0
            [2, 0, 2], // C1
            [6, 0, 2], // C2
            [8, 0, 2], // C3
            [6, 8, 6], // C4
            [8, 8, 6], // C5
            [0, 8, 6], // C6
            [2, 8, 6], // C7
        ]);

        let edge_cubie_colors = {
            use Color::*;
            use EdgeCubicle::*;
            // (homeplace, [X]) where X colors are ordered to start with the UD/FB face
            EdgeCubicleIndexed::new([
                (C0, [White, Blue]),
                (C1, [White, Orange]),
                (C2, [White, Red]),
                (C3, [White, Green]),
                (C4, [Blue, Orange]),
                (C5, [Blue, Red]),
                (C6, [Green, Orange]),
                (C7, [Green, Red]),
                (C8, [Yellow, Blue]),
                (C9, [Yellow, Orange]),
                (C10, [Yellow, Red]),
                (C11, [Yellow, Green]),
            ])
        };

        // example:
        // (C2, [White, Red]):
        // edge_face_index_map[C2 as usize] = [5, 1]
        // => the white facelet of the C2 cubie goes in index 5 of the white face
        // => the red facelet of the C2 cubie goes in index 1 of the red face
        let edge_face_index_map = EdgeCubicleIndexed::new([
            [1, 1], // C0
            [3, 1], // C1
            [5, 1], // C2
            [7, 1], // C3
            [5, 3], // C4
            [3, 5], // C5
            [3, 5], // C6
            [5, 3], // C7
            [7, 7], // C8
            [3, 7], // C9
            [5, 7], // C10
            [1, 7], // C11
        ]);

        // yucky way to avoid using MaybeUninit
        let mut faces = [[Color::Blue; 9]; 6];

        for (home_cubicle, colors) in corner_cubie_colors {
            // `cornerstate` is the cubie that normally lives in `home_cubicle`.
            // `colors` are the colors of that cubie.
            // `cornerstate.cubicle()` is the current cubicle that cubie is in
            let cornerstate = state.get_corner(home_cubicle);
            let oriented_colors = corner_shift(colors, cornerstate.orientation());
            let face_map = corner_face_index_map[cornerstate.cubicle()];
            let cubicle_colors = corner_cubie_colors[cornerstate.cubicle()].1;
            faces[cubicle_colors[0] as usize][face_map[0]] = oriented_colors[0];
            faces[cubicle_colors[1] as usize][face_map[1]] = oriented_colors[1];
            faces[cubicle_colors[2] as usize][face_map[2]] = oriented_colors[2];
        }

        for (home_cubicle, colors) in edge_cubie_colors {
            let edgestate = state.get_edge(home_cubicle);
            let oriented_colors = edge_shift(colors, edgestate.orientation());
            let face_map = edge_face_index_map[edgestate.cubicle()];
            let cubicle_colors = edge_cubie_colors[edgestate.cubicle()].1;
            faces[cubicle_colors[0] as usize][face_map[0]] = oriented_colors[0];
            faces[cubicle_colors[1] as usize][face_map[1]] = oriented_colors[1];
        }

        for c in Color::all() {
            faces[c as usize][4] = c;
        }

        Self { faces }
    }

    /// Gets the face of the given center color.
    ///
    /// Indices within faces' arrays are arranged like this:
    /// ```text
    ///           ┌──┬──┬──┐
    ///           │W0│W1│W2│
    ///           ├──┼──┼──┤
    ///           │W3│W4│W5│
    ///           ├──┼──┼──┤
    ///           │W6│W7│W8│
    ///           └──┴──┴──┘
    /// ┌──┬──┬──┐┌──┬──┬──┐┌──┬──┬──┐┌──┬──┬──┐
    /// │O0│O1│O2││G0│G1│G2││R0│R1│R2││B0│B1│B2│
    /// ├──┼──┼──┤├──┼──┼──┤├──┼──┼──┤├──┼──┼──┤
    /// │O3│O4│O5││G3│G4│G5││R3│R4│R5││B3│B4│B5│
    /// ├──┼──┼──┤├──┼──┼──┤├──┼──┼──┤├──┼──┼──┤
    /// │O6│O7│O8││G6│G7│G8││R6│R7│R8││B6│B7│B8│
    /// └──┴──┴──┘└──┴──┴──┘└──┴──┴──┘└──┴──┴──┘
    ///           ┌──┬──┬──┐
    ///           │Y0│Y1│Y2│
    ///           ├──┼──┼──┤
    ///           │Y3│Y4│Y5│
    ///           ├──┼──┼──┤
    ///           │Y6│Y7│Y8│
    ///           └──┴──┴──┘
    /// ```
    pub fn get_face(&self, center: Color) -> [Color; 9] {
        self.faces[center as usize]
    }
}

#[derive(thiserror::Error, Debug)]
pub enum DumbConversionError {
    #[error("The cubie that lives in {cubicle:?} was not found in the DumbCube")]
    CornerCubieNotFound { cubicle: CornerCubicle },
    #[error("The cubie that lives in {cubicle:?} was not found in the DumbCube")]
    EdgeCubieNotFound { cubicle: EdgeCubicle },
    #[error("CubeState::try_new() failed")]
    CubeStateConstruction(CubeStateConstructionError),
}

impl From<CubeStateConstructionError> for DumbConversionError {
    fn from(e: CubeStateConstructionError) -> Self {
        Self::CubeStateConstruction(e)
    }
}

/// Rotates a corner's colors from the given solved orientation into the orientation `orientation`.
const fn corner_shift([a, b, c]: [Color; 3], orientation: CornerOrientation) -> [Color; 3] {
    match orientation {
        CornerOrientation::O0 => [a, b, c],
        CornerOrientation::O1 => [c, a, b],
        CornerOrientation::O2 => [b, c, a],
    }
}

/// Flips an edge's colors from the given solved orientation into the orientation `orientation`.
const fn edge_shift([a, b]: [Color; 2], orientation: EdgeOrientation) -> [Color; 2] {
    match orientation {
        EdgeOrientation::O0 => [a, b],
        EdgeOrientation::O1 => [b, a],
    }
}

#[derive(Debug)]
pub struct DumbCubeBuilder {
    initialized: [[bool; 9]; 6],
    faces: [[Color; 9]; 6],
}

impl DumbCubeBuilder {
    /// Returns `None` if not all faces were initialized
    pub fn build(self) -> Option<DumbCube> {
        if self.initialized.into_iter().flatten().any(|x| x == false) {
            None
        } else {
            Some(DumbCube { faces: self.faces })
        }
    }

    /// Set the facelet at the given index (on the given color's side) to the given color
    #[inline]
    pub fn set(&mut self, face: Color, index: usize, set_to: Color) {
        assert!(index <= 8, "Provided index ({index}) out of bounds");
        self.initialized[face as usize][index] = true;
        self.faces[face as usize][index] = set_to;
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum Color {
    Orange,
    Red,
    Yellow,
    White,
    Green,
    Blue,
}

impl Color {
    fn all() -> [Self; 6] {
        [
            Self::Orange,
            Self::Red,
            Self::Yellow,
            Self::White,
            Self::Green,
            Self::Blue,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dumb_conversions() {
        assert_eq!(
            CubeState::SOLVED.to_dumb().to_cubestate().unwrap(),
            CubeState::SOLVED
        );

        let tperm_dumb = {
            use Color::*;
            DumbCube {
                faces: [
                    [
                        Orange, Red, Orange, Orange, Orange, Orange, Orange, Orange, Orange,
                    ],
                    [Blue, Orange, Green, Red, Red, Red, Red, Red, Red],
                    [
                        Yellow, Yellow, Yellow, Yellow, Yellow, Yellow, Yellow, Yellow, Yellow,
                    ],
                    [
                        White, White, White, White, White, White, White, White, White,
                    ],
                    [Green, Green, Red, Green, Green, Green, Green, Green, Green],
                    [Red, Blue, Blue, Blue, Blue, Blue, Blue, Blue, Blue],
                ],
            }
        };
        let tperm_cubestate = tperm_dumb.to_cubestate().unwrap();

        assert_eq!(tperm_dumb, tperm_cubestate.to_dumb());
        assert_eq!(
            tperm_cubestate,
            tperm_cubestate.to_dumb().to_cubestate().unwrap()
        );
    }
}

////////////////////////////////
// TODO: Remove all the stuff below here once we get a good 3d rendering thing going
////////////////////////////////

impl Color {
    fn emoji(self) -> &'static str {
        match self {
            Self::Orange => "🟧",
            Self::Red => "🟥",
            Self::Yellow => "🟨",
            Self::White => "⬜",
            Self::Green => "🟩",
            Self::Blue => "🟦",
        }
    }
}

const TMPL: [&str; 7] = [
    "┌──┬──┬──┐",
    "│⬛│⬛│⬛",
    "├──┼──┼──┤",
    "│⬛│⬛│⬛",
    "├──┼──┼──┤",
    "│⬛│⬛│⬛",
    "└──┴──┴──┘",
];
const TMPLSPACE: &str = "          ";

fn print_template_line(lnr: usize, facelet_colors: [Color; 9]) {
    if TMPL[lnr].contains("⬛") {
        let x = TMPL[lnr]
            .split("⬛")
            .zip(facelet_colors.chunks(3).nth(lnr / 2).unwrap())
            .flat_map(|(a, color)| [a, color.emoji()])
            .collect::<Vec<_>>()
            .join("");

        print!("{x}│");
    } else {
        print!("{}", TMPL[lnr]);
    }
}

fn println_render_cube(state: &CubeState) {
    let render = DumbCube::from_cubestate(state);
    for i in 0..7 {
        print!("{TMPLSPACE}");
        print_template_line(i, render.get_face(Color::White));
        println!();
    }
    for i in 0..7 {
        print_template_line(i, render.get_face(Color::Orange));
        print_template_line(i, render.get_face(Color::Green));
        print_template_line(i, render.get_face(Color::Red));
        print_template_line(i, render.get_face(Color::Blue));
        println!();
    }
    for i in 0..7 {
        print!("{TMPLSPACE}");
        print_template_line(i, render.get_face(Color::Yellow));
        println!();
    }
}
