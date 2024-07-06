use std::fmt;
use std::mem::transmute;
use std::ops::{Index, IndexMut};

pub type Corners = CubicleArray<CornerCubie, 8>;
pub type Edges = CubicleArray<EdgeCubie, 12>;

pub trait Cubicle: fmt::Debug + Eq + Copy {
    /// Enumerate all values of the type
    fn all() -> impl Iterator<Item = Self>;
}

pub trait Orientation: fmt::Debug + Eq + Copy {
    /// Enumerate all values of the type
    fn all() -> impl Iterator<Item = Self>;

    /// The zero orientation (all cubies in a solved cube have an orientation of zero)
    fn zero() -> Self {
        Self::all().next().unwrap()
    }

    /// Returns the orientation that needs to be applied to this `self`
    /// orientation in order to return it to O0.
    fn inverse(self) -> Self;

    /// Combines two orientations
    fn add(self, rhs: Self) -> Self;

    /// Generate a random orientation
    fn random<R: rand::Rng>(rng: &mut R) -> Self;
}

pub trait Cubie<C, O>: fmt::Debug + Eq + Copy + Sized {
    #[must_use]
    fn new(c: C, o: O) -> Self;

    /// What cubicle this cubie is in
    #[must_use]
    fn cubicle(self) -> C;

    /// Get the orientation of this cubie
    #[must_use]
    fn orientation(self) -> O;

    /// Set this cubie's orientation in place
    fn set_orientation(&mut self, o: O);
}

pub trait Cubies:
    fmt::Debug
    + Eq
    + Copy
    + IntoIterator<Item = Self::Cubie>
    + Index<Self::Cubicle, Output = Self::Cubie>
    + IndexMut<Self::Cubicle>
{
    type Cubicle: Cubicle;
    type Orientation: Orientation;
    type Cubie: Cubie<Self::Cubicle, Self::Orientation>;
    /// Generic array that can be indexed by this cubie's cubicle type
    type CubicleArray<T>: Index<Self::Cubicle, Output = T>
        + IndexMut<Self::Cubicle>
        + IntoIterator<Item = T>;

    /// Swap the items at the given indices
    fn swap(&mut self, a: Self::Cubicle, b: Self::Cubicle);

    fn shuffle<R: rand::Rng>(&mut self, rng: &mut R);

    /// Create a cubicle-indexed array with every element intialized to `init`.
    fn new_array<T: Copy>(init: T) -> Self::CubicleArray<T>;
}

impl Cubies for Corners {
    type Cubicle = CornerCubicle;
    type Orientation = CornerOrientation;
    type Cubie = CornerCubie;
    type CubicleArray<T> = CubicleArray<T, 8>;

    fn swap(&mut self, a: Self::Cubicle, b: Self::Cubicle) {
        self.0.swap(a as usize, b as usize)
    }

    fn shuffle<R: rand::Rng>(&mut self, rng: &mut R) {
        use rand::seq::SliceRandom;
        self.0.shuffle(rng);
    }

    fn new_array<T: Copy>(init: T) -> Self::CubicleArray<T> {
        CubicleArray::new([init; 8])
    }
}

impl Cubies for Edges {
    type Cubicle = EdgeCubicle;
    type Orientation = EdgeOrientation;
    type Cubie = EdgeCubie;
    type CubicleArray<T> = CubicleArray<T, 12>;

    fn swap(&mut self, a: Self::Cubicle, b: Self::Cubicle) {
        self.0.swap(a as usize, b as usize)
    }

    fn shuffle<R: rand::Rng>(&mut self, rng: &mut R) {
        use rand::seq::SliceRandom;
        self.0.shuffle(rng);
    }

    fn new_array<T: Copy>(init: T) -> Self::CubicleArray<T> {
        CubicleArray::new([init; 12])
    }
}

/// Wrapper around `[T; N]` that is indexed by a cubicle instead of a usize.
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub struct CubicleArray<T, const N: usize>([T; N]);

impl<T, const N: usize> CubicleArray<T, N> {
    pub const fn new(items: [T; N]) -> Self {
        Self(items)
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

impl Cubicle for CornerCubicle {
    fn all() -> impl Iterator<Item = Self> {
        use CornerCubicle::*;
        [C0, C1, C2, C3, C4, C5, C6, C7].into_iter()
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

impl Orientation for CornerOrientation {
    fn all() -> impl Iterator<Item = Self> {
        use CornerOrientation::*;
        [O0, O1, O2].into_iter()
    }

    fn inverse(self) -> Self {
        match self {
            Self::O0 => Self::O0,
            Self::O1 => Self::O2,
            Self::O2 => Self::O1,
        }
    }

    fn add(self, rhs: Self) -> Self {
        let sum = self as u8 + rhs as u8;
        // SAFETY: Modulo 3 always produces a value 0..=2, which are all valid `CornerOrientation`s
        unsafe { transmute::<u8, CornerOrientation>(sum % 3) }
    }

    fn random<R: rand::Rng>(rng: &mut R) -> Self {
        unsafe { transmute::<u8, CornerOrientation>(rng.gen_range(0..=2)) }
    }
}

/// Permutation + orientation of a single corner cubie
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct CornerCubie(u8);

impl fmt::Debug for CornerCubie {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "CornerCubie({:?}, {:?})",
            self.cubicle(),
            self.orientation()
        )
    }
}

impl CornerCubie {
    #[must_use]
    pub const fn new(c: CornerCubicle, o: CornerOrientation) -> Self {
        Self(((o as u8) << 3) | (c as u8))
    }
}

impl Cubie<CornerCubicle, CornerOrientation> for CornerCubie {
    fn new(c: CornerCubicle, o: CornerOrientation) -> Self {
        Self::new(c, o)
    }

    fn cubicle(self) -> CornerCubicle {
        // SAFETY: All possible 3-bit numbers are a valid CornerCubicle
        unsafe { transmute::<u8, CornerCubicle>(self.0 & 0b111) }
    }

    fn orientation(self) -> CornerOrientation {
        // SAFETY: All ways of constructing a `CornerCubie` preserve this invariant
        unsafe { transmute::<u8, CornerOrientation>(self.0 >> 3) }
    }

    fn set_orientation(&mut self, o: CornerOrientation) {
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

impl Cubicle for EdgeCubicle {
    fn all() -> impl Iterator<Item = Self> {
        use EdgeCubicle::*;
        [C0, C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C11].into_iter()
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

impl Orientation for EdgeOrientation {
    fn all() -> impl Iterator<Item = Self> {
        use EdgeOrientation::*;
        [O0, O1].into_iter()
    }

    /// Edge orientations are their own inverse:
    /// - Flipping a flipped edge flips it back to O0.
    /// - By not flipping an unflipped edge you stay at the unflipped
    /// orientation
    fn inverse(self) -> Self {
        self
    }

    fn add(self, rhs: Self) -> Self {
        let sum = self as u8 + rhs as u8;
        unsafe { transmute::<u8, EdgeOrientation>(sum % 2) }
    }

    fn random<R: rand::Rng>(rng: &mut R) -> Self {
        unsafe { transmute::<u8, EdgeOrientation>(rng.gen_range(0..=1)) }
    }
}

/// Permutation + orientation of a single edge cubie
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct EdgeCubie(u8);

impl fmt::Debug for EdgeCubie {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "EdgeCubie({:?}, {:?})",
            self.cubicle(),
            self.orientation()
        )
    }
}

impl EdgeCubie {
    #[must_use]
    pub const fn new(c: EdgeCubicle, o: EdgeOrientation) -> Self {
        Self(((c as u8) << 1) | (o as u8))
    }
}

impl Cubie<EdgeCubicle, EdgeOrientation> for EdgeCubie {
    fn new(c: EdgeCubicle, o: EdgeOrientation) -> Self {
        Self::new(c, o)
    }

    fn cubicle(self) -> EdgeCubicle {
        // SAFETY: Invariant upheld by constructors
        unsafe { transmute::<u8, EdgeCubicle>(self.0 >> 1) }
    }

    fn orientation(self) -> EdgeOrientation {
        // SAFETY: All 1-bit numbers are a valid EdgeOrientation
        unsafe { transmute::<u8, EdgeOrientation>(self.0 & 1) }
    }

    fn set_orientation(&mut self, o: EdgeOrientation) {
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
                let state = CornerCubie::new(c, o);
                assert_eq!(c, state.cubicle());
                assert_eq!(o, state.orientation());
            }
        }
    }

    #[test]
    fn edgestate() {
        for c in EdgeCubicle::all() {
            for o in EdgeOrientation::all() {
                let state = EdgeCubie::new(c, o);
                assert_eq!(c, state.cubicle());
                assert_eq!(o, state.orientation());
            }
        }
    }
}
