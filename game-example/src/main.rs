use std::f32::consts::PI;

use engine_core::{App, run};
use engine_math::{Camera2D, Transform2D, Vec2};
use engine_renderer::Renderer;

struct MyGame {
    transform: Transform2D,
    camera: Camera2D,
}

impl App for MyGame {
    fn on_start(&mut self) {
        println!("started")
    }

    fn on_update(&mut self, dt: f32) {}

    fn on_stop(&mut self) {
        println!("stopped")
    }

    fn on_resize(&mut self, width: u32, height: u32) {}

    fn on_render(&mut self, renderer: &mut dyn Renderer) {
        renderer.draw_sprite(&self.transform, &self.camera);
    }
}

fn main() {
    run(MyGame {
        transform: Transform2D {
            position: Vec2 { x: 300.0, y: 200.0 },
            rotation_in_radians: 1.0,
            scale: Vec2 { x: 100.0, y: 100.0 },
        },
        camera: Camera2D {
            position: Vec2 { x: 0.0, y: 0.0 },
            zoom: 1.0,
        },
    })
    .unwrap();
}
