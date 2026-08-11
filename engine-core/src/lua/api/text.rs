use engine_math::Vec2;
use engine_renderer::FontHandle;
use mlua::prelude::*;

use crate::lua::context::{with_render_ctx, with_start_ctx};

pub fn register(lua: &Lua, engine: &LuaTable) -> LuaResult<()> {
    engine.set(
        "load_font",
        lua.create_function(|_, (path, size): (String, f32)| {
            let index = with_start_ctx(|ctx| {
                let bytes = std::fs::read(&path)
                    .unwrap_or_else(|_| panic!("Failed to read font: {}", path));
                ctx.renderer.load_font(&bytes, size).to_lua_id()
            });
            Ok(index)
        })?,
    )?;

    engine.set(
        "draw_text",
        lua.create_function(
            |_, (text, font_id, x, y, r, g, b, a): (String, u32, f32, f32, f32, f32, f32, f32)| {
                with_render_ctx(|ctx| {
                    ctx.renderer.draw_text(
                        &text,
                        FontHandle::from_lua_id(font_id),
                        Vec2::new(x, y),
                        [r, g, b, a],
                    );
                });
                Ok(())
            },
        )?,
    )?;

    engine.set(
        "measure_text",
        lua.create_function(|lua, (text, font_id): (String, u32)| {
            let size = with_render_ctx(|ctx| {
                ctx.renderer
                    .measure_text(&text, FontHandle::from_lua_id(font_id))
            })
            .unwrap_or(Vec2::ZERO);

            let table = lua.create_table()?;
            table.set("x", size.x)?;
            table.set("y", size.y)?;
            Ok(table)
        })?,
    )?;

    Ok(())
}
