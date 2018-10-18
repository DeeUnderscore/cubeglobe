//! A terrain generator for making generic landscapes

use std::clone::Clone;

use noise::{Fbm, MultiFractal, NoiseFn, Seedable};
use rand::random;

use map::generator::Generator;
use map::{Block, IsoMap};

/// A terrain generator which uses Perlin noise for heightmap generation.
///
/// `TerGenOne` is relatively simple, and will only fill the landscape with
/// `Rock` blocks.
///
/// ## Example use
/// ```
/// use cubeglobe::map::generator::{TerGenOne, Generator};
///
/// let gen = TerGenOne::new().set_len(32);
/// let iso_map = gen.generate();
/// ```
#[derive(Debug, Default)]
pub struct TerGenOne {
    /// Dimensions of the map
    len: usize,
    frequency: f64,
}

impl TerGenOne {
    /// Default dimension for a map
    const DEFAULT_LEN: usize = 64;

    /// Default frequency parameter for the noise generator
    const DEFAULT_FREQUENCY: f64 = 0.05;

    /// Set the edge length
    pub fn set_len(self, len: usize) -> TerGenOne {
        // level 0 should end up bewtween 40% and 60%
        TerGenOne { len, ..self }
    }

    /// Set the frequency parameter for the noise generator
    ///
    /// Values of 0.05 and below are recommended. At 0.001, terrain will be
    /// mostly gentle slopes; at 0.005, there will be significant hills; at
    /// 0.05, the terrain will feature a lot of mountain peaks.
    pub fn set_frequency(self, freq: f64) -> TerGenOne {
        TerGenOne {
            frequency: freq,
            ..self
        }
    }

    /// Get a new terrain generator with all default settings
    pub fn new() -> TerGenOne {
        TerGenOne {
            len: Self::DEFAULT_LEN,
            frequency: Self::DEFAULT_FREQUENCY,
        }
    }


    /// Generate a map, creating a snapshot each time one slice in the x-axis is
    /// added.
    ///
    /// The snapshots can be fed to the renderer to generate a series of images
    /// which show even those blocks that are obscured in the final render. This
    /// can be useful for testing or diagnostics.
    pub fn generate_slices(&self) -> Vec<IsoMap> {
        let noise = Fbm::new().set_seed(random()).set_frequency(self.frequency);
        let mut isomap = IsoMap::new_empty(self.len);
        let half_height: f64 = self.len as f64 / 2.0;
        let mut maps: Vec<IsoMap> = Vec::new();

        for x in 0..isomap.len() {
            for y in 0..isomap.len() {
                let height =
                    (half_height + ((noise.get([x as f64, y as f64])) * half_height)) as usize;

                let mut column = isomap.0.slice_mut(s![x, y, 0..height]);

                column.fill(Block::Rock);
            }

            maps.push(isomap.clone());
        }

        maps
    }
}

impl Generator for TerGenOne {
    fn generate(&self) -> IsoMap {
        let noise = Fbm::new().set_seed(random()).set_frequency(self.frequency);
        let mut isomap = IsoMap::new_empty(self.len);
        let half_height: f64 = self.len as f64 / 2.0;

        for x in 0..isomap.len() {
            for y in 0..isomap.len() {
                let height =
                    (half_height + ((noise.get([x as f64, y as f64])) * half_height)) as usize;

                let mut column = isomap.0.slice_mut(s![x, y, 0..height]);

                column.fill(Block::Rock);
            }
        }

        isomap
    }
}
