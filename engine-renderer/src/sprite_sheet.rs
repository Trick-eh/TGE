use engine_math::Vec2;

use crate::{TextureHandle, UvRegion};

pub struct SpriteSheet {
    pub texture: TextureHandle,
    pub tile_width: u32,
    pub tile_height: u32,
    pub texture_width: u32,
    pub texture_height: u32,
}

impl SpriteSheet {
    pub fn uv_for_tile(&self, index: u32) -> UvRegion {
        let columns = self.texture_width / self.tile_width;
        let rows = self.texture_height / self.tile_height;
        let column = index % columns;
        let row = index / columns;
        let flipped_row = (rows - 1) - row;

        let min = Vec2 {
            x: (column * self.tile_width) as f32 / (self.texture_width as f32),
            y: (flipped_row * self.tile_height) as f32 / (self.texture_height as f32),
        };
        let max = Vec2 {
            x: ((column + 1) * self.tile_width) as f32 / (self.texture_width as f32),
            y: ((flipped_row + 1) * self.tile_height) as f32 / (self.texture_height as f32),
        };

        UvRegion { min, max }
    }
}
