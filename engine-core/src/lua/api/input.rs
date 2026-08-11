use crate::lua::context::{with_start_ctx, with_update_ctx};
use engine_input::{GamepadAxis, GamepadButton, InputBinding, InputState, KeyCode, MouseButton};
use mlua::prelude::*;

pub fn register(lua: &Lua, engine: &LuaTable) -> LuaResult<()> {
    engine.set(
        "is_action_held",
        lua.create_function(|_, action: String| {
            Ok(with_update_ctx(|ctx| ctx.input.is_action_held(&action)).unwrap_or(false))
        })?,
    )?;

    engine.set(
        "is_action_pressed",
        lua.create_function(|_, action: String| {
            Ok(with_update_ctx(|ctx| ctx.input.is_action_pressed(&action)).unwrap_or(false))
        })?,
    )?;

    engine.set(
        "is_action_released",
        lua.create_function(|_, action: String| {
            Ok(with_update_ctx(|ctx| ctx.input.is_action_released(&action)).unwrap_or(false))
        })?,
    )?;

    engine.set(
        "is_key_held",
        lua.create_function(|_, key: String| {
            Ok(with_update_ctx(|ctx| {
                convert_key(&key)
                    .map(|k| ctx.input.is_key_held(k))
                    .unwrap_or(false)
            })
            .unwrap_or(false))
        })?,
    )?;

    engine.set(
        "is_key_pressed",
        lua.create_function(|_, key: String| {
            Ok(with_update_ctx(|ctx| {
                convert_key(&key)
                    .map(|k| ctx.input.is_key_pressed(k))
                    .unwrap_or(false)
            })
            .unwrap_or(false))
        })?,
    )?;

    engine.set(
        "is_key_released",
        lua.create_function(|_, key: String| {
            Ok(with_update_ctx(|ctx| {
                convert_key(&key)
                    .map(|k| ctx.input.is_key_released(k))
                    .unwrap_or(false)
            })
            .unwrap_or(false))
        })?,
    )?;

    engine.set(
        "is_mouse_button_held",
        lua.create_function(|_, button: String| {
            Ok(with_update_ctx(|ctx| {
                convert_mouse_button(&button)
                    .map(|b| ctx.input.is_mouse_button_held(b))
                    .unwrap_or(false)
            })
            .unwrap_or(false))
        })?,
    )?;

    engine.set(
        "is_mouse_button_pressed",
        lua.create_function(|_, button: String| {
            Ok(with_update_ctx(|ctx| {
                convert_mouse_button(&button)
                    .map(|b| ctx.input.is_mouse_button_pressed(b))
                    .unwrap_or(false)
            })
            .unwrap_or(false))
        })?,
    )?;

    engine.set(
        "is_mouse_button_released",
        lua.create_function(|_, button: String| {
            Ok(with_update_ctx(|ctx| {
                convert_mouse_button(&button)
                    .map(|b| ctx.input.is_mouse_button_released(b))
                    .unwrap_or(false)
            })
            .unwrap_or(false))
        })?,
    )?;

    engine.set(
        "mouse_position",
        lua.create_function(|lua, ()| {
            let pos = with_update_ctx(|ctx| ctx.input.mouse_position()).unwrap_or_default();
            let table = lua.create_table()?;
            table.set("x", pos.x)?;
            table.set("y", pos.y)?;
            Ok(table)
        })?,
    )?;

    engine.set(
        "scroll_delta",
        lua.create_function(|lua, ()| {
            let delta = with_update_ctx(|ctx| ctx.input.scroll_delta()).unwrap_or_default();
            let table = lua.create_table()?;
            table.set("x", delta.x)?;
            table.set("y", delta.y)?;
            Ok(table)
        })?,
    )?;

    engine.set(
        "is_gamepad_button_held",
        lua.create_function(|_, button: String| {
            Ok(with_update_ctx(|ctx| {
                convert_gamepad_button(&button)
                    .map(|b| ctx.input.is_gamepad_button_held(b))
                    .unwrap_or(false)
            })
            .unwrap_or(false))
        })?,
    )?;

    engine.set(
        "is_gamepad_button_pressed",
        lua.create_function(|_, button: String| {
            Ok(with_update_ctx(|ctx| {
                convert_gamepad_button(&button)
                    .map(|b| ctx.input.is_gamepad_button_pressed(b))
                    .unwrap_or(false)
            })
            .unwrap_or(false))
        })?,
    )?;

    engine.set(
        "is_gamepad_button_released",
        lua.create_function(|_, button: String| {
            Ok(with_update_ctx(|ctx| {
                convert_gamepad_button(&button)
                    .map(|b| ctx.input.is_gamepad_button_released(b))
                    .unwrap_or(false)
            })
            .unwrap_or(false))
        })?,
    )?;

    engine.set(
        "gamepad_axis",
        lua.create_function(|_, axis: String| {
            Ok(with_update_ctx(|ctx| {
                convert_gamepad_axis(&axis)
                    .map(|a| ctx.input.gamepad_axis(a))
                    .unwrap_or(0.0)
            })
            .unwrap_or(0.0))
        })?,
    )?;

    engine.set(
        "bind_action",
        lua.create_function(|_, (action, binding): (String, String)| {
            fn bind(input: &mut InputState, action: &str, binding: &str) {
                if let Some(input_binding) = convert_binding(&binding) {
                    input.bind_action(&action, input_binding);
                } else {
                    eprintln!("Lua input: unknown binding '{}'", binding);
                }
            }
            with_start_ctx(|ctx| bind(&mut ctx.input, &action, &binding))
                .or_else(|| with_update_ctx(|ctx| bind(&mut ctx.input, &action, &binding)));
            Ok(())
        })?,
    )?;

    engine.set(
        "unbind_action",
        lua.create_function(|_, action: String| {
            with_update_ctx(|ctx| ctx.input.unbind_action(&action));
            Ok(())
        })?,
    )?;

    Ok(())
}

// --- string → engine type converters ---

fn convert_key(key: &str) -> Option<KeyCode> {
    match key {
        "a" => Some(KeyCode::A),
        "b" => Some(KeyCode::B),
        "c" => Some(KeyCode::C),
        "d" => Some(KeyCode::D),
        "e" => Some(KeyCode::E),
        "f" => Some(KeyCode::F),
        "g" => Some(KeyCode::G),
        "h" => Some(KeyCode::H),
        "i" => Some(KeyCode::I),
        "j" => Some(KeyCode::J),
        "k" => Some(KeyCode::K),
        "l" => Some(KeyCode::L),
        "m" => Some(KeyCode::M),
        "n" => Some(KeyCode::N),
        "o" => Some(KeyCode::O),
        "p" => Some(KeyCode::P),
        "q" => Some(KeyCode::Q),
        "r" => Some(KeyCode::R),
        "s" => Some(KeyCode::S),
        "t" => Some(KeyCode::T),
        "u" => Some(KeyCode::U),
        "v" => Some(KeyCode::V),
        "w" => Some(KeyCode::W),
        "x" => Some(KeyCode::X),
        "y" => Some(KeyCode::Y),
        "z" => Some(KeyCode::Z),
        "0" => Some(KeyCode::Zero),
        "1" => Some(KeyCode::One),
        "2" => Some(KeyCode::Two),
        "3" => Some(KeyCode::Three),
        "4" => Some(KeyCode::Four),
        "5" => Some(KeyCode::Five),
        "6" => Some(KeyCode::Six),
        "7" => Some(KeyCode::Seven),
        "8" => Some(KeyCode::Eight),
        "9" => Some(KeyCode::Nine),
        "left" => Some(KeyCode::Left),
        "right" => Some(KeyCode::Right),
        "up" => Some(KeyCode::Up),
        "down" => Some(KeyCode::Down),
        "space" => Some(KeyCode::Space),
        "enter" => Some(KeyCode::Enter),
        "escape" => Some(KeyCode::Escape),
        "backspace" => Some(KeyCode::Backspace),
        "tab" => Some(KeyCode::Tab),
        "lshift" => Some(KeyCode::LShift),
        "rshift" => Some(KeyCode::RShift),
        "lctrl" => Some(KeyCode::LCtrl),
        "rctrl" => Some(KeyCode::RCtrl),
        "lalt" => Some(KeyCode::LAlt),
        "ralt" => Some(KeyCode::RAlt),
        "insert" => Some(KeyCode::Insert),
        "delete" => Some(KeyCode::Delete),
        "home" => Some(KeyCode::Home),
        "end" => Some(KeyCode::End),
        "pageup" => Some(KeyCode::PageUp),
        "pagedown" => Some(KeyCode::PageDown),
        "f1" => Some(KeyCode::F1),
        "f2" => Some(KeyCode::F2),
        "f3" => Some(KeyCode::F3),
        "f4" => Some(KeyCode::F4),
        "f5" => Some(KeyCode::F5),
        "f6" => Some(KeyCode::F6),
        "f7" => Some(KeyCode::F7),
        "f8" => Some(KeyCode::F8),
        "f9" => Some(KeyCode::F9),
        "f10" => Some(KeyCode::F10),
        "f11" => Some(KeyCode::F11),
        "f12" => Some(KeyCode::F12),
        _ => {
            eprintln!("Lua input: unknown key '{}'", key);
            None
        }
    }
}

fn convert_mouse_button(button: &str) -> Option<MouseButton> {
    match button {
        "left_click" => Some(MouseButton::Left),
        "right_click" => Some(MouseButton::Right),
        "middle_click" => Some(MouseButton::Middle),
        "back_click" => Some(MouseButton::Back),
        "forward_click" => Some(MouseButton::Forward),
        _ => {
            eprintln!("Lua input: unknown mouse button '{}'", button);
            None
        }
    }
}

fn convert_gamepad_button(button: &str) -> Option<GamepadButton> {
    match button {
        "south" => Some(GamepadButton::South),
        "north" => Some(GamepadButton::North),
        "east" => Some(GamepadButton::East),
        "west" => Some(GamepadButton::West),
        "lbumper" => Some(GamepadButton::LBumper),
        "rbumper" => Some(GamepadButton::RBumper),
        "ltrigger" => Some(GamepadButton::LTrigger),
        "rtrigger" => Some(GamepadButton::RTrigger),
        "start" => Some(GamepadButton::Start),
        "select" => Some(GamepadButton::Select),
        "dpad_up" => Some(GamepadButton::DPadUp),
        "dpad_down" => Some(GamepadButton::DPadDown),
        "dpad_left" => Some(GamepadButton::DPadLeft),
        "dpad_right" => Some(GamepadButton::DPadRight),
        "lstick" => Some(GamepadButton::LStick),
        "rstick" => Some(GamepadButton::RStick),
        _ => {
            eprintln!("Lua input: unknown gamepad button '{}'", button);
            None
        }
    }
}

fn convert_gamepad_axis(axis: &str) -> Option<GamepadAxis> {
    match axis {
        "left_x" => Some(GamepadAxis::LeftStickX),
        "left_y" => Some(GamepadAxis::LeftStickY),
        "right_x" => Some(GamepadAxis::RightStickX),
        "right_y" => Some(GamepadAxis::RightStickY),
        "left_trigger" => Some(GamepadAxis::LeftTrigger),
        "right_trigger" => Some(GamepadAxis::RightTrigger),
        _ => {
            eprintln!("Lua input: unknown gamepad axis '{}'", axis);
            None
        }
    }
}

fn convert_binding(binding: &str) -> Option<InputBinding> {
    use engine_input::InputBinding;

    // try key first
    if let Some(key) = convert_key(binding) {
        return Some(InputBinding::Key(key));
    }
    // try mouse button
    if let Some(btn) = convert_mouse_button(binding) {
        return Some(InputBinding::MouseButton(btn));
    }
    // try gamepad button
    if let Some(btn) = convert_gamepad_button(binding) {
        return Some(InputBinding::GamepadButton(btn));
    }
    None
}
