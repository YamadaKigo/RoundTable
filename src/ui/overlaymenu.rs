// ===========================================================================
//  模块 ui::overlaymenu：K 线上的「叠加均线」菜单
// ===========================================================================
//  点 K 线图（蜡烛图 / 美国线）标题栏上的「叠加」弹出：12 行 = MA5/10/20/60/120/250 +
//  EMA5/10/20/60/120/250，行高 / 行距 / 画法都跟指标菜单（ui::panemenu）同一套。
//  每行左边 1 格是这条均线的色块标记：实心 = 开着、空心框 = 关着，颜色就是那条线的线色
//  （theme().ma / theme().ema，和画在 K 线上的是同一个色源）。右边是名字，踩在内容格上。
//  点一行翻一位，然后把新配置灌给所有认叠加的图表并置脏重算；菜单不自动收起，可以连着勾。
//  面板同样画在每帧最上层（见 app.rs 的 ui()），底下的图表 / 控件抢不到这块地方。
use crate::app::*;
use crate::grid::*;
use crate::indicator;
use crate::theme::*;
use crate::ui::menu::*;
use eframe::egui;

/// 一条菜单项：是不是 MA（否则 EMA）+ 第几条（对应 MA_PERIODS / EMA_PERIODS 的下标）+ 标签。
pub(crate) struct OverlayItem {
    pub(crate) is_ma: bool,
    pub(crate) index: usize,
    pub(crate) label: String,
}

impl StockApp {
    /// 菜单的全部行：先 6 条 MA、后 6 条 EMA（顺序和 MA_PERIODS / EMA_PERIODS 一致）。
    pub(crate) fn overlay_items() -> Vec<OverlayItem> {
        let mut items = Vec::with_capacity(12);
        for (i, p) in indicator::MA_PERIODS.iter().enumerate() {
            items.push(OverlayItem {
                is_ma: true,
                index: i,
                label: format!("MA{p}"),
            });
        }
        for (i, p) in indicator::EMA_PERIODS.iter().enumerate() {
            items.push(OverlayItem {
                is_ma: false,
                index: i,
                label: format!("EMA{p}"),
            });
        }
        items
    }

    /// 菜单矩形：右边缘和「叠加」按钮的右边缘对齐（都落在那条网格线上）、上沿压在标题栏
    /// 下沿那条网格线上（两段共用这 1px），高 = 项数 x 2 格。
    pub(crate) fn overlay_menu_rect(&self, count: usize, window_bottom: f32) -> Option<egui::Rect> {
        if count == 0 {
            return None;
        }
        Some(MenuPanel::right(self.overlay_menu_anchor, OVERLAY_MENU_W, count, window_bottom).rect)
    }

    /// 把当前的叠加配置灌给所有认它的图表（只有 K 线图两种）并置脏，下帧重算均线。
    pub(crate) fn sync_overlay(&mut self) {
        let ov = self.overlay;
        for pane in &mut self.panes {
            if pane.indicator.supports_overlay() {
                pane.indicator.set_overlay_state(ov);
                pane.dirty = true;
            }
        }
        let (ma, ema) = (ov.ma.count_ones(), ov.ema.count_ones());
        self.status_msg = format!("叠加均线：MA {ma} 条 / EMA {ema} 条");
    }

    /// 「叠加」菜单：先铺底，再逐条画（hover 高亮 + 色块标记 + 名字），最后把 1px 硬边压在最上面
    /// - 先画边的话会被 hover 高亮啃掉一条像素。
    pub(crate) fn ui_overlay_menu(&mut self, ui: &mut egui::Ui) {
        let Some(pane) = self.overlay_menu else {
            return;
        };
        let items = Self::overlay_items();
        let Some(panel) = self.overlay_menu_rect(items.len(), ui.max_rect().max.y) else {
            return;
        };
        let painter = ui.painter().clone();
        let menu = MenuPanel { rect: panel, rows: items.len() };
        // 勾选状态从这块图表自己的指标上取（它就是「叠加」菜单服务的对象），线色取当前皮肤。
        let mut ov = self.panes[pane].indicator.overlay_state();
        let t = theme();
        let clicked = menu.ui_rows(self, ui, &painter, |app, painter, j, row| {
            let item = &items[j];
            // 左边那一格是色块标记：开着就实心填、关着就只描一个 1px 框。
            let on = if item.is_ma {
                ov.ma_on(item.index)
            } else {
                ov.ema_on(item.index)
            };
            let line_color = if item.is_ma {
                t.ma[item.index]
            } else {
                t.ema[item.index]
            };
            let mark = egui::Rect::from_min_size(
                egui::pos2(row.min.x + LINE_W, row.center().y - GRID / 2.0),
                egui::vec2(GRID, GRID),
            );
            if on {
                painter.rect_filled(mark, egui::CornerRadius::ZERO, line_color);
            } else {
                border_hard(painter, mark, line_color);
            }
            // 名字让开左边那一格标记：落点还是踩在内容格上（多让 1 格）。
            app.text(
                painter,
                // 文字的吸附原点 = 这块下拉面板自己那套网格（面板左边那条线）。
                egui::pos2(text_left_x(row.min.x, panel.min.x) + PITCH, row.center().y),
                egui::Align2::LEFT_CENTER,
                &item.label,
                theme().text,
            );
        });

        if let Some(j) = clicked {
            let item = &items[j];
            if item.is_ma {
                ov.toggle_ma(item.index);
            } else {
                ov.toggle_ema(item.index);
            }
            // 界面自己那份配置也一起更新，再灌回所有认叠加的图表（只有 K 线图两种）。
            self.overlay = ov;
            self.sync_overlay();
            return;
        }

        // 鼠标一移出面板和它所在的那条图表栏（图表自己的菜单栏）就收起 - 和主窗口的菜单栏
        // 一个规矩（见 ui::menu::pointer_left）；点面板和那条栏以外的地方同样收起。
        if pointer_left(ui, &[panel, self.overlay_menu_bar]) {
            self.overlay_menu = None;
        }
    }
}
