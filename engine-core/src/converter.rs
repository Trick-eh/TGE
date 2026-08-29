use winit::{
    event::{ElementState, KeyEvent, MouseButton as WinitMouseButton},
    keyboard::{KeyCode as WinitKeyCode, PhysicalKey},
};

use engine_input::{KeyCode, MouseButton};

/// returns None if the key is not supported
pub fn convert_key(event: KeyEvent) -> Option<(KeyCode, bool, Option<String>)> {
    let pressed = event.state == ElementState::Pressed;
    let text = event.text.as_ref().map(|s| s.to_string());
    let key = match event.physical_key {
        PhysicalKey::Code(code) => match code {
            // letters
            WinitKeyCode::KeyA => KeyCode::A,
            WinitKeyCode::KeyB => KeyCode::B,
            WinitKeyCode::KeyC => KeyCode::C,
            WinitKeyCode::KeyD => KeyCode::D,
            WinitKeyCode::KeyE => KeyCode::E,
            WinitKeyCode::KeyF => KeyCode::F,
            WinitKeyCode::KeyG => KeyCode::G,
            WinitKeyCode::KeyH => KeyCode::H,
            WinitKeyCode::KeyI => KeyCode::I,
            WinitKeyCode::KeyJ => KeyCode::J,
            WinitKeyCode::KeyK => KeyCode::K,
            WinitKeyCode::KeyL => KeyCode::L,
            WinitKeyCode::KeyM => KeyCode::M,
            WinitKeyCode::KeyN => KeyCode::N,
            WinitKeyCode::KeyO => KeyCode::O,
            WinitKeyCode::KeyP => KeyCode::P,
            WinitKeyCode::KeyQ => KeyCode::Q,
            WinitKeyCode::KeyR => KeyCode::R,
            WinitKeyCode::KeyS => KeyCode::S,
            WinitKeyCode::KeyT => KeyCode::T,
            WinitKeyCode::KeyU => KeyCode::U,
            WinitKeyCode::KeyV => KeyCode::V,
            WinitKeyCode::KeyW => KeyCode::W,
            WinitKeyCode::KeyX => KeyCode::X,
            WinitKeyCode::KeyY => KeyCode::Y,
            WinitKeyCode::KeyZ => KeyCode::Z,
            // numbers
            WinitKeyCode::Digit0 => KeyCode::Zero,
            WinitKeyCode::Digit1 => KeyCode::One,
            WinitKeyCode::Digit2 => KeyCode::Two,
            WinitKeyCode::Digit3 => KeyCode::Three,
            WinitKeyCode::Digit4 => KeyCode::Four,
            WinitKeyCode::Digit5 => KeyCode::Five,
            WinitKeyCode::Digit6 => KeyCode::Six,
            WinitKeyCode::Digit7 => KeyCode::Seven,
            WinitKeyCode::Digit8 => KeyCode::Eight,
            WinitKeyCode::Digit9 => KeyCode::Nine,
            // arrows
            WinitKeyCode::ArrowLeft => KeyCode::Left,
            WinitKeyCode::ArrowRight => KeyCode::Right,
            WinitKeyCode::ArrowUp => KeyCode::Up,
            WinitKeyCode::ArrowDown => KeyCode::Down,
            // function keys
            WinitKeyCode::F1 => KeyCode::F1,
            WinitKeyCode::F2 => KeyCode::F2,
            WinitKeyCode::F3 => KeyCode::F3,
            WinitKeyCode::F4 => KeyCode::F4,
            WinitKeyCode::F5 => KeyCode::F5,
            WinitKeyCode::F6 => KeyCode::F6,
            WinitKeyCode::F7 => KeyCode::F7,
            WinitKeyCode::F8 => KeyCode::F8,
            WinitKeyCode::F9 => KeyCode::F9,
            WinitKeyCode::F10 => KeyCode::F10,
            WinitKeyCode::F11 => KeyCode::F11,
            WinitKeyCode::F12 => KeyCode::F12,
            // control keys
            WinitKeyCode::Space => KeyCode::Space,
            WinitKeyCode::Enter => KeyCode::Enter,
            WinitKeyCode::Escape => KeyCode::Escape,
            WinitKeyCode::Backspace => KeyCode::Backspace,
            WinitKeyCode::Tab => KeyCode::Tab,
            WinitKeyCode::ShiftLeft => KeyCode::LShift,
            WinitKeyCode::ShiftRight => KeyCode::RShift,
            WinitKeyCode::ControlLeft => KeyCode::LCtrl,
            WinitKeyCode::ControlRight => KeyCode::RCtrl,
            WinitKeyCode::AltLeft => KeyCode::LAlt,
            WinitKeyCode::AltRight => KeyCode::RAlt,
            // navigation
            WinitKeyCode::Insert => KeyCode::Insert,
            WinitKeyCode::Delete => KeyCode::Delete,
            WinitKeyCode::Home => KeyCode::Home,
            WinitKeyCode::End => KeyCode::End,
            WinitKeyCode::PageUp => KeyCode::PageUp,
            WinitKeyCode::PageDown => KeyCode::PageDown,
            // everything else — pass through as raw scancode
            _ => KeyCode::Other(code as u32),
        },
        // non-standard key (no scancode mapping)
        PhysicalKey::Unidentified(_) => KeyCode::Other(0),
    };
    Some((key, pressed, text))
}

pub fn convert_mouse_button(button: WinitMouseButton, state: ElementState) -> (MouseButton, bool) {
    let pressed = state == ElementState::Pressed;
    let mouse_button = match button {
        WinitMouseButton::Left => MouseButton::Left,
        WinitMouseButton::Middle => MouseButton::Middle,
        WinitMouseButton::Right => MouseButton::Right,
        WinitMouseButton::Back => MouseButton::Back,
        WinitMouseButton::Forward => MouseButton::Forward,
        WinitMouseButton::Other(id) => MouseButton::Other(id),
    };

    (mouse_button, pressed)
}
