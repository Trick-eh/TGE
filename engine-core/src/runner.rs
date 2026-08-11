use crate::{
    GameConfig,
    contexts::{FixedContext, RenderContext, StartContext, UpdateContext},
    converter::{convert_key, convert_mouse_button},
    lua::LuaApp,
    save::SaveData,
    schedule::SystemSchedule,
    systems,
    time::Time,
};

use engine_audio::{AudioAssets, AudioManager};
use engine_ecs::World;
use engine_input::{InputState, KeyCode};
use engine_renderer::{Renderer, opengl::OpenGLRenderer};
use std::{
    sync::mpsc::Receiver,
    time::{Duration, Instant},
};
use winit::{
    application::ApplicationHandler,
    event::{MouseScrollDelta, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowId},
};

struct EngineRunner {
    app: Box<dyn crate::App>,
    window: Option<Window>,
    size: (u32, u32),
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
    reload_rx: Option<Receiver<()>>,
    save_data: SaveData,
}

impl ApplicationHandler for EngineRunner {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let (window, mut renderer) = engine_renderer::create(event_loop, &self.config.window_title);

        window.request_redraw();

        self.renderer = Some(renderer);
        self.window = Some(window);

        self.schedule.add_fixed_system(systems::snapshot_system);
        self.schedule.add_fixed_system(systems::velocity_system);
        self.schedule.add_update_system(systems::animation_system);
        self.schedule
            .add_render_system(systems::sprite_render_system);

        if let Some(renderer) = &mut self.renderer {
            self.app.on_start(&mut StartContext {
                world: &mut self.world,
                renderer: renderer.as_mut(),
                time: &mut self.time,
                audio: &mut self.audio,
                schedule: &mut self.schedule,
                input: &mut self.input,
                audio_assets: &mut self.audio_assets,
                save_data: &mut self.save_data,
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
                self.save_data.flush();
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                if let Some(renderer) = &mut self.renderer {
                    renderer.begin_frame();
                    let mut render_ctx = RenderContext {
                        world: &mut self.world,
                        time: &mut self.time,
                        renderer: renderer.as_mut(),
                        audio: &mut self.audio,
                    };
                    self.app.on_background(&mut render_ctx);
                    self.schedule.run_render(&mut render_ctx);
                    self.app.on_render(&mut render_ctx);
                    renderer.end_frame();
                    renderer.present();
                }
            }
            WindowEvent::Resized(size) => {
                let (width, height) = (size.width, size.height);
                self.size = (width, height);
                self.app.on_resize(width, height);
                if let Some(renderer) = &mut self.renderer {
                    renderer.resize(width, height);
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        let should_reload = self
            .app
            .as_any_mut()
            .downcast_mut::<LuaApp>()
            .map(|app| {
                let has_event = app.reload_rx.try_recv().is_ok();
                if has_event {
                    eprintln!(
                        "DEBUG: file change event received, elapsed={:?}",
                        app.last_reload.elapsed()
                    );
                }
                while app.reload_rx.try_recv().is_ok() {}

                if has_event && app.last_reload.elapsed() > Duration::from_millis(200) {
                    eprintln!("DEBUG: triggering reload");
                    app.last_reload = Instant::now();
                    true
                } else if has_event {
                    eprintln!("DEBUG: debounced, skipping reload");
                    false
                } else {
                    false
                }
            })
            .unwrap_or(false);

        if should_reload {
            if let Some(mut renderer) = self.renderer.take() {
                let entities: Vec<_> = self.world.iter().map(|e| e.entity()).collect();
                for e in entities {
                    let _ = self.world.despawn(e);
                }

                let mut ctx = StartContext {
                    world: &mut self.world,
                    renderer: renderer.as_mut(),
                    time: &mut self.time,
                    audio: &mut self.audio,
                    audio_assets: &mut self.audio_assets,
                    schedule: &mut self.schedule,
                    input: &mut self.input,
                    save_data: &mut self.save_data,
                };

                if let Some(lua_app) = self.app.as_any_mut().downcast_mut::<LuaApp>() {
                    lua_app.reload(&mut ctx);
                }

                self.renderer = Some(renderer);
            }
        }

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
            let mut fixed_time = Time {
                dt: FIXED_DT,
                ..self.time
            };
            let mut fixed_ctx = FixedContext {
                world: &mut self.world,
                time: &mut fixed_time,
                audio: &mut self.audio,
                audio_assets: &mut self.audio_assets,
                input: &mut self.input,
                save_data: &mut self.save_data,
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
            save_data: &mut self.save_data,
        };
        self.schedule.run_update(&mut update_ctx);
        self.app.on_update(&mut update_ctx);
        self.input.flush();

        if let Some(w) = &self.window {
            w.request_redraw();
        }

        self.save_data.flush();
    }
}

pub fn run(
    app: impl crate::App + 'static,
    config: GameConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);

    let save_data = SaveData::load(&config.window_title);

    let mut runner = EngineRunner {
        app: Box::new(app),
        window: None,
        size: (0, 0),
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
            is_paused: false,
        },
        schedule: SystemSchedule::new(),
        audio: AudioManager::new(),
        audio_assets: AudioAssets::new(),
        config,
        reload_rx: None,
        save_data,
    };

    runner.audio.set_master_volume(runner.config.master_volume);
    runner.audio.set_music_volume(runner.config.music_volume);
    runner.audio.set_sfx_volume(runner.config.sfx_volume);

    event_loop.run_app(&mut runner)?;
    Ok(())
}
