use crate::cubie::*;
use crate::{CubieCube, Move};
use std::ops::{Index, IndexMut, Range};
use std::sync::OnceLock;

const NUM_CORNER_ORIS: u16 = 2187;
const NUM_EDGE_ORIS: u16 = 2048;
const NUM_UDSLICES: u16 = 495;

#[derive(Debug)]
pub struct CoordCube {
    /// Corner orientation ("twist" coordinate)
    /// Used in phase 1
    corner_ori: u16,
    /// Edge orientation ("flip" coordinate)
    /// Used in phase 1
    edge_ori: u16,
    /// Positions of the 4 equator edges ("udslice" coordinate)
    /// Used in phase 1
    udslice: u16,
}

impl CoordCube {
    pub(crate) const CORNER_ORI_RANGE: Range<u16> = 0..NUM_CORNER_ORIS;
    pub(crate) const EDGE_ORI_RANGE: Range<u16> = 0..NUM_EDGE_ORIS;
    pub(crate) const UDSLICE_RANGE: Range<u16> = 0..NUM_UDSLICES;

    fn from_cubie_cube(cubie_cube: &CubieCube) -> Self {
        Self {
            corner_ori: cubie_cube.get_ori_coord::<Corners>(),
            edge_ori: cubie_cube.get_ori_coord::<Edges>(),
            udslice: cubie_cube.get_udslice_coord(),
        }
    }

    fn to_cubie_cube(&self) -> CubieCube {
        // XXX: this pattern is here as a reminder to keep
        // this method up to date as new coords are added :-)
        #[deny(unused_variables)]
        let Self {
            corner_ori,
            edge_ori,
            udslice,
        } = self;

        let mut ret = CubieCube::SOLVED;
        ret.set_ori_coord::<Corners>(*corner_ori);
        ret.set_ori_coord::<Edges>(*edge_ori);
        ret.set_udslice_coord(*udslice);
        ret
    }

    pub fn apply_move(&mut self, moov: Move) {
        // XXX: this pattern is here as a reminder to keep
        // this method up to date as new coords are added :-)
        #[deny(unused_variables)]
        let Self {
            corner_ori,
            edge_ori,
            udslice,
        } = self;

        *corner_ori = corner_ori_move_table()[moov][*corner_ori as usize];
        *edge_ori = edge_ori_move_table()[moov][*edge_ori as usize];
        *udslice = udslice_move_table()[moov][*udslice as usize];
    }
}

fn udslice_move_table() -> &'static MoveTable<[u16; NUM_UDSLICES as usize]> {
    static TABLE: OnceLock<MoveTable<[u16; NUM_UDSLICES as usize]>> = OnceLock::new();

    TABLE.get_or_init(|| {
        let mut tbl = MoveTable([[0; NUM_UDSLICES as usize]; 18]);
        let mut cc = CubieCube::SOLVED;
        for coord in CoordCube::UDSLICE_RANGE {
            for moov in Move::all() {
                cc.set_udslice_coord(coord);
                cc.apply_move(moov);
                tbl[moov][coord as usize] = cc.get_udslice_coord();
            }
        }
        tbl
    })
}

fn edge_ori_move_table() -> &'static MoveTable<[u16; NUM_EDGE_ORIS as usize]> {
    static TABLE: OnceLock<MoveTable<[u16; NUM_EDGE_ORIS as usize]>> = OnceLock::new();

    TABLE.get_or_init(|| {
        let mut tbl = MoveTable([[0; NUM_EDGE_ORIS as usize]; 18]);
        let mut cc = CubieCube::SOLVED;
        for coord in CoordCube::EDGE_ORI_RANGE {
            for moov in Move::all() {
                cc.set_ori_coord::<Edges>(coord);
                cc.apply_move(moov);
                tbl[moov][coord as usize] = cc.get_ori_coord::<Edges>();
            }
        }
        tbl
    })
}

fn corner_ori_move_table() -> &'static MoveTable<[u16; NUM_CORNER_ORIS as usize]> {
    static TABLE: OnceLock<MoveTable<[u16; NUM_CORNER_ORIS as usize]>> = OnceLock::new();

    TABLE.get_or_init(|| {
        let mut tbl = MoveTable([[0; NUM_CORNER_ORIS as usize]; 18]);
        let mut cc = CubieCube::SOLVED;
        for coord in CoordCube::CORNER_ORI_RANGE {
            for moov in Move::all() {
                cc.set_ori_coord::<Corners>(coord);
                cc.apply_move(moov);
                tbl[moov][coord as usize] = cc.get_ori_coord::<Corners>();
            }
        }
        tbl
    })
}

#[derive(Debug)]
struct MoveTable<T>([T; 18]);

impl<T> Index<Move> for MoveTable<T> {
    type Output = T;

    fn index(&self, index: Move) -> &Self::Output {
        &self.0[index as usize]
    }
}

impl<T> IndexMut<Move> for MoveTable<T> {
    fn index_mut(&mut self, index: Move) -> &mut Self::Output {
        &mut self.0[index as usize]
    }
}

pub(crate) fn udslice_bitmask_to_coord(bitmask: u16) -> u16 {
    debug_assert!(
        bitmask.count_ones() == 4,
        "wrong # of bits set: {bitmask:b}"
    );
    debug_assert!(bitmask & 0x0fff == bitmask, "too high bit set: {bitmask:b}");

    let bitmask = ((bitmask << 4) & 0xff0) | (bitmask >> 8);

    static TABLE: OnceLock<[u16; 0xf01]> = OnceLock::new();
    TABLE.get_or_init(|| {
        let mut ret = [0; 0xf01];
        let mut idx = 0;
        for i in 0usize..=0xf00 {
            if i.count_ones() == 4 {
                ret[i] = idx;
                idx += 1;
            }
        }
        ret
    })[bitmask as usize]
}

pub(crate) fn udslice_coord_to_bitmask(coord: u16) -> u16 {
    debug_assert!((0..495).contains(&coord));

    static TABLE: OnceLock<[u16; 496]> = OnceLock::new();
    TABLE.get_or_init(|| {
        let mut ret = [0; 496];
        let mut idx = 0;
        for i in 0u16..=0xf00 {
            if i.count_ones() == 4 {
                let i = ((i << 8) & 0xf00) | (i >> 4);
                ret[idx] = i;
                idx += 1;
            }
        }
        ret
    })[coord as usize]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_valid_ranges(c: &CoordCube) {
        assert!(
            CoordCube::CORNER_ORI_RANGE.contains(&c.corner_ori),
            "Invalid corner_ori coord: {}",
            c.corner_ori
        );
        assert!(
            CoordCube::EDGE_ORI_RANGE.contains(&c.edge_ori),
            "Invalid edge_ori coord: {}",
            c.edge_ori
        );
        assert!(
            CoordCube::UDSLICE_RANGE.contains(&c.udslice),
            "Invalid udslice coord: {}",
            c.udslice
        );
    }

    #[test]
    fn udslice_to_from_bitmask() {
        for i in 0u16..=0xf00 {
            if i.count_ones() == 4 {
                println!("{i:b}");
                assert_eq!(udslice_coord_to_bitmask(udslice_bitmask_to_coord(i)), i);
            }
        }
    }

    #[test]
    fn assert_valid_coord_ranges_for_random_cubie_cubes() {
        for _ in 0..1000 {
            let cubie_cube = CubieCube::random_possible();
            let coord_cube = CoordCube::from_cubie_cube(&cubie_cube);
            assert_valid_ranges(&coord_cube);
        }
    }
}
