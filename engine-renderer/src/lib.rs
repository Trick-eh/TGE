use engine_math::{Camera2D, Transform2D};

pub mod opengl;

#[cfg(all(feature = "opengl", feature = "vulkan"))]
compile_error!(
    "Features 'opengl' and 'vulkan' are mutually exclusive. 
Enable only one at a time."
);

pub trait Renderer {
    fn begin_frame(&mut self);
    fn end_frame(&mut self);
    fn resize(&mut self, width: u32, height: u32);
    fn draw_sprite(&mut self, transform: &Transform2D, camera: &Camera2D);
}
