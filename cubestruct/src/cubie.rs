use std::fmt;
use std::mem::transmute;
use std::ops::{Index, IndexMut};

pub type Corners = CubicleArray<CornerState, 8>;
pub type Edges = CubicleArray<EdgeState, 12>;

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub struct CubicleArray<T, const N: usize>([T; N]);

impl<T, const N: usize> CubicleArray<T, N> {
    pub const fn new(items: [T; N]) -> Self {
        Self(items)
    }

    pub fn shuffle<R: rand::Rng>(&mut self, rng: &mut R) {
        use rand::seq::SliceRandom;
        self.0.shuffle(rng);
    }
}

impl<T> CubicleArray<T, 8> {
    pub fn swap(&mut self, a: CornerCubicle, b: CornerCubicle) {
        self.0.swap(a as usize, b as usize);
    }
}

impl<T> CubicleArray<T, 12> {
    pub fn swap(&mut self, a: EdgeCubicle, b: EdgeCubicle) {
        self.0.swap(a as usize, b as usize);
    }
}

impl<T, const N: usize> IntoIterator for CubicleArray<T, N> {
    type IntoIter = <[T; N] as IntoIterator>::IntoIter;
    type Item = T;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<T> Index<CornerCubicle> for CubicleArray<T, 8> {
    type Output = T;
    fn index(&self, index: CornerCubicle) -> &Self::Output {
        &self.0[index as usize]
    }
}

impl<T> IndexMut<CornerCubicle> for CubicleArray<T, 8> {
    fn index_mut(&mut self, index: CornerCubicle) -> &mut Self::Output {
        &mut self.0[index as usize]
    }
}

impl<T> Index<EdgeCubicle> for CubicleArray<T, 12> {
    type Output = T;
    fn index(&self, index: EdgeCubicle) -> &Self::Output {
        &self.0[index as usize]
    }
}

impl<T> IndexMut<EdgeCubicle> for CubicleArray<T, 12> {
    fn index_mut(&mut self, index: EdgeCubicle) -> &mut Self::Output {
        &mut self.0[index as usize]
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u8)]
pub enum CornerCubicle {
    C0 = 0,
    C1,
    C2,
    C3,
    C4,
    C5,
    C6,
    C7,
}

impl CornerCubicle {
    pub fn all() -> [Self; 8] {
        use CornerCubicle::*;
        [C0, C1, C2, C3, C4, C5, C6, C7]
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u8)]
pub enum CornerOrientation {
    /// No twist
    O0 = 0,
    /// Clockwise twist
    O1,
    /// Counterclockwise twist
    O2,
}

impl CornerOrientation {
    pub fn all() -> [Self; 3] {
        use CornerOrientation::*;
        [O0, O1, O2]
    }

    /// Returns the orientation that needs to be applied to this `self`
    /// orientation in order to return it to O0.
    pub fn inverse(self) -> Self {
        match self {
            Self::O0 => Self::O0,
            Self::O1 => Self::O2,
            Self::O2 => Self::O1,
        }
    }

    /// Combines two orientations
    pub fn add(self, rhs: Self) -> Self {
        let sum = self as u8 + rhs as u8;
        // SAFETY: Modulo 3 always produces a value 0..=2, which are all valid `CornerOrientation`s
        unsafe { transmute::<u8, CornerOrientation>(sum % 3) }
    }

    pub fn random<R: rand::Rng>(rng: &mut R) -> Self {
        unsafe { transmute::<u8, CornerOrientation>(rng.gen_range(0..=2)) }
    }
}

/// Permutation + orientation of a single corner cubie
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct CornerState(u8);

impl fmt::Debug for CornerState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "CornerState({:?}, {:?})",
            self.cubicle(),
            self.orientation()
        )
    }
}

impl CornerState {
    #[must_use]
    pub const fn new(p: CornerCubicle, o: CornerOrientation) -> Self {
        Self(((o as u8) << 3) | (p as u8))
    }

    /// What cubicle this corner is in
    #[must_use]
    pub const fn cubicle(self) -> CornerCubicle {
        // SAFETY: All possible 3-bit numbers are a valid CornerCubicle
        unsafe { transmute::<u8, CornerCubicle>(self.0 & 0b111) }
    }

    #[must_use]
    pub const fn orientation(self) -> CornerOrientation {
        // SAFETY: All ways of constructing a `CornerState` preserve this invariant
        unsafe { transmute::<u8, CornerOrientation>(self.0 >> 3) }
    }

    pub fn set_orientation(&mut self, o: CornerOrientation) {
        *self = Self::new(self.cubicle(), o);
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u8)]
pub enum EdgeCubicle {
    C0 = 0,
    C1,
    C2,
    C3,
    C4,
    C5,
    C6,
    C7,
    C8,
    C9,
    C10,
    C11,
}

impl EdgeCubicle {
    pub fn all() -> [Self; 12] {
        use EdgeCubicle::*;
        [C0, C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C11]
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u8)]
pub enum EdgeOrientation {
    /// Not flipped
    O0 = 0,
    /// Flipped
    O1,
}

impl EdgeOrientation {
    pub fn all() -> [Self; 2] {
        use EdgeOrientation::*;
        [O0, O1]
    }

    /// Returns the orientation that needs to be applied to this `self`
    /// orientation in order to return it to O0.
    ///
    /// *Edge* orientations are their own inverse:
    /// - Flipping a flipped edge flips it back to O0.
    /// - By not flipping an unflipped edge you stay at the unflipped
    /// orientation
    pub fn inverse(self) -> Self {
        self
    }

    /// Combines two orientations
    pub fn add(self, rhs: Self) -> Self {
        let sum = self as u8 + rhs as u8;
        unsafe { transmute::<u8, EdgeOrientation>(sum % 2) }
    }

    pub fn random<R: rand::Rng>(rng: &mut R) -> Self {
        unsafe { transmute::<u8, EdgeOrientation>(rng.gen_range(0..=1)) }
    }
}

/// Permutation + orientation of a single edge cubie
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct EdgeState(u8);

impl fmt::Debug for EdgeState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "EdgeState({:?}, {:?})",
            self.cubicle(),
            self.orientation()
        )
    }
}

impl EdgeState {
    #[must_use]
    pub const fn new(p: EdgeCubicle, o: EdgeOrientation) -> Self {
        Self(((p as u8) << 1) | (o as u8))
    }

    /// What cubicle this edge is in
    #[must_use]
    pub const fn cubicle(self) -> EdgeCubicle {
        // SAFETY: Invariant upheld by constructors
        unsafe { transmute::<u8, EdgeCubicle>(self.0 >> 1) }
    }

    #[must_use]
    pub const fn orientation(self) -> EdgeOrientation {
        // SAFETY: All 1-bit numbers are a valid EdgeOrientation
        unsafe { transmute::<u8, EdgeOrientation>(self.0 & 1) }
    }

    pub fn set_orientation(&mut self, o: EdgeOrientation) {
        *self = Self::new(self.cubicle(), o);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cornerstate() {
        for c in CornerCubicle::all() {
            for o in CornerOrientation::all() {
                let state = CornerState::new(c, o);
                assert_eq!(c, state.cubicle());
                assert_eq!(o, state.orientation());
            }
        }
    }

    #[test]
    fn edgestate() {
        for c in EdgeCubicle::all() {
            for o in EdgeOrientation::all() {
                let state = EdgeState::new(c, o);
                assert_eq!(c, state.cubicle());
                assert_eq!(o, state.orientation());
            }
        }
    }
}
