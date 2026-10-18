use crate::ecs::AppWorld;
use crate::grid::{BAR_COUNT, BAR_START};

pub(crate) fn mark_panes_dirty(world: &mut AppWorld) {
    for pane in &mut world.panes {
        pane.dirty = true;
    }
}

pub(crate) fn ensure_pane_computed(world: &mut AppWorld, pane: usize) {
    if !world.panes[pane].dirty {
        return;
    }
    let cmds = world.panes[pane].indicator.compute(&world.bars);
    world.panes[pane].cmds = cmds;
    world.panes[pane].dirty = false;
}

pub(crate) fn regenerate_market(world: &mut AppWorld) {
    let bars = crate::data::random_walk::generate(world.bar_seed, BAR_COUNT, BAR_START);
    world.bars.replace(bars);
    mark_panes_dirty(world);
}
