use engine_math::Vec2;
pub use hecs::*;
use std::collections::HashMap;

pub struct LuaComponents {
    pub data: HashMap<String, LuaData>,
}

#[derive(Clone, Debug)]
pub enum LuaData {
    Number(f64),
    Bool(bool),
    String(String),
    Nil,
}

pub struct Velocity {
    pub value: Vec2,
}

pub struct Acceleration {
    pub value: Vec2,
}

pub struct Tag {
    pub value: String,
}

/// Marker for moving entities that may appear to stutter
pub struct PreviousTransform {
    pub position: Vec2,
    pub rotation: f32,
}

/// Zero sized marker, useful for querying when trying to sync input with render
pub struct Player;
/// Zero sized mmarker, used for having the camera as a world resource
pub struct ActiveCamera;
