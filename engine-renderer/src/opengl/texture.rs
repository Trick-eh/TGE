use std::path::Path;

use glow::HasContext;

pub struct Texture {
    pub handle: glow::Texture,
    width: u32,
    height: u32,
    // set the scaling config
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
}
