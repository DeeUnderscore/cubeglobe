//! Renderer for rendering `IsoMap`s to image
//!
//! The renderer loads a configuration from a TOML file, and then uses that
//! configuration to render a provided `IsoMap`
//!
//! ## Tiles
//! Tiles are assumed to be a 2:1 isometric projection. They can represent a
//! cube or a cuboid, in which case the top face should adjut against the top of
//! the tile and take up the whole width – the rest of the cube will be below.
//!
//! Multiple tiles can be present in one file, forming a spritesheet. The TOML
//! can specify multiple files to load.
//!
//! ## TOML configuration file
//! The configuration file specifies what tiles are in what sprite sheets. For
//! example:
//!
//! ```TOML
//! # Width and height of an individual tile in pixels
//! width = 24
//! height = 24
//!
//! [[files]]
//! filename = "cubes.png"
//!
//!     [[files.tiles]]
//!     kind = "Rock"
//!
//!     # Offsets are optional, and assumed to be 0,0 if not specified. 0,0 is upper left
//!
//!     [[files.tiles]]
//!     kind = "Rock"
//!     x = 25
//!     y = 0
//! ```
//!

use std::collections::HashMap;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::path::PathBuf;
use std::rc::Rc;

use enum_iterator::IntoEnumIterator;
use ndarray::Axis;
use rand::Rng;
use sdl2::image::LoadSurface;
use sdl2::pixels::{PixelFormatEnum, Color};
use sdl2::rect::{Point, Rect};
pub use sdl2::surface::Surface;
pub use sdl2::rwops::RWops;
use toml;

use map::{Block, IsoMap};

mod errors;
pub use renderer::errors::*;

macro_rules! DEFAULT_BACKGROUND_COLOR{
    () => ( Color::RGB(154, 216, 224) )

}

/// Deserialized tiles.toml
#[derive(Deserialize)]
struct TilesConfig {
    width: u32,
    height: u32,
    files: Vec<File>,
    base_path: String,
}

#[derive(Deserialize)]
struct File {
    filename: String,
    tiles: Vec<TileDef>,
}

#[derive(Deserialize)]
struct TileDef {
    kind: Block,
    x: Option<i32>,
    y: Option<i32>,
}

/// A single tile, to be used in rendering the map
///
/// Under the assumption that copying tiles out of spritesheets and into
/// individual `Surface`s would impose an additional penalty on config load, we
/// instead keep track of the sheet and the rectangle with the tile, and copy
/// out of the sheet at the time we render
struct Tile<'a> {
    sheet: Rc<Surface<'a>>,
    pos: Rect,
}

/// Config used by the renderer to pick tiles
pub struct Renderer<'a> {
    /// width of a tile
    width: u32,

    /// height of a tile
    height: u32,

    /// A hashmap of block type to possible tiles to use for that block.
    ///
    /// Each block type can have multiple tiles. The tile to use is picked at
    /// random every time a block is rendered.
    tiles: HashMap<Block, Vec<Tile<'a>>>,
}

impl<'a> Renderer<'a> {
    /// Create new RendererConfig from a TOML configuration provided in `input`
    ///
    /// `s` is the string with the config TOML.
    pub fn from_config_str(s: &str) -> Result<Self, ConfigLoadError> {
        use failure::ResultExt;

        let parsed: TilesConfig = toml::from_str(s).context(ConfigLoadErrorKind::TomlParseError)?;
        let tile_width = parsed.width;
        let tile_height = parsed.height;
        let base_dir = parsed.base_path;

        let files_with_tiles: Vec<Vec<(Block, Tile)>> = parsed
            .files
            .into_iter()
            .map(|file| -> Result<Vec<(Block, Tile)>, _> {
                let mut filepath = PathBuf::new();
                filepath.push(&base_dir);
                filepath.push(&file.filename);

                // load each file to a surface...
                let surf = Rc::new(
                    Surface::from_file(filepath)
                        .map_err(ConfigLoadErrorKind::from_sdl_string_err)?,
                );

                // ...and then refer to that surface in Tile instances, along
                // with the offsets
                Ok(file
                    .tiles
                    .into_iter()
                    .map(|tiledef| -> (Block, Tile) {
                        let x = tiledef.x.unwrap_or(0);
                        let y = tiledef.y.unwrap_or(0);

                        (
                            tiledef.kind,
                            Tile {
                                sheet: Rc::clone(&surf),
                                pos: Rect::new(x, y, tile_width, tile_height),
                            },
                        )
                    }).collect::<Vec<(Block, Tile)>>())
            }).collect::<Result<Vec<Vec<(Block, Tile)>>, ConfigLoadError>>()?;

        let mut tiles_map: HashMap<Block, Vec<Tile>> = HashMap::new();

        // Each file is a vector of tiles, so we flatten out all the files here
        for (block, tile) in files_with_tiles.into_iter().flatten() {
            tiles_map
                .entry(block)
                .or_insert_with(Vec::new)
                .push(tile)
        }

        // Ensure each block has at least one tile
        for block in Block::into_enum_iter() {
            if block == Block::Air {
                continue; // We special-case air since it doesn't need tiles
            }

            if !tiles_map.contains_key(&block) {
                return Err(ConfigLoadError::from(ConfigLoadErrorKind::MissingBlock(
                    block,
                )));
            }
        }

        Ok(Renderer {
            width: tile_width,
            height: tile_height,
            tiles: tiles_map,
        })
    }

    /// Render an `IsoMap` using a `Renderer`
    pub fn render_map<'b>(&self, isomap: &IsoMap) -> Result<Surface<'b>, RendererError> {
        
        // Pixel height of the top face of the cube. Since we're in a 2:1
        // projection, it's half the tile's width. 
        let top_height: u32 = self.width/2;

        // Pixel height of the sides of the cube. This is the remainder of the
        // pixel height of a tile, after we account for the top face.
        let sides_height: u32 = self.height - top_height;

        // How much a single floor takes up in pixels, in the vertical. The longest part vertically
        // is the diagnoal. If we walk up or down the diagonal, we'll move by one full top_height
        // for each tile. Then, we'll also be able to see the frontmost tile's sides, so we add
        // sides_height.
        let floor_height: u32 = (isomap.len() as u32 * top_height) + sides_height; 

        // We make the surface wide enough to take the width of a floor and then
        // add a margin
        let surf_width: u32 = (self.width * isomap.len() as u32) + (self.width * 2);

        // We need enough room for a single floor, then every floor stack on top
        // of it, then some margins
        let surf_height: u32 = floor_height + (sides_height * isomap.len() as u32) + (self.height * 2);

        let mut out = Surface::new(surf_width, surf_height, PixelFormatEnum::RGB24)?;
        out.fill_rect(None, DEFAULT_BACKGROUND_COLOR!())?;

        // In the x axis, we find the midpoint, and then shift a bit to the
        // left, so that half of the tile is to the left of the midpoint, and
        // the other half to the right.
        //
        // In the y axis, we start from the bottom, go up to account for the
        // margin, and then go up to account for the floor height.
        let mut current_origin = Point::new(
            (surf_width / 2 - self.width / 2) as i32,
            surf_height as i32 - self.height as i32 - floor_height as i32, 
        );

        for floor in isomap.0.axis_iter(Axis(2)) {
            for ((x, y), tile) in floor.indexed_iter() {
                if tile == &Block::Air {
                    continue; // blank, do nothing
                }

                let tile_dest = self.get_tile_pos(current_origin, x, y);
                let tile_sprite = self.get_random_sprite(tile);

                tile_sprite.sheet.clone().blit(
                    tile_sprite.pos,
                    &mut out,
                    Rect::new(tile_dest.x, tile_dest.y, self.width, self.height),
                )?;
            }

            // Shift to the floor above
            current_origin = current_origin.offset(0, -(sides_height as i32));
        }

        Ok(out)
    }

    /// Get pixel position for a tile at map position `x_index`, `y_index`, assuming tile 0,0 is at `origin`.
    fn get_tile_pos(&self, origin: Point, x_index: usize, y_index: usize) -> Point {
        // Tile tops (the top surfaces of the cube) are assumed to be at a 2:1
        // ratio, twice as wide as they are tall.
        origin.offset(
            (x_index as i32 - y_index as i32) * (self.width as i32 / 2),
            (x_index as i32 + y_index as i32) * (self.width as i32 / 4),
        )
    }

    fn get_random_sprite(&self, tile_type: &Block) -> &Tile {
        // We unwrap here because from_str should never leave us in a state
        // where some tiles are missing
        let potential_tiles = self
            .tiles
            .get(&tile_type)
            .expect("renderer config missing tiles for a block type");

        rand::thread_rng()
            .choose(potential_tiles)
            .expect("renderer config has an empty vector for a block type")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn load_config() {
        let config_toml = include_str!("../../assets/test-tiles.toml");

        // TODO: Using cwd here is probably a bit flaky
        let mut assets_dir = env::current_dir().unwrap();
        assets_dir.push("assets/");

        let rconfig = Renderer::from_config_str(config_toml).unwrap();

        assert_eq!(rconfig.width, 24);
        match rconfig.tiles.get(&Block::Rock) {
            Some(t) => assert_eq!(t[0].pos, Rect::new(0, 0, 24, 26)),
            None => panic!(),
        }
    }
}
