use std::collections::HashMap;

use engine_math::Vec2;

pub struct InputState {
    keys: HashMap<KeyCode, KeyState>,
    mouse_buttons: HashMap<MouseButton, KeyState>,
    gamepad_buttons: HashMap<GamepadButton, KeyState>,
    gamepad_axes: HashMap<GamepadAxis, f32>,
    mouse_position: Vec2,
    scroll_delta: Vec2,
    action_map: ActionMap,
    gilrs: Option<gilrs::Gilrs>,
    text_input_active: bool,
    text_input_buffer: String,
}

pub struct ActionMap {
    bindings: HashMap<String, Vec<InputBinding>>,
}

#[derive(Clone, Copy, PartialEq)]
pub enum KeyState {
    Up,
    Pressed,
    Held,
    Released,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyCode {
    // controls keys
    Space,
    Enter,
    Escape,
    Backspace,
    Tab,
    LShift,
    RShift,
    LCtrl,
    RCtrl,
    LAlt,
    RAlt,
    // numbers
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Zero,
    // letters
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
    // arrows
    Left,
    Up,
    Down,
    Right,
    // f keys
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    // navigation
    Insert,
    Delete,
    Home,
    End,
    PageUp,
    PageDown,
    // other
    Other(u32),
}
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum MouseButton {
    Left,
    Middle,
    Right,
    Back,
    Forward,
    Other(u16),
}
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum InputBinding {
    Key(KeyCode),
    MouseButton(MouseButton),
    GamepadButton(GamepadButton),
}
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum GamepadButton {
    South,
    North,
    East,
    West,
    LBumper,
    RBumper,
    LTrigger,
    RTrigger,
    Start,
    Select,
    DPadUp,
    DPadDown,
    DPadLeft,
    DPadRight,
    LStick,
    RStick,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum GamepadAxis {
    LeftStickX,
    LeftStickY,
    RightStickX,
    RightStickY,
    LeftTrigger,
    RightTrigger,
}

impl InputState {
    pub fn new() -> InputState {
        InputState {
            keys: HashMap::new(),
            mouse_position: Vec2::ZERO,
            mouse_buttons: HashMap::new(),
            scroll_delta: Vec2::ZERO,
            action_map: ActionMap::new(),
            gamepad_buttons: HashMap::new(),
            gamepad_axes: HashMap::new(),
            gilrs: gilrs::Gilrs::new().ok(),
            text_input_active: false,
            text_input_buffer: String::new(),
        }
    }

    pub fn process_key_event(&mut self, key: KeyCode, pressed: bool) {
        let current_state = self.keys.get(&key).copied().unwrap_or(KeyState::Up);
        let new_state = match (current_state, pressed) {
            (KeyState::Up, true) => KeyState::Pressed,
            (KeyState::Pressed, true) => KeyState::Held,
            (KeyState::Held, true) => KeyState::Held,
            (KeyState::Pressed, false) => KeyState::Released,
            (KeyState::Held, false) => KeyState::Released,
            (KeyState::Released, true) => KeyState::Pressed,
            (KeyState::Released, false) => KeyState::Released,
            (KeyState::Up, false) => KeyState::Up,
        };
        self.keys.insert(key, new_state);
    }
    pub fn is_key_held(&self, key: KeyCode) -> bool {
        matches!(
            self.keys.get(&key),
            Some(KeyState::Held) | Some(KeyState::Pressed)
        )
    }
    pub fn is_key_pressed(&self, key: KeyCode) -> bool {
        matches!(self.keys.get(&key), Some(KeyState::Pressed))
    }
    pub fn is_key_released(&self, key: KeyCode) -> bool {
        matches!(self.keys.get(&key), Some(KeyState::Released))
    }

    pub fn process_mouse_button(&mut self, button: MouseButton, pressed: bool) {
        let current_state = self
            .mouse_buttons
            .get(&button)
            .copied()
            .unwrap_or(KeyState::Up);
        let new_state = match (current_state, pressed) {
            (KeyState::Up, true) => KeyState::Pressed,
            (KeyState::Pressed, true) => KeyState::Held,
            (KeyState::Held, true) => KeyState::Held,
            (KeyState::Pressed, false) => KeyState::Released,
            (KeyState::Held, false) => KeyState::Released,
            (KeyState::Released, true) => KeyState::Pressed,
            (KeyState::Released, false) => KeyState::Released,
            (KeyState::Up, false) => KeyState::Up,
        };
        self.mouse_buttons.insert(button, new_state);
    }
    pub fn is_mouse_button_held(&self, button: MouseButton) -> bool {
        matches!(
            self.mouse_buttons.get(&button),
            Some(KeyState::Held) | Some(KeyState::Pressed)
        )
    }
    pub fn is_mouse_button_pressed(&self, button: MouseButton) -> bool {
        matches!(self.mouse_buttons.get(&button), Some(KeyState::Pressed))
    }
    pub fn is_mouse_button_released(&self, button: MouseButton) -> bool {
        matches!(self.mouse_buttons.get(&button), Some(KeyState::Released))
    }

    pub fn flush(&mut self) {
        flush_button_map(&mut self.keys);
        flush_button_map(&mut self.mouse_buttons);
        flush_button_map(&mut self.gamepad_buttons);

        self.scroll_delta = Vec2::ZERO;
    }

    pub fn set_mouse_position(&mut self, x: f32, y: f32) {
        self.mouse_position = Vec2::new(x, y)
    }
    pub fn mouse_position(&self) -> Vec2 {
        self.mouse_position
    }
    pub fn add_scroll_delta(&mut self, dx: f32, dy: f32) {
        self.scroll_delta.x += dx;
        self.scroll_delta.y += dy;
    }
    pub fn scroll_delta(&self) -> Vec2 {
        self.scroll_delta
    }

    pub fn start_text_input(&mut self, initial: &str) {
        self.text_input_active = true;
        self.text_input_buffer = initial.to_string();
    }
    pub fn stop_text_input(&mut self) {
        self.text_input_active = false;
    }
    pub fn is_text_input_active(&self) -> bool {
        self.text_input_active
    }
    pub fn text_input_buffer(&self) -> &str {
        &self.text_input_buffer
    }
    pub fn set_text_input_buffer(&mut self, text: &str) {
        self.text_input_buffer = text.to_string();
    }
    pub fn process_text_input(&mut self, text: &str) {
        if !self.text_input_active {
            return;
        }
        for c in text.chars() {
            if !c.is_control() {
                self.text_input_buffer.push(c);
            }
        }
    }
    pub fn text_input_backspace(&mut self) {
        if self.text_input_active {
            self.text_input_buffer.pop();
        }
    }

    pub fn is_action_held(&self, action: &str) -> bool {
        self.action_map
            .bindings
            .get(action)
            .map(|bindings| {
                bindings.iter().any(|binding| match binding {
                    InputBinding::Key(keycode) => self.is_key_held(*keycode),
                    InputBinding::MouseButton(button) => self.is_mouse_button_held(*button),
                    InputBinding::GamepadButton(button) => self.is_gamepad_button_held(*button),
                })
            })
            .unwrap_or(false)
    }
    pub fn is_action_pressed(&self, action: &str) -> bool {
        self.action_map
            .bindings
            .get(action)
            .map(|bindings| {
                bindings.iter().any(|binding| match binding {
                    InputBinding::Key(keycode) => self.is_key_pressed(*keycode),
                    InputBinding::MouseButton(button) => self.is_mouse_button_pressed(*button),
                    InputBinding::GamepadButton(button) => self.is_gamepad_button_pressed(*button),
                })
            })
            .unwrap_or(false)
    }
    pub fn is_action_released(&self, action: &str) -> bool {
        self.action_map
            .bindings
            .get(action)
            .map(|bindings| {
                bindings.iter().any(|binding| match binding {
                    InputBinding::Key(keycode) => self.is_key_released(*keycode),
                    InputBinding::MouseButton(button) => self.is_mouse_button_released(*button),
                    InputBinding::GamepadButton(button) => self.is_gamepad_button_released(*button),
                })
            })
            .unwrap_or(false)
    }

    pub fn bind_action(&mut self, action: &str, binding: InputBinding) {
        self.action_map
            .bindings
            .entry(action.to_string())
            .or_insert_with(Vec::new)
            .push(binding);
    }
    pub fn unbind_action(&mut self, action: &str) {
        self.action_map.bindings.remove(action);
    }

    pub fn poll_gamepad_events(&mut self) {
        let Some(gilrs) = &mut self.gilrs else {
            return;
        };

        let mut events = Vec::new();
        while let Some(gilrs::Event { event, .. }) = gilrs.next_event() {
            events.push(event);
        }

        for event in events {
            match event {
                gilrs::EventType::ButtonPressed(button, _) => {
                    if let Some(btn) = convert_gamepad_button(button) {
                        self.process_gamepad_button(btn, true);
                    }
                }
                gilrs::EventType::ButtonReleased(button, _) => {
                    if let Some(btn) = convert_gamepad_button(button) {
                        self.process_gamepad_button(btn, false);
                    }
                }
                gilrs::EventType::AxisChanged(axis, value, _) => {
                    if let Some(ax) = convert_gamepad_axis(axis) {
                        self.gamepad_axes.insert(ax, value);
                    }
                }
                _ => {}
            }
        }
    }

    pub fn process_gamepad_button(&mut self, button: GamepadButton, pressed: bool) {
        let current = self
            .gamepad_buttons
            .get(&button)
            .copied()
            .unwrap_or(KeyState::Up);
        let new_state = match (current, pressed) {
            (KeyState::Up, true) => KeyState::Pressed,
            (KeyState::Pressed, true) => KeyState::Held,
            (KeyState::Held, true) => KeyState::Held,
            (KeyState::Pressed, false) => KeyState::Released,
            (KeyState::Held, false) => KeyState::Released,
            (KeyState::Released, true) => KeyState::Pressed,
            (KeyState::Released, false) => KeyState::Released,
            (KeyState::Up, false) => KeyState::Up,
        };
        self.gamepad_buttons.insert(button, new_state);
    }
    pub fn is_gamepad_button_held(&self, button: GamepadButton) -> bool {
        matches!(
            self.gamepad_buttons.get(&button),
            Some(KeyState::Held) | Some(KeyState::Pressed)
        )
    }
    pub fn is_gamepad_button_pressed(&self, button: GamepadButton) -> bool {
        matches!(self.gamepad_buttons.get(&button), Some(KeyState::Pressed))
    }
    pub fn is_gamepad_button_released(&self, button: GamepadButton) -> bool {
        matches!(self.gamepad_buttons.get(&button), Some(KeyState::Released))
    }

    pub fn gamepad_axis(&self, axis: GamepadAxis) -> f32 {
        const DEAD_ZONE: f32 = 0.1;
        let value = self.gamepad_axes.get(&axis).copied().unwrap_or(0.0);
        if value.abs() < DEAD_ZONE { 0.0 } else { value }
    }
}
impl ActionMap {
    fn new() -> ActionMap {
        ActionMap {
            bindings: HashMap::new(),
        }
    }
}

fn convert_gamepad_button(button: gilrs::Button) -> Option<GamepadButton> {
    match button {
        gilrs::Button::South => Some(GamepadButton::South),
        gilrs::Button::North => Some(GamepadButton::North),
        gilrs::Button::East => Some(GamepadButton::East),
        gilrs::Button::West => Some(GamepadButton::West),
        gilrs::Button::LeftTrigger => Some(GamepadButton::LBumper),
        gilrs::Button::RightTrigger => Some(GamepadButton::RBumper),
        gilrs::Button::LeftTrigger2 => Some(GamepadButton::LTrigger),
        gilrs::Button::RightTrigger2 => Some(GamepadButton::RTrigger),
        gilrs::Button::Start => Some(GamepadButton::Start),
        gilrs::Button::Select => Some(GamepadButton::Select),
        gilrs::Button::DPadUp => Some(GamepadButton::DPadUp),
        gilrs::Button::DPadDown => Some(GamepadButton::DPadDown),
        gilrs::Button::DPadLeft => Some(GamepadButton::DPadLeft),
        gilrs::Button::DPadRight => Some(GamepadButton::DPadRight),
        gilrs::Button::LeftThumb => Some(GamepadButton::LStick),
        gilrs::Button::RightThumb => Some(GamepadButton::RStick),
        _ => None,
    }
}
fn convert_gamepad_axis(axis: gilrs::Axis) -> Option<GamepadAxis> {
    match axis {
        gilrs::Axis::LeftStickX => Some(GamepadAxis::LeftStickX),
        gilrs::Axis::LeftStickY => Some(GamepadAxis::LeftStickY),
        gilrs::Axis::RightStickX => Some(GamepadAxis::RightStickX),
        gilrs::Axis::RightStickY => Some(GamepadAxis::RightStickY),
        gilrs::Axis::LeftZ => Some(GamepadAxis::LeftTrigger),
        gilrs::Axis::RightZ => Some(GamepadAxis::RightTrigger),
        _ => None,
    }
}
fn flush_button_map<T: Eq + std::hash::Hash>(map: &mut HashMap<T, KeyState>) {
    for state in map.values_mut() {
        *state = match *state {
            KeyState::Pressed => KeyState::Held,
            KeyState::Released => KeyState::Up,
            other => other,
        };
    }
}
