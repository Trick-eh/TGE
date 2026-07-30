use std::path::Path;

use crate::{TextureHandle, UvRegion};
use engine_math::Vec2;
use glow::HasContext;

pub struct Texture {
    pub handle: glow::Texture,
    pub width: u32,
    pub height: u32,
    // set the scaling config
}

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
        let column = index % columns;
        let rows = self.texture_height / self.tile_height;
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

impl Texture {
    pub fn new(gl: &glow::Context, path: &Path) -> Texture {
        let img = image::open(path).unwrap().into_rgba8();
        let width = img.width();
        let height = img.height();
        let pixels = img.as_raw();

        let texture = unsafe { gl.create_texture().unwrap() };
        unsafe {
            gl.bind_texture(glow::TEXTURE_2D, Some(texture));
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MIN_FILTER,
                glow::NEAREST as i32,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MAG_FILTER,
                glow::NEAREST as i32,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_WRAP_S,
                glow::CLAMP_TO_EDGE as i32,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_WRAP_T,
                glow::CLAMP_TO_EDGE as i32,
            );
            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::RGBA as i32,
                width as i32,
                height as i32,
                0,
                glow::RGBA,
                glow::UNSIGNED_BYTE,
                glow::PixelUnpackData::Slice(Some(pixels)),
            );
            gl.generate_mipmap(glow::TEXTURE_2D);
            gl.bind_texture(glow::TEXTURE_2D, None);
        };

        Texture {
            handle: texture,
            width,
            height,
        }
    }
    pub fn from_rgba8(gl: &glow::Context, width: u32, height: u32, pixels: &[u8]) -> Texture {
        let texture = unsafe { gl.create_texture().unwrap() };
        unsafe {
            gl.bind_texture(glow::TEXTURE_2D, Some(texture));
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MIN_FILTER,
                glow::NEAREST as i32,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MAG_FILTER,
                glow::NEAREST as i32,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_WRAP_S,
                glow::CLAMP_TO_EDGE as i32,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_WRAP_T,
                glow::CLAMP_TO_EDGE as i32,
            );
            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::RGBA as i32,
                width as i32,
                height as i32,
                0,
                glow::RGBA,
                glow::UNSIGNED_BYTE,
                glow::PixelUnpackData::Slice(Some(pixels)),
            );
            gl.generate_mipmap(glow::TEXTURE_2D);
            gl.bind_texture(glow::TEXTURE_2D, None);
        };

        Texture {
            handle: texture,
            width,
            height,
        }
    }
}
