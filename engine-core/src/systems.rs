use crate::{
    contexts::{FixedContext, RenderContext, UpdateContext},
    time::Time,
};
use engine_ecs::{ActiveCamera, PreviousTransform, Velocity};
use engine_math::{Camera2D, Transform2D};
use engine_renderer::{AnimatedSprite, Sprite};

pub fn animation_system(ctx: &mut UpdateContext) {
    for anim in ctx.world.query::<&mut AnimatedSprite>().iter() {
        anim.timer += ctx.time.dt;
        if anim.timer >= anim.frame_duration {
            anim.timer -= anim.frame_duration;
            if anim.current_frame + 1 < anim.frames.len() {
                anim.current_frame += 1;
            } else if anim.looping {
                anim.current_frame = 0;
            }
        }
    }
}

fn interpolate(transform: &Transform2D, previous: &PreviousTransform, time: &Time) -> Transform2D {
    let pos = previous.position.lerp(transform.position, time.alpha);
    let rot = previous.rotation + (transform.rotation_in_radians - previous.rotation) * time.alpha;
    Transform2D {
        position: pos,
        rotation_in_radians: rot,
        scale: transform.scale,
    }
}

pub fn sprite_render_system(ctx: &mut RenderContext) {
    let mut camera_query = ctx.world.query::<(&Camera2D, &ActiveCamera)>();
    let camera = camera_query.iter().next().map(|(cam, _)| cam);
    let Some(camera) = camera else { return };

    ctx.renderer.set_camera(&camera);

    for (transform, previous, sprite) in ctx
        .world
        .query::<(&Transform2D, &PreviousTransform, &Sprite)>()
        .without::<&AnimatedSprite>()
        .iter()
    {
        let interpolated = interpolate(transform, previous, ctx.time);
        ctx.renderer
            .draw_sprite(&interpolated, sprite.sprite_sheet, sprite.index);
    }

    for (transform, previous, anim) in ctx
        .world
        .query::<(&Transform2D, &PreviousTransform, &AnimatedSprite)>()
        .iter()
    {
        let index = anim.frames[anim.current_frame];
        let interpolated = interpolate(transform, previous, ctx.time);
        ctx.renderer
            .draw_sprite(&interpolated, anim.sprite_sheet, index);
    }

    for (transform, sprite) in ctx
        .world
        .query::<(&Transform2D, &Sprite)>()
        .without::<&AnimatedSprite>()
        .without::<&PreviousTransform>()
        .iter()
    {
        ctx.renderer
            .draw_sprite(transform, sprite.sprite_sheet, sprite.index);
    }

    for (transform, anim) in ctx
        .world
        .query::<(&Transform2D, &AnimatedSprite)>()
        .without::<&PreviousTransform>()
        .iter()
    {
        let index = anim.frames[anim.current_frame];
        ctx.renderer
            .draw_sprite(transform, anim.sprite_sheet, index);
    }
}

pub fn snapshot_system(ctx: &mut FixedContext) {
    for (transform, previous) in ctx
        .world
        .query::<(&Transform2D, &mut PreviousTransform)>()
        .iter()
    {
        previous.position = transform.position;
        previous.rotation = transform.rotation_in_radians;
    }
}

pub fn velocity_system(ctx: &mut FixedContext) {
    if ctx.time.is_paused {
        return;
    }
    for (transform, velocity) in ctx.world.query::<(&mut Transform2D, &Velocity)>().iter() {
        transform.position += velocity.value * ctx.time.dt;
    }
}
