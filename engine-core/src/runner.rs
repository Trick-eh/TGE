use crate::{
    GameConfig,
    contexts::{FixedContext, RenderContext, StartContext, UpdateContext},
    converter::{convert_key, convert_mouse_button},
    schedule::{self, SystemSchedule},
    systems,
    time::Time,
};

use engine_audio::{AudioAssets, AudioManager};
use engine_ecs::World;
use engine_input::InputState;
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
use std::{
    ffi::{CStr, CString},
    ops::DerefMut,
    time::Instant,
};
use winit::{
    application::ApplicationHandler,
    event::{KeyEvent, MouseScrollDelta, WindowEvent},
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
    input: InputState,
    world: World,
    last_frame_time: Option<Instant>,
    accumulator: f32,
    time: Time,
    schedule: SystemSchedule,
    audio: AudioManager,
    audio_assets: AudioAssets,
    config: GameConfig,
}

impl ApplicationHandler for EngineRunner {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let template_builder = ConfigTemplateBuilder::new();
        let display_builder = DisplayBuilder::new().with_window_attributes(Some(
            Window::default_attributes().with_title(&self.config.window_title),
        ));

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
        self.schedule.add_fixed_system(systems::snapshot_system);
        self.schedule.add_update_system(systems::animation_system);
        self.schedule
            .add_render_system(systems::sprite_render_system);

        if let Some(renderer) = &mut self.renderer {
            self.app.on_start(&mut StartContext {
                world: &mut self.world,
                renderer: renderer.as_mut(),
                audio: &mut self.audio,
                schedule: &mut self.schedule,
                input: &mut self.input,
                audio_assets: &mut self.audio_assets,
            });
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::MouseWheel { delta, .. } => match delta {
                MouseScrollDelta::LineDelta(x, y) => {
                    self.input.add_scroll_delta(x, y);
                }
                MouseScrollDelta::PixelDelta(pos) => {
                    self.input.add_scroll_delta(pos.x as f32, pos.y as f32);
                }
            },
            WindowEvent::MouseInput { state, button, .. } => {
                let (button, pressed) = convert_mouse_button(button, state);
                self.input.process_mouse_button(button, pressed);
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.input
                    .set_mouse_position(position.x as f32, position.y as f32);
            }
            WindowEvent::KeyboardInput {
                event: key_event,
                is_synthetic,
                ..
            } => {
                if is_synthetic {
                    return;
                }
                if let Some((key, pressed)) = convert_key(key_event) {
                    self.input.process_key_event(key, pressed);
                }
            }
            WindowEvent::CloseRequested => {
                self.app.on_stop();
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                if let (Some(renderer), Some(surface), Some(ctx)) =
                    (&mut self.renderer, &self.gl_surface, &self.gl_context)
                {
                    renderer.begin_frame();
                    let mut render_ctx = RenderContext {
                        world: &mut self.world,
                        time: &mut self.time,
                        renderer: renderer.as_mut(),
                        audio: &mut self.audio,
                    };
                    self.schedule.run_render(&mut render_ctx);
                    self.app.on_render(&mut render_ctx);
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
        const FIXED_DT: f32 = 1.0 / 60.0;

        let now = Instant::now();
        let dt = match self.last_frame_time {
            Some(last) => (now - last).as_secs_f32(),
            None => FIXED_DT,
        };
        let dt = dt.min(0.25);
        self.last_frame_time = Some(now);

        self.time.dt = dt;
        self.time.elapsed += dt;
        self.time.frame_count += 1;

        self.input.poll_gamepad_events();

        self.accumulator += dt;
        while self.accumulator >= FIXED_DT {
            let fixed_time = Time {
                dt: FIXED_DT,
                ..self.time
            };
            let mut fixed_ctx = FixedContext {
                world: &mut self.world,
                time: &fixed_time,
                audio: &mut self.audio,
            };
            self.schedule.run_fixed(&mut fixed_ctx);
            self.app.on_fixed_update(&mut fixed_ctx);
            self.accumulator -= FIXED_DT;
        }
        self.time.alpha = self.accumulator / FIXED_DT;

        let mut update_ctx = UpdateContext {
            world: &mut self.world,
            time: &mut self.time,
            input: &mut self.input,
            audio: &mut self.audio,
            audio_assets: &mut self.audio_assets,
            config: &self.config,
        };
        self.schedule.run_update(&mut update_ctx);
        self.app.on_update(&mut update_ctx);
        self.input.flush();

        if let Some(w) = &self.window {
            w.request_redraw();
        }
    }
}

pub fn run(
    app: impl crate::App + 'static,
    config: GameConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);
    let mut runner = EngineRunner {
        app: Box::new(app),
        window: None,
        size: (0, 0),
        gl_surface: None,
        gl_context: None,
        renderer: None,
        input: InputState::new(),
        world: World::new(),
        last_frame_time: None,
        accumulator: 0.0,
        time: Time {
            dt: 0.0,
            elapsed: 0.0,
            frame_count: 0,
            alpha: 0.0,
        },
        schedule: SystemSchedule::new(),
        audio: AudioManager::new(),
        audio_assets: AudioAssets::new(),
        config,
    };

    runner.audio.set_master_volume(runner.config.master_volume);
    runner.audio.set_music_volume(runner.config.music_volume);
    runner.audio.set_sfx_volume(runner.config.sfx_volume);

    event_loop.run_app(&mut runner)?;
    Ok(())
}
