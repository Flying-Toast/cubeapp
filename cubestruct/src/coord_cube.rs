use crate::cubie::*;
use crate::CubieCube;
use std::ops::Range;
use std::sync::OnceLock;

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
    pub(crate) const CORNER_ORI_RANGE: Range<u16> = 0..2187;
    pub(crate) const EDGE_ORI_RANGE: Range<u16> = 0..2048;
    pub(crate) const UDSLICE_RANGE: Range<u16> = 0..495;

    fn from_cubie_cube(cubie_cube: &CubieCube) -> Self {
        Self {
            corner_ori: cubie_cube.get_ori_coord::<Corners>(),
            edge_ori: cubie_cube.get_ori_coord::<Edges>(),
            udslice: cubie_cube.get_udslice_coord(),
        }
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
