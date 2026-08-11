use std::{env, process::Command};

fn main() {
    let task = env::args()
        .nth(1)
        .unwrap_or_else(|| "no xtask command".to_string());

    match task.as_str() {
        "build" => build(),
        "run" => run(env::args()
            .nth(2)
            .unwrap_or_else(|| "".to_string())
            .as_str()),
        "check" => check(),
        "init-lua-project" => init_lua_project(),
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

fn run(game: &str) {
    match game {
        "test" => cargo(&["run", "--package", "game-example"]),
        "snake" => cargo(&["run", "--package", "snake-clone"]),
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

    if !std::path::Path::new("main.lua").exists() {
        std::fs::write(
            "main.lua",
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

    if !std::path::Path::new("main.rs").exists() {
        std::fs::write(
            "main.rs",
            r#"fn main_lua() {
    use engine_core::{GameConfig, lua::LuaApp, run};

    let config = GameConfig {
        window_title: "game example".to_string(),
        ..GameConfig::default()
    };

    run(
        LuaApp::new(concat!(env!("CARGO_MANIFEST_DIR"), "/src/main.lua")),
        config,
    )
    .unwrap();
}
"#,
        )
        .unwrap();
    }

    println!("Lua project initialized. Run your game with: cargo xtask run");
}

fn print_help() {
    println!(
        "Available commands:
- build: compile the engine-core crate
- run: run the game-example crate
- check: run cargo check and cargo clippy over the workspace
"
    )
}

fn cargo(args: &[&str]) {
    let cargo = env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let status = Command::new(cargo)
        .args(args)
        .status()
        .expect("failed to run cargo");
    assert!(status.success());
}
