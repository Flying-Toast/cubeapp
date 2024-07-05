use crate::cubie::*;
use crate::dumb::DumbCube;
use crate::iter_2cycles::{corner_2cycles, edge_2cycles};

/// The Rubik's Cube group, *G*.
///
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
    /// e.g. `corners[C4].cubicle()` returns the cubicle in which the cubie that lives at C4 currently is located.
    corners: Corners,
    /// `edges[i]` is the state of the edge whose home is cubicle `i`
    /// e.g. `edges[C0].cubicle()` returns the cubicle in which the cubie that lives at C0 currently is located.
    edges: Edges,
}

impl CubeState {
    /// Returns `None` if the given cubie arrays are invalid (i.e. the put multiple cubies in the same cubicle).
    pub(crate) fn try_new(
        corners: Corners,
        edges: Edges,
    ) -> Result<Self, CubeStateConstructionError> {
        let mut seen_corners = CubicleArray::new([false; 8]);
        let mut seen_edges = CubicleArray::new([false; 12]);

        for i in corners {
            seen_corners[i.cubicle()] = true;
        }
        for i in edges {
            seen_edges[i.cubicle()] = true;
        }

        if seen_corners.into_iter().any(|x| x == false)
            || seen_edges.into_iter().any(|x| x == false)
        {
            Err(CubeStateConstructionError::EmptyCubicles)
        } else {
            Ok(Self { corners, edges })
        }
    }

    pub fn random_possible() -> Self {
        let Self {
            mut corners,
            mut edges,
        } = Self::SOLVED;
        let mut rng = rand::thread_rng();

        let mut total_corner_ori = CornerOrientation::O0;
        for cubicle in CornerCubicle::all().into_iter().skip(1) {
            let o = CornerOrientation::random(&mut rng);
            total_corner_ori = total_corner_ori.add(o);
            corners[cubicle].set_orientation(o);
        }
        corners[CornerCubicle::C0].set_orientation(total_corner_ori.inverse());

        let mut total_edge_ori = EdgeOrientation::O0;
        for cubicle in EdgeCubicle::all().into_iter().skip(1) {
            let o = EdgeOrientation::random(&mut rng);
            total_edge_ori = total_edge_ori.add(o);
            edges[cubicle].set_orientation(o);
        }
        edges[EdgeCubicle::C0].set_orientation(total_edge_ori.inverse());

        corners.shuffle(&mut rng);
        edges.shuffle(&mut rng);
        if corner_2cycles(corners).count() & 1 != edge_2cycles(edges).count() & 1 {
            edges.swap(EdgeCubicle::C0, EdgeCubicle::C1);
        }

        Self { corners, edges }
    }

    pub fn is_possible_state(&self) -> bool {
        let total_edge_orientation = self
            .edges
            .into_iter()
            .map(|x| x.orientation())
            .reduce(EdgeOrientation::add)
            .unwrap();
        let total_corner_orientation = self
            .corners
            .into_iter()
            .map(|x| x.orientation())
            .reduce(CornerOrientation::add)
            .unwrap();

        let corners_odd = corner_2cycles(self.corners).count() & 1 == 1;
        let edges_odd = edge_2cycles(self.edges).count() & 1 == 1;

        total_edge_orientation == EdgeOrientation::O0
            && total_corner_orientation == CornerOrientation::O0
            && corners_odd == edges_odd
    }

    #[must_use]
    pub fn to_dumb(&self) -> DumbCube {
        DumbCube::from_cubestate(self)
    }

    pub const SOLVED: Self = Self {
        corners: {
            use CornerCubicle::*;
            use CornerOrientation::O0;
            use CornerState as S;
            CubicleArray::new([
                S::new(C0, O0),
                S::new(C1, O0),
                S::new(C2, O0),
                S::new(C3, O0),
                S::new(C4, O0),
                S::new(C5, O0),
                S::new(C6, O0),
                S::new(C7, O0),
            ])
        },
        edges: {
            use EdgeCubicle::*;
            use EdgeOrientation::O0;
            use EdgeState as S;
            CubicleArray::new([
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
            ])
        },
    };

    /// Get the state of the corner whose home is the given `cubicle`
    pub(crate) fn get_corner(&self, cubicle: CornerCubicle) -> CornerState {
        self.corners[cubicle]
    }

    /// Get the state of the edge whose home is the given `cubicle`
    pub(crate) fn get_edge(&self, cubicle: EdgeCubicle) -> EdgeState {
        self.edges[cubicle]
    }

    #[must_use]
    pub fn inverse(&self) -> Self {
        // Doesn't actually need to be `SOLVED`, but we just use that as initialization
        // to avoid MaybeUninit
        let mut ret = Self::SOLVED;

        for (current, home) in self.corners.into_iter().zip(CornerCubicle::all()) {
            // The cubie that lives in `home` now has state `current`.
            // So, the inverse has to put `current` back at `home`
            ret.corners[current.cubicle()] =
                CornerState::new(home, current.orientation().inverse());
        }
        for (current, home) in self.edges.into_iter().zip(EdgeCubicle::all()) {
            // The cubie that lives in `home` now has state `current`.
            // So, the inverse has to put `current` back at `home`
            ret.edges[current.cubicle()] = EdgeState::new(home, current.orientation().inverse());
        }

        ret
    }

    /// Multiply ("*" group operation) `self` by `rhs`.
    #[must_use]
    pub fn mul(&self, rhs: &Self) -> Self {
        // Doesn't actually need to be `SOLVED`, but we just use that as initialization
        // to avoid MaybeUninit
        let mut ret = Self::SOLVED;

        for (self_state, home) in self.corners.into_iter().zip(CornerCubicle::all()) {
            // `home` goes to `self_state` by `self`.
            // `self_state` goes to `rhs_state` by `rhs`.
            let rhs_state = rhs.corners[self_state.cubicle()];
            ret.corners[home] = CornerState::new(
                rhs_state.cubicle(),
                self_state.orientation().add(rhs_state.orientation()),
            );
        }

        for (self_state, home) in self.edges.into_iter().zip(EdgeCubicle::all()) {
            // `home` goes to `self_state` by `self`.
            // `self_state` goes to `rhs_state` by `rhs`.
            let rhs_state = rhs.edges[self_state.cubicle()];
            ret.edges[home] = EdgeState::new(
                rhs_state.cubicle(),
                self_state.orientation().add(rhs_state.orientation()),
            );
        }

        ret
    }
}

#[derive(thiserror::Error, Debug)]
pub enum CubeStateConstructionError {
    #[error("One or more cubicle(s) did not have a cube in them")]
    EmptyCubicles,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn possible_states() {
        assert!(CubeState::SOLVED.is_possible_state());
        assert!(TPERM.is_possible_state());
        assert!(RMOVE.is_possible_state());
        let one_edge_flipped = {
            let mut ret = CubeState::SOLVED;
            ret.edges[EdgeCubicle::C0].set_orientation(EdgeOrientation::O1);
            ret
        };
        assert!(!one_edge_flipped.is_possible_state());

        let two_corners_swapped = {
            let mut ret = CubeState::SOLVED;
            ret.corners.swap(CornerCubicle::C0, CornerCubicle::C1);
            ret
        };

        assert!(!two_corners_swapped.is_possible_state());
    }

    #[test]
    fn random_possible_states_are_possible() {
        for _ in 0..1000 {
            let state = CubeState::random_possible();
            assert!(state.is_possible_state(), "{state:?}");
        }
    }

    #[test]
    fn group_ops() {
        assert_eq!(CubeState::SOLVED, CubeState::SOLVED.inverse());
        assert_eq!(CubeState::SOLVED.mul(&CubeState::SOLVED), CubeState::SOLVED);
        assert_eq!(RMOVE.mul(&CubeState::SOLVED), RMOVE);
        assert_eq!(CubeState::SOLVED.mul(&TPERM), TPERM);

        assert_eq!(TPERM.inverse(), TPERM);
        assert_eq!(TPERM.mul(&TPERM.inverse()), CubeState::SOLVED);
        assert_eq!(TPERM.mul(&TPERM), CubeState::SOLVED);

        assert_ne!(RMOVE.inverse(), RMOVE);
        assert_ne!(RMOVE.inverse(), CubeState::SOLVED);
        assert_ne!(RMOVE.mul(&RMOVE), RMOVE);
        assert_ne!(RMOVE.mul(&RMOVE), CubeState::SOLVED);
        assert_ne!(RMOVE.mul(&RMOVE).mul(&RMOVE), RMOVE);
        assert_ne!(RMOVE.mul(&RMOVE).mul(&RMOVE), CubeState::SOLVED);
        assert_eq!(RMOVE.mul(&RMOVE).mul(&RMOVE).mul(&RMOVE), CubeState::SOLVED);

        assert_eq!(
            CubeState::SOLVED,
            TPERM
                .mul(&RMOVE)
                .mul(&RMOVE.inverse())
                .mul(&TPERM.inverse())
        );
    }

    const TPERM: CubeState = CubeState {
        corners: {
            use CornerCubicle::*;
            use CornerOrientation::O0;
            CubicleArray::new([
                CornerState::new(C0, O0),
                CornerState::new(C3, O0),
                CornerState::new(C2, O0),
                CornerState::new(C1, O0),
                CornerState::new(C4, O0),
                CornerState::new(C5, O0),
                CornerState::new(C6, O0),
                CornerState::new(C7, O0),
            ])
        },
        edges: {
            use EdgeCubicle::*;
            use EdgeOrientation::O0;
            CubicleArray::new([
                EdgeState::new(C0, O0),
                EdgeState::new(C2, O0),
                EdgeState::new(C1, O0),
                EdgeState::new(C3, O0),
                EdgeState::new(C4, O0),
                EdgeState::new(C5, O0),
                EdgeState::new(C6, O0),
                EdgeState::new(C7, O0),
                EdgeState::new(C8, O0),
                EdgeState::new(C9, O0),
                EdgeState::new(C10, O0),
                EdgeState::new(C11, O0),
            ])
        },
    };

    const RMOVE: CubeState = CubeState {
        corners: CubicleArray::new([
            CornerState::new(CornerCubicle::C0, CornerOrientation::O0),
            CornerState::new(CornerCubicle::C5, CornerOrientation::O2),
            CornerState::new(CornerCubicle::C2, CornerOrientation::O0),
            CornerState::new(CornerCubicle::C1, CornerOrientation::O1),
            CornerState::new(CornerCubicle::C4, CornerOrientation::O0),
            CornerState::new(CornerCubicle::C7, CornerOrientation::O1),
            CornerState::new(CornerCubicle::C6, CornerOrientation::O0),
            CornerState::new(CornerCubicle::C3, CornerOrientation::O2),
        ]),
        edges: CubicleArray::new([
            EdgeState::new(EdgeCubicle::C0, EdgeOrientation::O0),
            EdgeState::new(EdgeCubicle::C1, EdgeOrientation::O0),
            EdgeState::new(EdgeCubicle::C5, EdgeOrientation::O0),
            EdgeState::new(EdgeCubicle::C3, EdgeOrientation::O0),
            EdgeState::new(EdgeCubicle::C4, EdgeOrientation::O0),
            EdgeState::new(EdgeCubicle::C10, EdgeOrientation::O0),
            EdgeState::new(EdgeCubicle::C6, EdgeOrientation::O0),
            EdgeState::new(EdgeCubicle::C2, EdgeOrientation::O0),
            EdgeState::new(EdgeCubicle::C8, EdgeOrientation::O0),
            EdgeState::new(EdgeCubicle::C9, EdgeOrientation::O0),
            EdgeState::new(EdgeCubicle::C7, EdgeOrientation::O0),
            EdgeState::new(EdgeCubicle::C11, EdgeOrientation::O0),
        ]),
    };
}
