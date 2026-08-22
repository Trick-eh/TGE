use engine_math::{Camera2D, Transform2D, Vec2};
use glow::HasContext;
use glutin::surface::GlSurface;

use crate::{
    FontHandle, Renderer, SpriteSheetHandle, TextureHandle, UvRegion,
    batch::{BatchMode, MAX_SPRITES},
    colors::WHITE,
    font_atlas::measure_text,
    opengl::{OpenGLRenderer, font::FontAtlas, texture::Texture},
    sprite_sheet::SpriteSheet,
};

impl Renderer for OpenGLRenderer {
    fn begin_frame(&mut self) {
        let (r, g, b, a) = (
            self.clear_color[0],
            self.clear_color[1],
            self.clear_color[2],
            self.clear_color[3],
        );
        unsafe {
            self.gl.clear_color(r, g, b, a);
            self.gl.clear(glow::COLOR_BUFFER_BIT);
        }
    }
    fn present(&mut self) {
        self.gl_surface.swap_buffers(&self.gl_context).unwrap();
    }
    fn end_frame(&mut self) {
        if self.batch.sprite_count > 0 {
            self.flush_batch();
        }
        unsafe {
            self.gl.bind_vertex_array(None);
        }
    }
    fn resize(&mut self, width: u32, height: u32) {
        self.viewport_size = (width as f32, height as f32);
        unsafe {
            self.gl.viewport(0, 0, width as i32, height as i32);
        }
        self.gl_surface.resize(
            &self.gl_context,
            std::num::NonZeroU32::new(width.max(1)).unwrap(),
            std::num::NonZeroU32::new(height.max(1)).unwrap(),
        );
    }
    fn request_redraw(&self) {
        self.window.request_redraw();
    }

    fn draw_sprite(
        &mut self,
        transform: &Transform2D,
        spritesheet: SpriteSheetHandle,
        tile_index: u32,
    ) {
        let sheet = self.sprite_sheets.get(spritesheet.0 as usize).unwrap();
        let uv_region = sheet.uv_for_tile(tile_index);
        let target_mode = BatchMode::Textured(sheet.texture);

        if self.batch.sprite_count > 0 && self.batch.mode != target_mode {
            self.flush_batch();
        }
        self.batch.mode = target_mode;

        if self.batch.sprite_count >= MAX_SPRITES {
            self.flush_batch();
            self.batch.mode = target_mode;
        }

        self.batch.push_sprite(transform, &uv_region, WHITE);
    }
    fn draw_colored_rect(&mut self, transform: &Transform2D, r: f32, g: f32, b: f32, a: f32) {
        let target_mode = BatchMode::Untextured;

        if self.batch.sprite_count > 0 && self.batch.mode != target_mode {
            self.flush_batch();
        }
        self.batch.mode = target_mode;

        if self.batch.sprite_count >= MAX_SPRITES {
            self.flush_batch();
            self.batch.mode = target_mode;
        }

        let uv = UvRegion {
            min: Vec2::ZERO,
            max: Vec2::ONE,
        };
        self.batch.push_sprite(transform, &uv, [r, g, b, a]);
    }

    fn load_texture(&mut self, bytes: &[u8]) -> crate::TextureHandle {
        let texture = {
            let img = image::load_from_memory(bytes).unwrap().into_rgba8();
            Texture::from_rgba8(&self.gl, img.width(), img.height(), img.as_raw())
        };

        let index = self.textures.len();
        self.textures.push(texture);

        TextureHandle(index as u32)
    }

    fn create_sprite_sheet(
        &mut self,
        texture: crate::TextureHandle,
        tile_width: u32,
        tile_height: u32,
    ) -> SpriteSheetHandle {
        let texture_index = texture.0 as usize;
        let texture_width = self.textures.get(texture_index).unwrap().width;
        let texture_height = self.textures.get(texture_index).unwrap().height;
        let sprite_sheet = SpriteSheet {
            texture,
            tile_width,
            tile_height,
            texture_width,
            texture_height,
        };
        let index = self.sprite_sheets.len();
        self.sprite_sheets.push(sprite_sheet);

        SpriteSheetHandle(index as u32)
    }

    fn uv_for_tile(&self, sheet: SpriteSheetHandle, index: u32) -> UvRegion {
        let sprite_sheet = self.sprite_sheets.get(sheet.0 as usize).unwrap();

        sprite_sheet.uv_for_tile(index)
    }

    fn set_camera(&mut self, camera: &Camera2D) {
        let (w, h) = self.viewport_size;
        self.cached_projection = camera.projection_matrix(w, h);
    }

    fn clear_assets(&mut self) {
        for texture in &self.textures {
            unsafe {
                self.gl.delete_texture(texture.handle);
            }
        }
        for font in &self.fonts {
            unsafe {
                self.gl.delete_texture(font.texture.handle);
            }
        }
        self.textures.clear();
        self.sprite_sheets.clear();
        self.fonts.clear();
    }

    fn load_font(&mut self, bytes: &[u8], size: f32) -> crate::FontHandle {
        let atlas = FontAtlas::build(&self.gl, bytes, size);
        let index = self.fonts.len();
        self.fonts.push(atlas);

        FontHandle(index as u32)
    }

    fn draw_text(&mut self, text: &str, font: FontHandle, position: Vec2, color: [f32; 4]) {
        let target_mode = BatchMode::Text(font);

        if self.batch.sprite_count > 0 && self.batch.mode != target_mode {
            self.flush_batch();
        }
        self.batch.mode = target_mode;

        let mut cursor_x = position.x;

        for c in text.chars() {
            let Some(glyph) = self.fonts[font.0 as usize].glyphs.get(&c).copied() else {
                continue;
            };

            if glyph.width > 0.0 && glyph.height > 0.0 {
                if self.batch.sprite_count >= MAX_SPRITES {
                    self.flush_batch();
                    self.batch.mode = target_mode;
                }

                let x = cursor_x + glyph.offset_x + glyph.width / 2.0;
                let y = position.y + glyph.offset_y + glyph.height / 2.0;

                let transform = Transform2D {
                    position: Vec2::new(x, y),
                    rotation_in_radians: 0.0,
                    scale: Vec2::new(glyph.width, glyph.height),
                };
                let uv = UvRegion {
                    min: glyph.uv_min,
                    max: glyph.uv_max,
                };

                self.batch.push_sprite(&transform, &uv, color);
            }

            cursor_x += glyph.advance;
        }
    }

    fn measure_text(&self, text: &str, font: FontHandle) -> Vec2 {
        match self.fonts.get(font.0 as usize) {
            Some(atlas) => measure_text(&atlas.glyphs, text),
            None => Vec2::ZERO,
        }
    }

    fn set_clear_color(&mut self, color: [f32; 4]) {
        self.clear_color = color;
    }
}
