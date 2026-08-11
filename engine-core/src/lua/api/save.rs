use crate::lua::context::with_save_data;
use mlua::prelude::*;

pub fn register(lua: &Lua, engine: &LuaTable) -> LuaResult<()> {
    engine.set(
        "save_data",
        lua.create_function(|_, (key, value): (String, LuaValue)| {
            with_save_data(|save| {
                let json_value = match value {
                    LuaValue::Number(n) => serde_json::Value::from(n),
                    LuaValue::Integer(i) => serde_json::Value::from(i),
                    LuaValue::Boolean(b) => serde_json::Value::from(b),
                    LuaValue::String(s) => serde_json::Value::from(s.to_string_lossy()),
                    _ => serde_json::Value::Null,
                };
                save.set(key, json_value);
            });
            Ok(())
        })?,
    )?;

    engine.set(
        "load_data",
        lua.create_function(|lua, key: String| {
            let result = with_save_data(|save| save.get(&key).cloned()).flatten();

            match result {
                Some(serde_json::Value::Number(n)) => {
                    Ok(LuaValue::Number(n.as_f64().unwrap_or(0.0)))
                }
                Some(serde_json::Value::Bool(b)) => Ok(LuaValue::Boolean(b)),
                Some(serde_json::Value::String(s)) => Ok(LuaValue::String(lua.create_string(&s)?)),
                _ => Ok(LuaValue::Nil),
            }
        })?,
    )?;

    Ok(())
}
