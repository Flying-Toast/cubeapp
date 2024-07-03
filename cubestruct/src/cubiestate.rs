use std::fmt;
use std::mem::transmute;

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
    pub fn all() -> impl Iterator<Item = Self> {
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

impl CornerOrientation {
    pub fn all() -> impl Iterator<Item = Self> {
        use CornerOrientation::*;
        [O0, O1, O2].into_iter()
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
    pub fn mul(self, rhs: Self) -> Self {
        let sum = self as u8 + rhs as u8;
        // SAFETY: Modulo 3 always produces a value 0..=2, which are all valid `CornerOrientation`s
        unsafe { transmute::<u8, CornerOrientation>(sum % 3) }
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
    pub fn all() -> impl Iterator<Item = Self> {
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

impl EdgeOrientation {
    pub fn all() -> impl Iterator<Item = Self> {
        use EdgeOrientation::*;
        [O0, O1].into_iter()
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
    pub fn mul(self, rhs: Self) -> Self {
        // SAFETY: all ways of ORing {1,0} will yield 1 or 0
        unsafe { transmute::<u8, EdgeOrientation>(self as u8 | rhs as u8) }
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
