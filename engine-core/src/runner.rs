use std::ffi::{CStr, CString};

use engine_renderer::{Renderer, opengl::OpenGLRenderer};
use glutin::{
    api::egl::{config, surface},
    config::{ConfigTemplateBuilder, GlConfig},
    context::{ContextAttributesBuilder, PossiblyCurrentContext},
    display::GetGlDisplay,
    prelude::*,
    prelude::{GlDisplay, NotCurrentGlContext},
    surface::{GlSurface, Surface, WindowSurface},
};
use glutin_winit::{DisplayBuilder, GlWindow};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    raw_window_handle::HasRawWindowHandle,
    window::{Window, WindowId},
};

struct EngineRunner {
    app: Box<dyn crate::App>,
    window: Option<Window>,
    size: (u32, u32),
    gl_surface: Option<Surface<WindowSurface>>,
    gl_context: Option<PossiblyCurrentContext>,
    renderer: Option<Box<dyn engine_renderer::Renderer>>,
}

impl ApplicationHandler for EngineRunner {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let template_builder = ConfigTemplateBuilder::new();
        let display_builder = DisplayBuilder::new()
            .with_window_attributes(Some(Window::default_attributes().with_title("engine")));

        let (window, gl_config) = display_builder
            .build(event_loop, template_builder, |configs| {
                configs
                    .reduce(|a, b| {
                        if a.num_samples() > b.num_samples() {
                            a
                        } else {
                            b
                        }
                    })
                    .unwrap()
            })
            .unwrap();
        let window = window.unwrap();

        let context_attrs =
            ContextAttributesBuilder::new().build(Some(window.raw_window_handle().unwrap()));
        let not_current_ctx = unsafe {
            gl_config
                .display()
                .create_context(&gl_config, &context_attrs)
                .unwrap()
        };

        let surface_attrs = window.build_surface_attributes(Default::default()).unwrap();
        let gl_surface = unsafe {
            gl_config
                .display()
                .create_window_surface(&gl_config, &surface_attrs)
                .unwrap()
        };

        let gl_context = not_current_ctx.make_current(&gl_surface).unwrap();

        let gl = unsafe {
            glow::Context::from_loader_function(|s| {
                gl_config
                    .display()
                    .get_proc_address(CString::new(s).unwrap().as_c_str())
                    as *const _
            })
        };

        let size = window.inner_size();
        let mut renderer = Box::new(OpenGLRenderer::new(gl));
        renderer.resize(size.width, size.height);

        window.request_redraw();

        self.gl_surface = Some(gl_surface);
        self.gl_context = Some(gl_context);
        self.renderer = Some(renderer);
        self.window = Some(window);

        self.app.on_start();
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                self.app.on_stop();
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                if let (Some(renderer), Some(surface), Some(ctx)) =
                    (&mut self.renderer, &self.gl_surface, &self.gl_context)
                {
                    renderer.begin_frame();
                    self.app.on_render(renderer.as_mut());
                    renderer.end_frame();
                    surface.swap_buffers(ctx).unwrap();
                }
            }
            WindowEvent::Resized(size) => {
                let (width, height) = (size.width, size.height);
                self.size = (width, height);
                self.app.on_resize(width, height);
                if let Some(renderer) = &mut self.renderer {
                    renderer.resize(width, height);
                }
                if let (Some(surface), Some(ctx)) = (&self.gl_surface, &self.gl_context) {
                    surface.resize(
                        ctx,
                        std::num::NonZeroU32::new(width.max(1)).unwrap(),
                        std::num::NonZeroU32::new(height.max(1)).unwrap(),
                    );
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        self.app.on_update(0.0);
        if let Some(w) = &self.window {
            w.request_redraw();
        }
    }
}

pub fn run(app: impl crate::App + 'static) -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
    let mut runner = EngineRunner {
        app: Box::new(app),
        window: None,
        size: (0, 0),
        gl_surface: None,
        gl_context: None,
        renderer: None,
    };

    event_loop.run_app(&mut runner)?;
    Ok(())
}
