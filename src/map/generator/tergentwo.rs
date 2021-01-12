//! A terrain generator for making generic landscapes

use noise::{Billow, Abs, Fbm, MultiFractal, NoiseFn, Seedable};
use rand::{thread_rng, Rng};

use map::generator::Generator;
use map::{Block, IsoMap};

/// A terrain generator which uses Perlin noise for heightmap generation.
///
/// Unlike `TerGenOne`, `TerGenTwo` can use all available tile types, so it will
/// try to do advanced things like layering soil on top of rock.
///
/// ## Example use
/// ```
/// use cubeglobe::map::generator::{TerGenTwo, Generator};
///
/// let gen = TerGenTwo::new().set_len(128);
/// let iso_map = gen.generate();
/// ```
#[derive(Debug, Default)]
pub struct TerGenTwo {
    len: usize,
    frequency: f64,
    layer_height: usize,
    min_soil_cutoff: usize,
    max_water_level: usize,
}

impl TerGenTwo {
    const DEFAULT_LEN: usize = 64;
    const DEFAULT_FREQUENCY: f64 = 0.05;
    const DEFAULT_LAYER_HEIGHT: usize = 15;
    const DEFAULT_MIN_SOIL_CUTOFF: usize = 45;
    const DEFAULT_MAX_WATER_LEVEL: usize = 40;

    /// Set the edge length
    pub fn set_len(self, len: usize) -> TerGenTwo {
        TerGenTwo { len, ..self }
    }

    /// Set the frequency parameter for the noise generator
    ///
    /// Values of 0.05 and below are recommended. At 0.001, terrain will be
    /// mostly gentle slopes; at 0.005, there will be significant hills; at
    /// 0.05, the terrain will feature a lot of mountain peaks.
    pub fn set_frequency(self, freq: f64) -> TerGenTwo {
        TerGenTwo {
            frequency: freq,
            ..self
        }
    }

    /// Set the layer height parameter
    ///
    /// The layer height determines how deep the soil layer will be. The actual
    /// layer height is subject to a noise function, so this is the maximum that
    /// it can actually be
    pub fn set_layer_height(self, layer_height: usize) -> TerGenTwo {
        TerGenTwo {
            layer_height,
            ..self
        }
    }

    /// Set the maximum possible water level
    ///
    /// Empty space below the water level will be filled with water. This
    /// parameter determines how high the water level can go – the actual water
    /// level is subject to randomization, up to this value.
    pub fn set_max_water_level(self, max_water_level: usize) -> TerGenTwo {
        TerGenTwo {
            max_water_level,
            ..self
        }
    }

    /// Set the minimum possible soil line
    ///
    /// The soil line determines how high the soil will go. Terrain above this
    /// level will not have soil, thus imitating bare rock of a mountain. This
    /// parameter sets the minimum possible soil level – the actual level is
    /// subject to randomization, between this value and the height of the map.
    pub fn set_min_soil_cutoff(self, min_soil_cutoff: usize) -> TerGenTwo {
        TerGenTwo {
            min_soil_cutoff,
            ..self
        }
    }

    /// Get a new terrain generator with all default settings
    pub fn new() -> TerGenTwo {
        TerGenTwo {
            len: Self::DEFAULT_LEN,
            frequency: Self::DEFAULT_FREQUENCY,
            layer_height: Self::DEFAULT_LAYER_HEIGHT,
            min_soil_cutoff: Self::DEFAULT_MIN_SOIL_CUTOFF,
            max_water_level: Self::DEFAULT_MAX_WATER_LEVEL,
        }
    }
}

impl Generator for TerGenTwo {
    fn generate(&self) -> IsoMap {
        let mut rng = thread_rng();

        let height_noise = Fbm::new().set_seed(rng.gen()).set_frequency(self.frequency);
        let billow = Billow::new()
            .set_seed(rng.gen())
            .set_frequency(self.frequency);

        // Billow returns negative values 
        let layer_noise = Abs::new(&billow);

        let water_level: usize = rng.gen_range(0, self.max_water_level + 1);
        let soil_level: usize = rng.gen_range(self.min_soil_cutoff, self.len);

        let mut isomap = IsoMap::new_empty(self.len);
        let half_height: f64 = self.len as f64 / 2.0;

        for x in 0..isomap.len() {
            for y in 0..isomap.len() {
                let height = (half_height
                    + ((height_noise.get([x as f64, y as f64])) * half_height))
                    as usize;

                if height < water_level {
                    // Rock, and then water up to the water level
                    isomap.0.slice_mut(s![x, y, 0..height-1]).fill(Block::Rock);
                    isomap
                        .0
                        .slice_mut(s![x, y, height-1..water_level-1])
                        .fill(Block::Water);
                } else if height < soil_level {
                    // Rock, and then soil, then a single block of grass
                    let soil_depth =
                        (layer_noise.get([x as f64, y as f64]) * self.layer_height as f64) as usize;

                    let rock_height: usize = height.saturating_sub(soil_depth);

                    isomap
                        .0
                        .slice_mut(s![x, y, 0..rock_height])
                        .fill(Block::Rock);

                    if rock_height < height-1 {
                        isomap
                            .0
                            .slice_mut(s![x, y, rock_height..(height - 1)])
                            .fill(Block::Soil);
                    } 
                    
                    if rock_height < height {
                        isomap.0[[x, y, height-1]] = Block::Grass;
                    }
                } else {
                    // Just rock
                    isomap.0.slice_mut(s![x, y, 0..height]).fill(Block::Rock);
                }
            }
        }

        isomap
    }
}
