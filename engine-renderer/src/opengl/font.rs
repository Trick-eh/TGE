use std::collections::HashMap;

use crate::{
    font_atlas::{GlyphInfo, pack_atlas},
    opengl::texture::Texture,
};
use glow::HasContext;

pub struct FontAtlas {
    pub texture: Texture,
    pub glyphs: HashMap<char, GlyphInfo>,
}

impl FontAtlas {
    pub fn build(gl: &glow::Context, font_data: &[u8], size: f32) -> Self {
        let packed = pack_atlas(font_data, size);
        let texture = unsafe { Self::upload_atlas(gl, &packed.pixels, packed.atlas_size) };

        FontAtlas {
            texture,
            glyphs: packed.glyphs,
        }
    }

    unsafe fn upload_atlas(gl: &glow::Context, pixels: &[u8], size: u32) -> Texture {
        unsafe {
            let handle = gl.create_texture().unwrap();
            gl.bind_texture(glow::TEXTURE_2D, Some(handle));

            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MIN_FILTER,
                glow::LINEAR as i32,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MAG_FILTER,
                glow::LINEAR as i32,
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

            gl.pixel_store_i32(glow::UNPACK_ALIGNMENT, 1);

            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::R8 as i32,
                size as i32,
                size as i32,
                0,
                glow::RED,
                glow::UNSIGNED_BYTE,
                glow::PixelUnpackData::Slice(Some(pixels)),
            );

            gl.pixel_store_i32(glow::UNPACK_ALIGNMENT, 4);

            Texture {
                handle,
                width: size,
                height: size,
            }
        }
    }
}
