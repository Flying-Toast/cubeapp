///! Array types that are indexed by cubicles instead of `usize`
// TODO: DRY this stuff
use crate::cubiestate::{CornerCubicle, EdgeCubicle};
use std::ops::{Index, IndexMut};

/// Newtype wrapper around `[T; 8]` to allow indexing by a `CornerCubicle`
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
#[repr(transparent)]
pub struct CornerCubicleIndexed<T>([T; 8]);

impl<T> CornerCubicleIndexed<T> {
    pub const fn new(array: [T; 8]) -> Self {
        Self(array)
    }

    pub fn swap(&mut self, a: CornerCubicle, b: CornerCubicle) {
        self.0.swap(a as usize, b as usize);
    }

    pub fn shuffle<R: rand::Rng>(&mut self, rng: &mut R) {
        use rand::seq::SliceRandom;
        self.0.shuffle(rng);
    }
}

impl<T> Index<CornerCubicle> for CornerCubicleIndexed<T> {
    type Output = T;
    fn index(&self, index: CornerCubicle) -> &Self::Output {
        &self.0[index as usize]
    }
}

impl<T> IndexMut<CornerCubicle> for CornerCubicleIndexed<T> {
    fn index_mut(&mut self, index: CornerCubicle) -> &mut Self::Output {
        &mut self.0[index as usize]
    }
}

impl<T> IntoIterator for CornerCubicleIndexed<T> {
    type Item = T;
    type IntoIter = <[T; 8] as IntoIterator>::IntoIter;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

/// Newtype wrapper around `[T; 12]` to allow indexing by an `EdgeCubicle`
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
#[repr(transparent)]
pub struct EdgeCubicleIndexed<T>([T; 12]);

impl<T> EdgeCubicleIndexed<T> {
    pub const fn new(array: [T; 12]) -> Self {
        Self(array)
    }

    pub fn swap(&mut self, a: EdgeCubicle, b: EdgeCubicle) {
        self.0.swap(a as usize, b as usize);
    }

    pub fn shuffle<R: rand::Rng>(&mut self, rng: &mut R) {
        use rand::seq::SliceRandom;
        self.0.shuffle(rng);
    }
}

impl<T> Index<EdgeCubicle> for EdgeCubicleIndexed<T> {
    type Output = T;
    fn index(&self, index: EdgeCubicle) -> &Self::Output {
        &self.0[index as usize]
    }
}

impl<T> IndexMut<EdgeCubicle> for EdgeCubicleIndexed<T> {
    fn index_mut(&mut self, index: EdgeCubicle) -> &mut Self::Output {
        &mut self.0[index as usize]
    }
}

impl<T> IntoIterator for EdgeCubicleIndexed<T> {
    type Item = T;
    type IntoIter = <[T; 12] as IntoIterator>::IntoIter;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
