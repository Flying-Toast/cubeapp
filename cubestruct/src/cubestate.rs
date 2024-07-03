use crate::cubiestate::*;
use crate::dumb::DumbCube;

/// Corner cubicle numbering:
/// ```text
/// ┌──┬──┬──┐  ┌──┬──┬──┐  ┌──┬──┬──┐
/// │ 0│  │ 1│  │  │  │  │  │ 4│  │ 5│
/// ├──┼──┼──┤  ├──┼──┼──┤  ├──┼──┼──┤
/// │  │  │  │  │  │  │  │  │  │  │  │
/// ├──┼──┼──┤  ├──┼──┼──┤  ├──┼──┼──┤
/// │ 2│  │ 3│  │  │  │  │  │ 6│  │ 7│
/// └──┴──┴──┘  └──┴──┴──┘  └──┴──┴──┘
/// Top Layer   Middle Lyr  Botm Layer
/// ```
///
/// Edge cubicle numbering:
/// ```text
/// ┌──┬──┬──┐  ┌──┬──┬──┐  ┌──┬──┬──┐
/// │  │ 0│  │  │ 4│  │ 5│  │  │ 8│  │
/// ├──┼──┼──┤  ├──┼──┼──┤  ├──┼──┼──┤
/// │ 1│  │ 2│  │  │  │  │  │ 9│  │10│
/// ├──┼──┼──┤  ├──┼──┼──┤  ├──┼──┼──┤
/// │  │ 3│  │  │ 6│  │ 7│  │  │11│  │
/// └──┴──┴──┘  └──┴──┴──┘  └──┴──┴──┘
/// Top Layer   Middle Lyr  Botm Layer
/// ```
///
/// A cubie is said to "live"/have a "home" in a cubicle if the cubie belongs in that cubicle *for a solved cube*.
#[derive(Debug, Eq, PartialEq)]
pub struct CubeState {
    /// `corners[i]` is the state of the corner whose home is cubicle `i`.
    /// e.g. `corners[0].cubicle()` returns the cubicle in which the cubie that lives at C0 currently is located.
    corners: [CornerState; 8],
    /// `edges[i]` is the state of the edge whose home is cubicle `i`
    /// e.g. `edges[0].cubicle()` returns the cubicle in which the cubie that lives at C0 currently is located.
    edges: [EdgeState; 12],
}

impl CubeState {
    /// Returns `None` if the given cubie arrays are invalid (i.e. the put multiple cubies in the same cubicle).
    pub(crate) fn try_new(
        corners: [CornerState; 8],
        edges: [EdgeState; 12],
    ) -> Result<Self, CubeStateConstructionError> {
        let mut seen_corners = [false; 8];
        let mut seen_edges = [false; 12];

        for i in corners {
            seen_corners[i.cubicle() as usize] = true;
        }
        for i in edges {
            seen_edges[i.cubicle() as usize] = true;
        }

        if seen_corners.into_iter().any(|x| x == false)
            || seen_edges.into_iter().any(|x| x == false)
        {
            Err(CubeStateConstructionError::EmptyCubicles)
        } else {
            Ok(Self { corners, edges })
        }
        ////////////////////////////////////////////////////////////////
        // TODO: return Err::ImpossibleState if it's an impossible state
        ////////////////////////////////////////////////////////////////
    }

    pub fn to_dumb(&self) -> DumbCube {
        DumbCube::from_cubestate(self)
    }

    pub const SOLVED: Self = Self {
        corners: {
            use CornerCubicle::*;
            use CornerOrientation::O0;
            use CornerState as S;
            [
                S::new(C0, O0),
                S::new(C1, O0),
                S::new(C2, O0),
                S::new(C3, O0),
                S::new(C4, O0),
                S::new(C5, O0),
                S::new(C6, O0),
                S::new(C7, O0),
            ]
        },
        edges: {
            use EdgeCubicle::*;
            use EdgeOrientation::O0;
            use EdgeState as S;
            [
                S::new(C0, O0),
                S::new(C1, O0),
                S::new(C2, O0),
                S::new(C3, O0),
                S::new(C4, O0),
                S::new(C5, O0),
                S::new(C6, O0),
                S::new(C7, O0),
                S::new(C8, O0),
                S::new(C9, O0),
                S::new(C10, O0),
                S::new(C11, O0),
            ]
        },
    };

    /// Get the state of the corner whose home is the given `cubicle`
    pub(crate) fn get_corner(&self, cubicle: CornerCubicle) -> CornerState {
        self.corners[cubicle as usize]
    }

    /// Get the state of the edge whose home is the given `cubicle`
    pub(crate) fn get_edge(&self, cubicle: EdgeCubicle) -> EdgeState {
        self.edges[cubicle as usize]
    }
}

#[derive(thiserror::Error, Debug)]
pub enum CubeStateConstructionError {
    #[error("One or more cubicle(s) did not have a cube in them")]
    EmptyCubicles,
    #[error("Attempted to create a CubeState for an impossible state")]
    ImpossibleState,
}
