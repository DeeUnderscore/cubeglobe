extern crate cubeglobe;

use cubeglobe::renderer::Renderer;
use cubeglobe::map::generator::{Generator, TerGenTwo};


pub fn main() {
    let config_toml = include_str!("../assets/full-tiles.toml");

    let rconfig = Renderer::from_str(config_toml).unwrap();

    let iso_map = TerGenTwo::new()
        .set_len(32)
        .set_frequency(0.01)
        .set_layer_height(7)
        .set_max_water_level(15)
        .set_min_soil_cutoff(30)
        .generate();

    rconfig.render_map(&iso_map).unwrap().save_bmp("out.bmp").unwrap();
}