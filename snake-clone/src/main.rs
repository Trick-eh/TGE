use engine_core::{GameConfig, lua::LuaApp, run};

fn main() {
    run(
        LuaApp::new(concat!(env!("CARGO_MANIFEST_DIR"), "/src/main.lua")),
        GameConfig {
            window_title: "Snake".to_string(),
            ..GameConfig::default()
        },
    )
    .unwrap();
}
