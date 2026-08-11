use std::{collections::HashMap, path::PathBuf};

pub struct SaveData {
    data: HashMap<String, serde_json::Value>,
    path: PathBuf,
    dirty: bool,
}

impl SaveData {
    pub fn load(game_title: &str) -> Self {
        let path = save_path(game_title);
        let data = if path.exists() {
            std::fs::read_to_string(&path)
                .ok()
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default()
        } else {
            HashMap::new()
        };

        SaveData {
            data,
            path,
            dirty: false,
        }
    }

    pub fn set(&mut self, key: String, value: serde_json::Value) {
        self.data.insert(key, value);
        self.dirty = true;
    }

    pub fn get(&self, key: &str) -> Option<&serde_json::Value> {
        self.data.get(key)
    }

    pub fn flush(&mut self) {
        if !self.dirty {
            return;
        }
        if let Some(parent) = self.path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(json) = serde_json::to_string_pretty(&self.data) {
            let _ = std::fs::write(&self.path, json);
        }
        self.dirty = false;
    }
}

fn save_path(game_title: &str) -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(game_title)
        .join("save.json")
}
