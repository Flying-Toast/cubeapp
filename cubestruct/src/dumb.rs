use crate::cubestate::CubeState;
use crate::cubiestate::{CornerCubicle, CornerOrientation, EdgeCubicle, EdgeOrientation};

/// A simpler cube representation than [`CubeState`]. A `DumbCube` is just an array of
/// 6 faces where each face is an array of 9 colors.
#[derive(Debug)]
pub struct DumbCube {
    /// See [`Self::get_face()`] for the layout of this array
    faces: [[Color; 9]; 6],
}

impl DumbCube {
    pub fn from_cubestate(state: &CubeState) -> Self {
        let corner_map = {
            use Color::*;
            use CornerCubicle::*;
            // (homecubicle, [clockwise_colors])
            [
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
            ]
        };
        // O1: shift colors right
        const fn corner_o1_shift([a, b, c]: [Color; 3]) -> [Color; 3] {
            [c, a, b]
        }
        // O2: shift colors left
        const fn corner_o2_shift([a, b, c]: [Color; 3]) -> [Color; 3] {
            [b, c, a]
        }

        let corner_face_map = [
            // `corner_map` tells us that the cubie who lives at C0 has colors [W, O, B];
            // elements in the [0, 0, 2] array here correspond to the order of colors in that [W, O, B] array.
            // here, corner_face_map[C0] = [0, 0, 2] tells us that the C0 cubie has its white facelet
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
        ];

        let edge_map = {
            use Color::*;
            use EdgeCubicle::*;
            // (homeplace, [X]) where X colors are ordered to start with the UD/FB face
            [
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
            ]
        };

        // example:
        // (C2, [White, Red]):
        // edge_face_map[C2 as usize] = [5, 1]
        // => the white facelet of the C2 cubie goes in index 5 of the white face
        // => the red facelet of the C2 cubie goes in index 1 of the red face
        let edge_face_map = [
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
        ];

        // yucky way to avoid using MaybeUninit
        let mut faces = [[Color::Blue; 9]; 6];

        for (home_cubicle, colors) in corner_map {
            // `cornerstate` is the cubie that normally lives in `home_cubicle`.
            // `colors` are the colors of that cubie.
            // `cornerstate.cubicle()` is the current cubicle that cubie is in
            let cornerstate = state.get_corner(home_cubicle);
            let oriented_colors = match cornerstate.orientation() {
                CornerOrientation::O0 => colors,
                CornerOrientation::O1 => corner_o1_shift(colors),
                CornerOrientation::O2 => corner_o2_shift(colors),
            };
            let cidx = cornerstate.cubicle() as usize;
            let face_map = corner_face_map[cidx];
            let cubicle_colors = corner_map[cidx].1;
            faces[cubicle_colors[0] as usize][face_map[0]] = oriented_colors[0];
            faces[cubicle_colors[1] as usize][face_map[1]] = oriented_colors[1];
            faces[cubicle_colors[2] as usize][face_map[2]] = oriented_colors[2];
        }

        const fn edge_o1_shift([a, b]: [Color; 2]) -> [Color; 2] {
            [b, a]
        }

        for (home_cubicle, colors) in edge_map {
            let edgestate = state.get_edge(home_cubicle);
            let oriented_colors = match edgestate.orientation() {
                EdgeOrientation::O0 => colors,
                EdgeOrientation::O1 => edge_o1_shift(colors),
            };
            let cidx = edgestate.cubicle() as usize;
            let face_map = edge_face_map[cidx];
            let cubicle_colors = edge_map[cidx].1;
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
