use crate::ecs::ChartWorld;
use crate::grid::{BAR_COUNT, BAR_START};

pub(crate) fn mark_panes_dirty(world: &mut ChartWorld) {
    for pane in &mut world.panes {
        pane.dirty = true;
    }
}

pub(crate) fn ensure_pane_computed(world: &mut ChartWorld, pane: usize) {
    if !world.panes[pane].dirty {
        return;
    }
    let script = world.panes[pane].script.clone();
    let cmds = if let Some(script) = script {
        match script.render(&world.bars, &world.panes_palette) {
            Ok(render) => {
                world.panes[pane].script_render = Some(render.clone());
                render.cmds
            }
            Err(error) => {
                world.panes[pane].script_error = Some(error);
                Vec::new()
            }
        }
    } else {
        world.panes[pane].script_render = None;
        Vec::new()
    };
    world.panes[pane].cmds = cmds;
    world.panes[pane].dirty = false;
}

pub(crate) fn regenerate_market(world: &mut ChartWorld) {
    let bars = crate::data::random_walk::generate(world.bar_seed, BAR_COUNT, BAR_START);
    world.bars.replace(bars);
    mark_panes_dirty(world);
}
