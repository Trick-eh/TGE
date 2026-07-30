use std::{collections::HashMap, f32::consts::PI};

use engine_audio::AudioAssets;
use engine_core::{App, FixedContext, GameConfig, RenderContext, StartContext, UpdateContext, run};
use engine_ecs::{ActiveCamera, Player, Velocity};
use engine_input::KeyCode;
use engine_math::{Camera2D, Transform2D, Vec2};
use engine_renderer::{AnimatedSprite, Sprite, SpriteSheetHandle};

struct MyGame {
    spritesheets: HashMap<String, SpriteSheetHandle>,
}

impl App for MyGame {
    fn on_start(&mut self, ctx: &mut StartContext) {
        {
            ctx.input
                .bind_action("move_up", engine_input::InputBinding::Key(KeyCode::Up));
            ctx.input
                .bind_action("move_down", engine_input::InputBinding::Key(KeyCode::Down));
            ctx.input
                .bind_action("move_left", engine_input::InputBinding::Key(KeyCode::Left));
            ctx.input.bind_action(
                "move_right",
                engine_input::InputBinding::Key(KeyCode::Right),
            );
            ctx.input
                .bind_action("rotate_left", engine_input::InputBinding::Key(KeyCode::D));
            ctx.input
                .bind_action("rotate_right", engine_input::InputBinding::Key(KeyCode::A));
        }
        ctx.schedule.add_update_system(player_movement_system);

        let sprite_sheet_name = "test".to_string();
        let texture = ctx
            .renderer
            .load_texture(include_bytes!("../assets/test.png"));

        self.spritesheets.insert(
            sprite_sheet_name,
            ctx.renderer.create_sprite_sheet(texture, 90, 90),
        );
        {
            ctx.audio_assets.add_sound(
                "rotate",
                ctx.audio.load_sound(include_bytes!("../assets/rotate.mp3")),
            );
            ctx.audio_assets.add_music(
                "bg",
                ctx.audio
                    .load_music(include_bytes!("../assets/background.mp3")),
            );
        }

        {
            ctx.world.spawn((
                Camera2D {
                    position: Vec2::ZERO,
                    zoom: 1.0,
                },
                ActiveCamera,
            ));
            ctx.world.spawn((
                Transform2D {
                    position: Vec2 { x: 100.0, y: 0.0 },
                    rotation_in_radians: PI / 4.0,
                    scale: Vec2 { x: 200.0, y: 100.0 },
                },
                Sprite {
                    sprite_sheet: *self.spritesheets.get("test").unwrap(),
                    index: 0,
                },
            ));
            ctx.world.spawn((
                Transform2D {
                    position: Vec2 { x: 300.0, y: 200.0 },
                    rotation_in_radians: PI / 5.0 + 1.0,
                    scale: Vec2 { x: 50.0, y: 50.0 },
                },
                Sprite {
                    sprite_sheet: *self.spritesheets.get("test").unwrap(),
                    index: 1,
                },
            ));
            ctx.world.spawn((
                Transform2D {
                    position: Vec2 {
                        x: -100.0,
                        y: -200.0,
                    },
                    rotation_in_radians: PI / 3.0,
                    scale: Vec2 { x: 50.0, y: 100.0 },
                },
                Sprite {
                    sprite_sheet: *self.spritesheets.get("test").unwrap(),
                    index: 2,
                },
            ));
            ctx.world.spawn((
                Transform2D {
                    position: Vec2 { x: 0.0, y: 0.0 },
                    rotation_in_radians: 0.0,
                    scale: Vec2 { x: 100.0, y: 100.0 },
                },
                AnimatedSprite {
                    sprite_sheet: *self.spritesheets.get("test").unwrap(),
                    frames: vec![0, 1, 2, 3],
                    frame_duration: 0.5,
                    current_frame: 0,
                    timer: 0.0,
                    looping: true,
                },
                Velocity {
                    value: Vec2 { x: 120.0, y: 120.0 },
                },
                Player,
            ));
        }
        if let Some(handle) = ctx.audio_assets.music.get("bg") {
            ctx.audio.play_music(handle);
        }
    }

    fn on_fixed_update(&mut self, _ctx: &mut FixedContext) {}

    fn on_update(&mut self, _ctx: &mut UpdateContext) {}

    fn on_render(&mut self, _ctx: &mut RenderContext) {}
    fn on_stop(&mut self) {
        println!("stopped")
    }

    fn on_resize(&mut self, _width: u32, _height: u32) {}
}

fn player_movement_system(ctx: &mut UpdateContext) {
    let query = ctx
        .world
        .query_mut::<(&mut Player, &mut Transform2D, &mut Velocity)>();
    let dt = ctx.time.dt;
    for (_, transform, velocity) in query {
        if ctx.input.is_action_held("move_up") {
            transform.position.y += velocity.value.y * dt;
        }
        if ctx.input.is_action_held("move_left") {
            transform.position.x -= velocity.value.x * dt;
        }
        if ctx.input.is_action_held("move_down") {
            transform.position.y -= velocity.value.y * dt;
        }
        if ctx.input.is_action_held("move_right") {
            transform.position.x += velocity.value.x * dt;
        }
        if ctx.input.is_action_held("rotate_left") {
            transform.rotation_in_radians -= 0.10 * dt;
            if let Some(handle) = ctx.audio_assets.get_sound("rotate") {
                ctx.audio.play_sound(handle);
            }
        }
        if ctx.input.is_action_held("rotate_right") {
            transform.rotation_in_radians += 0.10 * dt;
            if let Some(handle) = ctx.audio_assets.get_sound("rotate") {
                ctx.audio.play_sound(handle);
            }
        }
    }
}

fn main() {
    let config = GameConfig {
        window_title: "game example".to_string(),
        music_volume: 0.2,
        ..GameConfig::default()
    };
    run(
        MyGame {
            spritesheets: HashMap::new(),
        },
        config,
    )
    .unwrap();
}
