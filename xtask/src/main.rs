use std::{env, process::Command};

fn main() {
    let task = env::args()
        .nth(1)
        .unwrap_or_else(|| "no xtask command".to_string());

    match task.as_str() {
        "build" => build(),
        "run" => run(
            env::args().nth(2).unwrap_or_default().as_str(),
            env::args()
                .nth(3)
                .unwrap_or_else(|| "opengl".to_string())
                .as_str(),
        ),
        "check" => check(),
        "init-lua-project" => init_lua_project(),
        "init-rust-project" => init_rust_project(),
        "no xtask command" => {
            println!("no xtask command was given");
            print_help();
        }
        other => {
            println!("there is no xtask command called {}", other);
            print_help();
        }
    }
}

fn build() {
    cargo(&["build", "--package", "engine-core"]);
}

fn run(game: &str, renderer: &str) {
    match (game, renderer) {
        (package, "opengl") => cargo(&["run", "--package", package]),
        (package, "vulkan") => cargo(&[
            "run",
            "--package",
            package,
            "--no-default-features",
            "--features",
            "vulkan",
        ]),
        _ => println!("There is no runnable package called {}", game),
    }
}

fn check() {
    cargo(&["check", "--workspace"]);
    cargo(&["clippy", "--workspace"]);
}

fn init_lua_project() {
    std::fs::create_dir_all("definitions").unwrap();
    std::fs::write("definitions/engine.lua", engine_core::lua::LUA_DEFINITIONS).unwrap();

    std::fs::write(
        ".luarc.json",
        r#"{
    "workspace.library": ["./definitions"],
    "runtime.version": "Lua 5.4"
}"#,
    )
    .unwrap();

    std::fs::create_dir_all("src").unwrap();

    if !std::path::Path::new("src/main.lua").exists() {
        std::fs::write(
            "src/main.lua",
            r#"function on_start()
        -- initialization code
end

function on_update(dt)
-- code executed every frame
end

function on_fixed_update(dt)
-- physics related code (capped delta time) (not obligatory to use)
end

function on_render()
-- code executed every frame related to visuals (not obligatory to use)

function on_stop()
-- code executed on close (not obligatory to use)
end

function on_resize(width, height)
-- code executed every time the resize event is dispatched (not obligatory to use)
end
"#,
        )
        .unwrap();
    }

    assets_rs();

    if !std::path::Path::new("src/main.rs").exists() {
        std::fs::write(
            "src/main.rs",
            r#"use engine_core::{FilesystemAssets, GameConfig, lua::LuaApp, run};

mod assets;
use assets::EmbeddedAssets;

fn main() {
    let config = GameConfig {
        window_title: "generic_title".to_string(),
        ..GameConfig::default()
    };

    #[cfg(debug_assertions)]
    {
        let script_path = concat!(env!("CARGO_MANIFEST_DIR"), "/src/main.lua");
        run(LuaApp::new(script_path).with_hot_reload(), config, FilesystemAssets).unwrap();
    }

    #[cfg(not(debug_assertions))]
    {
        run(LuaApp::new("src/main.lua", config, EmbeddedAssets).unwrap();
    }
}
"#,
        )
        .unwrap();
    }

    println!("Lua project initialized. Run your game with: cargo xtask run");
    println!(
        "Note: add `rust-embed = \"8\"` to this project's Cargo.toml dependencies for release builds."
    );
}

fn init_rust_project() {
    std::fs::create_dir_all("src").unwrap();

    assets_rs();

    if !std::path::Path::new("src/main.rs").exists() {
        std::fs::write(
            "src/main.rs",
            r#"use engine_core::{FilesystemAssets, GameConfig, lua::LuaApp, run};

mod assets;
use assets::EmbeddedAssets;

fn main() {
    let config = GameConfig {
        window_title: "generic_title".to_string(),
        ..GameConfig::default()
    };

    #[cfg(debug_assertions)]
    {
        todo!("impl App trait for your game and substitute it below")
        run(App::new(config, FilesystemAssets)).unwrap();
    }

    #[cfg(not(debug_assertions))]
    {
        todo!("impl App trait for your game and substitute it below")
        run(App::new(config, EmbeddedAssets)).unwrap();
    }
}
"#,
        )
        .unwrap();
    }

    println!("Rust project initialized. Run your game with: cargo xtask run");
    println!(
        "Note: add `rust-embed = \"8\"` to this project's Cargo.toml dependencies for release builds."
    );
}

fn print_help() {
    println!(
        "Available commands:
- build: compile the engine-core crate
- run $game_crate_name ($renderer): run the game-crate (with an specific renderer if selected. defaults to opengl)
- check: run cargo check and cargo clippy over the workspace
- init-lua-project: run inside a game-crate to initialize a lua project
- init-rust-project: run inside a game-crate to initialize a rust project
"
    )
}

fn assets_rs() {
    if !std::path::Path::new("src/assets.rs").exists() {
        std::fs::write(
            "src/assets.rs",
            r#"use engine_core::AssetSource;
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "."]
struct GameAssets;

pub struct EmbeddedAssets;

impl AssetSource for EmbeddedAssets {
    fn read(&self, path: &str) -> Option<Vec<u8>> {
        GameAssets::get(path).map(|f| f.data.into_owned())
    }
}
"#,
        )
        .unwrap();
    }
}

fn cargo(args: &[&str]) {
    let cargo = env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let status = Command::new(cargo)
        .args(args)
        .status()
        .expect("failed to run cargo");
    assert!(status.success());
}
