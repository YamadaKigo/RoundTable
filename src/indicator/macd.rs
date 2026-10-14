// ===========================================================================
//  指标：MACD
// ===========================================================================
//  参数 12 / 26 / 9：DIF = EMA(快) - EMA(慢)；DEA = DIF 的 EMA(信号)；
//  柱子 = (DIF - DEA) * 2（国内习惯乘 2）。慢线要 26 根才稳，所以前 slow-1 根没有值。
//
//  指令顺序 = 绘制顺序（0 轴 -> 柱子 -> 两条折线），重叠处谁盖谁才不会变。
//  0 轴走 HLine：它是装饰线，不参与值域扫描（重放时值域跨过 0 就自动画出来）。
//  纵轴以 0 对称（value_range 把值域掰正）；状态带上是 DIF / DEA / MACD 三条序列的当前值。

// rgb_u32 只有单元测试用（按颜色认折线），正式构建里用不上。
#[allow(unused_imports)]
use super::{
    axis_ticks, chart_cross, chart_cross_bg, chart_text, chart_tick, chart_zero, crosshair, ema,
    rgb_u32, ChartCtx, ChartLayout, DrawCmd, Indicator, Palette, Rgb,
};
use crate::data::random_walk::Bar;

/// 两条线的经典配色（#3333CC 蓝 / #CC9900 橙）：也是「经典」皮肤里 dif / dea 的默认值，
/// 单元测试拿它们当参照色。真正画线用的是界面通过 set_palette 灌进来的那份（见 Palette）。
#[allow(dead_code)]
pub const DIF_RGB: Rgb = (0x33, 0x33, 0xCC);
#[allow(dead_code)]
pub const DEA_RGB: Rgb = (0xCC, 0x99, 0x00);

/// 这块图表要一条 2 格的状态带（一个文本行盒，放 DIF / DEA / MACD 的当前值），不要时间带。
/// 状态带 2 格（一个文本行盒），文字才和网格对齐。
const STATUS_ROWS: i32 = 2;
/// 右侧纵轴刻度几条。
const VALUE_TICKS: usize = 3;

/// MACD。
pub struct Macd {
    fast: usize,
    slow: usize,
    signal: usize,
    title: String,
    /// 界面灌进来的整套颜色（柱子用 K 线涨跌色、两条线用 dif / dea）。
    pal: Palette,
}

impl Macd {
    pub fn new(fast: usize, slow: usize, signal: usize) -> Self {
        Self {
            fast,
            slow,
            signal,
            title: format!("MACD({fast},{slow},{signal})"),
            pal: Palette::default(),
        }
    }
}

impl Default for Macd {
    fn default() -> Self {
        Self::new(12, 26, 9)
    }
}

/// 收盘价序列。
fn closes(bars: &[Bar]) -> Vec<f64> {
    bars.iter().map(|bar| bar.c).collect()
}

// ema（指数移动平均）在 indicator/mod.rs 里：叠加的 EMA 均线也是同一条递推，全工程只有一份。

/// 数值 -> 图例文字：保留 2 位小数；没有值（慢线还没成型 / 没数据）就写「--」。
fn fmt_value(v: Option<f64>) -> String {
    match v {
        Some(v) => format!("{v:.2}"),
        None => "--".to_owned(),
    }
}

impl Indicator for Macd {
    fn name(&self) -> &str {
        &self.title
    }

    /// 自己的源码：路径相对本文件所在目录解析，所以直接写自己的文件名。
    fn source(&self) -> &'static str {
        include_str!("macd.rs")
    }

    /// 自己的源文件路径：file!() 宏给的就是「相对仓库根」的那个路径，不手写常量。
    fn path(&self) -> &'static str {
        file!()
    }

    /// 纵轴以 0 对称：正负两边留一样的高度，0 轴才会落在正中。
    fn value_range(&self, lo: f64, hi: f64) -> (f64, f64) {
        let m = lo.abs().max(hi.abs()).max(f64::EPSILON);
        (-m, m)
    }

    fn layout(&self) -> ChartLayout {
        ChartLayout::with_status(STATUS_ROWS)
    }

    /// 右侧纵轴刻度：值域里均匀 3 条（已经以 0 对称，所以 0 轴天然在正中）。
    fn axis(&self, ctx: &ChartCtx) -> Vec<DrawCmd> {
        axis_ticks(ctx, VALUE_TICKS, chart_text(), chart_tick(), |v| {
            format!("{v:.2}")
        })
    }

    /// 最上层：十字光标。
    fn overlay(&self, ctx: &ChartCtx) -> Vec<DrawCmd> {
        crosshair(ctx, chart_cross(), chart_text(), chart_cross_bg(), |v| {
            format!("{v:.2}")
        })
    }

    fn compute(&self, bars: &[Bar]) -> Vec<DrawCmd> {
        let closes = closes(bars);
        let len = closes.len();
        if len == 0 {
            return Vec::new();
        }

        let fast = ema(&closes, self.fast);
        let slow = ema(&closes, self.slow);
        let dif: Vec<f64> = (0..len).map(|i| fast[i] - slow[i]).collect();

        // DEA 是 DIF 的 EMA：只从第一根有效 DIF（下标 warm）开始递推，前面没值。
        let warm = self.slow.saturating_sub(1).min(len);
        let k = 2.0 / (self.signal as f64 + 1.0);
        let mut dea = vec![0.0; len];
        let mut prev = 0.0;
        for i in warm..len {
            prev = if i == warm { dif[i] } else { prev + k * (dif[i] - prev) };
            dea[i] = prev;
        }

        let mut cmds = Vec::with_capacity(2 * (len - warm) + 3);
        // 0 轴：先画，后面柱子和折线会盖住它（和原来一个顺序）。
        cmds.push(DrawCmd::HLine {
            p: 0.0,
            color: chart_zero(),
            w: 1,
        });
        // 柱子：每根 (DIF - DEA) * 2，正负分别用涨跌色。
        for i in warm..len {
            let hist = (dif[i] - dea[i]) * 2.0;
            cmds.push(DrawCmd::Bar {
                i,
                v0: 0.0,
                v1: hist,
                hollow: false,
                color: if hist < 0.0 {
                    self.pal.candle_down
                } else {
                    self.pal.candle_up
                },
            });
        }
        // 折线：warm 之前没有值，整段从 warm 开始（中间不断档，所以各是一条）。
        cmds.push(DrawCmd::Polyline {
            pts: (warm..len).map(|i| (i, dif[i])).collect(),
            color: self.pal.dif,
            w: 1,
        });
        cmds.push(DrawCmd::Polyline {
            pts: (warm..len).map(|i| (i, dea[i])).collect(),
            color: self.pal.dea,
            w: 1,
        });
        cmds
    }

    /// 状态带：名字 + 三条序列在光标那一根上的值。值跟着光标变，所以每帧从缓存好的指令里读
    /// （指标本身不重算）。顺序固定成 DIF / DEA / MACD。
    fn status(&self, ctx: &ChartCtx) -> Vec<DrawCmd> {
        let Some(index) = ctx.focus_bar() else {
            return Vec::new();
        };
        let mut dif = None;
        let mut dea = None;
        let mut macd = None;
        for cmd in ctx.cmds {
            match cmd {
                DrawCmd::Polyline { pts, color, .. } => {
                    let v = pts.iter().find(|(i, _)| *i == index).map(|(_, p)| *p);
                    if *color == self.pal.dif {
                        dif = v;
                    } else if *color == self.pal.dea {
                        dea = v;
                    }
                }
                DrawCmd::Bar { i, v1, .. } if *i == index => macd = Some(*v1),
                _ => {}
            }
        }
        let macd_color = if macd.is_some_and(|v| v < 0.0) {
            self.pal.candle_down
        } else {
            self.pal.candle_up
        };
        vec![
            DrawCmd::StatusText {
                s: self.name().to_owned(),
                color: chart_text(),
            },
            DrawCmd::StatusText {
                s: format!("DIF {}", fmt_value(dif)),
                color: self.pal.dif,
            },
            DrawCmd::StatusText {
                s: format!("DEA {}", fmt_value(dea)),
                color: self.pal.dea,
            },
            DrawCmd::StatusText {
                s: format!("MACD {}", fmt_value(macd)),
                color: macd_color,
            },
        ]
    }

    fn set_palette(&mut self, pal: Palette) {
        self.pal = pal;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::random_walk::DATE_START;

    /// 造一段简单的收盘价序列（只要 c 有用，别的不参与指标计算）。
    fn bars_from(closes: &[f64]) -> Vec<Bar> {
        closes
            .iter()
            .enumerate()
            .map(|(i, c)| Bar {
                o: *c,
                h: *c + 1.0,
                l: *c - 1.0,
                c: *c,
                vol: 1.0,
                date: DATE_START.add_days(i as i32),
            })
            .collect()
    }

    /// 从指令里挑出某一条折线的点（按颜色认）。
    fn polyline(cmds: &[DrawCmd], color: u32) -> Vec<(usize, f64)> {
        cmds.iter()
            .find_map(|cmd| match cmd {
                DrawCmd::Polyline { pts, color: c, .. } if *c == color => Some(pts.clone()),
                _ => None,
            })
            .expect("没找到这条折线")
    }

    /// 从指令里挑出柱子：全部下标一起给出来。
    fn bars_of(cmds: &[DrawCmd]) -> Vec<(usize, f64)> {
        cmds.iter()
            .filter_map(|cmd| match cmd {
                DrawCmd::Bar { i, v1, .. } => Some((*i, *v1)),
                _ => None,
            })
            .collect()
    }

    /// EMA 的递推：第一根是自己，第二根 = 第一根 + k * (第二根 - 第一根)。
    #[test]
    fn ema_starts_at_the_first_value() {
        let e = ema(&[10.0, 20.0, 20.0], 9);
        let k = 2.0 / 10.0;
        assert_eq!(e[0], 10.0);
        assert!((e[1] - (10.0 + k * 10.0)).abs() < 1e-12);
        assert!((e[2] - (e[1] + k * (20.0 - e[1]))).abs() < 1e-12);
        assert!(e[2] > e[1] && e[2] < 20.0);
    }

    /// 指令覆盖的根数必须和 K 线一致；慢线没成型之前（前 25 根）一根指令都没有。
    #[test]
    fn macd_cmds_align_with_bars_and_warm_up() {
        let closes: Vec<f64> = (0..120).map(|i| 20.0 + (i as f64 * 0.1).sin()).collect();
        let bars = bars_from(&closes);
        let macd = Macd::default();
        let cmds = macd.compute(&bars);

        assert_eq!(macd.name(), "MACD(12,26,9)");
        assert_eq!(macd.value_range(1.0, 2.0), (-2.0, 2.0), "纵轴要以 0 对称");
        let dif = polyline(&cmds, rgb_u32(DIF_RGB));
        let dea = polyline(&cmds, rgb_u32(DEA_RGB));
        let hist = bars_of(&cmds);
        assert_eq!(dif.len(), bars.len() - 25);
        assert_eq!(dea.len(), bars.len() - 25);
        assert_eq!(hist.len(), bars.len() - 25);
        // 第一根有效指令落在下标 25（slow - 1）上。
        assert_eq!(dif[0].0, 25);
        assert_eq!(dea[0].0, 25);
        assert_eq!(hist[0].0, 25);
        assert_eq!(dif[dif.len() - 1].0, bars.len() - 1);
        // 0 轴是一条 HLine，值 0。
        assert!(cmds.iter().any(|cmd| matches!(
            cmd,
            DrawCmd::HLine { p, .. } if *p == 0.0
        )));
    }

    /// 柱子的定义：(DIF - DEA) * 2，一根都不能差。
    #[test]
    fn macd_hist_is_twice_the_gap() {
        let closes: Vec<f64> = (0..80).map(|i| 20.0 + (i as f64 * 0.37).cos()).collect();
        let bars = bars_from(&closes);
        let cmds = Macd::default().compute(&bars);
        let dif = polyline(&cmds, rgb_u32(DIF_RGB));
        let dea = polyline(&cmds, rgb_u32(DEA_RGB));
        for (k, (i, h)) in bars_of(&cmds).into_iter().enumerate() {
            let d = dif.iter().find(|(j, _)| *j == i).unwrap().1;
            let s = dea.iter().find(|(j, _)| *j == i).unwrap().1;
            assert!((h - (d - s) * 2.0).abs() < 1e-12, "第 {k} 根柱子不对");
        }
    }

    /// 单边上涨时 DIF 必须为正、柱子也该在 0 上方；空数据不能炸（一根指令都没有）。
    #[test]
    fn macd_is_positive_in_a_steady_uptrend() {
        let closes: Vec<f64> = (0..60).map(|i| 20.0 + i as f64).collect();
        let cmds = Macd::default().compute(&bars_from(&closes));
        let dif = polyline(&cmds, rgb_u32(DIF_RGB));
        assert!(dif[dif.len() - 1].1 > 0.0, "一路上涨时 DIF 应该为正");
        let hist = bars_of(&cmds);
        assert!(hist[hist.len() - 1].1 > 0.0);

        assert!(Macd::default().compute(&[]).is_empty());
    }
}
