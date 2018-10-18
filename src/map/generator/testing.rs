//! Testing map generator that produces a simple map
//!
//! This is not a unit/integration test, but a simple generator, which can be
//! used in tests.

use map::generator::Generator;
use map::{Block, IsoMap};

/// a simple generator that produces a mostly flat map.
///
/// The minimum size is 6. `dim` lower than that will peg the size to 6.
#[derive(Debug)]
pub struct TestingGenerator {
    /// Map dimension to use
    pub dim: usize,
}

impl Generator for TestingGenerator {
    fn generate(&self) -> IsoMap {
        let dim = if self.dim >= 6 { self.dim } else { 6 };

        let mut new_map = IsoMap::new_empty(dim);
        let halfway = (dim as f64 / 2.0).floor() as usize;

        // Fill a slice of half of the map cube, up to halfway in the z axis
        new_map
            .0
            .slice_mut(s![.., .., 0..halfway])
            .fill(Block::Rock);

        // Fill the level immediately above the already filled part with a
        // smaller square
        new_map
            .0
            .slice_mut(s![2..(dim - 2), 2..(dim - 2), halfway ])
            .fill(Block::Rock);
        new_map
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ndarray::{arr2, Axis};

    #[test]
    fn peg_to_six() {
        let map = TestingGenerator { dim: 1 }.generate();

        assert_eq!(map.0.shape(), &[6, 6, 6])
    }

    #[test]
    fn gen_map() {
        let map = TestingGenerator { dim: 1 }.generate();

        
        let r = Block::Rock;
        let a = Block::Air;
        // Check that the plane at the halfway point is all filled
        assert_eq!(
            map.0.subview(Axis(2), 2),
            arr2(&[ [r, r, r, r, r, r],
                    [r, r, r, r, r, r],
                    [r, r, r, r, r, r],
                    [r, r, r, r, r, r],
                    [r, r, r, r, r, r],
                    [r, r, r, r, r, r] ])
        );

        // Check that the plane immeidately above the halfway point is there
        assert_eq!(
            map.0.subview(Axis(2), 3),
            arr2(&[ [a, a, a, a, a, a],
                    [a, a, a, a, a, a],
                    [a, a, r, r, a, a],
                    [a, a, r, r, a, a],
                    [a, a, a, a, a, a],
                    [a, a, a, a, a, a] ])
        );
    }
}
