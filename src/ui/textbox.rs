// ===========================================================================
//  通用自绘输入框（单行）
// ===========================================================================
//  「一切都自己来」：不用 egui::TextEdit，也不碰任何默认控件外观 - 没有圆角 / 阴影 /
//  渐变，只有 1px 硬边和纯色块。
//  底子和源码编辑器共用同一份：
//    TextBuffer         行 + 光标（字符索引）+ 选区 + 撤销栈；
//    apply_edit_events  键盘 / 剪贴板 / 输入法事件 -> 缓冲区；
//    网格口径           1px 硬边、文字左对齐、2px 光标，坐标一律落在整数像素上。
//  差别只在这儿是单行：不认回车、没有行号、不做横向滚动，超长就在框里截断。
//  以后哪儿要输入框都拿这个，别再各写一份手搓的（颜色面板的 Hex 框就是头一个用户）。
//
//  外框怎么画由宿主说了算（颜色面板那一段：上下两条边是段间网格线、左右两条是面板外框，
//  所以传 edge = None，别让两条边框叠成 2px - 这个工程对线宽抠得很死）。
//  这儿只管内容：底色 + 选区 + 文字 + 输入法候选串 + 光标。

use crate::app::*;
use crate::font::{char_advance, text_advance};
use crate::grid::*;
use crate::theme::*;
use crate::ui::editor::apply_edit_events;
use crate::ui::textbuf::*;
use eframe::egui;

/// 内容区怎么画：底色 +（可选的）四周 1px 硬边。
#[derive(Clone, Copy)]
pub(crate) struct TextInputStyle {
    pub(crate) bg: egui::Color32,
    /// 四周那圈 1px 硬边的颜色；None = 不画（外面已经有框了）。
    pub(crate) edge: Option<egui::Color32>,
}

/// 一块单行输入框的全部跨帧状态。
#[derive(Clone)]
pub(crate) struct TextInput {
    /// 文本缓冲区（单行，永远只有第 0 行）。
    pub(crate) buf: TextBuffer,
    /// 输入法候选串：画在光标处、底下加一根 1px 下划线。
    pub(crate) preedit: String,
    /// 最多收几个字符（0 = 不限）；超了就把末尾裁掉。
    pub(crate) max_chars: usize,
}

impl Default for TextInput {
    fn default() -> Self {
        Self::new(0)
    }
}

/// 把「离文字起点多少像素」换成列号：一个字符一个字符累加字宽，落到哪个字符上就是哪一列。
/// 列是字符索引（中文 3 字节，绝不能拿字节数当列），越过行尾就是行尾。
pub(crate) fn col_at(line: &str, dx: f32) -> usize {
    let mut w = 0.0f32;
    for (i, ch) in line.chars().enumerate() {
        let cw = char_advance(ch);
        if dx < w + cw / 2.0 {
            return i;
        }
        w += cw;
    }
    line.chars().count()
}

impl TextInput {
    pub(crate) fn new(max_chars: usize) -> Self {
        let mut buf = TextBuffer::from_text("");
        buf.move_doc_end();
        Self {
            buf,
            preedit: String::new(),
            max_chars,
        }
    }

    /// 整份换掉（打开面板 / 别处改了值）：光标收到末尾、候选串清掉，撤销栈从头来过。
    pub(crate) fn set_text(&mut self, text: &str) {
        self.buf = TextBuffer::from_text(text);
        self.buf.move_doc_end();
        self.preedit.clear();
        self.join_lines();
        self.limit_chars(self.max_chars);
    }

    /// 单行框的兜底：剪贴板里若带换行（粘进来会断成好几行），把行并回去 - 单行框里
    /// 不该出现第二行。返回 true = 真的并过。
    pub(crate) fn join_lines(&mut self) -> bool {
        if self.buf.lines.len() <= 1 {
            return false;
        }
        self.buf.lines = vec![self.buf.lines.join("")];
        self.buf.sel_anchor = None;
        self.buf.move_doc_end();
        true
    }

    /// 缓冲区里的整份文本（单行，不会带换行）。
    pub(crate) fn text(&self) -> String {
        self.buf.to_text()
    }

    /// 限长：文档总字符数超过 max 就把末尾多出来的裁掉（单行输入框用）。
    /// 光标 / 选区落在被裁掉的那一段里就往末尾收，别的照旧。
    pub(crate) fn limit_chars(&mut self, max: usize) -> bool {
        if max == 0 {
            return false;
        }
        let total: usize = self.buf.lines.iter().map(|l| l.chars().count()).sum();
        if total <= max {
            return false;
        }
        let mut left = max;
        let mut kept: Vec<String> = Vec::new();
        for line in &self.buf.lines {
            if left == 0 {
                break;
            }
            let n = line.chars().count();
            if n <= left {
                left -= n;
                kept.push(line.clone());
            } else {
                kept.push(line.chars().take(left).collect());
                left = 0;
            }
        }
        if kept.is_empty() {
            kept.push(String::new());
        }
        self.buf.lines = kept;
        self.buf.clamp();
        true
    }

    /// 画一块单行输入框，并把这一帧的键盘 / 输入法事件喂给它。返回 true = 内容改过了。
    /// 焦点：点框里收过来、点别处还回去；只有拿到焦点才收键盘事件。
    /// filter 决定哪些字符收得下（Hex 输入框只收 16 进制数字）。
    pub(crate) fn ui(
        &mut self,
        app: &mut StockApp,
        ui: &mut egui::Ui,
        rect: egui::Rect,
        id: egui::Id,
        filter: &dyn Fn(char) -> bool,
        style: TextInputStyle,
    ) -> bool {
        let painter = ui.painter().clone();
        let inner = rect.shrink(LINE_W);
        let text_x = text_left_x(rect.min.x);
        let mid_y = inner.center().y;

        // ---- 焦点：谁最后点谁说了算 ----
        // 用 interact 而不是 allocate_rect：egui 只认「这一帧登记过的」控件才留住焦点。
        let resp = ui.interact(rect, id, egui::Sense::click());
        if resp.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::Text);
        }
        if resp.clicked() {
            ui.memory_mut(|m| m.request_focus(id));
        } else if ui.input(|i| i.pointer.primary_clicked()) {
            // 点在别处（面板里别的控件 / 面板外）就把焦点还回去。
            ui.memory_mut(|m| m.surrender_focus(id));
        }
        let focused = ui.memory(|m| m.has_focus(id));

        // ---- 点一下定位光标：按点击的 x 反算列 ----
        if focused && resp.clicked() {
            if let Some(p) = ui.input(|i| i.pointer.interact_pos()) {
                let col = col_at(self.buf.line(0), p.x - text_x);
                self.buf.set_cursor((0, col), false);
            }
        }

        // ---- 键盘 / 输入法：只认焦点在自己身上的那些 ----
        let mut changed = false;
        if focused {
            ui.memory_mut(|m| {
                m.set_focus_lock_filter(
                    id,
                    egui::EventFilter {
                        tab: true,
                        horizontal_arrows: true,
                        vertical_arrows: false,
                        escape: true,
                    },
                )
            });
            let events = ui.input(|i| i.events.clone());
            let out = apply_edit_events(&mut self.buf, &mut self.preedit, &events, false, filter);
            changed = out.changed;
            if let Some(text) = out.copy {
                ui.ctx().copy_text(text);
            }
            if self.join_lines() {
                changed = true;
            }
            if self.limit_chars(self.max_chars) {
                changed = true;
            }
        } else {
            self.preedit.clear();
        }

        // ---- 画：底色 -> 选区 -> 文字 -> 候选串 -> 光标 ----
        painter.rect_filled(inner, egui::CornerRadius::ZERO, style.bg);
        if let Some(edge) = style.edge {
            border_hard(&painter, rect, edge);
        }
        let text_painter = painter.with_clip_rect(inner);
        let line = self.buf.line(0).to_owned();
        if let Some((start, end)) = self.buf.selection() {
            let x0 = text_x + text_advance(&line.chars().take(start.1).collect::<String>());
            let x1 = text_x + text_advance(&line.chars().take(end.1).collect::<String>());
            painter.rect_filled(
                egui::Rect::from_min_size(
                    egui::pos2(x0, inner.min.y),
                    egui::vec2(x1 - x0, inner.height()),
                ),
                egui::CornerRadius::ZERO,
                theme().sel_bg,
            );
        }
        app.text(
            &text_painter,
            egui::pos2(text_x, mid_y),
            egui::Align2::LEFT_CENTER,
            &line,
            theme().text,
        );

        // 光标（2px 宽的竖线，整数像素、不闪）+ 候选串（在它底下画一根下划线）。
        let mut caret: Option<egui::Rect> = None;
        if focused {
            let head: String = line.chars().take(self.buf.cursor.1).collect();
            let mut x = (text_x + text_advance(&head)).round();
            if !self.preedit.is_empty() {
                let pw = text_advance(&self.preedit);
                app.text(
                    &text_painter,
                    egui::pos2(x, mid_y),
                    egui::Align2::LEFT_CENTER,
                    &self.preedit,
                    theme().text,
                );
                hline_hard(&text_painter, x, x + pw, inner.max.y - LINE_W, theme().text);
                x += pw;
            }
            let bar = egui::Rect::from_min_size(
                egui::pos2(x, inner.min.y),
                egui::vec2(2.0 * LINE_W, inner.height()),
            );
            text_painter.rect_filled(bar, egui::CornerRadius::ZERO, theme().text);
            caret = Some(bar);
        }

        // 输入法候选窗贴哪儿：不报的话系统的拼音框会弹到屏幕左上角去。
        // egui 0.36.2 的入口就是 PlatformOutput::ime（egui::output::IMEOutput，屏幕坐标），
        // egui-winit 拿它去调 winit 的 set_ime_cursor_area。
        if let Some(bar) = caret {
            let to_global = ui
                .ctx()
                .layer_transform_to_global(ui.layer_id())
                .unwrap_or_default();
            ui.output_mut(|o| {
                o.ime = Some(egui::output::IMEOutput {
                    purpose: egui::IMEPurpose::Normal,
                    rect: to_global * rect,
                    cursor_rect: to_global * bar,
                    should_interrupt_composition: false,
                });
            });
        }

        changed
    }
}