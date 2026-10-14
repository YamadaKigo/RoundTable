// ===========================================================================
//  模块 ui::panemenu：Pane 的指标切换下拉面板
// ===========================================================================
//  点 Pane 标题栏上的「切换」弹出：宽 20 格、每项 2 格 - 和菜单栏那份下拉面板同一套做法
//  （面板上沿压在标题栏下沿那条网格线上、行与行之间共用网格线、1px 硬边框最后画）。
//  面板画在每帧最上层（见 app.rs 的 ui()），所以底下的图表 / 控件都抢不到这块地方。
//  点某一项 = 把那个内置指标换到这个 Pane 上并置脏；点别处 / 再点「切换」= 收起。
use crate::app::*;
use crate::grid::*;
use crate::indicator;
use crate::theme::*;
use crate::ui::menu::*;
use eframe::egui;

impl StockApp {
    /// 下拉面板的矩形：右边缘和「切换」按钮对齐（都在 Pane 右边框那条线上）、上沿压在
    /// 标题栏下沿那条网格线上（两段共用这 1px），高 = 项数 x 2 格；底下放不下就翻上去。
    /// 几何全部来自 ui::menu 的 MenuPanel - 和菜单栏那份下拉是同一套。
    pub(crate) fn pane_menu_rect(&self, count: usize, window_bottom: f32) -> Option<egui::Rect> {
        if count == 0 {
            return None;
        }
        Some(MenuPanel::right(self.pane_menu_anchor, PANE_MENU_W, count, window_bottom).rect)
    }

    /// 换掉某个 Pane 的指标：从 builtin() 现取那一个、灌上当前涨跌色，置脏让它下帧重算。
    /// 源码查看窗要是正开着这个 Pane，就顺手换成新指标的源码（不然标题和内容对不上）。
    pub(crate) fn set_pane_indicator(&mut self, pane: usize, index: usize) {
        let mut list = indicator::builtin();
        if index >= list.len() {
            return;
        }
        let mut ind = list.swap_remove(index);
        // 新指标照样灌当前整套颜色（K 线涨跌 + 数字涨跌 + MACD 两线 / KDJ / RSI / 均线）。
        ind.set_palette(self.current_palette());
        // 换成 K 线图（蜡烛图 / 美国线）就接着用当前的叠加均线配置。
        if ind.supports_overlay() {
            ind.set_overlay_state(self.overlay);
        }
        let ov_ok = ind.supports_overlay();
        let name = ind.name().to_owned();
        self.panes[pane].indicator = ind;
        self.panes[pane].dirty = true;
        // 换成不认叠加的指标时，那块图表的「叠加」菜单按钮就没了，菜单也顺手收起。
        if !ov_ok {
            self.overlay_menu = None;
        }
        self.status_msg = format!("Pane{} 换成：{}", pane + 1, name);
        // 编辑窗正开着这个 Pane：它手里的缓冲区还是旧指标的，得整份换成新指标那一份
        // （保存是按当前指标的 path() 写盘的，不换就会把旧文字写进新文件里）。
        if self.source.win.open && self.source.pane == pane {
            self.reload_source(pane);
        }
    }

    /// 指标切换下拉面板：先铺底，再逐条画（hover 高亮 + 名字 + 点击换指标），
    /// 最后把 1px 硬边压在最上面 - 先画边的话会被 hover 高亮啃掉一条像素。
    pub(crate) fn ui_pane_menu(&mut self, ui: &mut egui::Ui) {
        let Some(pane) = self.pane_menu else {
            return;
        };
        // 名字先拷成自己的（下面要调 self.text / self.set_pane_indicator，不能再借着 self.panes）。
        let names = indicator::builtin_names();
        let Some(panel) = self.pane_menu_rect(names.len(), ui.max_rect().max.y) else {
            return;
        };
        let painter = ui.painter().clone();
        let menu = MenuPanel { rect: panel, rows: names.len() };
        let clicked = menu.ui_rows(self, ui, &painter, |app, painter, j, row| {
            app.text(
                painter,
                egui::pos2(text_left_x(row.min.x), row.center().y),
                egui::Align2::LEFT_CENTER,
                &names[j],
                theme().text,
            );
        });

        if let Some(index) = clicked {
            self.set_pane_indicator(pane, index);
            self.pane_menu = None;
            return;
        }

        // 点面板和「切换」按钮以外的地方 -> 收起面板。
        let anchor = self.pane_menu_anchor;
        let outside = ui.ctx().input(|i| {
            i.pointer.primary_clicked()
                && i.pointer
                    .interact_pos()
                    .is_some_and(|p| !panel.contains(p) && !anchor.contains(p))
        });
        if outside {
            self.pane_menu = None;
        }
    }
}
