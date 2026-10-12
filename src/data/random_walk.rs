// ===========================================================================
//  随机游走行情生成器
// ===========================================================================
//  暂时没有真实行情源，先用随机游走造一段能复现的假 K 线：
//  同一个 seed 一定生成同一串数据，调试时可以对得上。

/// 每天最多波动多少（价格单位）。
const DAILY_MOVE: f64 = 0.3;

/// 一根 K 线：开盘 / 最高 / 最低 / 收盘。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Bar {
    pub o: f64,
    pub h: f64,
    pub l: f64,
    pub c: f64,
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

/// 生成 count 根 K 线。价格从 start 起步，每天收盘价在开盘价基础上随机走 ±DAILY_MOVE；
/// 高低点在实体之外再各随机延伸一段，保证 h >= max(o, c) 且 l <= min(o, c)。
pub fn generate(seed: u64, count: usize, start: f64) -> Vec<Bar> {
    let mut rng = Rng::new(seed);
    let mut bars = Vec::with_capacity(count);
    let mut prev_close = start;

    for _ in 0..count {
        let o = prev_close;
        let c = o + rng.signed() * DAILY_MOVE;
        let h = o.max(c) + rng.unit() * DAILY_MOVE;
        let l = o.min(c) - rng.unit() * DAILY_MOVE;
        bars.push(Bar { o, h, l, c });
        prev_close = c;
    }

    bars
}
