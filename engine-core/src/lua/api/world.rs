use crate::lua::api::{from_lua_value, to_lua_value};
use crate::lua::context::{with_fixed_ctx, with_render_ctx, with_start_ctx, with_update_ctx};
use engine_ecs::{Entity, LuaComponents, LuaData, Player, PreviousTransform, Velocity, World};
use engine_math::{Transform2D, Vec2};
use engine_renderer::{AnimatedSprite, Renderer, Sprite, SpriteSheetHandle, TextureHandle};
use mlua::prelude::*;
use std::cell::RefCell;
use std::collections::HashMap;

thread_local! {
    static ENTITY_MAP: RefCell<HashMap<u64, Entity>> = RefCell::new(HashMap::new());
    static NEXT_ID: RefCell<u64> = RefCell::new(1);
}

fn next_lua_id() -> u64 {
    NEXT_ID.with(|n| {
        let id = *n.borrow();
        *n.borrow_mut() += 1;
        id
    })
}

fn register_entity(lua_id: u64, entity: Entity) {
    ENTITY_MAP.with(|m| m.borrow_mut().insert(lua_id, entity));
}

fn find_entity(lua_id: u64) -> Option<Entity> {
    ENTITY_MAP.with(|m| m.borrow().get(&lua_id).copied())
}

fn remove_entity(lua_id: u64) {
    ENTITY_MAP.with(|m| m.borrow_mut().remove(&lua_id));
}

pub fn register(lua: &Lua, engine: &LuaTable) -> LuaResult<()> {
    engine.set(
        "set_component",
        lua.create_function(|_, (lua_id, key, value): (u64, String, LuaValue)| {
            with_update_ctx(|ctx| {
                if let Some(entity) = find_entity(lua_id) {
                    let data = from_lua_value(value);
                    if let Ok(mut components) = ctx.world.get::<&mut LuaComponents>(entity) {
                        components.data.insert(key, data);
                    } else {
                        let mut map = HashMap::new();
                        map.insert(key, data);
                        let _ = ctx.world.insert_one(entity, LuaComponents { data: map });
                    }
                }
            });
            Ok(())
        })?,
    )?;

    engine.set(
        "get_component",
        lua.create_function(|lua, (lua_id, key): (u64, String)| {
            let result = with_update_ctx(|ctx| {
                find_entity(lua_id).and_then(|entity| {
                    ctx.world
                        .get::<&LuaComponents>(entity)
                        .ok()
                        .and_then(|c| c.data.get(&key).cloned())
                })
            })
            .flatten();

            match result {
                Some(data) => to_lua_value(&lua, &data),
                None => Ok(LuaValue::Nil),
            }
        })?,
    )?;

    engine.set(
        "has_component",
        lua.create_function(|_, (lua_id, key): (u64, String)| {
            Ok(with_update_ctx(|ctx| {
                find_entity(lua_id).and_then(|entity| {
                    ctx.world
                        .get::<&LuaComponents>(entity)
                        .ok()
                        .map(|c| c.data.contains_key(&key))
                })
            })
            .flatten()
            .unwrap_or(false))
        })?,
    )?;

    engine.set(
        "remove_component",
        lua.create_function(|_, (lua_id, key): (u64, String)| {
            with_update_ctx(|ctx| {
                if let Some(entity) = find_entity(lua_id) {
                    if let Ok(mut components) = ctx.world.get::<&mut LuaComponents>(entity) {
                        components.data.remove(&key);
                    }
                }
            });
            Ok(())
        })?,
    )?;

    engine.set(
        "spawn_entity",
        lua.create_function(|_, components: LuaTable| {
            let lua_id = next_lua_id();

            let did_spawn = with_start_ctx(|ctx| {
                let entity = spawn_from_table(&mut ctx.world, &components);
                register_entity(lua_id, entity);
            })
            .or_else(|| {
                with_update_ctx(|ctx| {
                    let entity = spawn_from_table(&mut ctx.world, &components);
                    register_entity(lua_id, entity);
                })
            })
            .or_else(|| {
                with_fixed_ctx(|ctx| {
                    let entity = spawn_from_table(&mut ctx.world, &components);
                    register_entity(lua_id, entity);
                })
            });

            if did_spawn.is_none() {
                eprintln!("Lua world: spawn_entity called outside valid context");
            }

            Ok(lua_id)
        })?,
    )?;

    engine.set(
        "despawn",
        lua.create_function(|_, lua_id: u64| {
            with_start_ctx(|ctx| {
                if let Some(entity) = find_entity(lua_id) {
                    let _ = ctx.world.despawn(entity);
                    remove_entity(lua_id);
                } else {
                    eprintln!("Lua world: despawn called with unknown id {}", lua_id);
                }
            })
            .or_else(|| {
                with_update_ctx(|ctx| {
                    if let Some(entity) = find_entity(lua_id) {
                        let _ = ctx.world.despawn(entity);
                        remove_entity(lua_id);
                    } else {
                        eprintln!("Lua world: despawn called with unknown id {}", lua_id);
                    }
                })
            })
            .or_else(|| {
                with_fixed_ctx(|ctx| {
                    if let Some(entity) = find_entity(lua_id) {
                        let _ = ctx.world.despawn(entity);
                        remove_entity(lua_id);
                    } else {
                        eprintln!("Lua world: despawn called with unknown id {}", lua_id);
                    }
                })
            });

            Ok(())
        })?,
    )?;

    engine.set(
        "get_position",
        lua.create_function(|lua, lua_id: u64| {
            let pos = with_update_ctx(|ctx| {
                find_entity(lua_id)
                    .and_then(|e| ctx.world.get::<&Transform2D>(e).ok().map(|t| t.position))
            })
            .flatten();

            match pos {
                Some(p) => {
                    let t = lua.create_table()?;
                    t.set("x", p.x)?;
                    t.set("y", p.y)?;
                    Ok(LuaValue::Table(t))
                }
                None => Ok(LuaValue::Nil),
            }
        })?,
    )?;

    engine.set(
        "set_position",
        lua.create_function(|_, (lua_id, x, y): (u64, f32, f32)| {
            with_start_ctx(|ctx| {
                if let Some(e) = find_entity(lua_id) {
                    if let Ok(mut t) = ctx.world.get::<&mut Transform2D>(e) {
                        t.position = Vec2::new(x, y);
                    }
                }
            })
            .or_else(|| {
                with_update_ctx(|ctx| {
                    if let Some(e) = find_entity(lua_id) {
                        if let Ok(mut t) = ctx.world.get::<&mut Transform2D>(e) {
                            t.position = Vec2::new(x, y);
                        }
                    }
                })
            })
            .or_else(|| {
                with_fixed_ctx(|ctx| {
                    if let Some(e) = find_entity(lua_id) {
                        if let Ok(mut t) = ctx.world.get::<&mut Transform2D>(e) {
                            t.position = Vec2::new(x, y);
                        }
                    }
                })
            });

            Ok(())
        })?,
    )?;

    engine.set(
        "get_scale",
        lua.create_function(|lua, lua_id: u64| {
            let scale = with_update_ctx(|ctx| {
                find_entity(lua_id)
                    .and_then(|e| ctx.world.get::<&Transform2D>(e).ok().map(|t| t.scale))
            })
            .flatten();

            match scale {
                Some(s) => {
                    let t = lua.create_table()?;
                    t.set("x", s.x)?;
                    t.set("y", s.y)?;
                    Ok(LuaValue::Table(t))
                }
                None => Ok(LuaValue::Nil),
            }
        })?,
    )?;

    engine.set(
        "set_scale",
        lua.create_function(|_, (lua_id, x, y): (u64, f32, f32)| {
            with_start_ctx(|ctx| {
                if let Some(e) = find_entity(lua_id) {
                    if let Ok(mut t) = ctx.world.get::<&mut Transform2D>(e) {
                        t.scale = Vec2::new(x, y);
                    }
                }
            })
            .or_else(|| {
                with_update_ctx(|ctx| {
                    if let Some(e) = find_entity(lua_id) {
                        if let Ok(mut t) = ctx.world.get::<&mut Transform2D>(e) {
                            t.scale = Vec2::new(x, y);
                        }
                    }
                })
            })
            .or_else(|| {
                with_fixed_ctx(|ctx| {
                    if let Some(e) = find_entity(lua_id) {
                        if let Ok(mut t) = ctx.world.get::<&mut Transform2D>(e) {
                            t.scale = Vec2::new(x, y);
                        }
                    }
                })
            });
            Ok(())
        })?,
    )?;

    engine.set(
        "get_rotation",
        lua.create_function(|_, lua_id: u64| {
            Ok(with_update_ctx(|ctx| {
                find_entity(lua_id).and_then(|e| {
                    ctx.world
                        .get::<&Transform2D>(e)
                        .ok()
                        .map(|t| t.rotation_in_radians)
                })
            })
            .flatten()
            .unwrap_or(0.0))
        })?,
    )?;

    engine.set(
        "set_rotation",
        lua.create_function(|_, (lua_id, radians): (u64, f32)| {
            with_start_ctx(|ctx| {
                if let Some(e) = find_entity(lua_id) {
                    if let Ok(mut t) = ctx.world.get::<&mut Transform2D>(e) {
                        t.rotation_in_radians = radians;
                    }
                }
            })
            .or_else(|| {
                with_update_ctx(|ctx| {
                    if let Some(e) = find_entity(lua_id) {
                        if let Ok(mut t) = ctx.world.get::<&mut Transform2D>(e) {
                            t.rotation_in_radians = radians;
                        }
                    }
                })
            })
            .or_else(|| {
                with_fixed_ctx(|ctx| {
                    if let Some(e) = find_entity(lua_id) {
                        if let Ok(mut t) = ctx.world.get::<&mut Transform2D>(e) {
                            t.rotation_in_radians = radians;
                        }
                    }
                })
            });
            Ok(())
        })?,
    )?;

    engine.set(
        "get_velocity",
        lua.create_function(|lua, lua_id: u64| {
            let vel = with_update_ctx(|ctx| {
                find_entity(lua_id)
                    .and_then(|e| ctx.world.get::<&Velocity>(e).ok().map(|v| v.value))
            })
            .flatten();

            match vel {
                Some(v) => {
                    let t = lua.create_table()?;
                    t.set("x", v.x)?;
                    t.set("y", v.y)?;
                    Ok(LuaValue::Table(t))
                }
                None => Ok(LuaValue::Nil),
            }
        })?,
    )?;

    engine.set(
        "set_velocity",
        lua.create_function(|_, (lua_id, x, y): (u64, f32, f32)| {
            with_start_ctx(|ctx| {
                if let Some(e) = find_entity(lua_id) {
                    if let Ok(mut v) = ctx.world.get::<&mut Velocity>(e) {
                        v.value = Vec2::new(x, y);
                    }
                }
            })
            .or_else(|| {
                with_update_ctx(|ctx| {
                    if let Some(e) = find_entity(lua_id) {
                        if let Ok(mut v) = ctx.world.get::<&mut Velocity>(e) {
                            v.value = Vec2::new(x, y);
                        }
                    }
                })
            })
            .or_else(|| {
                with_fixed_ctx(|ctx| {
                    if let Some(e) = find_entity(lua_id) {
                        if let Ok(mut v) = ctx.world.get::<&mut Velocity>(e) {
                            v.value = Vec2::new(x, y);
                        }
                    }
                })
            });
            Ok(())
        })?,
    )?;

    engine.set(
        "set_sprite",
        lua.create_function(|_, (lua_id, sheet_index, tile_index): (u64, u32, u32)| {
            fn insert(world: &mut World, lua_id: u64, sheet_index: u32, tile_index: u32) {
                if let Some(e) = find_entity(lua_id) {
                    let sprite = Sprite {
                        sprite_sheet: SpriteSheetHandle::from_lua_id(sheet_index),
                        index: tile_index,
                    };
                    let _ = world.insert_one(e, sprite);
                }
            }

            with_start_ctx(|ctx| insert(&mut ctx.world, lua_id, sheet_index, tile_index))
                .or_else(|| {
                    with_update_ctx(|ctx| insert(&mut ctx.world, lua_id, sheet_index, tile_index))
                })
                .or_else(|| {
                    with_fixed_ctx(|ctx| insert(&mut ctx.world, lua_id, sheet_index, tile_index))
                });

            Ok(())
        })?,
    )?;

    engine.set(
        "set_sprite_index",
        lua.create_function(|_, (lua_id, tile_index): (u64, u32)| {
            with_start_ctx(|ctx| {
                if let Some(e) = find_entity(lua_id) {
                    if let Ok(mut s) = ctx.world.get::<&mut Sprite>(e) {
                        s.index = tile_index;
                    }
                }
            })
            .or_else(|| {
                with_update_ctx(|ctx| {
                    if let Some(e) = find_entity(lua_id) {
                        if let Ok(mut s) = ctx.world.get::<&mut Sprite>(e) {
                            s.index = tile_index;
                        }
                    }
                })
            })
            .or_else(|| {
                with_fixed_ctx(|ctx| {
                    if let Some(e) = find_entity(lua_id) {
                        if let Ok(mut s) = ctx.world.get::<&mut Sprite>(e) {
                            s.index = tile_index;
                        }
                    }
                })
            });
            Ok(())
        })?,
    )?;

    engine.set(
        "set_animated_sprite",
        lua.create_function(
            |_,
             (lua_id, sheet_index, frames_table, frame_duration, looping): (
                u64,
                u32,
                LuaTable,
                f32,
                bool,
            )| {
                fn insert(
                    world: &mut World,
                    lua_id: u64,
                    sheet_index: u32,
                    frames: Vec<u32>,
                    frame_duration: f32,
                    looping: bool,
                ) {
                    if let Some(e) = find_entity(lua_id) {
                        let anim = AnimatedSprite {
                            sprite_sheet: SpriteSheetHandle::from_lua_id(sheet_index),
                            frames,
                            frame_duration,
                            current_frame: 0,
                            timer: 0.0,
                            looping,
                        };
                        let _ = world.insert_one(e, anim);
                    }
                }

                let frames: Vec<u32> = frames_table
                    .sequence_values::<u32>()
                    .filter_map(|r| r.ok())
                    .collect();

                with_start_ctx(|ctx| {
                    insert(
                        &mut ctx.world,
                        lua_id,
                        sheet_index,
                        frames.clone(),
                        frame_duration,
                        looping,
                    )
                })
                .or_else(|| {
                    with_update_ctx(|ctx| {
                        insert(
                            &mut ctx.world,
                            lua_id,
                            sheet_index,
                            frames.clone(),
                            frame_duration,
                            looping,
                        )
                    })
                })
                .or_else(|| {
                    with_fixed_ctx(|ctx| {
                        insert(
                            &mut ctx.world,
                            lua_id,
                            sheet_index,
                            frames,
                            frame_duration,
                            looping,
                        )
                    })
                });

                Ok(())
            },
        )?,
    )?;

    engine.set(
        "set_camera",
        lua.create_function(|_, (x, y, zoom): (f32, f32, f32)| {
            use engine_ecs::ActiveCamera;
            use engine_math::Camera2D;

            fn set_or_spawn(world: &mut World, x: f32, y: f32, zoom: f32) {
                let mut camera_query = world.query::<(&mut Camera2D, &ActiveCamera)>();

                if let Some((cam, _)) = camera_query.iter().next() {
                    cam.position = Vec2::new(x, y);
                    cam.zoom = zoom;
                } else {
                    drop(camera_query);
                    world.spawn((
                        Camera2D {
                            position: Vec2::new(x, y),
                            zoom,
                        },
                        ActiveCamera,
                    ));
                }
            };

            with_start_ctx(|ctx| set_or_spawn(&mut ctx.world, x, y, zoom))
                .or_else(|| with_update_ctx(|ctx| set_or_spawn(&mut ctx.world, x, y, zoom)));
            Ok(())
        })?,
    )?;

    engine.set(
        "load_texture",
        lua.create_function(|_, path: String| {
            let index = with_start_ctx(|ctx| {
                let bytes = std::fs::read(&path)
                    .unwrap_or_else(|_| panic!("Failed to read texture: {}", path));
                ctx.renderer.load_texture(&bytes).to_lua_id()
            });
            Ok(index)
        })?,
    )?;

    engine.set(
        "create_sprite_sheet",
        lua.create_function(|_, (texture_index, tile_w, tile_h): (u32, u32, u32)| {
            let index = with_start_ctx(|ctx| {
                ctx.renderer
                    .create_sprite_sheet(TextureHandle::from_lua_id(texture_index), tile_w, tile_h)
                    .to_lua_id()
            });
            Ok(index)
        })?,
    )?;

    engine.set(
        "draw_rect",
        lua.create_function(|_, (x, y, w, h, r, g, b, a)| {
            fn draw(
                renderer: &mut dyn Renderer,
                x: f32,
                y: f32,
                w: f32,
                h: f32,
                r: f32,
                g: f32,
                b: f32,
                a: f32,
            ) {
                let transform = Transform2D {
                    position: Vec2::new(x, y),
                    rotation_in_radians: 0.0,
                    scale: Vec2::new(w, h),
                };
                renderer.draw_colored_rect(&transform, r, g, b, a);
            }
            with_render_ctx(|ctx| draw(ctx.renderer, x, y, w, h, r, g, b, a));
            Ok(())
        })?,
    )?;

    engine.set(
        "set_paused",
        lua.create_function(|_, (pause): (bool)| {
            with_update_ctx(|ctx| {
                ctx.time.is_paused = pause;
            })
            .or_else(|| {
                with_fixed_ctx(|ctx| {
                    ctx.time.is_paused = pause;
                })
            });
            Ok(())
        })?,
    )?;

    Ok(())
}

fn spawn_from_table(world: &mut World, components: &LuaTable) -> Entity {
    let transform = components
        .get::<LuaTable>("transform")
        .ok()
        .map(|t| Transform2D {
            position: Vec2::new(
                t.get::<f32>("x").unwrap_or(0.0),
                t.get::<f32>("y").unwrap_or(0.0),
            ),
            rotation_in_radians: t.get::<f32>("rotation").unwrap_or(0.0),
            scale: Vec2::new(
                t.get::<f32>("scale_x").unwrap_or(1.0),
                t.get::<f32>("scale_y").unwrap_or(1.0),
            ),
        });
    let previous_transform = match transform.as_ref() {
        Some(t) => Some(PreviousTransform {
            position: t.position.clone(),
            rotation: t.rotation_in_radians.clone(),
        }),
        None => None,
    };

    let velocity = components
        .get::<LuaTable>("velocity")
        .ok()
        .map(|v| Velocity {
            value: Vec2::new(
                v.get::<f32>("x").unwrap_or(0.0),
                v.get::<f32>("y").unwrap_or(0.0),
            ),
        });

    let is_player = components.get::<bool>("player").unwrap_or(false);

    let mut extra = HashMap::new();
    for pair in components.clone().pairs::<String, LuaValue>() {
        if let Ok((key, value)) = pair {
            match key.as_str() {
                "transform" | "velocity" | "player" => continue,
                _ => {
                    extra.insert(key, from_lua_value(value));
                }
            }
        }
    }

    let lua_components = if !extra.is_empty() {
        Some(LuaComponents { data: extra })
    } else {
        None
    };

    // let entity = world.spawn(());

    let entity = match (transform, velocity, is_player, lua_components) {
        (Some(t), Some(v), true, Some(lc)) => world.spawn((t, v, Player, lc)),
        (Some(t), Some(v), true, None) => world.spawn((t, v, Player)),
        (Some(t), Some(v), false, Some(lc)) => world.spawn((t, v, lc)),
        (Some(t), Some(v), false, None) => world.spawn((t, v)),
        (Some(t), None, true, Some(lc)) => world.spawn((t, Player, lc)),
        (Some(t), None, true, None) => world.spawn((t, Player)),
        (Some(t), None, false, Some(lc)) => world.spawn((t, lc)),
        (Some(t), None, false, None) => world.spawn((t,)),
        (None, Some(v), _, Some(lc)) => world.spawn((v, lc)),
        (None, Some(v), _, None) => world.spawn((v,)),
        (None, None, true, Some(lc)) => world.spawn((Player, lc)),
        (None, None, true, None) => world.spawn((Player,)),
        (None, None, false, Some(lc)) => world.spawn((lc,)),
        (None, None, false, None) => world.spawn(()),
    };

    if let Some(pt) = previous_transform {
        let _ = world.insert_one(entity, pt);
    }

    entity
}
