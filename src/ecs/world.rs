use crate::ecs::ComponentTable;
#[cfg(test)]
use crate::ecs::Entity;
use crate::ecs::entity::EntityAllocator;
use crate::ecs::{MarketBars, Pane};
use crate::grid::*;
use crate::indicator;
use crate::theme::{DOWN_RGB_DEFAULT, UP_RGB_DEFAULT};
use crate::ui::panel::{IND_H_DEFAULT, KLINE_H_DEFAULT, VOL_H_DEFAULT};

pub(crate) struct AppWorld {
    pub(crate) bars: MarketBars,
    pub(crate) bar_seed: u64,
    pub(crate) panes: ComponentTable<Pane>,
    pub(crate) panes_palette: indicator::Palette,
    pub(crate) kline_h: f32,
    pub(crate) vol_h: f32,
    pub(crate) ind_h: f32,
    pub(crate) up_color: (u8, u8, u8),
    pub(crate) down_color: (u8, u8, u8),
    pub(crate) chart_follows_theme: bool,
    pub(crate) overlay: indicator::Overlay,
    entities: EntityAllocator,
}

impl AppWorld {
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
            kline_h: KLINE_H_DEFAULT,
            vol_h: VOL_H_DEFAULT,
            ind_h: IND_H_DEFAULT,
            up_color: UP_RGB_DEFAULT,
            down_color: DOWN_RGB_DEFAULT,
            chart_follows_theme: true,
            overlay: indicator::Overlay::DEFAULT,
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
