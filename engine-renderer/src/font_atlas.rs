use std::collections::HashMap;

use engine_math::Vec2;

#[derive(Clone, Copy)]
pub struct GlyphInfo {
    pub uv_min: Vec2,
    pub uv_max: Vec2,
    pub width: f32,
    pub height: f32,
    pub advance: f32,
    pub offset_x: f32,
    pub offset_y: f32,
}

pub struct PackedAtlas {
    pub pixels: Vec<u8>,
    pub atlas_size: u32,
    pub glyphs: HashMap<char, GlyphInfo>,
}

pub fn pack_atlas(font_data: &[u8], size: f32) -> PackedAtlas {
    let font = fontdue::Font::from_bytes(font_data, fontdue::FontSettings::default())
        .expect("Failed to load font");

    let atlas_size = 512u32;
    let mut atlas_pixels = vec![0u8; (atlas_size * atlas_size) as usize];
    let mut glyphs = HashMap::new();

    let mut cursor_x = 0u32;
    let mut cursor_y = 0u32;
    let mut row_height = 0u32;
    let padding = 1u32;

    for c in ' '..='~' {
        let (metrics, bitmap) = font.rasterize(c, size);

        if metrics.width == 0 || metrics.height == 0 {
            glyphs.insert(
                c,
                GlyphInfo {
                    uv_min: Vec2::ZERO,
                    uv_max: Vec2::ZERO,
                    width: 0.0,
                    height: 0.0,
                    advance: metrics.advance_width,
                    offset_x: 0.0,
                    offset_y: 0.0,
                },
            );
            continue;
        }

        if cursor_x + metrics.width as u32 + padding > atlas_size {
            cursor_x = 0;
            cursor_y += row_height + padding;
            row_height = 0;
        }

        for y in 0..metrics.height {
            for x in 0..metrics.width {
                let atlas_idx = (cursor_y + y as u32) * atlas_size + (cursor_x + x as u32);
                atlas_pixels[atlas_idx as usize] = bitmap[y * metrics.width + x];
            }
        }

        let uv_min = Vec2::new(
            cursor_x as f32 / atlas_size as f32,
            (cursor_y + metrics.height as u32) as f32 / atlas_size as f32,
        );
        let uv_max = Vec2::new(
            (cursor_x + metrics.width as u32) as f32 / atlas_size as f32,
            cursor_y as f32 / atlas_size as f32,
        );

        glyphs.insert(
            c,
            GlyphInfo {
                uv_min,
                uv_max,
                width: metrics.width as f32,
                height: metrics.height as f32,
                advance: metrics.advance_width,
                offset_x: metrics.xmin as f32,
                offset_y: metrics.ymin as f32,
            },
        );

        cursor_x += metrics.width as u32 + padding;
        row_height = row_height.max(metrics.height as u32);
    }

    let mut flipped_pixels = vec![0u8; (atlas_size * atlas_size) as usize];
    for y in 0..atlas_size {
        let src_row = y * atlas_size;
        let dst_row = (atlas_size - 1 - y) * atlas_size;
        flipped_pixels[dst_row as usize..(dst_row + atlas_size) as usize]
            .copy_from_slice(&atlas_pixels[src_row as usize..(src_row + atlas_size) as usize]);
    }

    PackedAtlas {
        pixels: flipped_pixels,
        atlas_size,
        glyphs,
    }
}

pub fn measure_text(glyphs: &HashMap<char, GlyphInfo>, text: &str) -> Vec2 {
    let mut width = 0.0f32;
    let mut max_height = 0.0f32;

    for c in text.chars() {
        if let Some(glyph) = glyphs.get(&c) {
            width += glyph.advance;
            max_height = max_height.max(glyph.height);
        }
    }
    Vec2::new(width, max_height)
}
