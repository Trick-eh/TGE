use std::collections::HashMap;

use kira::{
    manager::{AudioManager as KiraManager, AudioManagerSettings, backend::DefaultBackend},
    sound::{
        FromFileError,
        static_sound::{StaticSoundData, StaticSoundSettings},
        streaming::{StreamingSoundData, StreamingSoundHandle},
    },
    track::{TrackBuilder, TrackHandle},
    tween::Tween,
};

#[derive(Clone)]
pub struct SoundHandle(usize);
#[derive(Clone)]
pub struct MusicHandle(usize);

pub struct AudioManager {
    manager: KiraManager<DefaultBackend>,
    sounds: Vec<StaticSoundData>,
    music_bytes: Vec<&'static [u8]>,
    sfx_track: TrackHandle,
    music_track: TrackHandle,
    current_music: Option<StreamingSoundHandle<FromFileError>>,
}

impl AudioManager {
    pub fn new() -> Self {
        let mut manager =
            KiraManager::new(AudioManagerSettings::default()).expect("Failed to initialize audio");
        let sfx_track = manager
            .add_sub_track(TrackBuilder::default())
            .expect("Failed to create SFX track");
        let music_track = manager
            .add_sub_track(TrackBuilder::default())
            .expect("Failed to create music track");

        AudioManager {
            manager,
            sounds: Vec::new(),
            music_bytes: Vec::new(),
            sfx_track,
            music_track,
            current_music: None,
        }
    }

    pub fn load_sound(&mut self, bytes: &'static [u8]) -> SoundHandle {
        let data = StaticSoundData::from_cursor(std::io::Cursor::new(bytes))
            .expect("Failed to load sound");
        let index = self.sounds.len();
        self.sounds.push(data);
        SoundHandle(index)
    }
    pub fn play_sound(&mut self, handle: &SoundHandle) {
        let settings = StaticSoundSettings::new().output_destination(&self.sfx_track);
        let data = self.sounds[handle.0].clone().with_settings(settings);

        self.manager.play(data).expect("Failed to play sound");
    }
    pub fn play_sound_with_volume(&mut self, handle: &SoundHandle, volume: f64) {
        let settings = StaticSoundSettings::new().volume(volume);
        let data = self.sounds[handle.0].clone().with_settings(settings);
        self.manager.play(data).expect("Failed to play sound");
    }

    pub fn load_music(&mut self, bytes: &'static [u8]) -> MusicHandle {
        let index = self.music_bytes.len();
        self.music_bytes.push(bytes);
        MusicHandle(index)
    }
    pub fn play_music(&mut self, handle: &MusicHandle) {
        self.stop_music();

        let data =
            StreamingSoundData::from_cursor(std::io::Cursor::new(self.music_bytes[handle.0]))
                .expect("Failed to load music")
                .output_destination(&self.music_track)
                .loop_region(..);

        let music_handle = self.manager.play(data).expect("Failed to play music");
        self.current_music = Some(music_handle);
    }
    pub fn pause_music(&mut self) {
        if let Some(music) = &mut self.current_music {
            music.pause(Tween::default());
        }
    }
    pub fn resume_music(&mut self) {
        if let Some(music) = &mut self.current_music {
            music.resume(Tween::default());
        }
    }
    pub fn stop_music(&mut self) {
        if let Some(mut music) = self.current_music.take() {
            music.stop(Tween::default());
        }
    }

    pub fn set_master_volume(&mut self, volume: f64) {
        self.manager
            .main_track()
            .set_volume(volume, Tween::default());
    }
    pub fn set_sfx_volume(&mut self, volume: f64) {
        self.sfx_track.set_volume(volume, Tween::default());
    }
    pub fn set_music_volume(&mut self, volume: f64) {
        self.music_track.set_volume(volume, Tween::default());
    }
}

pub struct AudioAssets {
    pub sounds: HashMap<String, SoundHandle>,
    pub music: HashMap<String, MusicHandle>,
}
impl AudioAssets {
    pub fn new() -> AudioAssets {
        AudioAssets {
            sounds: HashMap::new(),
            music: HashMap::new(),
        }
    }
    pub fn add_sound(&mut self, name: &str, handle: SoundHandle) {
        self.sounds.insert(name.to_string(), handle);
    }
    pub fn get_sound(&self, name: &str) -> Option<&SoundHandle> {
        self.sounds.get(name)
    }
    pub fn add_music(&mut self, name: &str, handle: MusicHandle) {
        self.music.insert(name.to_string(), handle);
    }
    pub fn get_music(&self, name: &str) -> Option<&MusicHandle> {
        self.music.get(name)
    }
}
