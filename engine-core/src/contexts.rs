use engine_audio::{AudioAssets, AudioManager};
use engine_ecs::World;
use engine_input::InputState;
use engine_renderer::Renderer;

use crate::{GameConfig, SystemSchedule, Time, save::SaveData};

pub struct StartContext<'a> {
    pub schedule: &'a mut SystemSchedule,
    pub world: &'a mut World,
    pub time: &'a mut Time,
    pub input: &'a mut InputState,
    pub renderer: &'a mut dyn Renderer,
    pub audio: &'a mut AudioManager,
    pub audio_assets: &'a mut AudioAssets,
    pub save_data: &'a mut SaveData,
}
pub struct UpdateContext<'a> {
    pub world: &'a mut World,
    pub time: &'a mut Time,
    pub input: &'a mut InputState,
    pub audio: &'a mut AudioManager,
    pub audio_assets: &'a AudioAssets,
    pub config: &'a GameConfig,
    pub save_data: &'a mut SaveData,
}
pub struct FixedContext<'a> {
    pub world: &'a mut World,
    pub time: &'a mut Time,
    pub audio: &'a mut AudioManager,
    pub audio_assets: &'a AudioAssets,
    pub input: &'a mut InputState,
    pub save_data: &'a mut SaveData,
}
pub struct RenderContext<'a> {
    pub world: &'a mut World,
    pub time: &'a mut Time,
    pub renderer: &'a mut dyn Renderer,
    pub audio: &'a mut AudioManager,
    pub input: &'a mut InputState,
}
