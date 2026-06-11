mod runner;

use engine_renderer::Renderer;

pub use crate::runner::run;

pub trait App {
    fn on_start(&mut self);
    fn on_update(&mut self, dt: f32);
    fn on_render(&mut self, renderer: &mut dyn Renderer);
    fn on_stop(&mut self);
    fn on_resize(&mut self, width: u32, height: u32);
}
