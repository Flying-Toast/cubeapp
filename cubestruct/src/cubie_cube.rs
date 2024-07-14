use crate::cubie::*;
use crate::facelet_cube::FaceletCube;
use crate::iter_2cycles::perm_2cycles;
use crate::Move;
use std::ops::{Index, IndexMut, Mul, MulAssign};

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
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub struct CubieCube {
    /// `corners[i]` is the state of the corner whose home is cubicle `i`.
    /// e.g. `corners[C4].cubicle()` returns the cubicle in which the cubie that lives at C4 currently is located.
    corners: Corners,
    /// `edges[i]` is the state of the edge whose home is cubicle `i`
    /// e.g. `edges[C0].cubicle()` returns the cubicle in which the cubie that lives at C0 currently is located.
    edges: Edges,
}

impl CubieCube {
    /// Returns `None` if the given cubie arrays are invalid (i.e. the put multiple cubies in the same cubicle).
    pub(crate) fn try_new(
        corners: Corners,
        edges: Edges,
    ) -> Result<Self, CubieCubeConstructionError> {
        fn aux<C: Cubies>(cubies: C) -> Result<C, CubieCubeConstructionError> {
            if all_cubies_seen::<C>(cubies) {
                Ok(cubies)
            } else {
                Err(CubieCubeConstructionError::EmptyCubicles)
            }
        }

        Ok(Self {
            corners: aux(corners)?,
            edges: aux(edges)?,
        })
    }

    pub fn random_possible() -> Self {
        fn aux<C: Cubies, R: rand::Rng>(cubies: &mut C, rng: &mut R) {
            let mut total_ori = C::Orientation::zero();
            for cubicle in C::Cubicle::all().skip(1) {
                let o = C::Orientation::random(rng);
                total_ori = total_ori.add(o);
                cubies[cubicle].set_orientation(o);
            }
            cubies[C::Cubicle::all().next().unwrap()].set_orientation(total_ori.inverse());

            cubies.shuffle(rng);
        }

        let Self {
            mut corners,
            mut edges,
        } = Self::SOLVED;
        let mut rng = rand::thread_rng();

        aux(&mut corners, &mut rng);
        aux(&mut edges, &mut rng);

        if (perm_2cycles(corners).count() + perm_2cycles(edges).count()) & 1 == 1 {
            edges.swap(EdgeCubicle::C0, EdgeCubicle::C1);
        }

        Self { corners, edges }
    }

    pub fn is_possible_state(&self) -> bool {
        fn is_zero_ori<C: Cubies>(cubies: C) -> bool {
            cubies
                .into_iter()
                .map(|x| x.orientation())
                .reduce(C::Orientation::add)
                .unwrap()
                == C::Orientation::zero()
        }

        let total_2cycles = perm_2cycles(self.corners).count() + perm_2cycles(self.edges).count();

        is_zero_ori(self.corners) && is_zero_ori(self.edges) && total_2cycles & 1 == 0
    }

    #[must_use]
    pub fn to_facelet_cube(&self) -> FaceletCube {
        FaceletCube::from_cubie_cube(self)
    }

    pub const SOLVED: Self = Self {
        corners: {
            use CornerCubicle::*;
            use CornerCubie as S;
            use CornerOrientation::O0;
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
            use EdgeCubie as S;
            use EdgeOrientation::O0;
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

    /// Returns a new CubieCube that is the inverse of `self`
    #[must_use]
    pub fn inverse(&self) -> Self {
        fn aux<C: Cubies>(cubies: C, ret: &mut C) {
            for (current, home) in cubies.into_iter().zip(C::Cubicle::all()) {
                // The cubie that lives in `home` now has state `current`.
                // So, the inverse has to put `current` back at `home`
                ret[current.cubicle()] = C::Cubie::new(home, current.orientation().inverse());
            }
        }

        // Doesn't actually need to be `SOLVED`, but we just use that as initialization
        // to avoid MaybeUninit
        let mut ret = Self::SOLVED;

        aux(self.corners, &mut ret.corners);
        aux(self.edges, &mut ret.edges);

        ret
    }

    pub fn apply_move(&mut self, moov: Move) {
        match moov {
            Move::L => {
                *self *= LMOVE;
            }
            Move::Li => {
                *self *= LMOVE * LMOVE * LMOVE;
            }
            Move::L2 => {
                *self *= LMOVE * LMOVE;
            }
            Move::R => {
                *self *= RMOVE;
            }
            Move::Ri => {
                *self *= RMOVE * RMOVE * RMOVE;
            }
            Move::R2 => {
                *self *= RMOVE * RMOVE;
            }
            Move::D => {
                *self *= DMOVE;
            }
            Move::Di => {
                *self *= DMOVE * DMOVE * DMOVE;
            }
            Move::D2 => {
                *self *= DMOVE * DMOVE;
            }
            Move::U => {
                *self *= UMOVE;
            }
            Move::Ui => {
                *self *= UMOVE * UMOVE * UMOVE;
            }
            Move::U2 => {
                *self *= UMOVE * UMOVE;
            }
            Move::F => {
                *self *= FMOVE;
            }
            Move::Fi => {
                *self *= FMOVE * FMOVE * FMOVE;
            }
            Move::F2 => {
                *self *= FMOVE * FMOVE;
            }
            Move::B => {
                *self *= BMOVE;
            }
            Move::Bi => {
                *self *= BMOVE * BMOVE * BMOVE;
            }
            Move::B2 => {
                *self = BMOVE * BMOVE;
            }
        }
    }

    pub(crate) fn set_ori_coord<C: Cubies>(&mut self, coord: u16)
    where
        Self: Index<C::Cubicle, Output = C::Cubie> + IndexMut<C::Cubicle>,
    {
        debug_assert!(C::ORI_COORD_RANGE.contains(&coord));
        let base = C::Orientation::all().count() as u16;
        let max_cubicle = C::Cubicle::all().last().unwrap();
        let mut max_cubicle_home = None;
        let mut total_ori = C::Orientation::zero();
        for home_cubicle in C::Cubicle::all() {
            let state = self[home_cubicle];
            if state.cubicle() == max_cubicle {
                max_cubicle_home = Some(home_cubicle);
                continue;
            }
            let divided = if state.cubicle().as_u8() == 0 {
                coord
            } else {
                coord / base.pow(state.cubicle().as_u8().into())
            };
            let digit = divided % base;
            let target_ori = C::Orientation::from_u8(digit.try_into().unwrap()).unwrap();
            total_ori = total_ori.add(target_ori);
            self[home_cubicle].set_orientation(target_ori);
        }
        self[max_cubicle_home.unwrap()].set_orientation(total_ori.inverse());
    }

    pub(crate) fn get_ori_coord<C: Cubies>(&self) -> u16
    where
        Self: Index<C::Cubicle, Output = C::Cubie>,
    {
        let base = C::Orientation::all().count() as u16;
        let mut sum = 0;
        let max_cubicle = C::Cubicle::all().last().unwrap();
        for home_cubicle in C::Cubicle::all() {
            let state = self[home_cubicle];
            if state.cubicle() == max_cubicle {
                continue;
            }
            sum += base.pow(state.cubicle().as_u8().into()) * state.orientation().as_u8() as u16;
        }
        sum
    }

    pub(crate) fn get_udslice_coord(&self) -> u16 {
        let udslice_cubicles = [
            EdgeCubicle::C4,
            EdgeCubicle::C5,
            EdgeCubicle::C6,
            EdgeCubicle::C7,
        ];

        let mut mask = 0;
        for home_cubicle in udslice_cubicles {
            let loc = self[home_cubicle].cubicle();
            mask |= 1 << loc.as_u8();
        }

        crate::coord_cube::udslice_bitmask_to_coord(mask)
    }

    pub(crate) fn set_udslice_coord(&mut self, coord: u16) {
        debug_assert!(crate::coord_cube::CoordCube::UDSLICE_RANGE.contains(&coord));
        let mask = crate::coord_cube::udslice_coord_to_bitmask(coord);

        let mut udslice_cubicles = [
            EdgeCubicle::C4,
            EdgeCubicle::C5,
            EdgeCubicle::C6,
            EdgeCubicle::C7,
        ]
        .into_iter();
        let mut non_udslice_cubicles = {
            use EdgeCubicle::*;
            [C0, C1, C2, C3, C8, C9, C10, C11].into_iter()
        };

        for (i, cubicle) in <Edges as Cubies>::Cubicle::all().enumerate() {
            if mask & (1 << i) == 0 {
                self.edges[non_udslice_cubicles.next().unwrap()].set_cubicle(cubicle);
            } else {
                self.edges[udslice_cubicles.next().unwrap()].set_cubicle(cubicle);
            }
        }

        // maintain possibleness
        if (perm_2cycles(self.corners).count() + perm_2cycles(self.edges).count()) & 1 == 1 {
            self.edges.swap(EdgeCubicle::C0, EdgeCubicle::C1);
        }
    }
}

impl Index<CornerCubicle> for CubieCube {
    type Output = CornerCubie;
    fn index(&self, index: CornerCubicle) -> &Self::Output {
        &self.corners[index]
    }
}

impl IndexMut<CornerCubicle> for CubieCube {
    fn index_mut(&mut self, index: CornerCubicle) -> &mut Self::Output {
        &mut self.corners[index]
    }
}

impl Index<EdgeCubicle> for CubieCube {
    type Output = EdgeCubie;
    fn index(&self, index: EdgeCubicle) -> &Self::Output {
        &self.edges[index]
    }
}

impl IndexMut<EdgeCubicle> for CubieCube {
    fn index_mut(&mut self, index: EdgeCubicle) -> &mut Self::Output {
        &mut self.edges[index]
    }
}

impl Mul<CubieCube> for CubieCube {
    type Output = Self;

    fn mul(self, rhs: CubieCube) -> Self::Output {
        fn aux<C: Cubies>(lhs: C, rhs: C, ret: &mut C) {
            for (lhs_state, home) in lhs.into_iter().zip(C::Cubicle::all()) {
                // `home` goes to `lhs_state` by `lhs`.
                // `lhs_state` goes to `rhs_state` by `rhs`.
                let rhs_state = rhs[lhs_state.cubicle()];
                ret[home] = C::Cubie::new(
                    rhs_state.cubicle(),
                    lhs_state.orientation().add(rhs_state.orientation()),
                );
            }
        }

        // Doesn't actually need to be `SOLVED`, but we just use that as initialization
        // to avoid MaybeUninit
        let mut ret = CubieCube::SOLVED;

        aux(self.corners, rhs.corners, &mut ret.corners);
        aux(self.edges, rhs.edges, &mut ret.edges);

        ret
    }
}

impl MulAssign<CubieCube> for CubieCube {
    fn mul_assign(&mut self, rhs: CubieCube) {
        *self = self.mul(rhs);
    }
}

#[derive(thiserror::Error, Debug)]
pub enum CubieCubeConstructionError {
    #[error("One or more cubicle(s) did not have a cube in them")]
    EmptyCubicles,
}

fn all_cubies_seen<C: Cubies>(cubies: C) -> bool {
    let mut seen = C::new_array(false);
    for i in cubies {
        seen[i.cubicle()] = true;
    }
    seen.into_iter().all(|x| x == true)
}

const RMOVE: CubieCube = CubieCube {
    corners: CubicleArray::new([
        CornerCubie::new(CornerCubicle::C0, CornerOrientation::O0),
        CornerCubie::new(CornerCubicle::C5, CornerOrientation::O2),
        CornerCubie::new(CornerCubicle::C2, CornerOrientation::O0),
        CornerCubie::new(CornerCubicle::C1, CornerOrientation::O1),
        CornerCubie::new(CornerCubicle::C4, CornerOrientation::O0),
        CornerCubie::new(CornerCubicle::C7, CornerOrientation::O1),
        CornerCubie::new(CornerCubicle::C6, CornerOrientation::O0),
        CornerCubie::new(CornerCubicle::C3, CornerOrientation::O2),
    ]),
    edges: CubicleArray::new([
        EdgeCubie::new(EdgeCubicle::C0, EdgeOrientation::O0),
        EdgeCubie::new(EdgeCubicle::C1, EdgeOrientation::O0),
        EdgeCubie::new(EdgeCubicle::C5, EdgeOrientation::O0),
        EdgeCubie::new(EdgeCubicle::C3, EdgeOrientation::O0),
        EdgeCubie::new(EdgeCubicle::C4, EdgeOrientation::O0),
        EdgeCubie::new(EdgeCubicle::C10, EdgeOrientation::O0),
        EdgeCubie::new(EdgeCubicle::C6, EdgeOrientation::O0),
        EdgeCubie::new(EdgeCubicle::C2, EdgeOrientation::O0),
        EdgeCubie::new(EdgeCubicle::C8, EdgeOrientation::O0),
        EdgeCubie::new(EdgeCubicle::C9, EdgeOrientation::O0),
        EdgeCubie::new(EdgeCubicle::C7, EdgeOrientation::O0),
        EdgeCubie::new(EdgeCubicle::C11, EdgeOrientation::O0),
    ]),
};

const UMOVE: CubieCube = CubieCube {
    corners: CubicleArray::new([
        CornerCubie::new(CornerCubicle::C1, CornerOrientation::O0),
        CornerCubie::new(CornerCubicle::C3, CornerOrientation::O0),
        CornerCubie::new(CornerCubicle::C0, CornerOrientation::O0),
        CornerCubie::new(CornerCubicle::C2, CornerOrientation::O0),
        CornerCubie::new(CornerCubicle::C4, CornerOrientation::O0),
        CornerCubie::new(CornerCubicle::C5, CornerOrientation::O0),
        CornerCubie::new(CornerCubicle::C6, CornerOrientation::O0),
        CornerCubie::new(CornerCubicle::C7, CornerOrientation::O0),
    ]),
    edges: CubicleArray::new([
        EdgeCubie::new(EdgeCubicle::C2, EdgeOrientation::O0),
        EdgeCubie::new(EdgeCubicle::C0, EdgeOrientation::O0),
        EdgeCubie::new(EdgeCubicle::C3, EdgeOrientation::O0),
        EdgeCubie::new(EdgeCubicle::C1, EdgeOrientation::O0),
        EdgeCubie::new(EdgeCubicle::C4, EdgeOrientation::O0),
        EdgeCubie::new(EdgeCubicle::C5, EdgeOrientation::O0),
        EdgeCubie::new(EdgeCubicle::C6, EdgeOrientation::O0),
        EdgeCubie::new(EdgeCubicle::C7, EdgeOrientation::O0),
        EdgeCubie::new(EdgeCubicle::C8, EdgeOrientation::O0),
        EdgeCubie::new(EdgeCubicle::C9, EdgeOrientation::O0),
        EdgeCubie::new(EdgeCubicle::C10, EdgeOrientation::O0),
        EdgeCubie::new(EdgeCubicle::C11, EdgeOrientation::O0),
    ]),
};

const FMOVE: CubieCube = CubieCube {
    corners: CubicleArray::new([
        CornerCubie::new(CornerCubicle::C0, CornerOrientation::O0),
        CornerCubie::new(CornerCubicle::C1, CornerOrientation::O0),
        CornerCubie::new(CornerCubicle::C3, CornerOrientation::O1),
        CornerCubie::new(CornerCubicle::C7, CornerOrientation::O2),
        CornerCubie::new(CornerCubicle::C4, CornerOrientation::O0),
        CornerCubie::new(CornerCubicle::C5, CornerOrientation::O0),
        CornerCubie::new(CornerCubicle::C2, CornerOrientation::O2),
        CornerCubie::new(CornerCubicle::C6, CornerOrientation::O1),
    ]),
    edges: CubicleArray::new([
        EdgeCubie::new(EdgeCubicle::C0, EdgeOrientation::O0),
        EdgeCubie::new(EdgeCubicle::C1, EdgeOrientation::O0),
        EdgeCubie::new(EdgeCubicle::C2, EdgeOrientation::O0),
        EdgeCubie::new(EdgeCubicle::C7, EdgeOrientation::O1),
        EdgeCubie::new(EdgeCubicle::C4, EdgeOrientation::O0),
        EdgeCubie::new(EdgeCubicle::C5, EdgeOrientation::O0),
        EdgeCubie::new(EdgeCubicle::C3, EdgeOrientation::O1),
        EdgeCubie::new(EdgeCubicle::C11, EdgeOrientation::O1),
        EdgeCubie::new(EdgeCubicle::C8, EdgeOrientation::O0),
        EdgeCubie::new(EdgeCubicle::C9, EdgeOrientation::O0),
        EdgeCubie::new(EdgeCubicle::C10, EdgeOrientation::O0),
        EdgeCubie::new(EdgeCubicle::C6, EdgeOrientation::O1),
    ]),
};

const LMOVE: CubieCube = CubieCube {
    corners: CubicleArray::new([
        CornerCubie::new(CornerCubicle::C2, CornerOrientation::O1),
        CornerCubie::new(CornerCubicle::C1, CornerOrientation::O0),
        CornerCubie::new(CornerCubicle::C6, CornerOrientation::O2),
        CornerCubie::new(CornerCubicle::C3, CornerOrientation::O0),
        CornerCubie::new(CornerCubicle::C0, CornerOrientation::O2),
        CornerCubie::new(CornerCubicle::C5, CornerOrientation::O0),
        CornerCubie::new(CornerCubicle::C4, CornerOrientation::O1),
        CornerCubie::new(CornerCubicle::C7, CornerOrientation::O0),
    ]),
    edges: CubicleArray::new([
        EdgeCubie::new(EdgeCubicle::C0, EdgeOrientation::O0),
        EdgeCubie::new(EdgeCubicle::C6, EdgeOrientation::O0),
        EdgeCubie::new(EdgeCubicle::C2, EdgeOrientation::O0),
        EdgeCubie::new(EdgeCubicle::C3, EdgeOrientation::O0),
        EdgeCubie::new(EdgeCubicle::C1, EdgeOrientation::O0),
        EdgeCubie::new(EdgeCubicle::C5, EdgeOrientation::O0),
        EdgeCubie::new(EdgeCubicle::C9, EdgeOrientation::O0),
        EdgeCubie::new(EdgeCubicle::C7, EdgeOrientation::O0),
        EdgeCubie::new(EdgeCubicle::C8, EdgeOrientation::O0),
        EdgeCubie::new(EdgeCubicle::C4, EdgeOrientation::O0),
        EdgeCubie::new(EdgeCubicle::C10, EdgeOrientation::O0),
        EdgeCubie::new(EdgeCubicle::C11, EdgeOrientation::O0),
    ]),
};

const BMOVE: CubieCube = CubieCube {
    corners: CubicleArray::new([
        CornerCubie::new(CornerCubicle::C4, CornerOrientation::O2),
        CornerCubie::new(CornerCubicle::C0, CornerOrientation::O1),
        CornerCubie::new(CornerCubicle::C2, CornerOrientation::O0),
        CornerCubie::new(CornerCubicle::C3, CornerOrientation::O0),
        CornerCubie::new(CornerCubicle::C5, CornerOrientation::O1),
        CornerCubie::new(CornerCubicle::C1, CornerOrientation::O2),
        CornerCubie::new(CornerCubicle::C6, CornerOrientation::O0),
        CornerCubie::new(CornerCubicle::C7, CornerOrientation::O0),
    ]),
    edges: CubicleArray::new([
        EdgeCubie::new(EdgeCubicle::C4, EdgeOrientation::O1),
        EdgeCubie::new(EdgeCubicle::C1, EdgeOrientation::O0),
        EdgeCubie::new(EdgeCubicle::C2, EdgeOrientation::O0),
        EdgeCubie::new(EdgeCubicle::C3, EdgeOrientation::O0),
        EdgeCubie::new(EdgeCubicle::C8, EdgeOrientation::O1),
        EdgeCubie::new(EdgeCubicle::C0, EdgeOrientation::O1),
        EdgeCubie::new(EdgeCubicle::C6, EdgeOrientation::O0),
        EdgeCubie::new(EdgeCubicle::C7, EdgeOrientation::O0),
        EdgeCubie::new(EdgeCubicle::C5, EdgeOrientation::O1),
        EdgeCubie::new(EdgeCubicle::C9, EdgeOrientation::O0),
        EdgeCubie::new(EdgeCubicle::C10, EdgeOrientation::O0),
        EdgeCubie::new(EdgeCubicle::C11, EdgeOrientation::O0),
    ]),
};

const DMOVE: CubieCube = CubieCube {
    corners: CubicleArray::new([
        CornerCubie::new(CornerCubicle::C0, CornerOrientation::O0),
        CornerCubie::new(CornerCubicle::C1, CornerOrientation::O0),
        CornerCubie::new(CornerCubicle::C2, CornerOrientation::O0),
        CornerCubie::new(CornerCubicle::C3, CornerOrientation::O0),
        CornerCubie::new(CornerCubicle::C6, CornerOrientation::O0),
        CornerCubie::new(CornerCubicle::C4, CornerOrientation::O0),
        CornerCubie::new(CornerCubicle::C7, CornerOrientation::O0),
        CornerCubie::new(CornerCubicle::C5, CornerOrientation::O0),
    ]),
    edges: CubicleArray::new([
        EdgeCubie::new(EdgeCubicle::C0, EdgeOrientation::O0),
        EdgeCubie::new(EdgeCubicle::C1, EdgeOrientation::O0),
        EdgeCubie::new(EdgeCubicle::C2, EdgeOrientation::O0),
        EdgeCubie::new(EdgeCubicle::C3, EdgeOrientation::O0),
        EdgeCubie::new(EdgeCubicle::C4, EdgeOrientation::O0),
        EdgeCubie::new(EdgeCubicle::C5, EdgeOrientation::O0),
        EdgeCubie::new(EdgeCubicle::C6, EdgeOrientation::O0),
        EdgeCubie::new(EdgeCubicle::C7, EdgeOrientation::O0),
        EdgeCubie::new(EdgeCubicle::C9, EdgeOrientation::O0),
        EdgeCubie::new(EdgeCubicle::C11, EdgeOrientation::O0),
        EdgeCubie::new(EdgeCubicle::C8, EdgeOrientation::O0),
        EdgeCubie::new(EdgeCubicle::C10, EdgeOrientation::O0),
    ]),
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn possible_states() {
        assert!(CubieCube::SOLVED.is_possible_state());
        assert!(TPERM.is_possible_state());
        assert!(RMOVE.is_possible_state());
        let one_edge_flipped = {
            let mut ret = CubieCube::SOLVED;
            ret.edges[EdgeCubicle::C0].set_orientation(EdgeOrientation::O1);
            ret
        };
        assert!(!one_edge_flipped.is_possible_state());

        let two_corners_swapped = {
            let mut ret = CubieCube::SOLVED;
            ret.corners.swap(CornerCubicle::C0, CornerCubicle::C1);
            ret
        };

        assert!(!two_corners_swapped.is_possible_state());
    }

    #[test]
    fn set_udslice_coord() {
        let mut cube = CubieCube::SOLVED;
        for coord in crate::coord_cube::CoordCube::UDSLICE_RANGE {
            cube.set_udslice_coord(coord);
            assert_eq!(cube.get_udslice_coord(), coord);
            assert!(cube.is_possible_state());
            assert!(all_cubies_seen(cube.edges));
        }
    }

    #[test]
    fn set_ori_coord() {
        fn aux<C: Cubies>()
        where
            CubieCube: Index<C::Cubicle, Output = C::Cubie> + IndexMut<C::Cubicle>,
        {
            let mut cube = CubieCube::SOLVED;
            for coord in C::ORI_COORD_RANGE {
                cube.set_ori_coord::<C>(coord);
                assert!(cube.is_possible_state());
                assert_eq!(cube.get_ori_coord::<C>(), coord);
            }
        }

        aux::<Corners>();
        aux::<Edges>();
    }

    #[test]
    fn random_possible_states_are_possible() {
        for _ in 0..1000 {
            let state = CubieCube::random_possible();
            assert!(state.is_possible_state(), "{state:?}");
        }
    }

    #[test]
    fn move_application() {
        use Move::*;
        let mut tperm = CubieCube::SOLVED;
        for moov in [R, U, Ri, Ui, Ri, F, R2, Ui, Ri, Ui, R, U, Ri, Fi] {
            tperm.apply_move(moov);
        }

        assert_eq!(tperm, TPERM);

        let mut rmove = CubieCube::SOLVED;
        rmove.apply_move(R);
        assert_eq!(rmove, RMOVE);
        rmove.apply_move(Ri);
        assert_eq!(rmove, CubieCube::SOLVED);
    }

    #[test]
    fn group_ops() {
        assert_eq!(CubieCube::SOLVED, CubieCube::SOLVED.inverse());
        assert_eq!(CubieCube::SOLVED * CubieCube::SOLVED, CubieCube::SOLVED);
        assert_eq!(RMOVE * CubieCube::SOLVED, RMOVE);
        assert_eq!(CubieCube::SOLVED * TPERM, TPERM);

        assert_eq!(TPERM.inverse(), TPERM);
        assert_eq!(TPERM * TPERM.inverse(), CubieCube::SOLVED);
        assert_eq!(TPERM * TPERM, CubieCube::SOLVED);

        assert_ne!(RMOVE.inverse(), RMOVE);
        assert_ne!(RMOVE.inverse(), CubieCube::SOLVED);
        assert_ne!(RMOVE * RMOVE, RMOVE);
        assert_ne!(RMOVE * RMOVE, CubieCube::SOLVED);
        assert_ne!(RMOVE * RMOVE * RMOVE, CubieCube::SOLVED);
        assert_eq!(RMOVE * RMOVE * RMOVE * RMOVE, CubieCube::SOLVED);

        assert_eq!(
            CubieCube::SOLVED,
            TPERM * RMOVE * RMOVE.inverse() * TPERM.inverse()
        );
    }

    const TPERM: CubieCube = CubieCube {
        corners: {
            use CornerCubicle::*;
            use CornerOrientation::O0;
            CubicleArray::new([
                CornerCubie::new(C0, O0),
                CornerCubie::new(C3, O0),
                CornerCubie::new(C2, O0),
                CornerCubie::new(C1, O0),
                CornerCubie::new(C4, O0),
                CornerCubie::new(C5, O0),
                CornerCubie::new(C6, O0),
                CornerCubie::new(C7, O0),
            ])
        },
        edges: {
            use EdgeCubicle::*;
            use EdgeOrientation::O0;
            CubicleArray::new([
                EdgeCubie::new(C0, O0),
                EdgeCubie::new(C2, O0),
                EdgeCubie::new(C1, O0),
                EdgeCubie::new(C3, O0),
                EdgeCubie::new(C4, O0),
                EdgeCubie::new(C5, O0),
                EdgeCubie::new(C6, O0),
                EdgeCubie::new(C7, O0),
                EdgeCubie::new(C8, O0),
                EdgeCubie::new(C9, O0),
                EdgeCubie::new(C10, O0),
                EdgeCubie::new(C11, O0),
            ])
        },
    };
}
