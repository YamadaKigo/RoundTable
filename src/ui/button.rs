use crate::application::app::*;
use crate::domain::layout::*;
use crate::engine::kit;
use crate::engine::theme::*;
#[cfg(test)]
use crate::ui::icon::*;

pub(crate) const BTN_ROWS: i32 = TEXT_ROWS;
pub(crate) const BTN_H: f32 = cells(BTN_ROWS);

pub(crate) const BTN_STEP: f32 = grid_line(BTN_ROWS);

pub(crate) const BTN_MAX_COLS: i32 = 24;
pub(crate) const BTN_MAX_W: f32 = cells(BTN_MAX_COLS);

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum ButtonState {
    Idle,
    Hover,
    Pressed,
}

impl ButtonState {
    pub(crate) fn face(self) -> kit::Color32 {
        match self {
            ButtonState::Idle => theme().btn_face,
            ButtonState::Hover => theme().btn_hover,
            ButtonState::Pressed => theme().btn_press,
        }
    }
}

#[derive(Clone)]
pub(crate) enum ButtonContent {
    Text(String),

    Symbol(SymbolKind),

    #[cfg(test)]
    Icon(IconKind),
}

impl ButtonContent {
    pub(crate) fn tooltip(&self) -> String {
        match self {
            ButtonContent::Text(text) => text.clone(),
            ButtonContent::Symbol(kind) => match kind {
                SymbolKind::Minimize => "Minimize",
                SymbolKind::Maximize => "Maximize",
                SymbolKind::Restore => "Restore",
                SymbolKind::Close => "Close",
                SymbolKind::Pin => "Pin",
                SymbolKind::Unpin => "Pinned",
                SymbolKind::Exclusive => "Lock",
                SymbolKind::ExclusiveOn => "Free",
                SymbolKind::PageUp => "Previous Page",
                SymbolKind::PageDown => "Next Page",
            }
            .to_owned(),
            #[cfg(test)]
            ButtonContent::Icon(kind) => kind.label().to_owned(),
        }
    }

    #[cfg(test)]
    pub(crate) fn label(&self) -> String {
        match self {
            ButtonContent::Text(text) => text.clone(),
            ButtonContent::Symbol(kind) => kind.file_stem().to_owned(),
            #[cfg(test)]
            ButtonContent::Icon(kind) => kind.label().to_owned(),
        }
    }

    pub(crate) fn class(&self) -> ButtonClass {
        match self {
            ButtonContent::Text(_) => ButtonClass::Text,
            ButtonContent::Symbol(_) => ButtonClass::Icon,
            #[cfg(test)]
            ButtonContent::Icon(_) => ButtonClass::Icon,
        }
    }

    pub(crate) fn interaction_id(&self) -> kit::Id {
        match self {
            ButtonContent::Text(text) => kit::Id::new(("button_text", text)),
            ButtonContent::Symbol(kind) => kit::Id::new(("button_symbol", kind.file_stem())),
            #[cfg(test)]
            ButtonContent::Icon(kind) => kit::Id::new(("button_icon", kind.file_stem())),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum ButtonClass {
    Text,
    Icon,
}

#[derive(Clone)]
pub(crate) struct Button {
    pub(crate) content: ButtonContent,

    pub(crate) tip: String,

    pub(crate) w: f32,
}

impl Button {
    pub(crate) fn new(content: ButtonContent) -> Self {
        Self {
            content,
            tip: String::new(),
            w: 0.0,
        }
    }

    pub(crate) fn with_tip(mut self, tip: &str) -> Self {
        self.tip = tip.to_owned();
        self
    }

    pub(crate) fn measure(&mut self, app: &mut AppState) -> &mut Self {
        self.w = app.button_width(&self.content);
        self
    }
}

impl AppState {
    pub(crate) fn button_width(&mut self, content: &ButtonContent) -> f32 {
        let inner = match content {
            ButtonContent::Text(text) => self.text_width(text),
            ButtonContent::Symbol(_) => SYMBOL_W,
            #[cfg(test)]
            ButtonContent::Icon(_) => return cells(2),
        };
        snap_cells_up(inner + 2.0 * TEXT_PAD_X).min(BTN_MAX_W)
    }

    pub(crate) fn draw_button(
        &mut self,
        ui: &mut kit::Ui,
        painter: &kit::Painter,
        rect: kit::Rect,
        content: &ButtonContent,
        pressed: bool,
        tip: &str,
    ) -> bool {
        self.draw_button_with_text(
            ui,
            painter,
            rect,
            content,
            pressed,
            tip,
            theme().button_text,
        )
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "hard-edge primitive keeps geometry arguments explicit"
    )]
    pub(crate) fn draw_button_with_text(
        &mut self,
        ui: &mut kit::Ui,
        painter: &kit::Painter,
        rect: kit::Rect,
        content: &ButtonContent,
        pressed: bool,
        tip: &str,
        text_color: kit::Color32,
    ) -> bool {
        let resp = interact_rect(ui, rect, content.interaction_id(), kit::Sense::click());
        let active = ui.is_enabled() && !self.shell.win_silent;
        let state = if active && (pressed || resp.is_pointer_button_down_on()) {
            ButtonState::Pressed
        } else if active && resp.hovered() {
            ButtonState::Hover
        } else {
            ButtonState::Idle
        };

        painter.rect_filled(rect, kit::CornerRadius::ZERO, state.face());
        border_hard(painter, rect, theme().button_border);

        if active && resp.hovered() {
            let text = if tip.is_empty() {
                content.tooltip()
            } else {
                tip.to_owned()
            };
            if !text.is_empty() {
                self.shell.tooltip = Some((rect, text));
            }
        }

        match content {
            ButtonContent::Text(text) => {
                let text = text.clone();
                self.text(
                    painter,
                    rect.center(),
                    kit::Align2::CENTER_CENTER,
                    &text,
                    text_color,
                );
            }
            ButtonContent::Symbol(kind) => {
                let symbol = self.assets.symbols.get(*kind);
                let rect =
                    kit::Rect::from_center_size(rect.center(), kit::vec2(SYMBOL_W, SYMBOL_H));
                symbol.paint(painter, rect, theme().symbol);
            }
            #[cfg(test)]
            ButtonContent::Icon(kind) => {
                let icon = kit::Rect::from_min_size(
                    kit::pos2(rect.min.x + LINE_W, rect.min.y + LINE_W),
                    kit::vec2(ICON_W, ICON_H),
                );
                let img = *self.assets.icons.get(*kind);
                img.paint(painter, icon);
            }
        }
        active && resp.pressed()
    }

    pub(crate) fn paint_tooltip(&mut self, ctx: &kit::Context, layout: kit::Rect) {
        let Some((anchor, text)) = self.shell.tooltip.take() else {
            return;
        };
        if text.is_empty() {
            return;
        }
        let w = snap_cells_up(self.text_width(&text) + 2.0 * TEXT_PAD_X);
        let h = cells(TEXT_ROWS);

        let x = clamp_soft(anchor.min.x, layout.min.x, layout.max.x - w);
        let below = anchor.max.y - LINE_W;
        let y = if below + h <= layout.max.y - LINE_W {
            below
        } else {
            (anchor.min.y + LINE_W - h).max(layout.min.y)
        };
        let rect = kit::Rect::from_min_size(kit::pos2(x, y), kit::vec2(w, h));
        let layer = kit::LayerId::new(kit::Order::Tooltip, kit::Id::new("rt_tooltip"));
        ctx.with_layer(kit::PaintLayer::Dynamic, || {
            let painter = ctx.layer_painter(layer);
            tag_box(&painter, rect, theme().tooltip_bg, theme().tooltip_border);
            self.text(
                &painter,
                rect.center(),
                kit::Align2::CENTER_CENTER,
                &text,
                theme().tooltip_text,
            );
        });
    }
}
