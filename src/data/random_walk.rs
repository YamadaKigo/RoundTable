// ===========================================================================
//  随机游走行情生成器
// ===========================================================================
//  暂时没有真实行情源，先用随机游走造一段能复现的假 K 线：
//  同一个 seed 一定生成同一串数据，调试时可以对得上。

/// 每天最多波动多少（价格单位）。
const DAILY_MOVE: f64 = 0.3;

/// 成交量的基准档：再乘上 0.35~1.5 的随机系数与走势强度，落点大约 1.5e7~1.2e8，
/// 这样刻度里「万」和「亿」两种单位都会出现（见 main.rs 的 fmt_volume）。
const VOL_BASE: f64 = 6.0e7;

/// 第一根 K 线的日期：2024-01-02（周二）。
pub const DATE_START: Date = Date::new(2024, 1, 2);

/// 日期（精确到「日」）。本工程只用它做显示和时间轴标签，不涉及时区 / 闰秒。
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Date {
    pub y: i32,
    pub m: u8,
    pub d: u8,
}

impl Date {
    pub const fn new(y: i32, m: u8, d: u8) -> Self {
        Self { y, m, d }
    }

    /// 往后推 n 天（n >= 0）。大小月和闰年都在这儿算清楚。
    pub fn add_days(self, n: i32) -> Self {
        let mut y = self.y;
        let mut m = self.m as i32;
        let mut d = self.d as i32 + n;
        loop {
            let len = days_in_month(y, m) as i32;
            if d <= len {
                break;
            }
            d -= len;
            m += 1;
            if m > 12 {
                m = 1;
                y += 1;
            }
        }
        Self {
            y,
            m: m as u8,
            d: d as u8,
        }
    }

    /// "MM-DD"：时间轴上同一年里那种短标签。
    pub fn md(self) -> String {
        format!("{:02}-{:02}", self.m, self.d)
    }

    /// "YY-MM-DD"：时间轴上跨年那一格用的短标签。
    pub fn ymd_short(self) -> String {
        format!("{:02}-{:02}-{:02}", self.y.rem_euclid(100), self.m, self.d)
    }

    /// "YYYY-MM-DD"：状态条里的完整日期。
    pub fn full(self) -> String {
        format!("{:04}-{:02}-{:02}", self.y, self.m, self.d)
    }
}

fn is_leap(y: i32) -> bool {
    (y % 4 == 0 && y % 100 != 0) || y % 400 == 0
}

fn days_in_month(y: i32, m: i32) -> u8 {
    match m {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        _ => if is_leap(y) { 29 } else { 28 },
    }
}

/// 一根 K 线：开盘 / 最高 / 最低 / 收盘 + 成交量 + 日期。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Bar {
    pub o: f64,
    pub h: f64,
    pub l: f64,
    pub c: f64,
    pub vol: f64,
    pub date: Date,
}

/// splitmix64：不依赖任何外部库的伪随机数发生器，结果完全由 seed 决定。
struct Rng(u64);

impl Rng {
    fn new(seed: u64) -> Self {
        Self(seed)
    }

    fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// 均匀落到 [0, 1)。
    fn unit(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }

    /// 均匀落到 [-1, 1)。
    fn signed(&mut self) -> f64 {
        self.unit() * 2.0 - 1.0
    }
}

/// 生成 count 根 K 线。价格从 start 起步，每天收盘价在开盘价基础上随机走 +/-DAILY_MOVE；
/// 高低点在实体之外再各随机延伸一段，保证 h >= max(o, c) 且 l <= min(o, c)。
pub fn generate(seed: u64, count: usize, start: f64) -> Vec<Bar> {
    let mut rng = Rng::new(seed);
    let mut bars = Vec::with_capacity(count);
    let mut prev_close = start;
    // 日期只走工作日：从周二起步，周五收盘之后直接跳到下周一（跳过周末）。
    let mut date = DATE_START;
    let mut weekday = 1; // 0 = 周一 ... 4 = 周五

    for i in 0..count {
        if i > 0 {
            let step = if weekday == 4 { 3 } else { 1 };
            date = date.add_days(step);
            weekday = (weekday + 1) % 5;
        }
        let o = prev_close;
        let c = o + rng.signed() * DAILY_MOVE;
        let h = o.max(c) + rng.unit() * DAILY_MOVE;
        let l = o.min(c) - rng.unit() * DAILY_MOVE;
        // 成交量：走势越猛量越大（跟真实盘面一个方向），基准 +/-，上下再抖一抖。
        let amp = ((c - o).abs() / DAILY_MOVE).clamp(0.0, 1.0);
        let vol = VOL_BASE * (0.35 + 1.15 * rng.unit()) * (0.7 + 0.6 * amp);
        bars.push(Bar {
            o,
            h,
            l,
            c,
            vol,
            date,
        });
        prev_close = c;
    }

    bars
}
