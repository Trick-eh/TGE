use crate::lua::context::{with_fixed_ctx, with_start_ctx, with_update_ctx};
use engine_audio::{AudioAssets, AudioManager};
use mlua::prelude::*;

pub fn register(lua: &Lua, engine: &LuaTable) -> LuaResult<()> {
    engine.set(
        "load_sound",
        lua.create_function(|_, (name, path): (String, String)| {
            with_start_ctx(|ctx| {
                let bytes = std::fs::read(&path)
                    .unwrap_or_else(|_| panic!("Failed to read sound file: {}", path));
                let handle = ctx.audio.load_sound(bytes);
                ctx.audio_assets.add_sound(&name, handle);
            });
            Ok(())
        })?,
    )?;

    engine.set(
        "load_music",
        lua.create_function(|_, (name, path): (String, String)| {
            with_start_ctx(|ctx| {
                println!("{}", "game-example".to_owned() + &path);
                let bytes = std::fs::read(&path)
                    .unwrap_or_else(|_| panic!("Failed to read music file: {}", path));
                let handle = ctx.audio.load_music(bytes);
                ctx.audio_assets.add_music(&name, handle);
            });
            Ok(())
        })?,
    )?;

    engine.set(
        "play_sound",
        lua.create_function(|_, name: String| {
            fn play(audio: &mut AudioManager, assets: &AudioAssets, name: &str) {
                if let Some(handle) = assets.get_sound(&name) {
                    audio.play_sound(handle);
                } else {
                    eprintln!("Lua audio: sound '{}' not found", name);
                }
            }
            with_start_ctx(|ctx| play(&mut ctx.audio, &ctx.audio_assets, &name))
                .or_else(|| with_update_ctx(|ctx| play(&mut ctx.audio, &ctx.audio_assets, &name)))
                .or_else(|| with_fixed_ctx(|ctx| play(&mut ctx.audio, &ctx.audio_assets, &name)));

            Ok(())
        })?,
    )?;

    engine.set(
        "play_sound_with_volume",
        lua.create_function(|_, (name, volume): (String, f64)| {
            fn play(audio: &mut AudioManager, assets: &AudioAssets, name: &str, volume: f64) {
                if let Some(handle) = assets.get_sound(&name) {
                    audio.play_sound_with_volume(handle, volume);
                } else {
                    eprintln!("Lua audio: sound '{}' not found", name);
                }
            }

            with_start_ctx(|ctx| play(&mut ctx.audio, &ctx.audio_assets, &name, volume))
                .or_else(|| {
                    with_update_ctx(|ctx| play(&mut ctx.audio, &ctx.audio_assets, &name, volume))
                })
                .or_else(|| {
                    with_fixed_ctx(|ctx| play(&mut ctx.audio, &ctx.audio_assets, &name, volume))
                });

            Ok(())
        })?,
    )?;

    engine.set(
        "play_music",
        lua.create_function(|_, name: String| {
            fn play(audio: &mut AudioManager, assets: &AudioAssets, name: &str) {
                if let Some(handle) = assets.get_music(&name) {
                    audio.play_music(handle);
                } else {
                    eprintln!("Lua audio: music '{}' not found", name);
                }
            }
            with_start_ctx(|ctx| play(&mut ctx.audio, &ctx.audio_assets, &name))
                .or_else(|| with_update_ctx(|ctx| play(&mut ctx.audio, &ctx.audio_assets, &name)))
                .or_else(|| with_fixed_ctx(|ctx| play(&mut ctx.audio, &ctx.audio_assets, &name)));
            Ok(())
        })?,
    )?;

    engine.set(
        "pause_music",
        lua.create_function(|_, ()| {
            with_update_ctx(|ctx| ctx.audio.pause_music())
                .or_else(|| with_fixed_ctx(|ctx| ctx.audio.pause_music()));
            Ok(())
        })?,
    )?;

    engine.set(
        "resume_music",
        lua.create_function(|_, ()| {
            with_update_ctx(|ctx| ctx.audio.resume_music())
                .or_else(|| with_fixed_ctx(|ctx| ctx.audio.resume_music()));
            Ok(())
        })?,
    )?;

    engine.set(
        "stop_music",
        lua.create_function(|_, ()| {
            with_update_ctx(|ctx| ctx.audio.stop_music())
                .or_else(|| with_fixed_ctx(|ctx| ctx.audio.stop_music()));
            Ok(())
        })?,
    )?;

    engine.set(
        "set_master_volume",
        lua.create_function(|_, volume: f64| {
            with_start_ctx(|ctx| ctx.audio.set_master_volume(volume))
                .or_else(|| with_update_ctx(|ctx| ctx.audio.set_master_volume(volume)))
                .or_else(|| with_fixed_ctx(|ctx| ctx.audio.set_master_volume(volume)));
            Ok(())
        })?,
    )?;

    engine.set(
        "set_sfx_volume",
        lua.create_function(|_, volume: f64| {
            with_start_ctx(|ctx| ctx.audio.set_sfx_volume(volume))
                .or_else(|| with_update_ctx(|ctx| ctx.audio.set_sfx_volume(volume)))
                .or_else(|| with_fixed_ctx(|ctx| ctx.audio.set_sfx_volume(volume)));
            Ok(())
        })?,
    )?;

    engine.set(
        "set_music_volume",
        lua.create_function(|_, volume: f64| {
            with_start_ctx(|ctx| ctx.audio.set_music_volume(volume))
                .or_else(|| with_update_ctx(|ctx| ctx.audio.set_music_volume(volume)))
                .or_else(|| with_fixed_ctx(|ctx| ctx.audio.set_music_volume(volume)));
            Ok(())
        })?,
    )?;

    Ok(())
}
