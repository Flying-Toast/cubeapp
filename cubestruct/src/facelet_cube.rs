use crate::cubie::*;
use crate::cubie_cube::{CubieCube, CubieCubeConstructionError};
use std::ops::{Index, IndexMut};

trait CubiesExt: Cubies {
    type FaceletArray<T: Eq + Copy>: Copy
        + Eq
        + Index<usize, Output = T>
        + IndexMut<usize>
        + IntoIterator<Item = T>;

    fn new_facelet_array<T: Eq + Copy>(init: T) -> Self::FaceletArray<T>;

    fn make_missing_cubie_err(cubicle: Self::Cubicle) -> FaceletConversionError;

    fn rotate_facelet_array<T: Eq + Copy>(
        arr: Self::FaceletArray<T>,
        amt: Self::Orientation,
    ) -> Self::FaceletArray<T>;
}

impl CubiesExt for Corners {
    type FaceletArray<T: Eq + Copy> = [T; 3];

    fn new_facelet_array<T: Eq + Copy>(init: T) -> Self::FaceletArray<T> {
        [init; 3]
    }

    fn make_missing_cubie_err(cubicle: Self::Cubicle) -> FaceletConversionError {
        FaceletConversionError::CornerCubieNotFound { cubicle }
    }

    fn rotate_facelet_array<T: Eq + Copy>(
        mut arr: Self::FaceletArray<T>,
        amt: Self::Orientation,
    ) -> Self::FaceletArray<T> {
        arr.rotate_right(amt as usize);
        arr
    }
}

impl CubiesExt for Edges {
    type FaceletArray<T: Eq + Copy> = [T; 2];

    fn new_facelet_array<T: Eq + Copy>(init: T) -> Self::FaceletArray<T> {
        [init; 2]
    }

    fn make_missing_cubie_err(cubicle: Self::Cubicle) -> FaceletConversionError {
        FaceletConversionError::EdgeCubieNotFound { cubicle }
    }

    fn rotate_facelet_array<T: Eq + Copy>(
        mut arr: Self::FaceletArray<T>,
        amt: Self::Orientation,
    ) -> Self::FaceletArray<T> {
        arr.rotate_right(amt as usize);
        arr
    }
}

/// A simpler cube representation than [`CubieCube`]. A `FaceletCube` is just an array of
/// 6 faces where each face is an array of 9 colors.
#[derive(Debug, Eq, PartialEq)]
pub struct FaceletCube {
    /// See [`Self::get_face()`] for the layout of this array
    faces: [[Color; 9]; 6],
}

impl FaceletCube {
    pub fn builder() -> FaceletCubeBuilder {
        FaceletCubeBuilder {
            initialized: [[false; 9]; 6],
            faces: [[Color::Blue; 9]; 6],
        }
    }

    pub fn to_cubie_cube(&self) -> Result<CubieCube, FaceletConversionError> {
        let corner_map = {
            use Color::*;
            use CornerCubicle::*;
            CubicleArray::new([
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

        let edge_map = {
            use Color::*;
            use EdgeCubicle::*;
            CubicleArray::new([
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

        fn aux<C: CubiesExt>(
            map: C::CubicleArray<(C::Cubicle, C::FaceletArray<Color>, C::FaceletArray<usize>)>,
            output_cubies: &mut C,
            facelet_cube: &FaceletCube,
        ) -> Result<(), FaceletConversionError> {
            'outer: for (home, home_colors, _) in map {
                for orientation in C::Orientation::all() {
                    // list of `(cubicle, oriented colors of the cubie in that cubicle)`
                    let colored_cubies = map.into_iter().map(|(cubicle, faces, indices)| {
                        // dummy init value
                        let mut arr = C::new_facelet_array(Color::Blue);

                        for (i, (face, index)) in faces.into_iter().zip(indices).enumerate() {
                            arr[i] = facelet_cube.get_face(face)[index];
                        }

                        (cubicle, arr)
                    });

                    for (cubicle, colors_in_cubicle) in colored_cubies {
                        if C::rotate_facelet_array(home_colors, orientation) == colors_in_cubicle {
                            output_cubies[home] = C::Cubie::new(cubicle, orientation);
                            continue 'outer;
                        }
                    }
                }
                // The `home` cubie doesn't exist in the FaceletCube
                return Err(C::make_missing_cubie_err(home));
            }
            Ok(())
        }

        // another yucky hack to avoid MaybeUninit (because logic error is easier to debug than UB)
        let mut corners =
            CubicleArray::new([CornerCubie::new(CornerCubicle::C0, CornerOrientation::O0); 8]);
        let mut edges =
            CubicleArray::new([EdgeCubie::new(EdgeCubicle::C0, EdgeOrientation::O0); 12]);

        aux(corner_map, &mut corners, self)?;
        aux(edge_map, &mut edges, self)?;

        Ok(CubieCube::try_new(corners, edges)?)
    }

    /// Use [`CubieCube::from_facelet_cube`] for a `pub` interface to this
    pub(crate) fn from_cubie_cube(cubie_cube: &CubieCube) -> Self {
        let corner_cubie_colors = {
            use Color::*;
            use CornerCubicle::*;
            // (homecubicle, [clockwise_colors])
            CubicleArray::new([
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

        let corner_face_index_map = CubicleArray::new([
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
            CubicleArray::new([
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
        let edge_face_index_map = CubicleArray::new([
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

        fn aux<C: CubiesExt>(
            faces: &mut [[Color; 9]; 6],
            face_index_map: C::CubicleArray<C::FaceletArray<usize>>,
            cubie_colors: C::CubicleArray<(C::Cubicle, C::FaceletArray<Color>)>,
            cubie_cube: &CubieCube,
        ) where
            CubieCube: Index<C::Cubicle, Output = C::Cubie>,
        {
            for (home_cubicle, colors) in cubie_colors {
                // `cubie` is the cubie that normally lives in `home_cubicle`.
                // `colors` are the colors of that cubie.
                // `cornerstate.cubicle()` is the current cubicle that cubie is in
                let cubie = cubie_cube[home_cubicle];
                let oriented_colors = C::rotate_facelet_array(colors, cubie.orientation());
                let face_map = face_index_map[cubie.cubicle()];
                let cubicle_colors = cubie_colors[cubie.cubicle()].1;

                for (i, face_map_item) in face_map.into_iter().enumerate() {
                    faces[cubicle_colors[i] as usize][face_map_item] = oriented_colors[i];
                }
            }
        }

        aux::<Corners>(
            &mut faces,
            corner_face_index_map,
            corner_cubie_colors,
            cubie_cube,
        );
        aux::<Edges>(
            &mut faces,
            edge_face_index_map,
            edge_cubie_colors,
            cubie_cube,
        );

        // center pieces
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
pub enum FaceletConversionError {
    #[error("The cubie that lives in {cubicle:?} was not found in the FaceletCube")]
    CornerCubieNotFound { cubicle: CornerCubicle },
    #[error("The cubie that lives in {cubicle:?} was not found in the FaceletCube")]
    EdgeCubieNotFound { cubicle: EdgeCubicle },
    #[error("CubieCube::try_new() failed")]
    CubieCubeConstruction(CubieCubeConstructionError),
}

impl From<CubieCubeConstructionError> for FaceletConversionError {
    fn from(e: CubieCubeConstructionError) -> Self {
        Self::CubieCubeConstruction(e)
    }
}

#[derive(Debug)]
pub struct FaceletCubeBuilder {
    initialized: [[bool; 9]; 6],
    faces: [[Color; 9]; 6],
}

impl FaceletCubeBuilder {
    /// Returns `None` if not all faces were initialized
    pub fn build(self) -> Option<FaceletCube> {
        if self.initialized.into_iter().flatten().any(|x| x == false) {
            None
        } else {
            Some(FaceletCube { faces: self.faces })
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
    fn facelet_cube_conversions() {
        assert_eq!(
            CubieCube::SOLVED.to_facelet_cube().to_cubie_cube().unwrap(),
            CubieCube::SOLVED
        );

        assert_eq!(TPERM, TPERM.to_cubie_cube().unwrap().to_facelet_cube());
        assert_eq!(RMOVE, RMOVE.to_cubie_cube().unwrap().to_facelet_cube());
    }

    const TPERM: FaceletCube = {
        use Color::*;
        FaceletCube {
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

    const RMOVE: FaceletCube = {
        use Color::*;
        FaceletCube {
            faces: [
                [
                    Orange, Orange, Orange, Orange, Orange, Orange, Orange, Orange, Orange,
                ],
                [Red, Red, Red, Red, Red, Red, Red, Red, Red],
                [
                    Yellow, Yellow, Blue, Yellow, Yellow, Blue, Yellow, Yellow, Blue,
                ],
                [
                    White, White, Green, White, White, Green, White, White, Green,
                ],
                [
                    Green, Green, Yellow, Green, Green, Yellow, Green, Green, Yellow,
                ],
                [White, Blue, Blue, White, Blue, Blue, White, Blue, Blue],
            ],
        }
    };
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

fn println_render_cube(render: &FaceletCube) {
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
