use crate::app::*;
use crate::grid::*;
use crate::theme::*;
use crate::ui::button::*;
use crate::ui::menu::*;
use crate::ui::pattern::*;
use eframe::egui;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(crate) enum SysButton {
    Exclusive,
    Pin,
    Minimize,
    Maximize,
    Close,
}

impl SysButton {
    pub(crate) fn symbol(
        self,
        maximized: bool,
        restore_minimized: bool,
        pinned: bool,
        exclusive: bool,
    ) -> SymbolKind {
        match self {
            SysButton::Exclusive if exclusive => SymbolKind::ExclusiveOn,
            SysButton::Exclusive => SymbolKind::Exclusive,
            SysButton::Pin if pinned => SymbolKind::Unpin,
            SysButton::Pin => SymbolKind::Pin,
            SysButton::Minimize if restore_minimized => SymbolKind::Restore,
            SysButton::Minimize => SymbolKind::Minimize,
            SysButton::Maximize if maximized => SymbolKind::Restore,
            SysButton::Maximize => SymbolKind::Maximize,
            SysButton::Close => SymbolKind::Close,
        }
    }

    pub(crate) fn label(
        self,
        maximized: bool,
        restore_minimized: bool,
        pinned: bool,
        exclusive: bool,
    ) -> &'static str {
        match self {
            SysButton::Exclusive if exclusive => "解除独占",
            SysButton::Exclusive => "独占",
            SysButton::Pin if pinned => "解除置顶",
            SysButton::Pin => "置顶",
            SysButton::Minimize if restore_minimized => "还原",
            SysButton::Minimize => "最小化",
            SysButton::Maximize if maximized => "还原",
            SysButton::Maximize => "最大化",
            SysButton::Close => "关闭",
        }
    }
}

pub(crate) fn title_text_slot(app: &mut StockApp, title: egui::Rect, text: &str) -> egui::Rect {
    let w = tab_width(app, text).min(title.width());
    egui::Rect::from_min_size(title.min, egui::vec2(w, title.height()))
}

pub(crate) fn paint_title_text_edge(painter: &egui::Painter, slot: egui::Rect) {
    vline_hard(
        painter,
        slot.max.x - LINE_W,
        slot.min.y + LINE_W,
        slot.max.y - LINE_W,
        theme().window_border,
    );
}

pub(crate) fn paint_title_fill(
    painter: &egui::Painter,
    title: egui::Rect,
    text_slot: egui::Rect,
    keys_left: f32,
) {
    let fill = egui::Rect::from_min_max(
        egui::pos2(text_slot.max.x, title.min.y),
        egui::pos2(keys_left.max(text_slot.max.x), title.max.y),
    );
    paint_fill(painter, fill, TITLE_FILL, theme().title_fill);
}

pub(crate) fn sys_button_rects_n(bar: egui::Rect, count: usize) -> Vec<egui::Rect> {
    let mut rects = vec![egui::Rect::NOTHING; count];
    let mut right = bar.max.x;
    for i in (0..count).rev() {
        rects[i] = egui::Rect::from_min_size(
            egui::pos2(right - SYS_BTN_W, bar.min.y),
            egui::vec2(SYS_BTN_W, SYS_BTN_H),
        );
        right = rects[i].min.x + LINE_W;
    }
    rects
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_sys_button(
    app: &mut StockApp,
    painter: &egui::Painter,
    rect: egui::Rect,
    hot: bool,
    kind: SysButton,
    maximized: bool,
    restore_minimized: bool,
    pinned: bool,
    exclusive: bool,
) {
    let face = if hot {
        theme().title_button_hover
    } else {
        theme().title_button_bg
    };
    painter.rect_filled(rect, egui::CornerRadius::ZERO, face);
    border_hard(painter, rect, theme().title_button_border);

    let content =
        ButtonContent::Symbol(kind.symbol(maximized, restore_minimized, pinned, exclusive));
    debug_assert_eq!(content.class(), ButtonClass::Icon);
    let ButtonContent::Symbol(symbol) = content else {
        return;
    };
    let symbol_rect = egui::Rect::from_center_size(rect.center(), egui::vec2(SYMBOL_W, SYMBOL_H));
    app.symbols
        .get(symbol)
        .paint(painter, symbol_rect, theme().title_button_symbol);

    if hot {
        app.tooltip = Some((
            rect,
            kind.label(maximized, restore_minimized, pinned, exclusive)
                .to_owned(),
        ));
    }
}

impl StockApp {
    pub(crate) fn ui_titlebar(&mut self, ui: &mut egui::Ui, bar: egui::Rect) {
        let ctx = ui.ctx().clone();
        let painter = ui.painter().clone();

        let name_slot =
            egui::Rect::from_min_size(bar.min, egui::vec2(tab_width(self, APP_NAME), bar.height()));
        self.text(
            &painter,
            name_slot.center(),
            egui::Align2::CENTER_CENTER,
            APP_NAME,
            theme().title_text,
        );

        let buttons = sys_button_rects_n(bar, 2);
        paint_title_fill(&painter, bar, name_slot, buttons[0].min.x);
        for (i, kind) in [SysButton::Minimize, SysButton::Close]
            .into_iter()
            .enumerate()
        {
            let slot = match kind {
                SysButton::Minimize => 0,
                SysButton::Close => 1,
                SysButton::Exclusive | SysButton::Pin | SysButton::Maximize => continue,
            };
            let rect = buttons[slot];
            let resp = ui.interact(rect, egui::Id::new(("rt_sys_btn", i)), egui::Sense::click());
            paint_sys_button(
                self,
                &painter,
                rect,
                resp.hovered(),
                kind,
                false,
                false,
                false,
                false,
            );

            if resp.clicked() {
                match kind {
                    SysButton::Minimize => {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                    }
                    SysButton::Close => {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                    SysButton::Exclusive | SysButton::Pin | SysButton::Maximize => {}
                }
            }
        }
    }
}
