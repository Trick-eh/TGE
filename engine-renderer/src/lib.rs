use engine_math::{Camera2D, Transform2D, Vec2};

pub mod batch;
pub mod colors;

#[cfg(all(feature = "opengl", not(target_arch = "wasm32")))]
pub mod opengl;

#[cfg(feature = "vulkan")]
pub mod vulkan;

#[cfg(all(feature = "opengl", feature = "vulkan"))]
compile_error!(
    "Features 'opengl' and 'vulkan' are mutually exclusive. 
Enable only one at a time."
);

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct TextureHandle(u32);
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct SpriteSheetHandle(u32);
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct FontHandle(u32);

impl TextureHandle {
    pub fn to_lua_id(&self) -> u32 {
        self.0
    }
    pub fn from_lua_id(id: u32) -> Self {
        TextureHandle(id)
    }
}
impl SpriteSheetHandle {
    pub fn to_lua_id(&self) -> u32 {
        self.0
    }
    pub fn from_lua_id(id: u32) -> Self {
        SpriteSheetHandle(id)
    }
}
impl FontHandle {
    pub fn to_lua_id(&self) -> u32 {
        self.0
    }
    pub fn from_lua_id(id: u32) -> Self {
        FontHandle(id)
    }
}

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
    fn present(&mut self);
    fn resize(&mut self, width: u32, height: u32);
    fn request_redraw(&self);

    fn set_camera(&mut self, camera: &Camera2D);

    fn load_texture(&mut self, bytes: &[u8]) -> TextureHandle;
    fn create_sprite_sheet(
        &mut self,
        texture: TextureHandle,
        tile_width: u32,
        tile_height: u32,
    ) -> SpriteSheetHandle;
    fn draw_sprite(
        &mut self,
        transform: &Transform2D,
        spritesheet: SpriteSheetHandle,
        tile_index: u32,
    );
    fn draw_colored_rect(&mut self, transform: &Transform2D, r: f32, g: f32, b: f32, a: f32);

    fn load_font(&mut self, bytes: &[u8], size: f32) -> FontHandle;
    fn draw_text(&mut self, text: &str, font: FontHandle, position: Vec2, color: [f32; 4]);
    fn measure_text(&self, text: &str, font: FontHandle) -> Vec2;

    fn set_clear_color(&mut self, color: [f32; 4]);

    fn uv_for_tile(&self, sheet: SpriteSheetHandle, index: u32) -> UvRegion;

    fn clear_assets(&mut self);
}

pub struct UvRegion {
    pub min: Vec2,
    pub max: Vec2,
}

#[cfg(all(feature = "opengl", not(target_arch = "wasm32")))]
pub fn create(event_loop: &winit::event_loop::ActiveEventLoop, title: &str) -> Box<dyn Renderer> {
    opengl::create_renderer(event_loop, title)
}

// #[cfg(feature = "vulkan")]
// pub fn create(
//     event_loop: &winit::event_loop::ActiveEventLoop,
//     title: &str,
// ) -> Box<dyn Renderer> {
//     vulkan::create_renderer(event_loop, title)
// }
