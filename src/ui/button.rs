use crate::app::*;
use crate::grid::*;
use crate::theme::*;
use crate::ui::icon::*;
use eframe::egui;

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
    pub(crate) fn face(self) -> egui::Color32 {
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

    #[allow(dead_code)]
    IconText(IconKind, String),

    #[allow(dead_code)]
    Icon(IconKind),
}

impl ButtonContent {
    #[cfg(test)]
    pub(crate) fn label(&self) -> String {
        match self {
            ButtonContent::Text(text) => text.clone(),
            ButtonContent::Symbol(kind) => kind.file_stem().to_owned(),
            ButtonContent::IconText(_, text) => text.clone(),
            ButtonContent::Icon(kind) => kind.label().to_owned(),
        }
    }

    pub(crate) fn class(&self) -> ButtonClass {
        match self {
            ButtonContent::Text(_) => ButtonClass::Text,
            ButtonContent::Symbol(_) | ButtonContent::Icon(_) | ButtonContent::IconText(..) => {
                ButtonClass::Icon
            }
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

    pub(crate) fn measure(&mut self, app: &mut StockApp) -> &mut Self {
        self.w = app.button_width(&self.content);
        self
    }
}

impl StockApp {
    pub(crate) fn button_width(&mut self, content: &ButtonContent) -> f32 {
        let inner = match content {
            ButtonContent::Text(text) => self.text_width(text),
            ButtonContent::Symbol(_) => SYMBOL_W,
            ButtonContent::IconText(_, text) => ICON_W + ICON_GAP + self.text_width(text),

            ButtonContent::Icon(_) => return cells(2),
        };
        snap_cells_up(inner + 2.0 * TEXT_PAD_X).min(BTN_MAX_W)
    }

    pub(crate) fn draw_button(
        &mut self,
        ui: &mut egui::Ui,
        painter: &egui::Painter,
        rect: egui::Rect,
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

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn draw_button_with_text(
        &mut self,
        ui: &mut egui::Ui,
        painter: &egui::Painter,
        rect: egui::Rect,
        content: &ButtonContent,
        pressed: bool,
        tip: &str,
        text_color: egui::Color32,
    ) -> bool {
        let resp = interact_rect(ui, rect, egui::Sense::click());
        let state = if pressed || resp.is_pointer_button_down_on() {
            ButtonState::Pressed
        } else if resp.hovered() {
            ButtonState::Hover
        } else {
            ButtonState::Idle
        };

        painter.rect_filled(rect, egui::CornerRadius::ZERO, state.face());
        border_hard(painter, rect, theme().button_border);

        if resp.hovered() && !tip.is_empty() {
            self.tooltip = Some((rect, tip.to_owned()));
        }

        match content {
            ButtonContent::Text(text) => {
                let text = text.clone();
                self.text(
                    painter,
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    &text,
                    text_color,
                );
            }
            ButtonContent::Symbol(kind) => {
                let symbol = self.symbols.get(*kind);
                let rect =
                    egui::Rect::from_center_size(rect.center(), egui::vec2(SYMBOL_W, SYMBOL_H));
                symbol.paint(painter, rect, theme().symbol);
            }
            ButtonContent::Icon(kind) => {
                let icon = egui::Rect::from_min_size(
                    egui::pos2(rect.min.x + LINE_W, rect.min.y + LINE_W),
                    egui::vec2(ICON_W, ICON_H),
                );
                let img = *self.icons.get(*kind);
                img.paint(painter, icon);
            }
            ButtonContent::IconText(kind, text) => {
                let text = text.clone();
                let total = ICON_W + ICON_GAP + self.text_width(&text);
                let icon_x = (rect.center().x - total / 2.0).round();

                let icon = egui::Rect::from_min_size(
                    egui::pos2(icon_x, rect.min.y + LINE_W),
                    egui::vec2(ICON_W, ICON_H),
                );
                let img = *self.icons.get(*kind);
                img.paint(painter, icon);
                self.text(
                    painter,
                    egui::pos2(icon.max.x + ICON_GAP, rect.center().y),
                    egui::Align2::LEFT_CENTER,
                    &text,
                    text_color,
                );
            }
        }
        resp.clicked()
    }

    pub(crate) fn paint_tooltip(&mut self, painter: &egui::Painter, layout: egui::Rect) {
        let Some((anchor, text)) = self.tooltip.take() else {
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
        let rect = egui::Rect::from_min_size(egui::pos2(x, y), egui::vec2(w, h));
        tag_box(painter, rect, theme().tooltip_bg, theme().tooltip_border);
        self.text(
            painter,
            rect.center(),
            egui::Align2::CENTER_CENTER,
            &text,
            theme().tooltip_text,
        );
    }
}
