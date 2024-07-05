use crate::cubie::*;

#[derive(Debug)]
struct CornerPerm2Cycles {
    corners: Corners,
}

impl CornerPerm2Cycles {
    fn new(corners: Corners) -> Self {
        Self { corners }
    }
}

impl Iterator for CornerPerm2Cycles {
    type Item = (CornerCubicle, CornerCubicle);

    fn next(&mut self) -> Option<Self::Item> {
        // ensure all cubicles appear
        for c in CornerCubicle::all() {
            assert!(self
                .corners
                .into_iter()
                .find(|x| x.cubicle() == c)
                .is_some());
        }

        let (first_unhomed_state, home) = self
            .corners
            .into_iter()
            .zip(CornerCubicle::all())
            .find(|(state, home)| state.cubicle() != *home)?;

        let ret = (first_unhomed_state.cubicle(), home);
        self.corners.swap(ret.0, ret.1);
        Some(ret)
    }
}

pub fn corner_2cycles(corners: Corners) -> impl Iterator<Item = (CornerCubicle, CornerCubicle)> {
    CornerPerm2Cycles::new(corners)
}

#[derive(Debug)]
struct EdgePerm2Cycles {
    edges: Edges,
}

impl EdgePerm2Cycles {
    fn new(edges: Edges) -> Self {
        Self { edges }
    }
}

impl Iterator for EdgePerm2Cycles {
    type Item = (EdgeCubicle, EdgeCubicle);

    fn next(&mut self) -> Option<Self::Item> {
        // ensure all cubicles appear
        for c in EdgeCubicle::all() {
            assert!(self.edges.into_iter().find(|x| x.cubicle() == c).is_some());
        }

        let (first_unhomed_state, home) = self
            .edges
            .into_iter()
            .zip(EdgeCubicle::all())
            .find(|(state, home)| state.cubicle() != *home)?;

        let ret = (first_unhomed_state.cubicle(), home);
        self.edges.swap(ret.0, ret.1);
        Some(ret)
    }
}

pub fn edge_2cycles(edges: Edges) -> impl Iterator<Item = (EdgeCubicle, EdgeCubicle)> {
    EdgePerm2Cycles::new(edges)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cubie::{CornerOrientation, EdgeOrientation};

    #[test]
    fn corner_cycle_decomposition() {
        use CornerCubicle::*;
        assert_eq!(
            CornerPerm2Cycles::new(CubicleArray::new([
                CornerState::new(C2, CornerOrientation::O0),
                CornerState::new(C5, CornerOrientation::O0),
                CornerState::new(C6, CornerOrientation::O0),
                CornerState::new(C0, CornerOrientation::O0),
                CornerState::new(C7, CornerOrientation::O0),
                CornerState::new(C4, CornerOrientation::O0),
                CornerState::new(C3, CornerOrientation::O0),
                CornerState::new(C1, CornerOrientation::O0),
            ]))
            .collect::<Vec<_>>(),
            vec![(C2, C0), (C6, C0), (C3, C0), (C5, C1), (C4, C1), (C7, C1)]
        );
        assert_eq!(
            CornerPerm2Cycles::new(CubicleArray::new([
                CornerState::new(C0, CornerOrientation::O0),
                CornerState::new(C1, CornerOrientation::O0),
                CornerState::new(C2, CornerOrientation::O0),
                CornerState::new(C3, CornerOrientation::O0),
                CornerState::new(C4, CornerOrientation::O0),
                CornerState::new(C5, CornerOrientation::O0),
                CornerState::new(C6, CornerOrientation::O0),
                CornerState::new(C7, CornerOrientation::O0),
            ]))
            .collect::<Vec<_>>(),
            vec![]
        );
    }

    #[test]
    fn edge_cycle_decomposition() {
        use EdgeCubicle::*;
        assert_eq!(
            EdgePerm2Cycles::new(CubicleArray::new([
                EdgeState::new(C8, EdgeOrientation::O0),
                EdgeState::new(C6, EdgeOrientation::O0),
                EdgeState::new(C1, EdgeOrientation::O0),
                EdgeState::new(C7, EdgeOrientation::O0),
                EdgeState::new(C0, EdgeOrientation::O0),
                EdgeState::new(C10, EdgeOrientation::O0),
                EdgeState::new(C5, EdgeOrientation::O0),
                EdgeState::new(C4, EdgeOrientation::O0),
                EdgeState::new(C11, EdgeOrientation::O0),
                EdgeState::new(C2, EdgeOrientation::O0),
                EdgeState::new(C9, EdgeOrientation::O0),
                EdgeState::new(C3, EdgeOrientation::O0),
            ]))
            .collect::<Vec<_>>(),
            vec![
                (C8, C0),
                (C11, C0),
                (C3, C0),
                (C7, C0),
                (C4, C0),
                (C6, C1),
                (C5, C1),
                (C10, C1),
                (C9, C1),
                (C2, C1)
            ]
        );
        assert_eq!(
            EdgePerm2Cycles::new(CubicleArray::new([
                EdgeState::new(C0, EdgeOrientation::O0),
                EdgeState::new(C1, EdgeOrientation::O0),
                EdgeState::new(C2, EdgeOrientation::O0),
                EdgeState::new(C3, EdgeOrientation::O0),
                EdgeState::new(C4, EdgeOrientation::O0),
                EdgeState::new(C5, EdgeOrientation::O0),
                EdgeState::new(C6, EdgeOrientation::O0),
                EdgeState::new(C7, EdgeOrientation::O0),
                EdgeState::new(C8, EdgeOrientation::O0),
                EdgeState::new(C9, EdgeOrientation::O0),
                EdgeState::new(C10, EdgeOrientation::O0),
                EdgeState::new(C11, EdgeOrientation::O0),
            ]))
            .collect::<Vec<_>>(),
            vec![]
        );
    }
}
