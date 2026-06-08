use std::{env, process::Command};

fn main() {
    let task = env::args()
        .nth(1)
        .unwrap_or_else(|| "no xtask command".to_string());

    match task.as_str() {
        "build" => build(),
        "run" => run(),
        "check" => check(),
        "no xtask command" => {
            println!("no xtask command was given");
            print_help();
        }
        other => {
            println!("the is no xtask command called {}", other);
            print_help();
        }
    }
}

fn build() {
    cargo(&["build", "--package", "engine-core"]);
}

fn run() {
    cargo(&["run", "--package", "game-example"]);
}

fn check() {
    cargo(&["check", "--workspace"]);
    cargo(&["clippy", "--workspace"]);
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
