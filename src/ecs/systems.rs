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
    let rendered = world.panes[pane]
        .script
        .as_ref()
        .map(|script| script.render(&world.bars, &world.panes_palette));
    let pane = &mut world.panes[pane];
    match rendered {
        Some(Ok(mut render)) => {
            pane.cmds = std::mem::take(&mut render.cmds);
            pane.script_render = Some(render);
            pane.script_error = None;
        }
        Some(Err(error)) => {
            pane.cmds.clear();
            pane.script_error = Some(error);
        }
        None => {
            pane.cmds.clear();
            pane.script_render = None;
            pane.script_error = None;
        }
    }
    pane.dirty = false;
}

pub(crate) fn regenerate_market(world: &mut ChartWorld) {
    let bars = crate::data::random_walk::generate(world.bar_seed, BAR_COUNT, BAR_START);
    world.bars.replace(bars);
    mark_panes_dirty(world);
}
