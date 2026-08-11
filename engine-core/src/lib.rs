mod contexts;
mod converter;
pub mod lua;
mod runner;
mod save;
mod schedule;
mod systems;
mod time;

pub use crate::contexts::*;
pub use crate::runner::run;
pub use crate::schedule::SystemSchedule;
pub use crate::time::Time;

pub trait App {
    fn on_start(&mut self, ctx: &mut StartContext);
    fn on_fixed_update(&mut self, ctx: &mut FixedContext);
    fn on_update(&mut self, ctx: &mut UpdateContext);
    fn on_background(&mut self, ctx: &mut RenderContext) {}
    fn on_render(&mut self, ctx: &mut RenderContext) {}
    fn on_stop(&mut self);
    fn on_resize(&mut self, width: u32, height: u32);
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        panic!("as_any_mut not implemented - required for hot reload (lua only)")
    }
}

pub struct GameConfig {
    pub master_volume: f64,
    pub sfx_volume: f64,
    pub music_volume: f64,
    pub window_title: String,
    pub window_width: u32,
    pub window_height: u32,
    pub vsync: bool,
    pub fixed_timestep: f32,
}

impl Default for GameConfig {
    fn default() -> Self {
        GameConfig {
            master_volume: 1.0,
            sfx_volume: 1.0,
            music_volume: 0.5,
            window_title: "engine".to_string(),
            window_width: 800,
            window_height: 600,
            vsync: true,
            fixed_timestep: 1.0 / 60.0,
        }
    }
}
