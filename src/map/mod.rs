//! Things related to representing a map

pub mod generator;

use ndarray::{Array3, Axis};

/// A single block of a certain type
#[derive(Copy, IntoEnumIterator, Clone, PartialEq, Eq, Debug, Deserialize, Hash)]
pub enum Block {
    Air,
    Rock,
    Grass,
    Soil,
    Water,
}

impl Default for Block {
    fn default() -> Block {
        Block::Air
    }
}

/// Struct representing a three dimensional map of blocks
///
/// Order is (x,y,z), z+ is up. Although it's called `IsoMap`, there is nothing
/// inherently isometric about it, other than the fact it's intended to be
/// rendered in an isometric perspective.
#[derive(Clone)]
pub struct IsoMap(pub Array3<Block>);

#[cfg_attr(feature = "cargo-clippy", allow(len_without_is_empty))]
impl IsoMap {
    /// Create a new cube-shaped IsoMap, with `len` tiles in every direction,
    /// filled with [`Block::Air`](enum.Block.html#variant.Air).
    pub fn new_empty(len: usize) -> IsoMap {
        IsoMap(Array3::default((len, len, len)))
    }

    /// Get the length of the map
    ///
    /// The map is a cube, every edge is the same length. This function returns
    /// the edge length.
    pub fn len(&self) -> usize {
        self.0.len_of(Axis(0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ndarray::arr3;

    #[test]
    fn all_air() {
        let iso_map = IsoMap::new_empty(2);

        assert_eq!(
            iso_map.0,
            arr3(&[
                [[Block::Air, Block::Air], [Block::Air, Block::Air]],
                [[Block::Air, Block::Air], [Block::Air, Block::Air]]
            ])
        )
    }

    #[test]
    fn return_len() {
        let iso_map = IsoMap::new_empty(50);
        
        assert_eq!(iso_map.len(), 50)
    }
}
