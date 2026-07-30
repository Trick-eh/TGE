use engine_math::{Camera2D, Transform2D, Vec2};

pub mod opengl;

#[cfg(all(feature = "opengl", feature = "vulkan"))]
compile_error!(
    "Features 'opengl' and 'vulkan' are mutually exclusive. 
Enable only one at a time."
);

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct TextureHandle(u32);

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct SpriteSheetHandle(u32);

pub struct Sprite {
    pub sprite_sheet: SpriteSheetHandle,
    pub index: u32,
}

pub struct AnimatedSprite {
    pub sprite_sheet: SpriteSheetHandle,
    pub frames: Vec<u32>,
    pub frame_duration: f32,
    pub current_frame: usize,
    pub timer: f32,
    pub looping: bool,
}

pub trait Renderer {
    fn begin_frame(&mut self);
    fn end_frame(&mut self);
    fn resize(&mut self, width: u32, height: u32);

    fn load_texture(&mut self, bytes: &[u8]) -> TextureHandle;
    fn create_sprite_sheet(
        &mut self,
        texture: TextureHandle,
        tile_width: u32,
        tile_height: u32,
    ) -> SpriteSheetHandle;
    fn uv_for_tile(&self, sheet: SpriteSheetHandle, index: u32) -> UvRegion;

    fn draw_sprite(
        &mut self,
        transform: &Transform2D,
        spritesheet: SpriteSheetHandle,
        tile_index: u32,
    );

    fn set_camera(&mut self, camera: &Camera2D);
}

pub struct UvRegion {
    pub min: Vec2,
    pub max: Vec2,
}
