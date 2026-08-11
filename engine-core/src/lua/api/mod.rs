use crate::{contexts::StartContext, lua::context::with_persistent};
use engine_ecs::LuaData;
use mlua::prelude::*;

mod audio;
mod input;
mod save;
mod text;
mod world;

pub fn register(lua: &Lua) -> LuaResult<()> {
    let engine = lua.create_table()?;

    input::register(lua, &engine)?;
    audio::register(lua, &engine)?;
    world::register(lua, &engine)?;
    register_persist(lua, &engine)?;
    save::register(lua, &engine)?;
    text::register(lua, &engine)?;

    lua.globals().set("engine", engine)?;

    Ok(())
}

fn register_persist(lua: &Lua, engine: &LuaTable) -> LuaResult<()> {
    engine.set(
        "persist",
        lua.create_function(|_, (key, value): (String, LuaValue)| {
            with_persistent(|store| {
                store.insert(key, from_lua_value(value));
            });
            Ok(())
        })?,
    )?;

    engine.set(
        "get_persisted",
        lua.create_function(|lua, key: String| {
            let result = with_persistent(|store| store.get(&key).cloned()).flatten();
            match result {
                Some(data) => to_lua_value(&lua, &data),
                None => Ok(LuaValue::Nil),
            }
        })?,
    )?;

    Ok(())
}

fn to_lua_value(lua: &Lua, data: &LuaData) -> LuaResult<LuaValue> {
    match data {
        LuaData::Number(n) => Ok(LuaValue::Number(*n)),
        LuaData::Bool(b) => Ok(LuaValue::Boolean(*b)),
        LuaData::String(s) => Ok(LuaValue::String(lua.create_string(s)?)),
        LuaData::Nil => Ok(LuaValue::Nil),
    }
}
fn from_lua_value(value: LuaValue) -> LuaData {
    match value {
        LuaValue::Number(n) => LuaData::Number(n),
        LuaValue::Integer(i) => LuaData::Number(i as f64),
        LuaValue::Boolean(b) => LuaData::Bool(b),
        LuaValue::String(s) => LuaData::String(s.to_string_lossy()),
        _ => LuaData::Nil,
    }
}
