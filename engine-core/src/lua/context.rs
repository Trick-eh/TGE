use engine_ecs::LuaData;

use crate::{
    contexts::{FixedContext, RenderContext, StartContext, UpdateContext},
    save::SaveData,
};
use std::{cell::RefCell, collections::HashMap};

thread_local! {
    static UPDATE_CTX: RefCell<Option<*mut UpdateContext<'static>>> = RefCell::new(None);
    static FIXED_CTX: RefCell<Option<*mut FixedContext<'static>>> = RefCell::new(None);
    static RENDER_CTX: RefCell<Option<*mut RenderContext<'static>>> = RefCell::new(None);
    static START_CTX: RefCell<Option<*mut StartContext<'static>>> = RefCell::new(None);
    static PERSISTENT: RefCell<Option<*mut HashMap<String,LuaData>>> = RefCell::new(None);
    static SAVE_DATA: RefCell<Option<*mut SaveData>> = RefCell::new(None);

}

pub fn set_update_ctx(ctx: &mut UpdateContext) {
    UPDATE_CTX.with(|c| {
        *c.borrow_mut() = Some(ctx as *mut UpdateContext as *mut UpdateContext<'static>);
    });
}
pub fn set_fixed_ctx(ctx: &mut FixedContext) {
    FIXED_CTX.with(|c| {
        *c.borrow_mut() = Some(ctx as *mut FixedContext as *mut FixedContext<'static>);
    });
}
pub fn set_render_ctx(ctx: &mut RenderContext) {
    RENDER_CTX.with(|c| {
        *c.borrow_mut() = Some(ctx as *mut RenderContext as *mut RenderContext<'static>);
    });
}
pub fn set_start_ctx(ctx: &mut StartContext) {
    START_CTX.with(|c| {
        *c.borrow_mut() = Some(ctx as *mut StartContext as *mut StartContext<'static>);
    });
}
pub fn set_persistent(store: &mut HashMap<String, LuaData>) {
    PERSISTENT.with(|p| {
        *p.borrow_mut() = Some(store as *mut HashMap<String, LuaData>);
    });
}
pub fn set_save_data(save: &mut SaveData) {
    SAVE_DATA.with(|s| {
        *s.borrow_mut() = Some(save as *mut SaveData);
    });
}

pub fn clear_update_ctx() {
    UPDATE_CTX.with(|c| *c.borrow_mut() = None);
}
pub fn clear_fixed_ctx() {
    FIXED_CTX.with(|c| *c.borrow_mut() = None);
}
pub fn clear_render_ctx() {
    RENDER_CTX.with(|c| *c.borrow_mut() = None);
}
pub fn clear_start_ctx() {
    START_CTX.with(|c| *c.borrow_mut() = None);
}
pub fn clear_persistent() {
    PERSISTENT.with(|p| *p.borrow_mut() = None);
}
pub fn clear_save_data() {
    SAVE_DATA.with(|s| *s.borrow_mut() = None);
}

pub fn with_update_ctx<F, R>(f: F) -> Option<R>
where
    F: FnOnce(&mut UpdateContext) -> R,
{
    UPDATE_CTX.with(|c| c.borrow().map(|ptr| unsafe { f(&mut *ptr) }))
}
pub fn with_fixed_ctx<F, R>(f: F) -> Option<R>
where
    F: FnOnce(&mut FixedContext) -> R,
{
    FIXED_CTX.with(|c| c.borrow().map(|ptr| unsafe { f(&mut *ptr) }))
}
pub fn with_render_ctx<F, R>(f: F) -> Option<R>
where
    F: FnOnce(&mut RenderContext) -> R,
{
    RENDER_CTX.with(|c| c.borrow().map(|ptr| unsafe { f(&mut *ptr) }))
}
pub fn with_start_ctx<F, R>(f: F) -> Option<R>
where
    F: FnOnce(&mut StartContext) -> R,
{
    START_CTX.with(|c| c.borrow().map(|ptr| unsafe { f(&mut *ptr) }))
}
pub fn with_persistent<F, R>(f: F) -> Option<R>
where
    F: FnOnce(&mut HashMap<String, LuaData>) -> R,
{
    PERSISTENT.with(|p| p.borrow().map(|ptr| unsafe { f(&mut *ptr) }))
}
pub fn with_save_data<F, R>(f: F) -> Option<R>
where
    F: FnOnce(&mut SaveData) -> R,
{
    SAVE_DATA.with(|s| s.borrow().map(|ptr| unsafe { f(&mut *ptr) }))
}
