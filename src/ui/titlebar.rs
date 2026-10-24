use crate::application::app::*;
use crate::domain::layout::*;
use crate::engine::font::text_advance;
use crate::engine::kit;
use crate::engine::theme::*;
use crate::ui::button::*;
use crate::ui::menu::*;
use crate::ui::pattern::*;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(crate) enum SysButton {
    Exclusive,
    Pin,
    Minimize,
    Maximize,
    Close,
}

impl SysButton {
    fn is_text(self) -> bool {
        matches!(self, SysButton::Exclusive | SysButton::Pin)
    }

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
            SysButton::Exclusive if exclusive => "Free",
            SysButton::Exclusive => "Lock",
            SysButton::Pin if pinned => "Pinned",
            SysButton::Pin => "Pin",
            SysButton::Minimize if restore_minimized => "Restore",
            SysButton::Minimize => "Minimize",
            SysButton::Maximize if maximized => "Restore",
            SysButton::Maximize => "Maximize",
            SysButton::Close => "Close",
        }
    }

    fn width(self, maximized: bool, restore_minimized: bool, pinned: bool, exclusive: bool) -> f32 {
        if self.is_text() {
            snap_cells_up(
                text_advance(self.label(maximized, restore_minimized, pinned, exclusive))
                    + 2.0 * TEXT_PAD_X,
            )
        } else {
            SYS_BTN_W
        }
    }
}

pub(crate) fn title_text_slot(app: &mut AppState, title: kit::Rect, text: &str) -> kit::Rect {
    let w = tab_width(app, text).min(title.width());
    kit::Rect::from_min_size(title.min, kit::vec2(w, title.height()))
}

pub(crate) fn paint_title_text_edge(painter: &kit::Painter, slot: kit::Rect) {
    vline_hard(
        painter,
        slot.max.x - LINE_W,
        slot.min.y + LINE_W,
        slot.max.y - LINE_W,
        theme().window_border,
    );
}

pub(crate) fn paint_title_fill(
    painter: &kit::Painter,
    title: kit::Rect,
    text_slot: kit::Rect,
    keys_left: f32,
) {
    let fill = kit::Rect::from_min_max(
        kit::pos2(text_slot.max.x, title.min.y),
        kit::pos2(keys_left.max(text_slot.max.x), title.max.y),
    );
    paint_fill(painter, fill, TITLE_FILL, theme().title_fill);
}

pub(crate) fn sys_button_rects_n(bar: kit::Rect, count: usize) -> Vec<kit::Rect> {
    let mut rects = vec![kit::Rect::NOTHING; count];
    let mut right = bar.max.x;
    for i in (0..count).rev() {
        rects[i] = kit::Rect::from_min_size(
            kit::pos2(right - SYS_BTN_W, bar.min.y),
            kit::vec2(SYS_BTN_W, SYS_BTN_H),
        );
        right = rects[i].min.x + LINE_W;
    }
    rects
}

pub(crate) fn sys_button_rects(
    bar: kit::Rect,
    keys: &[(SysButton, usize)],
    maximized: bool,
    restore_minimized: bool,
    pinned: bool,
    exclusive: bool,
) -> Vec<kit::Rect> {
    let count = keys.iter().map(|(_, slot)| slot + 1).max().unwrap_or(0);
    let mut widths = vec![SYS_BTN_W; count];
    for &(kind, slot) in keys {
        widths[slot] = kind.width(maximized, restore_minimized, pinned, exclusive);
    }
    let mut rects = vec![kit::Rect::NOTHING; count];
    let mut right = bar.max.x;
    for i in (0..count).rev() {
        rects[i] = kit::Rect::from_min_size(
            kit::pos2(right - widths[i], bar.min.y),
            kit::vec2(widths[i], SYS_BTN_H),
        );
        right = rects[i].min.x + LINE_W;
    }
    rects
}

pub(crate) fn sys_keys_width(
    keys: &[(SysButton, usize)],
    maximized: bool,
    restore_minimized: bool,
    pinned: bool,
    exclusive: bool,
) -> f32 {
    if keys.is_empty() {
        return 0.0;
    }
    keys.iter()
        .map(|(kind, _)| kind.width(maximized, restore_minimized, pinned, exclusive))
        .sum::<f32>()
        - (keys.len().saturating_sub(1) as f32) * LINE_W
}

#[expect(
    clippy::too_many_arguments,
    reason = "hard-edge primitive keeps geometry arguments explicit"
)]
pub(crate) fn paint_sys_button(
    app: &mut AppState,
    painter: &kit::Painter,
    rect: kit::Rect,
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
    painter.rect_filled(rect, kit::CornerRadius::ZERO, face);
    border_hard(painter, rect, theme().title_button_border);

    let label = kind.label(maximized, restore_minimized, pinned, exclusive);
    if kind.is_text() {
        app.text(
            painter,
            rect.center(),
            kit::Align2::CENTER_CENTER,
            label,
            theme().button_text,
        );
    } else {
        let content =
            ButtonContent::Symbol(kind.symbol(maximized, restore_minimized, pinned, exclusive));
        debug_assert_eq!(content.class(), ButtonClass::Icon);
        let ButtonContent::Symbol(symbol) = content else {
            return;
        };
        let symbol_rect = kit::Rect::from_center_size(rect.center(), kit::vec2(SYMBOL_W, SYMBOL_H));
        app.assets
            .symbols
            .get(symbol)
            .paint(painter, symbol_rect, theme().title_button_symbol);
    }

    if hot {
        app.shell.tooltip = Some((rect, label.to_owned()));
    }
}

impl AppState {
    pub(crate) fn ui_titlebar(&mut self, ui: &mut kit::Ui, bar: kit::Rect) {
        let ctx = ui.ctx().clone();
        let painter = ui.painter().clone();

        let name_slot =
            kit::Rect::from_min_size(bar.min, kit::vec2(tab_width(self, APP_NAME), bar.height()));
        self.text(
            &painter,
            name_slot.center(),
            kit::Align2::CENTER_CENTER,
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
            let resp = ui.interact(rect, kit::Id::new(("rt_sys_btn", i)), kit::Sense::click());
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

            if resp.pressed() {
                match kind {
                    SysButton::Minimize => {
                        ctx.send_viewport_cmd(kit::ViewportCommand::Minimized(true));
                    }
                    SysButton::Close => {
                        ctx.send_viewport_cmd(kit::ViewportCommand::Close);
                    }
                    SysButton::Exclusive | SysButton::Pin | SysButton::Maximize => {}
                }
            }
        }
    }
}
