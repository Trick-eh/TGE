use engine_core::{FilesystemAssets, GameConfig, lua::LuaApp, run};

mod assets;
use assets::EmbeddedAssets;

fn main() {
    let config = GameConfig {
        window_title: "Snake".to_string(),
        ..GameConfig::default()
    };

    #[cfg(debug_assertions)]
    {
        let script_path = concat!(env!("CARGO_MANIFEST_DIR"), "/src/main.lua");
        run(
            LuaApp::new(script_path).with_hot_reload(),
            config,
            FilesystemAssets,
        )
        .unwrap();
    }

    #[cfg(not(debug_assertions))]
    {
        run(LuaApp::new("src/main.lua"), config, EmbeddedAssets).unwrap();
    }
}
