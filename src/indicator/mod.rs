// ===========================================================================
//  指标层
// ===========================================================================
//  一个指标 = 一次纯计算：喂给它整段 K 线，它还给若干条线的整段数值。
//  界面那边（main.rs 的指标区）只认 Indicator 这个接口和 Output 这份输出，
//  不认识任何具体指标 —— 以后加新指标只要在这儿再写一个 impl，绘图一行都不用改。
//
//  为什么坚持「纯函数 + 整段算」：
//    · 结果只跟 bars 有关，所以能按「数据代次 + 指标 id」缓存，不动视图就一次都不重算；
//    · 不碰 egui、不碰全局状态，以后要做可编程 / 自定义指标（把同一份输入输出搬到脚本里）
//      或者跑回测，都能直接把这一段挪过去复用。
//
//  画什么由 Output 描述：几条线（名字 + 颜色 + 每根一个值）+ 可选的一组柱子（MACD 柱）。
//  柱子不带颜色：以 0 为界的柱子按正负取涨跌色，这件事归界面管（用户能改涨跌色）。

use crate::data::random_walk::Bar;

/// 颜色一律 (u8, u8, u8)，和主程序那一套一致；画的时候再转 Color32。
pub type Rgb = (u8, u8, u8);

/// 一条线：名字 + 颜色 + 每根 K 线一个值（None = 这一根没有值，图上就断开不画）。
pub struct Line {
    pub name: String,
    pub rgb: Rgb,
    pub values: Vec<Option<f64>>,
}

/// 一组柱子（比如 MACD 柱）：每根 K 线一个值，正负由界面决定配色。
pub struct Hist {
    pub values: Vec<Option<f64>>,
}

/// 一个指标算完的结果。lines / hist 里的 values 长度都等于 K 线根数，下标一一对应。
pub struct Output {
    /// 图例标题，例如 "MACD(12,26,9)"。
    pub title: String,
    pub lines: Vec<Line>,
    pub hist: Option<Hist>,
    /// 纵轴要不要以 0 为中心对称：MACD 这种正负都有的指标一般要，
    /// 只要正值的那种（比如纯成交量类）就不要。
    pub symmetric: bool,
}

/// 指标统一接口。
pub trait Indicator {
    /// 稳定标识：缓存键用它。换实现（改了算法）就必须换 id，否则会拿到旧缓存。
    fn id(&self) -> &'static str;
    /// 图例标题（带上参数）。
    fn title(&self) -> String;
    /// 纯计算：只依赖 bars，不依赖界面状态。
    fn compute(&self, bars: &[Bar]) -> Output;
}

/// 内置指标表：界面按这个顺序摆出来。以后加指标就往这儿塞一个。
pub fn builtin() -> Vec<Box<dyn Indicator>> {
    vec![Box::new(Macd::default())]
}

/// 收盘价序列。
fn closes(bars: &[Bar]) -> Vec<f64> {
    bars.iter().map(|bar| bar.c).collect()
}

/// 指数移动平均：第一根就取它自己的收价，之后按 k = 2/(n+1) 递推。
/// 不搞「前 n 根才开始有效」那套，前面不留一大段空白。
fn ema(values: &[f64], n: usize) -> Vec<f64> {
    let k = 2.0 / (n as f64 + 1.0);
    let mut out = Vec::with_capacity(values.len());
    let mut prev = 0.0;
    for (i, v) in values.iter().enumerate() {
        prev = if i == 0 { *v } else { prev + k * (*v - prev) };
        out.push(prev);
    }
    out
}

/// MACD（参数 12 / 26 / 9）：
///   DIF = EMA(快) - EMA(慢)；DEA = DIF 的 EMA(信号)；柱子 = (DIF - DEA) * 2（国内习惯乘以 2）。
/// 慢线要 26 根才稳，所以前 slow-1 根一律留空（None），图上自然从那儿才开始有线。
pub struct Macd {
    pub fast: usize,
    pub slow: usize,
    pub signal: usize,
}

impl Default for Macd {
    fn default() -> Self {
        Self {
            fast: 12,
            slow: 26,
            signal: 9,
        }
    }
}

impl Indicator for Macd {
    fn id(&self) -> &'static str {
        "macd"
    }

    fn title(&self) -> String {
        format!("MACD({},{},{})", self.fast, self.slow, self.signal)
    }

    fn compute(&self, bars: &[Bar]) -> Output {
        let closes = closes(bars);
        let title = self.title();
        let len = closes.len();
        if len == 0 {
            return Output {
                title,
                lines: Vec::new(),
                hist: None,
                symmetric: true,
            };
        }

        let fast = ema(&closes, self.fast);
        let slow = ema(&closes, self.slow);
        let dif_raw: Vec<f64> = (0..len).map(|i| fast[i] - slow[i]).collect();

        // DEA 是 DIF 的 EMA：只从第一根有效 DIF（下标 warm）开始递推，前面留空。
        let warm = self.slow.saturating_sub(1).min(len);
        let k = 2.0 / (self.signal as f64 + 1.0);
        let mut dea_raw = vec![0.0; len];
        let mut prev = 0.0;
        for i in warm..len {
            prev = if i == warm {
                dif_raw[i]
            } else {
                prev + k * (dif_raw[i] - prev)
            };
            dea_raw[i] = prev;
        }

        // 拼成带 None 的序列：warm 之前是空的，之后每根都有值。
        let mut dif = vec![None; len];
        let mut dea = vec![None; len];
        let mut hist = vec![None; len];
        for i in warm..len {
            dif[i] = Some(dif_raw[i]);
            dea[i] = Some(dea_raw[i]);
            hist[i] = Some((dif_raw[i] - dea_raw[i]) * 2.0);
        }

        Output {
            title,
            lines: vec![
                Line {
                    name: "DIF".to_owned(),
                    rgb: DIF_RGB,
                    values: dif,
                },
                Line {
                    name: "DEA".to_owned(),
                    rgb: DEA_RGB,
                    values: dea,
                },
            ],
            hist: Some(Hist { values: hist }),
            symmetric: true,
        }
    }
}

/// 两条线的颜色（#3333CC 蓝 / #CC9900 橙），白底上够清楚，也符合本工程
/// 「每个通道都是 0x11 的整数倍」的色值规矩。
pub const DIF_RGB: Rgb = (0x33, 0x33, 0xCC);
pub const DEA_RGB: Rgb = (0xCC, 0x99, 0x00);

#[cfg(test)]
mod tests {
    use super::*;

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
                date: crate::data::random_walk::DATE_START.add_days(i as i32),
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
        // 输入封顶在 20 之后，EMA 只朝 20 逼近（还是按同一条递推式走），不会一下跳到 20。
        assert!((e[2] - (e[1] + k * (20.0 - e[1]))).abs() < 1e-12);
        assert!(e[2] > e[1] && e[2] < 20.0);
    }

    /// 输出长度必须和 K 线一一对应；慢线没成型之前一律是 None。
    #[test]
    fn macd_lines_align_with_bars_and_warm_up() {
        let closes: Vec<f64> = (0..120).map(|i| 20.0 + (i as f64 * 0.1).sin()).collect();
        let bars = bars_from(&closes);
        let macd = Macd::default();
        let out = macd.compute(&bars);
        assert_eq!(out.title, "MACD(12,26,9)");
        assert!(out.symmetric);
        assert_eq!(out.lines.len(), 2);
        let hist = out.hist.as_ref().expect("MACD 必须有柱子");
        for line in &out.lines {
            assert_eq!(line.values.len(), bars.len());
        }
        assert_eq!(hist.values.len(), bars.len());
        // 前 25 根（slow - 1）没有值，第 26 根开始有。
        for i in 0..25 {
            assert!(out.lines[0].values[i].is_none(), "第 {i} 根不该有 DIF");
            assert!(hist.values[i].is_none(), "第 {i} 根不该有柱子");
        }
        assert!(out.lines[0].values[25].is_some());
        assert!(hist.values[25].is_some());
    }

    /// 柱子的定义：(DIF - DEA) * 2，一根都不能差。
    #[test]
    fn macd_hist_is_twice_the_gap() {
        let closes: Vec<f64> = (0..80).map(|i| 20.0 + (i as f64 * 0.37).cos()).collect();
        let bars = bars_from(&closes);
        let out = Macd::default().compute(&bars);
        let (dif, dea) = (&out.lines[0].values, &out.lines[1].values);
        let hist = &out.hist.as_ref().unwrap().values;
        for i in 0..bars.len() {
            match (dif[i], dea[i], hist[i]) {
                (Some(d), Some(s), Some(h)) => {
                    assert!((h - (d - s) * 2.0).abs() < 1e-12, "第 {i} 根柱子不对")
                }
                (None, None, None) => {}
                other => panic!("第 {i} 根三条序列的有效性不一致：{other:?}"),
            }
        }
    }

    /// 单边上涨时 DIF 必须为正、柱子也该在 0 上方；空数据不能炸。
    #[test]
    fn macd_is_positive_in_a_steady_uptrend() {
        let closes: Vec<f64> = (0..60).map(|i| 20.0 + i as f64).collect();
        let out = Macd::default().compute(&bars_from(&closes));
        let last = out.lines[0].values[59].unwrap();
        assert!(last > 0.0, "一路上涨时 DIF 应该为正：{last}");
        assert!(out.hist.as_ref().unwrap().values[59].unwrap() > 0.0);

        let empty = Macd::default().compute(&[]);
        assert!(empty.lines.is_empty());
        assert!(empty.hist.is_none());
    }
}
