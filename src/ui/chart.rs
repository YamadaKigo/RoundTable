// ===========================================================================
//  模块 ui::chart：左栏图表
// ===========================================================================
//  K 线区（信息栏 / 主图 / 时间轴）、成交量区、指标区，以及滚轮缩放与左键平移。
//  本文件 = 拆分前那份单文件里的「九、功能区的蜡烛图」。
use crate::app::*;
use crate::font::*;
use crate::grid::*;
use crate::theme::*;
use crate::ui::colorpanel::*;
use crate::ui::panel::*;
use crate::data;
use eframe::egui;

// ===========================================================================
//  九、功能区的蜡烛图
// ===========================================================================
//  左栏从上到下拆成三块（尺寸都是 8 的倍数，中间各夹 1px 线格，见「十」里的 left_rows）：
//    K 线区（默认 600px）/ 1px / 横向拖拽条 8px / 1px / 成交量区（默认 96px）/ 下面留白。
//  K 线区内部再分三层：信息栏 24px / 1px / 主图（剩余吸到 8 的倍数）/ 1px / 时间轴 24px，
//  吸出来的零头（0..7px）留在时间轴下面。
//  主图：绘图区在左、价格轴在右 —— 价格轴占最右 8 个内容格（cells(8) = 73px）。
//  成交量区用同一套切法（所以价格轴和成交量轴上下一条线），刻度轴也是 cells(8) = 73px。
//  价格映射：取「可见 K 线」的最高价和最低价，线性映射到绘图区高度，上下各留 1 格，
//  结果吸到像素中心（整数行 + 0.5），所以实体和影线都落在整像素行上。
//  索引映射：绘图区宽度按 1 格一根均分，每根画在自己格子的正中。
//  只遍历可见的那一段（offset .. offset + visible），不碰全量数据。
//  十字光标：鼠标在绘图区里才画（离开就消失），两条 1px 硬边线严格限制在绘图区内部；
//  鼠标坐标先吸附到像素中心，再反算价格（y_of 的逆）和索引；右侧价格标签、
//  底部索引标签都用纯色矩形 + 1px 黑边，四边落在整数像素上。
//  信息栏：日期 开 高 低 收 涨跌 涨跌幅 成交量，显示鼠标悬停那根（鼠标不在图上就显示最后一根）；
//  每项宽度都是 8 的倍数，标签左对齐、数值右对齐，涨跌 / 涨跌幅按涨跌色上色。
//  时间轴：标注间隔按可见根数动态定（相邻标签中心至少隔 48px = 6 格），格式 MM-DD，跨年用 YY-MM-DD。
//  成交量：量柱宽度和左边缘跟蜡烛同一套算式（同一 offset / scale），颜色复用涨跌色；
//  纵轴按可见区间的最大成交量线性映射，上下各留 8px，右侧 3 条刻度按万 / 亿显示。
//  价格上下界和成交量上界都缓存在 view_cache 里 —— 键（代次 / offset / visible / scale）不变就不重算。

/// 成交量数值 → 文字：1e8 起步用「亿」，1e4 起步用「万」，都保留 1 位小数；更小的直接给整数。
pub(crate) fn fmt_volume(v: f64) -> String {
    if v >= YI {
        format!("{:.1}亿", v / YI)
    } else if v >= WAN {
        format!("{:.1}万", v / WAN)
    } else {
        format!("{v:.0}")
    }
}

/// 指标数值 → 图例文字：保留 2 位小数；没有值（慢线还没成型 / 没数据）就写「--」。
pub(crate) fn fmt_indicator(v: Option<f64>) -> String {
    match v {
        Some(v) => format!("{v:.2}"),
        None => "--".to_owned(),
    }
}

/// 日期 → "MM-DD"（时间轴上同一年里那种短标签）。
pub(crate) fn date_md(d: data::random_walk::Date) -> String {
    format!("{:02}-{:02}", d.m, d.d)
}

/// 日期 → "YY-MM-DD"（时间轴上跨年那一格用）。
pub(crate) fn date_ymd_short(d: data::random_walk::Date) -> String {
    format!("{:02}-{:02}-{:02}", d.y.rem_euclid(100), d.m, d.d)
}

/// 日期 → "YYYY-MM-DD"（信息栏里的完整日期）。
pub(crate) fn date_full(d: data::random_walk::Date) -> String {
    format!("{:04}-{:02}-{:02}", d.y, d.m, d.d)
}

/// 画一根蜡烛：1px 竖影线 + 实体（上涨空心 / 下跌实心）。参数全是像素：
/// left = 实体左边缘（整数像素），bar_w = 实体像素宽（奇数），
/// y_open / y_close / y_high / y_low = 四价换算好的像素中心 y。
/// 影线的中心列 = left + (bar_w - 1) / 2：bar_w 是奇数时它才落在整整一列像素的中间，
/// 实体左右两边的像素列数严格相等（tests::wick_is_centered_in_the_body_for_every_scale 盯着这条）。
/// 参数多是因为它只管画：几何全在调用处算好（那种「一堆像素坐标」的函数这里一律照旧）。
#[allow(clippy::too_many_arguments)]
pub(crate) fn paint_candle(
    painter: &egui::Painter,
    left: f32,
    bar_w: f32,
    rising: bool,
    color: egui::Color32,
    y_open: f32,
    y_close: f32,
    y_high: f32,
    y_low: f32,
) {
    let x = px_center(left + (bar_w - LINE_W) / 2.0);
    painter.line_segment(
        [egui::pos2(x, y_high), egui::pos2(x, y_low)],
        egui::Stroke::new(LINE_W, color),
    );

    let top = (y_open - 0.5).round().min((y_close - 0.5).round());
    let height = ((y_open - 0.5).round() - (y_close - 0.5).round()).abs() + LINE_W;
    let rect = egui::Rect::from_min_size(egui::pos2(left, top), egui::vec2(bar_w, height));

    // 上涨空心：先铺满再挖掉内部（四边各留 1px 硬边，中间填绘图区底色）；
    // 下跌实心：整块纯色。宽度或高度不足 3 像素时画不出「空心」，一律退化成实心。
    let hollow = rising && bar_w >= 3.0 && height >= 3.0;
    painter.rect_filled(rect, egui::CornerRadius::ZERO, color);
    if hollow {
        let inner = egui::Rect::from_min_max(
            egui::pos2(rect.min.x + LINE_W, rect.min.y + LINE_W),
            egui::pos2(rect.max.x - LINE_W, rect.max.y - LINE_W),
        );
        painter.rect_filled(inner, egui::CornerRadius::ZERO, C_CHART_BG);
    }
}

impl StockApp {
    /// 功能区左栏 →（绘图区, 价格轴）：价格轴占最右边 AXIS_COLS 个内容格，
    /// 两者共用中间那条网格线，各自尺寸都是整数格。
    /// 绘图区左边缘再让开 1px：最左那一列是窗口外框的线格（铁律第 2/8 条），
    /// 不让开的话，最左那根蜡烛会被黑边框啃掉一列，看着就不居中了。
    pub(crate) fn chart_rects(work: egui::Rect) -> (egui::Rect, egui::Rect) {
        let edge = work.max.x - AXIS_W;
        let plot = egui::Rect::from_min_max(
            egui::pos2(work.min.x + LINE_W, work.min.y),
            egui::pos2(edge, work.max.y),
        );
        let axis = egui::Rect::from_min_max(egui::pos2(edge, work.min.y), work.max);
        (plot, axis)
    }

    /// K 线区内部再切三层：信息栏 24px / 1px 线格 / 主图（剩余吸到 8 的倍数）/ 1px 线格 / 时间轴 24px。
    /// 主图右边的价格轴还是 AXIS_W；信息栏和时间轴横贯整块宽，三块上下共用那两条 1px 线格 ——
    /// 所以主图高度 = 剩余高度扣掉时间轴和它上面那条线格，再向下吸到 8 的倍数，
    /// 吸掉的零头（0..7px）留在时间轴下面，空着不塞东西。
    pub(crate) fn kline_layers(area: egui::Rect) -> [egui::Rect; 3] {
        let info = egui::Rect::from_min_size(area.min, egui::vec2(area.width(), INFO_H));
        let main_y = area.min.y + INFO_H + LINE_W;
        let main_h = snap8_down((area.max.y - main_y - TIME_AXIS_H - LINE_W).max(0.0));
        let main = egui::Rect::from_min_size(
            egui::pos2(area.min.x, main_y),
            egui::vec2(area.width(), main_h),
        );
        let time = egui::Rect::from_min_size(
            egui::pos2(area.min.x, main_y + main_h + LINE_W),
            egui::vec2(area.width(), TIME_AXIS_H),
        );
        [info, main, time]
    }

    /// 画 K 线区：信息栏 + 主图（白底绘图区、右侧灰色价格轴、蜡烛、刻度、十字光标）+ 时间轴。
    /// 缩放 / 平移也在这里处理：鼠标位置这一帧只取一次，先动视图、再按最终的
    /// scale / offset 出图，十字光标的索引标签和信息栏的内容才不会慢一帧。
    pub(crate) fn ui_kline(&mut self, ui: &mut egui::Ui, area: egui::Rect) {
        let painter = ui.painter().clone();
        let [info, main, time] = Self::kline_layers(area);
        let (plot, axis) = Self::chart_rects(main);
        // 整块先铺工作区白（信息栏 / 时间轴用它）；主图白底、价格轴灰底。
        // 价格轴那一条灰底从主图顶端一直铺到 K 线区底端：时间轴那一段、以及时间轴下面
        // 吸零头剩下的余量（0..7px）都在里面 —— 右边从上到下一条灰，不会露出白色横条。
        painter.rect_filled(area, egui::CornerRadius::ZERO, C_WORK_BG);
        painter.rect_filled(plot, egui::CornerRadius::ZERO, C_CHART_BG);
        painter.rect_filled(
            egui::Rect::from_min_max(
                egui::pos2(axis.min.x, main.min.y),
                egui::pos2(axis.max.x, area.max.y),
            ),
            egui::CornerRadius::ZERO,
            C_AXIS_BG,
        );
        // 三块之间那两条 1px 线格（信息栏下沿、时间轴上沿）：它们落在两块之间的空行上，
        // 谁也吃不掉谁的像素。这里就画掉 —— 没有数据提前返回时，这两条线也照样在。
        hline_hard(&painter, area.min.x, area.max.x, info.max.y, C_LINE);
        hline_hard(&painter, area.min.x, area.max.x, time.min.y - LINE_W, C_LINE);

        // 绘图区里真正允许画东西的范围：上下各留 1 格（那条网格线也留出来）。
        let pad = grid_line(PLOT_PAD_COLS);
        let body = egui::Rect::from_min_max(
            egui::pos2(plot.min.x, plot.min.y + pad),
            egui::pos2(plot.max.x, plot.max.y - pad),
        );

        // 鼠标位置这一帧只取一次：缩放锚点、平移起点、十字光标都用它，互相不覆盖状态。
        // （拖拽中鼠标滑出绘图区也还留着它，十字线不会闪掉。）
        let hover = ui.input(|i| i.pointer.hover_pos());

        // ---- 视图操作：滚轮缩放（锚点缩放）+ 左键拖拽平移 ----
        let notches = self.take_wheel_notches(ui, plot, hover);
        if notches != 0 {
            if let Some(p) = hover {
                self.zoom_around(p.x - plot.min.x, notches);
            }
        }
        self.pan_by_drag(ui, plot);
        // 视图动完再重算一屏放几根，并把 offset 夹回合法范围（边界处钳住，拖不出空白）。
        self.clamp_view(plot.width());

        let visible = self.visible;
        if visible == 0 {
            return;
        }

        // 价格范围 / 成交量上界都走映射缓存：键（数据代次 + offset + visible + scale）没变就不重算，
        // 只有真的滚了 / 拖了 / 换了数据才扫一遍可见区间（规格：数据不变不重算）。
        self.refresh_view_cache();
        let (hi, lo) = (self.view_cache.price_hi, self.view_cache.price_lo);
        let span = (hi - lo).max(f64::EPSILON);

        // 价格 → 像素中心：最高价贴 body 最上一行，最低价贴最下一行。
        let y_of = |price: f64| -> f32 {
            let t = ((hi - price) / span) as f32;
            px_center(body.min.y + t * (body.height() - LINE_W))
        };

        // 刻度：从上到下 5 条横向参考线铺满绘图区，价格数字右对齐写在轴内。
        for k in 0..TICK_COUNT {
            let t = k as f32 / (TICK_COUNT - 1) as f32;
            let y = y_of(hi - (hi - lo) * t as f64);
            hline_hard(&painter, plot.min.x, plot.max.x, y, C_TICK);
            let label = format!("{:.2}", hi - (hi - lo) * t as f64);
            self.text(
                &painter,
                egui::pos2(axis.max.x - TEXT_PAD_X, y),
                egui::Align2::RIGHT_CENTER,
                &label,
                C_TEXT,
            );
        }

        // 蜡烛：横向按「实体 scale 像素 + 必要时那 1px 间隙」一根挨一根铺开 ——
        // 第 i 根的实体左边 = 绘图区左边缘 + i * 步进，都是整数像素，直接落在像素栅格上，
        // 所以实体永远是整数像素宽（不会出现 1.5px），一屏正好放得下 visible 根。
        // 每根交给 paint_candle：先影线（固定 1px 竖线，最高→最低，画在实体正中那一列），
        // 再实体（开盘→收盘）；实体宽是奇数（snap_scale 兜底），正中那一列才存在。
        let advance = bar_advance(self.scale);
        let bar_w = snap_scale(self.scale) as f32;
        let slice = &self.bars[self.offset..self.offset + visible];
        for (i, bar) in slice.iter().enumerate() {
            let left = (plot.min.x + i as f32 * advance).round();
            let rising = bar.c >= bar.o;
            // 涨跌色改从 struct 字段取（颜色面板改的就是它俩），不再写死 C_UP / C_DOWN。
            let rgb = if rising {
                self.up_color
            } else {
                self.down_color
            };
            let color = rgb32(rgb);
            paint_candle(
                &painter,
                left,
                bar_w,
                rising,
                color,
                y_of(bar.o),
                y_of(bar.c),
                y_of(bar.h),
                y_of(bar.l),
            );
        }

        // 价格轴左边那条 1px 硬边最后画，压在蜡烛上面。
        vline_hard(&painter, axis.min.x, axis.min.y, axis.max.y, C_LINE);

        // ---- 十字光标 ----
        // 用帧首取好的 hover：只有落在绘图区里才算数，鼠标在价格轴、菜单栏、状态栏或
        // 窗口外面时直接清空（十字线立刻消失）；正在拖拽平移时不清空、坐标夹进绘图区，
        // 十字线一直跟着（否则往左右拖出绘图区那一瞬间线就闪没了）。
        let offset = self.offset;
        let dragging = self.pan.is_some();
        let point = hover.or_else(|| {
            if dragging {
                ui.input(|i| i.pointer.latest_pos())
            } else {
                None
            }
        });
        self.crosshair = point
            .filter(|p| dragging || plot.contains(*p))
            .map(|p| {
                // 鼠标坐标是浮点，先吸附到像素中心再画：否则 1px 的线会横跨两列像素、
                // 被渲染成 2px 灰边。横竖都夹在绘图区里，右边界给价格轴那条分隔线留位置。
                let x = px_center(p.x.clamp(plot.min.x, plot.max.x - LINE_W));
                let y = px_center(p.y.clamp(plot.min.y, plot.max.y - LINE_W));
                // 价格 = y_of 的逆运算（同一个 body 高度、同一个 1px 修正），
                // 所以标签上的数字和十字线所在的高度严丝合缝。
                let t = ((y - body.min.y) / (body.height() - LINE_W)).clamp(0.0, 1.0);
                // 索引 = 这一列落在第几根里（按步进算）+ 屏幕起点 offset，越界夹在可见范围内。
                let slot = ((x - plot.min.x) / advance).floor().max(0.0) as usize;
                Crosshair {
                    x,
                    y,
                    price: hi - span * t as f64,
                    index: offset + slot.min(visible - 1),
                }
            });

        // 再画：两条 1px 硬边线（只落在绘图区内部）+ 右侧价格标签 + 底部索引标签。
        if let Some(cross) = self.crosshair {
            vline_hard(&painter, cross.x, plot.min.y, plot.max.y, C_CROSS);
            hline_hard(&painter, plot.min.x, plot.max.x, cross.y, C_CROSS);

            // 价格标签：贴在价格轴里右对齐（和刻度数字同一条基准线），底色盖住底下的刻度。
            let price_text = format!("{:.2}", cross.price);
            let w = self.cross_label_w(&price_text).min(axis.width() - 2.0 * LINE_W);
            let x0 = (axis.max.x - TEXT_PAD_X - w)
                .round()
                .clamp(axis.min.x + LINE_W, axis.max.x - w - LINE_W);
            let y0 = (cross.y - CROSS_LABEL_H / 2.0)
                .round()
                .clamp(plot.min.y, plot.max.y - CROSS_LABEL_H);
            self.cross_label(&painter, &price_text, egui::pos2(x0, y0), w);

            // 索引标签：水平方向居中在竖线上，垂直方向贴绘图区下沿。
            let index_text = format!("{}", cross.index);
            let w = self.cross_label_w(&index_text).min(plot.width());
            let x0 = (cross.x - w / 2.0)
                .round()
                .clamp(plot.min.x, plot.max.x - w);
            self.cross_label(
                &painter,
                &index_text,
                egui::pos2(x0, plot.max.y - CROSS_LABEL_H),
                w,
            );
        }

        // ---- 信息栏：鼠标悬停那根 K 线的数据（鼠标不在图上就显示最后一根） ----
        // 十字光标上一步已经算好，这里直接读它的下标，所以信息栏和十字线永远指着同一根。
        let info_index = match self.crosshair {
            Some(cross) => cross.index,
            None => self.bars.len().saturating_sub(1),
        };
        self.ui_info_bar(&painter, info, info_index);

        // ---- 时间轴：按可见根数动态决定标注间隔（相邻标签中心至少隔 48px = 6 格） ----
        self.ui_time_axis(&painter, time, plot, advance);
    }

    /// 重算映射缓存：可见区间的价格上下界 + 最大成交量。
    /// 键（数据代次 + offset + visible + scale）没变就直接返回 —— 不动视图的那些帧一次数据扫描都不做。
    pub(crate) fn refresh_view_cache(&mut self) {
        let c = self.view_cache;
        if c.generation == self.data_gen
            && c.offset == self.offset
            && c.visible == self.visible
            && c.scale == self.scale
        {
            return;
        }
        let (mut hi, mut lo, mut vol) = (f64::MIN, f64::MAX, 0.0f64);
        if self.visible > 0 {
            for bar in &self.bars[self.offset..self.offset + self.visible] {
                hi = hi.max(bar.h);
                lo = lo.min(bar.l);
                vol = vol.max(bar.vol);
            }
        }
        self.view_cache = ViewCache {
            generation: self.data_gen,
            offset: self.offset,
            visible: self.visible,
            scale: self.scale,
            price_hi: hi,
            price_lo: lo,
            vol_max: vol,
        };
    }

    /// 信息栏：日期 开 高 低 收 涨跌 涨跌幅 成交量，每一项宽度都是 8 的倍数。
    /// 标签左对齐、数值右对齐（数值右边缘在本项最后 1 格之前，那 1 格就是两项之间的空隙）；
    /// 涨跌和涨跌幅按涨跌色上色 —— 涨用涨色、跌用跌色。
    pub(crate) fn ui_info_bar(&mut self, painter: &egui::Painter, area: egui::Rect, index: usize) {
        if self.bars.is_empty() {
            return;
        }
        let index = index.min(self.bars.len() - 1);
        let bar = self.bars[index];
        // 涨跌 = 收盘 - 昨收（第一根没有昨收，就拿它自己的开盘当基准）。
        let prev_close = if index > 0 { self.bars[index - 1].c } else { bar.o };
        let change = bar.c - prev_close;
        let pct = if prev_close.abs() > f64::EPSILON {
            change / prev_close * 100.0
        } else {
            0.0
        };
        let (up, down) = (rgb32(self.up_color), rgb32(self.down_color));
        let trend = if change >= 0.0 { up } else { down };
        let values = [
            (date_full(bar.date), C_TEXT),
            (format!("{:.2}", bar.o), C_TEXT),
            (format!("{:.2}", bar.h), C_TEXT),
            (format!("{:.2}", bar.l), C_TEXT),
            (format!("{:.2}", bar.c), C_TEXT),
            (format!("{change:+.2}"), trend),
            (format!("{pct:+.2}%"), trend),
            (fmt_volume(bar.vol), C_TEXT),
        ];
        let mut x = area.min.x + GRID;
        for (i, (value, color)) in values.iter().enumerate() {
            let w = self.info_item_w(i);
            // 左栏被拖窄 / 窗口变小的时候放不下的项就不画了：宁可少显示一项，
            // 也不让文字越过左栏边界压到分割条和右栏上。
            if x + w > area.max.x {
                break;
            }
            self.text(
                painter,
                egui::pos2(x, area.center().y),
                egui::Align2::LEFT_CENTER,
                INFO_LABELS[i],
                C_TEXT,
            );
            self.text(
                painter,
                egui::pos2(x + w - GRID, area.center().y),
                egui::Align2::RIGHT_CENTER,
                value,
                *color,
            );
            x += w;
        }
    }

    /// 信息栏一项的宽度 = 标签宽 + 1 格空隙 + 模板数值宽 + 右侧 1 格空隙，向上吸到 8 的倍数。
    /// 量的是固定的「模板」而不是当前值：同一位数字换来换去时，项的宽度和位置不会跟着抖。
    pub(crate) fn info_item_w(&mut self, i: usize) -> f32 {
        let label = self.text_width(INFO_LABELS[i]);
        let value = self.text_width(INFO_TEMPLATES[i]);
        snap8_up(label + GRID + value + GRID)
    }

    /// 时间轴：按可见根数动态决定标注间隔，保证相邻标签中心间距 >= 48px（6 格）。
    /// 标注只落在「下标能被 step 整除」的那些 K 线上 —— 标签跟着数据走，拖动时不会一帧一个位置。
    /// 跨年那一格换成 YY-MM-DD（和上一根被标注的 K 线年份不同就算跨年），其余是 MM-DD。
    pub(crate) fn ui_time_axis(
        &mut self,
        painter: &egui::Painter,
        area: egui::Rect,
        plot: egui::Rect,
        advance: f32,
    ) {
        let visible = self.visible;
        if visible == 0 {
            return;
        }
        let step = self.time_step(advance);
        let bar_w = snap_scale(self.scale) as f32;
        for i in 0..visible {
            let idx = self.offset + i;
            if idx % step != 0 {
                continue;
            }
            let bar = self.bars[idx];
            let cross_year = idx < step || self.bars[idx - step].date.y != bar.date.y;
            let label = if cross_year {
                date_ymd_short(bar.date)
            } else {
                date_md(bar.date)
            };
            let w = self.text_width(&label);
            let cx = (plot.min.x + i as f32 * advance + bar_w / 2.0).round();
            // 只画整段都落在绘图区里的标签：压到价格轴上或只露出一半的，干脆不画。
            if cx - w / 2.0 < plot.min.x || cx + w / 2.0 > plot.max.x - LINE_W {
                continue;
            }
            self.text(
                painter,
                egui::pos2(cx, area.center().y),
                egui::Align2::CENTER_CENTER,
                &label,
                C_TEXT,
            );
        }
    }

    /// 时间轴标注间隔（单位：根）：让相邻标签中心至少隔 48px（6 格）。
    /// 用「MM-DD」和跨年那种「YY-MM-DD」里更宽的一个来算，窄屏 / 大 scale 下也不会挤成一团。
    pub(crate) fn time_step(&mut self, advance: f32) -> usize {
        let label_w = self.text_width("00-00").max(self.text_width("00-00-00"));
        let need = (label_w + GRID).max(TIME_LABEL_GAP_MIN);
        ((need / advance).ceil() as usize).max(1)
    }

    /// 成交量区：量柱的宽度和左边缘跟蜡烛用同一套算式（同一个 offset / scale / 步进），
    /// 所以两者严格对齐；颜色也复用涨跌色。
    /// 纵轴按可见区间的最大成交量线性映射，上下各留 8px 边距；右边 AXIS_W 是刻度轴，
    /// 2~3 条刻度，数值换算成万 / 亿保留 1 位小数。它不接交互 —— 滚动 / 平移都算在主图上，
    /// 两块共用 offset / scale，自然就同步了。
    pub(crate) fn ui_volume(&mut self, painter: &egui::Painter, area: egui::Rect) {
        let (plot, axis) = Self::chart_rects(area);
        painter.rect_filled(area, egui::CornerRadius::ZERO, C_CHART_BG);
        painter.rect_filled(axis, egui::CornerRadius::ZERO, C_AXIS_BG);
        let visible = self.visible;
        if visible == 0 {
            return;
        }
        // 基线和可用高度：上下各留 1 格（8px），最大的那根量柱正好顶到上边距。
        let vmax = self.view_cache.vol_max.max(f64::EPSILON);
        let base = area.max.y - VOL_PAD_Y;
        let usable = (area.height() - 2.0 * VOL_PAD_Y).max(LINE_W);
        let y_of = |v: f64| px_center(base - (v / vmax).clamp(0.0, 1.0) as f32 * usable);

        for k in 0..VOL_TICK_COUNT {
            let t = k as f32 / (VOL_TICK_COUNT - 1) as f32;
            let v = vmax * t as f64;
            let y = y_of(v);
            hline_hard(painter, plot.min.x, plot.max.x, y, C_TICK);
            self.text(
                painter,
                egui::pos2(axis.max.x - TEXT_PAD_X, y),
                egui::Align2::RIGHT_CENTER,
                &fmt_volume(v),
                C_TEXT,
            );
        }

        let advance = bar_advance(self.scale);
        let bar_w = snap_scale(self.scale) as f32;
        for (i, bar) in self.bars[self.offset..self.offset + visible].iter().enumerate() {
            let left = (plot.min.x + i as f32 * advance).round();
            let color = if bar.c >= bar.o {
                self.up_color
            } else {
                self.down_color
            };
            painter.rect_filled(
                egui::Rect::from_min_max(
                    egui::pos2(left, y_of(bar.vol)),
                    egui::pos2(left + bar_w, base),
                ),
                egui::CornerRadius::ZERO,
                rgb32(color),
            );
        }

        // 价格轴左边那条 1px 硬边最后画（和主图一个做法）。
        vline_hard(painter, axis.min.x, axis.min.y, axis.max.y, C_LINE);
    }

    /// 指标区：骨架和成交量区一样（白底绘图区 + 右侧灰色刻度轴 + 2~3 条刻度线），内容按
    /// 当前指标的 Output 画 —— 先刻度、再 0 轴、再柱子、再折线，图例最后压在左上角。
    /// 横轴和蜡烛 / 量柱共用同一个 offset / scale，所以上中下三层永远对齐。
    /// 这个函数不认识任何具体指标：以后加指标只改 src/indicator/，这儿一行都不用动。
    pub(crate) fn ui_indicator(&mut self, painter: &egui::Painter, area: egui::Rect) {
        let (plot, axis) = Self::chart_rects(area);
        painter.rect_filled(area, egui::CornerRadius::ZERO, C_CHART_BG);
        painter.rect_filled(axis, egui::CornerRadius::ZERO, C_AXIS_BG);

        let visible = self.visible;
        if visible == 0 {
            return;
        }
        let (up, down) = (rgb32(self.up_color), rgb32(self.down_color));
        let advance = bar_advance(self.scale);
        let bar_w = snap_scale(self.scale) as f32;
        // 图例显示鼠标悬停那一根的值（鼠标不在图上就是最后一根），和 K 线区信息栏一个规矩。
        let index = match self.crosshair {
            Some(cross) => cross.index,
            None => self.bars.len().saturating_sub(1),
        };

        // ---- 第一步：只读地把这一帧要画的东西算出来（像素坐标 + 图例文字） ----
        // 算的时候借 indicators / bars，后面画字要 &mut self；两步分开，借用互不打架。
        let generation = self.data_gen;
        let mut hist_rects: Vec<(egui::Rect, egui::Color32)> = Vec::new();
        let mut polylines: Vec<(egui::Color32, Vec<(f32, f32)>)> = Vec::new();
        let mut ticks: Vec<(String, f32)> = Vec::new();
        let mut legend: Vec<(String, egui::Color32)> = Vec::new();
        let mut zero_y = None;
        {
            let out = self.indicators.output(generation, &self.bars);
            let from = self.offset;
            let to = (self.offset + visible).min(self.bars.len());
            // 值域 = 可见区间里所有序列的上下界；以 0 对称的指标（MACD）再把 0 摆到正中。
            let (mut lo, mut hi) = (f64::MAX, f64::MIN);
            for i in from..to {
                for line in &out.lines {
                    if let Some(v) = line.values.get(i).copied().flatten() {
                        lo = lo.min(v);
                        hi = hi.max(v);
                    }
                }
                if let Some(hist) = &out.hist {
                    if let Some(v) = hist.values.get(i).copied().flatten() {
                        lo = lo.min(v);
                        hi = hi.max(v);
                    }
                }
            }
            if lo > hi {
                // 可见区间里一条值都没有：给个平的值域，别让下面除零。
                lo = 0.0;
                hi = 0.0;
            }
            if out.symmetric {
                let m = lo.abs().max(hi.abs()).max(f64::EPSILON);
                lo = -m;
                hi = m;
            }
            let span = (hi - lo).max(f64::EPSILON);
            let top = area.min.y + VOL_PAD_Y;
            let bottom = area.max.y - VOL_PAD_Y;
            // 值 → 像素行：hi 贴上面那条边距线、lo 贴下面那条（和量区同一套边距）。
            let y_of = |v: f64| {
                let t = ((hi - v) / span).clamp(0.0, 1.0) as f32;
                px_center(top + t * (bottom - top))
            };
            // 刻度：从上到下均匀几条，数值右对齐写进右边那条灰轴里。
            for k in 0..IND_TICK_COUNT {
                let t = k as f32 / (IND_TICK_COUNT - 1) as f32;
                let v = hi - (hi - lo) * t as f64;
                ticks.push((format!("{v:.2}"), y_of(v)));
            }
            // 0 轴：值域跨过 0 才画（以 0 对称的指标正好落在正中间）。
            if lo < 0.0 && hi > 0.0 {
                zero_y = Some(y_of(0.0));
            }
            // 柱子：每根和蜡烛同宽、同左边缘，正负分别用涨跌色。
            if let Some(hist) = &out.hist {
                let base = y_of(0.0);
                for i in from..to {
                    let Some(v) = hist.values.get(i).copied().flatten() else {
                        continue;
                    };
                    let left = (plot.min.x + (i - from) as f32 * advance).round();
                    let y = y_of(v);
                    hist_rects.push((
                        egui::Rect::from_min_max(
                            egui::pos2(left, y.min(base)),
                            egui::pos2(left + bar_w, y.max(base)),
                        ),
                        if v >= 0.0 { up } else { down },
                    ));
                }
            }
            // 折线：每根的 x 取蜡烛影线那一列（三层严格对齐），中间断档就另起一段。
            for line in &out.lines {
                let color = rgb32(line.rgb);
                let mut run: Vec<(f32, f32)> = Vec::new();
                for i in from..to {
                    match line.values.get(i).copied().flatten() {
                        Some(v) => {
                            let left = (plot.min.x + (i - from) as f32 * advance).round();
                            run.push((left + (bar_w - LINE_W) / 2.0, y_of(v)));
                        }
                        None => {
                            // 只有两个点以上才连得成线，单点直接丢掉。
                            if run.len() > 1 {
                                polylines.push((color, std::mem::take(&mut run)));
                            } else {
                                run.clear();
                            }
                        }
                    }
                }
                if run.len() > 1 {
                    polylines.push((color, run));
                }
            }
            // 图例：标题 + 每条线 / 柱子的名字和值（值取光标那一根）。
            legend.push((out.title.clone(), C_TEXT));
            for line in &out.lines {
                let v = line.values.get(index).copied().flatten();
                legend.push((format!("{} {}", line.name, fmt_indicator(v)), rgb32(line.rgb)));
            }
            if let Some(hist) = &out.hist {
                let v = hist.values.get(index).copied().flatten();
                let color = if v.is_some_and(|v| v < 0.0) { down } else { up };
                legend.push((format!("MACD {}", fmt_indicator(v)), color));
            }
        }

        // ---- 第二步：画 ----
        for (label, y) in &ticks {
            hline_hard(painter, plot.min.x, plot.max.x, *y, C_TICK);
            self.text(
                painter,
                egui::pos2(axis.max.x - TEXT_PAD_X, *y),
                egui::Align2::RIGHT_CENTER,
                label,
                C_TEXT,
            );
        }
        if let Some(y) = zero_y {
            hline_hard(painter, plot.min.x, plot.max.x, y, C_IND_ZERO);
        }
        for (rect, color) in &hist_rects {
            painter.rect_filled(*rect, egui::CornerRadius::ZERO, *color);
        }
        for (color, pts) in &polylines {
            polyline_hard(painter, pts, *color);
        }
        // 轴左边那条 1px 硬边（和主图 / 量区一个做法），压在曲线之上。
        vline_hard(painter, axis.min.x, axis.min.y, axis.max.y, C_LINE);

        // 图例压在左上角，画在最后（曲线盖不住它）；一项都放不下就不画了。
        let mut x = plot.min.x + GRID;
        let y = plot.min.y + FONT_SIZE / 2.0;
        for (text, color) in &legend {
            let w = self.text_width(text);
            if x + w > plot.max.x - GRID {
                break;
            }
            self.text(
                painter,
                egui::pos2(x, y),
                egui::Align2::LEFT_CENTER,
                text,
                *color,
            );
            x += w + GRID;
        }
    }

    /// 这一帧滚轮滚了几格（正 = 往上滚 = 放大）。只有鼠标在绘图区里才算数：
    /// 在控制面板或别处滚滚轮，K 线不动，也不会把增量攒在字段里留着下次发作。
    /// 不足一格的零头（触控板那种按点上报的增量）存在 zoom_acc 里，下一帧接着凑。
    pub(crate) fn take_wheel_notches(
        &mut self,
        ui: &egui::Ui,
        plot: egui::Rect,
        hover: Option<egui::Pos2>,
    ) -> i32 {
        if !hover.is_some_and(|p| plot.contains(p)) {
            self.zoom_acc = 0.0;
            return 0;
        }
        let mut acc = self.zoom_acc;
        ui.input(|i| {
            for event in &i.events {
                if let egui::Event::MouseWheel { unit, delta, .. } = event {
                    // Windows 上一格就是 1 行：winit 把 WM_MOUSEWHEEL 的 ±120 除以 120。
                    acc += match unit {
                        egui::MouseWheelUnit::Line => delta.y,
                        egui::MouseWheelUnit::Point => delta.y / WHEEL_POINTS_PER_NOTCH,
                        egui::MouseWheelUnit::Page => delta.y,
                    };
                }
            }
        });
        let whole = acc.trunc();
        self.zoom_acc = acc - whole;
        whole as i32
    }

    /// 锚点缩放：鼠标指着的那根 K 线，缩放前后必须停在同一屏幕 x 上（手感全靠这个）。
    ///   mouse_bar  = offset + (mouse_x - 绘图区左边) / 旧步进
    ///   new_offset = mouse_bar - (mouse_x - 绘图区左边) / 新步进
    /// 最后把这根 K 线的下标取整 —— offset 是整数索引，只能整根整根地挪。
    pub(crate) fn zoom_around(&mut self, x_rel: f32, notches: i32) {
        let old = snap_scale(self.scale);
        let new = scale_after(old, notches);
        if new == old {
            return;
        }
        let mouse_bar = self.offset as f32 + x_rel / bar_advance(old);
        self.scale = new;
        self.offset = (mouse_bar - x_rel / bar_advance(new)).round().max(0.0) as usize;
    }

    /// 左键拖拽平移：在绘图区按住左键左右拖，offset 按「位移 / 步进」根变化（取整）。
    /// 向右拖 = 看更早的数据（offset 变小），向左拖 = 看更晚的数据（offset 变大）。
    /// 记的是按下那一刻的鼠标 x 和 offset，每帧按总位移反算；拖到边界被钳住之后再往回拖，
    /// 视图会从钳住的位置接着走，不会「飘」。
    pub(crate) fn pan_by_drag(&mut self, ui: &egui::Ui, plot: egui::Rect) {
        let resp = ui.interact(plot, egui::Id::new("rt_chart_pan"), egui::Sense::drag());
        let down = ui.input(|i| i.pointer.button_down(egui::PointerButton::Primary));
        if resp.drag_started_by(egui::PointerButton::Primary) {
            let x = ui.input(|i| i.pointer.latest_pos()).map_or(0.0, |p| p.x);
            self.pan = Some((x, self.offset));
        }
        if resp.dragged_by(egui::PointerButton::Primary) {
            if let (Some((start_x, start_offset)), Some(p)) =
                (self.pan, ui.input(|i| i.pointer.latest_pos()))
            {
                let bars = (((p.x - start_x) / bar_advance(self.scale)) as f64).round();
                self.offset = (start_offset as f64 - bars).max(0.0) as usize;
            }
        }
        // 左键一松就结束这次平移（不只认这一帧的 drag_stopped）：pan 只在这个函数里维护，
        // 而且「还按着左键」是它唯一的存活条件 —— 十字光标靠它判断自己该不该继续显示。
        if !down {
            self.pan = None;
        }
    }

    /// 重算一屏放几根，并把 offset 夹回 [0, 总根数 - 可见根数]：边界处钳住，
    /// 既拖不出左边的空白，也不会让最后一根后面露出空槽。
    pub(crate) fn clamp_view(&mut self, plot_w: f32) {
        self.visible = visible_bars(plot_w, self.scale).min(self.bars.len());
        self.offset = self.offset.min(self.bars.len().saturating_sub(self.visible));
    }

    /// 十字光标标签的宽度 = 文字宽 + 左右各 9px 内边距，向上吸到整数格（最小 2 格）。
    pub(crate) fn cross_label_w(&mut self, text: &str) -> f32 {
        snap_cells_up(self.text_width(text) + 2.0 * TEXT_PAD_X).max(cells(2))
    }

    /// 画一个十字光标标签：纯色底 + 1px 黑边 + 居中文字，位置和尺寸都落在整数像素上。
    pub(crate) fn cross_label(&mut self, painter: &egui::Painter, text: &str, pos: egui::Pos2, w: f32) {
        let rect = egui::Rect::from_min_size(pos, egui::vec2(w, CROSS_LABEL_H));
        painter.rect_filled(rect, egui::CornerRadius::ZERO, C_CROSS_LABEL);
        border_hard(painter, rect, C_BORDER);
        self.text(
            painter,
            rect.center(),
            egui::Align2::CENTER_CENTER,
            text,
            C_TEXT,
        );
    }
}

