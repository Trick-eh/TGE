//! Minimal smoke test for the Vulkan renderer, isolated from the real
//! engine runner. Run with:
//!
//!   cargo run --example vulkan_smoke_test --no-default-features --features vulkan -p engine-renderer
//!
//! Success looks like: a window appears showing a solid dark blue-gray
//! clear color (matching the OpenGL backend's default clear), and stays
//! open, continuously redrawing (cycling through frames-in-flight) until
//! closed.

use engine_math::{Transform2D, Vec2};
use engine_renderer::Renderer;
use engine_renderer::vulkan::VulkanRenderer;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    window::WindowId,
};

struct SmokeTest {
    vulkan: Option<VulkanRenderer>,
}

impl ApplicationHandler for SmokeTest {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let vulkan = VulkanRenderer::new(event_loop, "vulkan smoke test");
        println!("VulkanRenderer created successfully.");
        vulkan.request_redraw();
        self.vulkan = Some(vulkan);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                self.vulkan = None;
                println!("VulkanRenderer destroyed cleanly.");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                if let Some(vulkan) = &mut self.vulkan {
                    vulkan.begin_frame();
                    for i in 0..1001 {
                        vulkan.draw_colored_rect(
                            &Transform2D {
                                position: Vec2 {
                                    x: i as f32 / 1000.0,
                                    y: 0.0,
                                },
                                rotation_in_radians: 0.0,
                                scale: Vec2 { x: 0.1, y: 0.1 },
                            },
                            1.0,
                            0.0,
                            0.0,
                            1.0,
                        );
                    }

                    vulkan.end_frame();
                    vulkan.present();
                    vulkan.request_redraw(); // keep the loop going
                }
            }
            WindowEvent::Resized(s) => {
                if let Some(vulkan) = &mut self.vulkan {
                    vulkan.resize(s.width, s.height);
                }
            }
            _ => {}
        }
    }
}

fn main() {
    let event_loop = EventLoop::new().expect("Failed to create event loop");
    let mut app = SmokeTest { vulkan: None };
    event_loop.run_app(&mut app).expect("Event loop failed");
}
