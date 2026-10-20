use crate::ecs::ComponentTable;
#[cfg(test)]
use crate::ecs::Entity;
use crate::ecs::entity::EntityAllocator;
use crate::ecs::{MarketBars, Pane};
use crate::grid::*;
use crate::indicator;

pub(crate) struct ChartWorld {
    pub(crate) bars: MarketBars,
    pub(crate) bar_seed: u64,
    pub(crate) panes: ComponentTable<Pane>,
    pub(crate) panes_palette: indicator::Palette,
    entities: EntityAllocator,
}

impl ChartWorld {
    pub(crate) fn new(panes: Vec<Pane>, palette: indicator::Palette) -> Self {
        let mut entities = EntityAllocator::default();
        let mut pane_components = ComponentTable::default();
        for pane in panes {
            let entity = entities.spawn();
            pane_components.push(entity, pane);
        }

        Self {
            bars: MarketBars::new(crate::data::random_walk::generate(
                BAR_SEED, BAR_COUNT, BAR_START,
            )),
            bar_seed: BAR_SEED,
            panes: pane_components,
            panes_palette: palette,
            entities,
        }
    }

    pub(crate) fn add_pane(&mut self, pane: Pane) -> usize {
        let entity = self.entities.spawn();
        self.panes.push(entity, pane);
        self.panes.len() - 1
    }

    pub(crate) fn replace_pane(&mut self, index: usize, pane: Pane) {
        let entity = self.entities.spawn();
        self.panes.replace(index, entity, pane);
    }

    #[cfg(test)]
    pub(crate) fn pane_entity_ids(&self) -> Vec<Entity> {
        (0..self.panes.len())
            .filter_map(|index| self.panes.entity(index))
            .collect()
    }
}
