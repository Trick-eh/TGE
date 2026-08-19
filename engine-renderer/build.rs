use std::{env, path::Path, process::Command};

fn main() {
    if env::var("CARGO_FEATURE_VULKAN").is_err() {
        return;
    }

    let out_dir = env::var("OUT_DIR").expect("OUT_DIR not set by cargo");
    let shader_dir = Path::new("src/vulkan/shaders");

    let shaders = ["quad.vert", "quad.frag"];

    for shader in shaders {
        let src_path = shader_dir.join(shader);
        let out_path = Path::new(&out_dir).join(format!("{shader}.spv"));

        println!("cargo:rerun-if-changed={}", src_path.display());

        let status = Command::new("glslc")
            .arg(&src_path)
            .arg("-o")
            .arg(&out_path)
            .status()
            .unwrap_or_else(|e| {
                panic!(
                    "Failed to run glslc (is it installed and on PATH? \
                error: {e}"
                )
            });

        if !status.success() {
            panic!("glslc failed to compile {}", src_path.display());
        }
    }
}
