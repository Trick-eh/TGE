use crate::{
    App,
    contexts::{FixedContext, RenderContext, StartContext, UpdateContext},
};
use engine_ecs::LuaData;
use mlua::prelude::*;
use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::mpsc::{Receiver, channel},
    time::{Duration, Instant},
};

pub mod api;
mod context;

pub const LUA_DEFINITIONS: &str = include_str!("../../../definitions/engine.lua");

pub struct LuaApp {
    lua: Lua,
    script_path: PathBuf,
    persistent: HashMap<String, LuaData>,
    pub reload_rx: Receiver<()>,
    _watcher: RecommendedWatcher,
    pub last_reload: Instant,
}

impl LuaApp {
    pub fn new(script_path: impl Into<PathBuf>) -> Self {
        let script_path = script_path.into();
        let (tx, rx) = channel::<()>();

        let tx_clone = tx.clone();
        let watch_path = script_path.clone();
        let watch_dir = script_path
            .parent()
            .expect("Script path has no parent")
            .to_path_buf();

        let mut watcher = notify::recommended_watcher(move |res: notify::Result<Event>| {
            if let Ok(event) = res {
                let is_script = event.paths.iter().any(|p| p == &watch_path);
                if is_script {
                    let _ = tx_clone.send(());
                }
            }
        })
        .expect("Failed to create file watcher");

        watcher
            .watch(&watch_dir, RecursiveMode::NonRecursive)
            .expect("Failed to watch directory");

        LuaApp {
            lua: Lua::new(),
            script_path,
            persistent: HashMap::new(),
            reload_rx: rx,
            _watcher: watcher,
            last_reload: Instant::now(),
        }
    }

    fn call_lua_fn<A: IntoLuaMulti>(&self, name: &str, args: A) {
        let result: LuaResult<()> = (|| {
            let globals = self.lua.globals();
            match globals.get::<LuaValue>(name)? {
                LuaValue::Function(func) => func.call::<()>(args)?,
                LuaValue::Nil => {}
                _ => eprintln!("Lua: '{}' exists but is not a function", name),
            }
            Ok(())
        })();

        if let Err(e) = result {
            eprintln!("Lua error in {}: {}", name, e);
        }
    }

    pub fn reload(&mut self, ctx: &mut StartContext) {
        eprintln!("Hot reload triggered...");

        ctx.renderer.clear_assets();
        ctx.audio.clear_assets();
        ctx.audio_assets.clear_assets();
        ctx.time.is_paused = false;

        self.lua = Lua::new();

        if let Err(e) = api::register(&self.lua) {
            eprintln!("Hot reload: failed to register API: {}", e);
            return;
        }

        let script = {
            let mut attempts = 0;
            loop {
                match std::fs::read_to_string(&self.script_path) {
                    Ok(s) => break s,
                    Err(e) => {
                        attempts += 1;
                        if attempts >= 10 {
                            eprintln!(
                                "Hot reload: failed to read script after {} attempts: {}",
                                attempts, e
                            );
                            return;
                        }
                        std::thread::sleep(Duration::from_millis(10));
                    }
                }
            }
        };

        if let Err(e) = self.lua.load(&script).exec() {
            eprintln!("Hot reload: script error: {}", e);
            return;
        }

        context::set_start_ctx(ctx);
        context::set_persistent(&mut self.persistent);
        context::set_save_data(&mut ctx.save_data);
        self.call_lua_fn("on_start", ());
        context::clear_start_ctx();
        context::clear_persistent();
        context::clear_save_data();

        eprintln!("Hot reload complete.");
    }
}

impl App for LuaApp {
    fn on_start(&mut self, ctx: &mut StartContext) {
        if let Err(e) = api::register(&self.lua) {
            eprintln!("Failed to register Lua API: {}", e);
            return;
        }
        let script = std::fs::read_to_string(&self.script_path).expect("Failed to read Lua script");

        if let Err(e) = self.lua.load(&script).exec() {
            eprintln!("Lua script error: {}", e);
            return;
        }

        context::set_start_ctx(ctx);
        context::set_persistent(&mut self.persistent);
        context::set_save_data(&mut ctx.save_data);
        self.call_lua_fn("on_start", ());
        context::clear_start_ctx();
        context::clear_persistent();
        context::clear_save_data();
    }

    fn on_update(&mut self, ctx: &mut UpdateContext) {
        context::set_update_ctx(ctx);
        context::set_persistent(&mut self.persistent);
        context::set_save_data(&mut ctx.save_data);
        self.call_lua_fn("on_update", ctx.time.dt);
        context::clear_update_ctx();
        context::clear_persistent();
        context::clear_save_data();
    }

    fn on_fixed_update(&mut self, ctx: &mut FixedContext) {
        context::set_fixed_ctx(ctx);
        context::set_persistent(&mut self.persistent);
        context::set_save_data(&mut ctx.save_data);
        self.call_lua_fn("on_fixed_update", ctx.time.dt);
        context::clear_fixed_ctx();
        context::clear_persistent();
        context::clear_save_data();
    }

    fn on_background(&mut self, ctx: &mut RenderContext) {
        context::set_render_ctx(ctx);
        context::set_persistent(&mut self.persistent);
        self.call_lua_fn("on_background", ());
        context::clear_render_ctx();
        context::clear_persistent();
    }

    fn on_render(&mut self, ctx: &mut RenderContext) {
        context::set_render_ctx(ctx);
        context::set_persistent(&mut self.persistent);
        self.call_lua_fn("on_render", ());
        context::clear_render_ctx();
        context::clear_persistent();
    }

    fn on_stop(&mut self) {
        self.call_lua_fn("on_stop", ());
    }

    fn on_resize(&mut self, width: u32, height: u32) {
        self.call_lua_fn("on_resize", (width, height));
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
