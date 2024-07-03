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
    #[cfg(test)]
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

impl CornerOrientation {
    #[cfg(test)]
    fn all() -> impl Iterator<Item = Self> {
        use CornerOrientation::*;
        [O0, O1, O2].into_iter()
    }
}

/// Permutation + orientation of a single corner cubie
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct CornerState(u8);

impl CornerState {
    #[must_use]
    #[inline]
    pub const fn new(p: CornerCubicle, o: CornerOrientation) -> Self {
        Self(((o as u8) << 3) | (p as u8))
    }

    /// What cubicle this corner is in
    #[must_use]
    #[inline]
    pub const fn cubicle(self) -> CornerCubicle {
        // SAFETY: All possible 3-bit numbers are a valid CornerCubicle
        unsafe { transmute::<u8, CornerCubicle>(self.0 & 0b111) }
    }

    #[must_use]
    #[inline]
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
    #[cfg(test)]
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

impl EdgeOrientation {
    #[cfg(test)]
    fn all() -> impl Iterator<Item = Self> {
        use EdgeOrientation::*;
        [O0, O1].into_iter()
    }
}

/// Permutation + orientation of a single edge cubie
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct EdgeState(u8);

impl EdgeState {
    #[must_use]
    #[inline]
    pub const fn new(p: EdgeCubicle, o: EdgeOrientation) -> Self {
        Self(((p as u8) << 1) | (o as u8))
    }

    /// What cubicle this edge is in
    #[must_use]
    #[inline]
    pub const fn cubicle(self) -> EdgeCubicle {
        // SAFETY: Invariant upheld by constructors
        unsafe { transmute::<u8, EdgeCubicle>(self.0 >> 1) }
    }

    #[must_use]
    #[inline]
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