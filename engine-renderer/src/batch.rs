use engine_math::{Transform2D, Vec2};

use crate::{FontHandle, TextureHandle, UvRegion};

pub const MAX_SPRITES: usize = 1000;
pub const VERTEX_SIZE: usize = 8; // x, y, u, v, r, g, b, a
pub const FLOATS_PER_SPRITE: usize = 4 * VERTEX_SIZE;
pub const INDICES_PER_SPRITE: usize = 6;

#[derive(Clone, Copy, PartialEq)]
pub enum BatchMode {
    Textured(TextureHandle),
    Text(FontHandle),
    Untextured,
    Empty,
}

pub struct SpriteBatch {
    pub vertices: Vec<f32>,
    pub indices: Vec<u32>,
    pub sprite_count: usize,
    pub mode: BatchMode,
}

impl SpriteBatch {
    pub fn new() -> SpriteBatch {
        SpriteBatch {
            vertices: Vec::with_capacity(MAX_SPRITES * FLOATS_PER_SPRITE),
            indices: Vec::with_capacity(MAX_SPRITES * INDICES_PER_SPRITE),
            sprite_count: 0,
            mode: BatchMode::Empty,
        }
    }
    pub fn clear(&mut self) {
        self.vertices.clear();
        self.indices.clear();
        self.sprite_count = 0;
        self.mode = BatchMode::Empty;
    }
    pub fn push_sprite(&mut self, transform: &Transform2D, uv: &UvRegion, color: [f32; 4]) {
        let half_w = transform.scale.x / 2.0;
        let half_h = transform.scale.y / 2.0;

        let corners = [
            Vec2::new(-half_w, -half_h),
            Vec2::new(half_w, -half_h),
            Vec2::new(half_w, half_h),
            Vec2::new(-half_w, half_h),
        ];

        let uvs = [
            Vec2::new(uv.min.x, uv.min.y),
            Vec2::new(uv.max.x, uv.min.y),
            Vec2::new(uv.max.x, uv.max.y),
            Vec2::new(uv.min.x, uv.max.y),
        ];

        let cos = transform.rotation_in_radians.cos();
        let sin = transform.rotation_in_radians.sin();

        for (corner, uv) in corners.iter().zip(uvs.iter()) {
            let rotated = Vec2::new(
                corner.x * cos - corner.y * sin,
                corner.x * sin + corner.y * cos,
            );

            let world = rotated + transform.position;

            self.vertices.extend_from_slice(&[
                world.x, world.y, uv.x, uv.y, color[0], color[1], color[2], color[3],
            ]);
        }

        let base = (self.sprite_count * 4) as u32;
        self.indices
            .extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
        self.sprite_count += 1;
    }
}
