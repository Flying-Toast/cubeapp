mod coord_cube;
mod cubie;
mod cubie_cube;
mod facelet_cube;
mod iter_2cycles;

pub use cubie_cube::CubieCube;
pub use facelet_cube::{Color, FaceletCube};

use std::fmt;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Move {
    Li,
    L,
    L2,
    Ri,
    R,
    R2,
    Di,
    D,
    D2,
    Ui,
    U,
    U2,
    Fi,
    F,
    F2,
    Bi,
    B,
    B2,
}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Li => write!(f, "L'"),
            Self::L => write!(f, "L"),
            Self::L2 => write!(f, "L2"),
            Self::Ri => write!(f, "R'"),
            Self::R => write!(f, "R"),
            Self::R2 => write!(f, "R2"),
            Self::Di => write!(f, "D'"),
            Self::D => write!(f, "D"),
            Self::D2 => write!(f, "D2"),
            Self::Ui => write!(f, "U'"),
            Self::U => write!(f, "U"),
            Self::U2 => write!(f, "U2"),
            Self::Fi => write!(f, "F'"),
            Self::F => write!(f, "F"),
            Self::F2 => write!(f, "F2"),
            Self::Bi => write!(f, "B'"),
            Self::B => write!(f, "B"),
            Self::B2 => write!(f, "B2"),
        }
    }
}
