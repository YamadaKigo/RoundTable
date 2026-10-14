// ===========================================================================
//  模块 ui::button：全工程统一的按钮（自绘）
// ===========================================================================
//  按钮只有这一个类：内容（纯文字 / 图标 / 图标 + 文字）+ 悬停便签 + 量好的宽度。
//  外观一律走这儿，别处不许再抄一份：
//    - 三种状态三块纯色（常态 / 悬停 / 按下），1px 硬边框，无圆角、无阴影、无渐变；
//    - 内容整体居中；带图标时把「图标 + 间隔 + 文字」当一个整体居中；
//    - 鼠标悬停且有便签文字时，把便签记到 StockApp::tooltip，由 ui() 结尾统一画在整帧最上层
//      （便签 = 1px 硬边小牌子，和十字光标那两块牌子同一套外观，见 paint_tooltip）。
//  宽度 = 内容宽 + 左右各 1 格空档，再吸到整数格（铁律：文字 / 图标左右各留 1 格）。
//
//  图标走 ui::icon 的 IconSet（16x16 PNG 读进来再自绘）；按钮本身不认识图标的画法 -
//  以后要加按钮样式（带下拉箭头的、可切换的），扩的都是这一个文件。
use crate::app::*;
use crate::grid::*;
use crate::theme::*;
use crate::ui::icon::*;
use eframe::egui;

/// 按钮的行高：2 格 = 一个文本行盒（cells(2) = 19px）。控制面板 / 编辑器按钮栏都用它。
pub(crate) const BTN_ROWS: i32 = TEXT_ROWS;
pub(crate) const BTN_H: f32 = cells(BTN_ROWS);
/// 相邻两个按钮共用中间那条网格线：步进 = grid_line(2) = 18px。
pub(crate) const BTN_STEP: f32 = grid_line(BTN_ROWS);
/// 按钮宽度的上限：24 格（再长就吸不住了，宁可截文字也不越出面板）。
pub(crate) const BTN_MAX_COLS: i32 = 24;
pub(crate) const BTN_MAX_W: f32 = cells(BTN_MAX_COLS);

/// 按钮的三种外观状态。
#[derive(Clone, Copy, PartialEq)]
pub(crate) enum ButtonState {
    Idle,
    Hover,
    Pressed,
}

impl ButtonState {
    /// 底色：三种状态各一块纯色，只有明暗差别。
    pub(crate) fn face(self) -> egui::Color32 {
        match self {
            ButtonState::Idle => theme().btn_face,
            ButtonState::Hover => theme().btn_hover,
            ButtonState::Pressed => theme().btn_press,
        }
    }
}

/// 按钮内容：纯文字 / 图标 + 文字 / 只有图标。
#[derive(Clone)]
pub(crate) enum ButtonContent {
    Text(String),
    IconText(IconKind, String),
    /// 只画一个图标（编辑器按钮栏那种方按钮）：宽高都是 cells(2)。
    Icon(IconKind),
}

impl ButtonContent {
    /// 状态栏回显 / 便签的默认名字。
    pub(crate) fn label(&self) -> String {
        match self {
            ButtonContent::Text(text) => text.clone(),
            ButtonContent::IconText(_, text) => text.clone(),
            ButtonContent::Icon(kind) => kind.label().to_owned(),
        }
    }
}

/// 一个按钮：内容 + 悬停便签 + 量好的宽度。
/// 宽度要用硬边字形的实际像素宽来量，所以得等 text_engine 就绪（见 StockApp::button_width）。
#[derive(Clone)]
pub(crate) struct Button {
    pub(crate) content: ButtonContent,
    /// 鼠标悬停时弹出来的便签上的字（空串 = 不弹便签）。
    pub(crate) tip: String,
    /// 已经量好的宽度（cells 形状；和摆在哪儿无关，量一次就够）。
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

    /// 配上悬停便签的文字（便签内容 = 按钮含义）。
    pub(crate) fn with_tip(mut self, tip: &str) -> Self {
        self.tip = tip.to_owned();
        self
    }

    /// 量宽度（要用 app 里的硬边文字引擎）。
    pub(crate) fn measure(&mut self, app: &mut StockApp) -> &mut Self {
        self.w = app.button_width(&self.content);
        self
    }

    /// 这个按钮叫什么（状态栏回显用）：有便签就用便签，没有就取内容本身。
    pub(crate) fn label(&self) -> String {
        if self.tip.is_empty() {
            self.content.label()
        } else {
            self.tip.clone()
        }
    }
}

impl StockApp {
    /// 按钮宽度 = 内容宽 + 左右各 9px 内边距，向上吸到整数格，最多 24 格。
    /// 内容宽：纯文字 = 文字宽；图标 + 文字 = 2 格 + 1 格 + 文字宽；只有图标 = 2 格。
    pub(crate) fn button_width(&mut self, content: &ButtonContent) -> f32 {
        let inner = match content {
            ButtonContent::Text(text) => self.text_width(text),
            ButtonContent::IconText(_, text) => ICON_W + ICON_GAP + self.text_width(text),
            // 只有图标：内容就是那一个 16x16 的方框，左右各留 1px 边框 = cells(2) 的正方形。
            ButtonContent::Icon(_) => return cells(2),
        };
        snap_cells_up(inner + 2.0 * TEXT_PAD_X).min(BTN_MAX_W)
    }

    /// 画一个按钮并登记点击：按状态铺底色和 1px 深灰硬边 -> 按内容画图标 / 文字，
    /// 悬停时把便签记到 self.tooltip。返回这一帧是否被点击。
    pub(crate) fn draw_button(
        &mut self,
        ui: &mut egui::Ui,
        painter: &egui::Painter,
        rect: egui::Rect,
        content: &ButtonContent,
        pressed: bool,
        tip: &str,
    ) -> bool {
        let resp = ui.allocate_rect(rect, egui::Sense::click());
        let state = if pressed || resp.is_pointer_button_down_on() {
            ButtonState::Pressed
        } else if resp.hovered() {
            ButtonState::Hover
        } else {
            ButtonState::Idle
        };

        // 先铺底色再描边：边框永远压在高亮之上，四条边都完整。
        painter.rect_filled(rect, egui::CornerRadius::ZERO, state.face());
        border_hard(painter, rect, theme().ctrl_border);

        // 悬停 -> 弹便签。真正画在整帧最上层（见 paint_tooltip），这儿只记一笔。
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
                    theme().text,
                );
            }
            ButtonContent::Icon(kind) => {
                // 只画图标：贴着内容区左上角（1px 边框 + 16x16 图标）。
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
                // 图标 16px 高，正好贴着内容区上沿那一行（按钮高 2 格 = 17px 内容）。
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
                    theme().text,
                );
            }
        }
        resp.clicked()
    }

    /// 悬停便签：排在整帧最后画（浮动面板都压不住它）。便签是一块 1px 硬边的小牌子：
    /// 默认开在按钮下方，上沿正好压在按钮下沿那条网格线上（和按钮下边框共用那 1px，
    /// 绝不叠成 2px）；下面放不下就翻到按钮上方（下沿压在按钮上沿那条线上）。
    /// 左右吸到竖网格线上，整块夹在窗口里。
    pub(crate) fn paint_tooltip(&mut self, painter: &egui::Painter, layout: egui::Rect) {
        let Some((anchor, text)) = self.tooltip.take() else {
            return;
        };
        if text.is_empty() {
            return;
        }
        let w = snap_cells_up(self.text_width(&text) + 2.0 * TEXT_PAD_X);
        let h = cells(TEXT_ROWS);
        let x = clamp_soft(grid_snap(anchor.min.x), layout.min.x, layout.max.x - w);
        let below = anchor.max.y - LINE_W;
        let y = if below + h <= layout.max.y - LINE_W {
            below
        } else {
            (anchor.min.y + LINE_W - h).max(layout.min.y)
        };
        let rect = egui::Rect::from_min_size(egui::pos2(x, y), egui::vec2(w, h));
        tag_box(painter, rect, theme().cross_label, theme().ctrl_border);
        self.text(
            painter,
            rect.center(),
            egui::Align2::CENTER_CENTER,
            &text,
            theme().text,
        );
    }
}
