const VOL_BASE: f64 = 6.0e7;

pub const DATE_START: Date = Date::new(2024, 1, 2);

const PRICE_TICK: f64 = 0.01;

const LIMIT_PCT: f64 = 0.10;

const GAP_SIGMA: f64 = 0.0065;

const INTRADAY_SIGMA: f64 = 0.013;

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

    pub fn md(self) -> String {
        format!("{:02}-{:02}", self.m, self.d)
    }

    pub fn ymd_short(self) -> String {
        format!("{:02}-{:02}-{:02}", self.y.rem_euclid(100), self.m, self.d)
    }

    #[allow(dead_code)]
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
        _ => {
            if is_leap(y) {
                29
            } else {
                28
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Bar {
    pub o: f64,
    pub h: f64,
    pub l: f64,
    pub c: f64,
    pub vol: f64,
    pub date: Date,
}

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

    fn unit(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }

    fn normal(&mut self) -> f64 {
        let u1 = self.unit().max(1.0e-12);
        let u2 = self.unit();
        (-2.0 * u1.ln()).sqrt() * (std::f64::consts::TAU * u2).cos()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BarState {
    Normal,
    LimitUp,
    LimitDown,
    Suspended,
}

impl BarState {
    #[allow(dead_code)]
    pub fn label(self) -> &'static str {
        match self {
            Self::Normal => "",
            Self::LimitUp => "涨停",
            Self::LimitDown => "跌停",
            Self::Suspended => "停牌",
        }
    }
}

fn round_tick(v: f64) -> f64 {
    (v.max(PRICE_TICK) * 100.0).round() / 100.0
}

fn tick_clamp(v: f64, lo: f64, hi: f64) -> f64 {
    round_tick(v.clamp(lo, hi))
}

pub fn limit_prices(prev_close: f64) -> (f64, f64) {
    let prev = round_tick(prev_close);
    (
        round_tick(prev * (1.0 + LIMIT_PCT)),
        round_tick(prev * (1.0 - LIMIT_PCT)),
    )
}

#[allow(dead_code)]
pub fn bar_state(bars: &[Bar], index: usize) -> BarState {
    let Some(bar) = bars.get(index) else {
        return BarState::Normal;
    };
    if bar.vol <= 0.0 {
        return BarState::Suspended;
    }
    if index == 0 {
        return BarState::Normal;
    }
    let (up, down) = limit_prices(bars[index - 1].c);
    if bar.c >= up - PRICE_TICK * 0.25 {
        BarState::LimitUp
    } else if bar.c <= down + PRICE_TICK * 0.25 {
        BarState::LimitDown
    } else {
        BarState::Normal
    }
}

fn forced_events(rng: &mut Rng, count: usize) -> Vec<BarState> {
    let mut out = vec![BarState::Normal; count];
    if count < 120 {
        return out;
    }
    let lo = 20usize;
    let hi = count.saturating_sub(10);
    if hi <= lo {
        return out;
    }
    let span = hi - lo;
    for kind in [BarState::LimitUp, BarState::LimitDown, BarState::Suspended] {
        let mut at = lo + (rng.next_u64() as usize % span);
        while out[at] != BarState::Normal {
            at = lo + (at - lo + 1) % span;
        }
        out[at] = kind;
    }
    out
}

pub fn generate(seed: u64, count: usize, start: f64) -> Vec<Bar> {
    let mut rng = Rng::new(seed);
    let mut bars = Vec::with_capacity(count);
    let mut prev_close = round_tick(start);

    let mut date = DATE_START;
    let mut weekday = 1;
    let forced = forced_events(&mut rng, count);

    for (i, state) in forced.into_iter().enumerate() {
        if i > 0 {
            let step = if weekday == 4 { 3 } else { 1 };
            date = date.add_days(step);
            weekday = (weekday + 1) % 5;
        }
        let (limit_up, limit_down) = limit_prices(prev_close);

        if state == BarState::Suspended {
            bars.push(Bar {
                o: prev_close,
                h: prev_close,
                l: prev_close,
                c: prev_close,
                vol: 0.0,
                date,
            });
            continue;
        }

        let gap = if i == 0 {
            0.0
        } else {
            (rng.normal() * GAP_SIGMA).clamp(-0.085, 0.085)
        };
        let open = tick_clamp(prev_close * (1.0 + gap), limit_down, limit_up);

        let kind = if state != BarState::Normal {
            state
        } else if rng.unit() < 0.0015 {
            if rng.unit() < 0.5 {
                BarState::LimitUp
            } else {
                BarState::LimitDown
            }
        } else {
            BarState::Normal
        };

        let (o, h, l, c, vol) = match kind {
            BarState::LimitUp => {
                if rng.unit() < 0.45 {
                    (
                        limit_up,
                        limit_up,
                        limit_up,
                        limit_up,
                        VOL_BASE * (0.03 + 0.12 * rng.unit()),
                    )
                } else {
                    let low = tick_clamp(
                        open.min(prev_close * (1.0 - rng.unit() * 0.025)),
                        limit_down,
                        limit_up,
                    )
                    .min(open);
                    (
                        open,
                        limit_up,
                        low,
                        limit_up,
                        VOL_BASE * (1.0 + 1.8 * rng.unit()),
                    )
                }
            }
            BarState::LimitDown => {
                if rng.unit() < 0.45 {
                    (
                        limit_down,
                        limit_down,
                        limit_down,
                        limit_down,
                        VOL_BASE * (0.03 + 0.12 * rng.unit()),
                    )
                } else {
                    let high = tick_clamp(
                        open.max(prev_close * (1.0 + rng.unit() * 0.025)),
                        limit_down,
                        limit_up,
                    )
                    .max(open);
                    (
                        open,
                        high,
                        limit_down,
                        limit_down,
                        VOL_BASE * (1.0 + 1.8 * rng.unit()),
                    )
                }
            }
            BarState::Normal | BarState::Suspended => {
                let gap_pull = -(open / prev_close - 1.0) * 0.18;
                let close = tick_clamp(
                    open * (1.0 + rng.normal() * INTRADAY_SIGMA + gap_pull),
                    limit_down,
                    limit_up,
                );
                let body_hi = open.max(close);
                let body_lo = open.min(close);
                let high = tick_clamp(body_hi * (1.0 + rng.unit() * 0.007), body_hi, limit_up)
                    .max(body_hi);
                let low = tick_clamp(body_lo * (1.0 - rng.unit() * 0.007), limit_down, body_lo)
                    .min(body_lo);
                let amp = ((close - open).abs() / prev_close).clamp(0.0, 0.2);
                let volume = VOL_BASE
                    * (0.35 + 1.15 * rng.unit())
                    * (0.7 + 35.0 * amp)
                    * (0.9 + 0.2 * (gap.abs() / GAP_SIGMA).min(3.0));
                (open, high, low, close, volume)
            }
        };

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
