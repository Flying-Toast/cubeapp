use crate::cubie::*;

#[derive(Debug)]
struct Perm2Cycles<C> {
    cubies: C,
}

impl<C: Cubies> Perm2Cycles<C> {
    fn new(cubies: C) -> Self {
        Self { cubies }
    }
}

impl<C: Cubies> Iterator for Perm2Cycles<C> {
    type Item = (C::Cubicle, C::Cubicle);

    fn next(&mut self) -> Option<Self::Item> {
        // ensure all cubicles appear
        for c in C::Cubicle::all() {
            assert!(self.cubies.into_iter().find(|x| x.cubicle() == c).is_some());
        }

        let (first_unhomed_state, home) = self
            .cubies
            .into_iter()
            .zip(C::Cubicle::all())
            .find(|(state, home)| state.cubicle() != *home)?;

        let ret = (first_unhomed_state.cubicle(), home);
        self.cubies.swap(ret.0, ret.1);
        Some(ret)
    }
}

pub fn perm_2cycles<C: Cubies>(cubies: C) -> impl Iterator<Item = (C::Cubicle, C::Cubicle)> {
    Perm2Cycles::new(cubies)
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::cubie::{CornerOrientation, EdgeOrientation};

    #[test]
    fn corner_cycle_decomposition() {
        use CornerCubicle::*;
        assert_eq!(
            Perm2Cycles::new(CubicleArray::new([
                CornerCubie::new(C2, CornerOrientation::O0),
                CornerCubie::new(C5, CornerOrientation::O0),
                CornerCubie::new(C6, CornerOrientation::O0),
                CornerCubie::new(C0, CornerOrientation::O0),
                CornerCubie::new(C7, CornerOrientation::O0),
                CornerCubie::new(C4, CornerOrientation::O0),
                CornerCubie::new(C3, CornerOrientation::O0),
                CornerCubie::new(C1, CornerOrientation::O0),
            ]))
            .collect::<Vec<_>>(),
            vec![(C2, C0), (C6, C0), (C3, C0), (C5, C1), (C4, C1), (C7, C1)]
        );
        assert_eq!(
            Perm2Cycles::new(CubicleArray::new([
                CornerCubie::new(C0, CornerOrientation::O0),
                CornerCubie::new(C1, CornerOrientation::O0),
                CornerCubie::new(C2, CornerOrientation::O0),
                CornerCubie::new(C3, CornerOrientation::O0),
                CornerCubie::new(C4, CornerOrientation::O0),
                CornerCubie::new(C5, CornerOrientation::O0),
                CornerCubie::new(C6, CornerOrientation::O0),
                CornerCubie::new(C7, CornerOrientation::O0),
            ]))
            .collect::<Vec<_>>(),
            vec![]
        );
    }

    #[test]
    fn edge_cycle_decomposition() {
        use EdgeCubicle::*;
        assert_eq!(
            Perm2Cycles::new(CubicleArray::new([
                EdgeCubie::new(C8, EdgeOrientation::O0),
                EdgeCubie::new(C6, EdgeOrientation::O0),
                EdgeCubie::new(C1, EdgeOrientation::O0),
                EdgeCubie::new(C7, EdgeOrientation::O0),
                EdgeCubie::new(C0, EdgeOrientation::O0),
                EdgeCubie::new(C10, EdgeOrientation::O0),
                EdgeCubie::new(C5, EdgeOrientation::O0),
                EdgeCubie::new(C4, EdgeOrientation::O0),
                EdgeCubie::new(C11, EdgeOrientation::O0),
                EdgeCubie::new(C2, EdgeOrientation::O0),
                EdgeCubie::new(C9, EdgeOrientation::O0),
                EdgeCubie::new(C3, EdgeOrientation::O0),
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
            Perm2Cycles::new(CubicleArray::new([
                EdgeCubie::new(C0, EdgeOrientation::O0),
                EdgeCubie::new(C1, EdgeOrientation::O0),
                EdgeCubie::new(C2, EdgeOrientation::O0),
                EdgeCubie::new(C3, EdgeOrientation::O0),
                EdgeCubie::new(C4, EdgeOrientation::O0),
                EdgeCubie::new(C5, EdgeOrientation::O0),
                EdgeCubie::new(C6, EdgeOrientation::O0),
                EdgeCubie::new(C7, EdgeOrientation::O0),
                EdgeCubie::new(C8, EdgeOrientation::O0),
                EdgeCubie::new(C9, EdgeOrientation::O0),
                EdgeCubie::new(C10, EdgeOrientation::O0),
                EdgeCubie::new(C11, EdgeOrientation::O0),
            ]))
            .collect::<Vec<_>>(),
            vec![]
        );
    }
}
