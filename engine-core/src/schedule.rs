use crate::contexts::{FixedContext, RenderContext, UpdateContext};

pub type UpdateSystem = fn(&mut UpdateContext);
pub type FixedSystem = fn(&mut FixedContext);
pub type RenderSystem = fn(&mut RenderContext);

pub struct SystemSchedule {
    update_systems: Vec<UpdateSystem>,
    fixed_systems: Vec<FixedSystem>,
    render_systems: Vec<RenderSystem>,
}

impl SystemSchedule {
    pub fn new() -> Self {
        SystemSchedule {
            update_systems: Vec::new(),
            fixed_systems: Vec::new(),
            render_systems: Vec::new(),
        }
    }

    pub fn add_update_system(&mut self, system: UpdateSystem) {
        self.update_systems.push(system);
    }

    pub fn add_fixed_system(&mut self, system: FixedSystem) {
        self.fixed_systems.push(system);
    }

    pub fn add_render_system(&mut self, system: RenderSystem) {
        self.render_systems.push(system);
    }

    pub fn run_update(&self, ctx: &mut UpdateContext) {
        for system in &self.update_systems {
            system(ctx);
        }
    }

    pub fn run_fixed(&self, ctx: &mut FixedContext) {
        for system in &self.fixed_systems {
            system(ctx);
        }
    }

    pub fn run_render(&self, ctx: &mut RenderContext) {
        for system in &self.render_systems {
            system(ctx);
        }
    }
}
