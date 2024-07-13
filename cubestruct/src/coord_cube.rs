use crate::cubie::*;
use crate::CubieCube;
use std::ops::Index;
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
    fn from_cubie_cube(cubie_cube: &CubieCube) -> Self {
        fn get_ori_coord<C: Cubies>(cubie_cube: &CubieCube) -> u16
        where
            CubieCube: Index<C::Cubicle, Output = C::Cubie>,
        {
            let base = C::Orientation::all().count() as u16;
            let mut sum = 0;
            let max_cubicle = C::Cubicle::all().rev().next().unwrap();
            for home_cubicle in C::Cubicle::all() {
                let state = cubie_cube[home_cubicle];
                if state.cubicle() == max_cubicle {
                    continue;
                }
                sum +=
                    base.pow(state.cubicle().as_u8().into()) * state.orientation().as_u8() as u16;
            }
            sum
        }

        let udslice_cubicles = [
            EdgeCubicle::C4,
            EdgeCubicle::C5,
            EdgeCubicle::C6,
            EdgeCubicle::C7,
        ];
        let mut mask = 0;
        for home_cubicle in udslice_cubicles {
            let loc = cubie_cube[home_cubicle].cubicle();
            mask |= 1 << loc.as_u8();
        }

        Self {
            corner_ori: get_ori_coord::<Corners>(cubie_cube),
            edge_ori: get_ori_coord::<Edges>(cubie_cube),
            udslice: udslice_bitmask_to_coord(mask),
        }
    }
}

fn udslice_bitmask_to_coord(bitmask: u16) -> u16 {
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

fn udslice_coord_to_bitmask(coord: u16) -> u16 {
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
            (0..2187).contains(&c.corner_ori),
            "Invalid corner_ori coord: {}",
            c.corner_ori
        );
        assert!(
            (0..2048).contains(&c.edge_ori),
            "Invalid edge_ori coord: {}",
            c.edge_ori
        );
        assert!(
            (0..495).contains(&c.udslice),
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
