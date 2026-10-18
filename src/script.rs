use crate::data::random_walk::Bar;
use crate::indicator::{self, DrawCmd, Palette};
use std::collections::HashMap;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Op {
    Add,
    Sub,
    Mul,
    Div,
    Gt,
    Lt,
    Ge,
    Le,
    Eq,
    Ne,
}

#[derive(Clone)]
enum Expr {
    Num(f64),
    Var(String),
    Call(String, Vec<Expr>),
    Binary(Op, Box<Expr>, Box<Expr>),
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum DrawKind {
    Line,
    Hist,
    HLine,
    Zero,
    Candle,
    Ohlc,
    Volume,
}

#[derive(Clone)]
struct DrawStmt {
    kind: DrawKind,
    args: Vec<Expr>,
}

#[derive(Clone)]
enum Statement {
    Assign(String, Expr),
    Draw(DrawStmt),
}

#[derive(Clone)]
pub(crate) struct ScriptProgram {
    statements: Vec<Statement>,
}

#[derive(Clone, PartialEq, Debug)]
enum Token {
    Number(f64),
    Ident(String),
    Plus,
    Minus,
    Star,
    Slash,
    Greater,
    Less,
    GreaterEqual,
    LessEqual,
    Equal,
    EqualEqual,
    BangEqual,
    Colon,
    Comma,
    LParen,
    RParen,
    Semi,
    End,
}

impl ScriptProgram {
    pub(crate) fn parse(source: &str) -> Result<Self, String> {
        let tokens = tokenize(source)?;
        let mut parser = Parser { tokens, at: 0 };
        let mut statements = Vec::new();
        while parser.peek() != &Token::End {
            let name = parser.ident()?;
            if parser.accept(&Token::Colon) || parser.accept(&Token::Equal) {
                let expr = parser.expr()?;
                parser.accept(&Token::Semi);
                statements.push(Statement::Assign(name, expr));
                continue;
            }
            if !parser.accept(&Token::LParen) {
                return Err(format!("{name} 后面缺少 : 或 ="));
            }
            let args = parser.args()?;
            parser.accept(&Token::Semi);
            let kind = draw_kind(&name).ok_or_else(|| format!("未知命令 {name}"))?;
            statements.push(Statement::Draw(DrawStmt { kind, args }));
        }
        if statements.is_empty() {
            return Err("指标代码不能为空".to_owned());
        }
        Ok(Self { statements })
    }

    pub(crate) fn compute(&self, bars: &[Bar], pal: &Palette) -> Result<Vec<DrawCmd>, String> {
        let len = bars.len();
        let mut env = base_env(bars);
        let mut out = Vec::new();
        for statement in &self.statements {
            match statement {
                Statement::Assign(name, expr) => {
                    let values = eval(expr, &env, len)?;
                    env.insert(name.clone(), values);
                }
                Statement::Draw(draw) => draw_commands(draw, &env, bars, pal, &mut out)?,
            }
        }
        Ok(out)
    }
}

fn tokenize(source: &str) -> Result<Vec<Token>, String> {
    let chars: Vec<char> = source.chars().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c.is_whitespace() {
            i += 1;
            continue;
        }
        if c == '#' {
            while i < chars.len() && chars[i] != '\n' {
                i += 1;
            }
            continue;
        }
        if c.is_ascii_digit() || c == '.' {
            let start = i;
            if c == '0' && i + 1 < chars.len() && matches!(chars[i + 1], 'x' | 'X') {
                i += 2;
                let digits = i;
                while i < chars.len() && chars[i].is_ascii_hexdigit() {
                    i += 1;
                }
                let text: String = chars[digits..i].iter().collect();
                let value =
                    u32::from_str_radix(&text, 16).map_err(|_| format!("颜色 {text} 无效"))? as f64;
                out.push(Token::Number(value));
                continue;
            }
            while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
                i += 1;
            }
            let text: String = chars[start..i].iter().collect();
            let value = text
                .parse::<f64>()
                .map_err(|_| format!("数字 {text} 无效"))?;
            out.push(Token::Number(value));
            continue;
        }
        if c.is_alphabetic() || c == '_' {
            let start = i;
            i += 1;
            while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            out.push(Token::Ident(chars[start..i].iter().collect()));
            continue;
        }
        let next = chars.get(i + 1).copied();
        let token = match (c, next) {
            ('>', Some('=')) => {
                i += 1;
                Token::GreaterEqual
            }
            ('<', Some('=')) => {
                i += 1;
                Token::LessEqual
            }
            ('=', Some('=')) => {
                i += 1;
                Token::EqualEqual
            }
            ('!', Some('=')) => {
                i += 1;
                Token::BangEqual
            }
            ('+', _) => Token::Plus,
            ('-', _) => Token::Minus,
            ('*', _) => Token::Star,
            ('/', _) => Token::Slash,
            ('>', _) => Token::Greater,
            ('<', _) => Token::Less,
            ('=', _) => Token::Equal,
            (':', _) => Token::Colon,
            (',', _) => Token::Comma,
            ('(', _) => Token::LParen,
            (')', _) => Token::RParen,
            (';', _) => Token::Semi,
            _ => return Err(format!("无法识别的字符 {c}")),
        };
        out.push(token);
        i += 1;
    }
    out.push(Token::End);
    Ok(out)
}

struct Parser {
    tokens: Vec<Token>,
    at: usize,
}

impl Parser {
    fn peek(&self) -> &Token {
        &self.tokens[self.at]
    }

    fn take(&mut self) -> Token {
        let token = self.tokens[self.at].clone();
        self.at += 1;
        token
    }

    fn accept(&mut self, token: &Token) -> bool {
        if self.peek() == token {
            self.at += 1;
            true
        } else {
            false
        }
    }

    fn ident(&mut self) -> Result<String, String> {
        match self.take() {
            Token::Ident(name) => Ok(name),
            other => Err(format!("这里需要名称，实际是 {other:?}")),
        }
    }

    fn args(&mut self) -> Result<Vec<Expr>, String> {
        let mut args = Vec::new();
        if self.accept(&Token::RParen) {
            return Ok(args);
        }
        loop {
            args.push(self.expr()?);
            if self.accept(&Token::RParen) {
                return Ok(args);
            }
            if !self.accept(&Token::Comma) {
                return Err("函数参数间缺少逗号".to_owned());
            }
        }
    }

    fn expr(&mut self) -> Result<Expr, String> {
        self.comparison()
    }

    fn comparison(&mut self) -> Result<Expr, String> {
        let mut expr = self.additive()?;
        loop {
            let op = match self.peek() {
                Token::Greater => Op::Gt,
                Token::Less => Op::Lt,
                Token::GreaterEqual => Op::Ge,
                Token::LessEqual => Op::Le,
                Token::Equal => Op::Eq,
                Token::EqualEqual => Op::Eq,
                Token::BangEqual => Op::Ne,
                _ => break,
            };
            self.at += 1;
            expr = Expr::Binary(op, Box::new(expr), Box::new(self.additive()?));
        }
        Ok(expr)
    }

    fn additive(&mut self) -> Result<Expr, String> {
        let mut expr = self.multiplicative()?;
        loop {
            let op = match self.peek() {
                Token::Plus => Op::Add,
                Token::Minus => Op::Sub,
                _ => break,
            };
            self.at += 1;
            expr = Expr::Binary(op, Box::new(expr), Box::new(self.multiplicative()?));
        }
        Ok(expr)
    }

    fn multiplicative(&mut self) -> Result<Expr, String> {
        let mut expr = self.primary()?;
        loop {
            let op = match self.peek() {
                Token::Star => Op::Mul,
                Token::Slash => Op::Div,
                _ => break,
            };
            self.at += 1;
            expr = Expr::Binary(op, Box::new(expr), Box::new(self.primary()?));
        }
        Ok(expr)
    }

    fn primary(&mut self) -> Result<Expr, String> {
        match self.take() {
            Token::Number(v) => Ok(Expr::Num(v)),
            Token::Minus => Ok(Expr::Binary(
                Op::Sub,
                Box::new(Expr::Num(0.0)),
                Box::new(self.primary()?),
            )),
            Token::Ident(name) => {
                if self.accept(&Token::LParen) {
                    Ok(Expr::Call(name, self.args()?))
                } else {
                    Ok(Expr::Var(name))
                }
            }
            Token::LParen => {
                let expr = self.expr()?;
                if !self.accept(&Token::RParen) {
                    return Err("缺少右括号".to_owned());
                }
                Ok(expr)
            }
            other => Err(format!("表达式里出现 {other:?}")),
        }
    }
}

fn draw_kind(name: &str) -> Option<DrawKind> {
    match name {
        "LINE" => Some(DrawKind::Line),
        "HIST" | "BAR" => Some(DrawKind::Hist),
        "HLINE" => Some(DrawKind::HLine),
        "ZERO" => Some(DrawKind::Zero),
        "CANDLE" => Some(DrawKind::Candle),
        "OHLC" => Some(DrawKind::Ohlc),
        "VOL" => Some(DrawKind::Volume),
        _ => None,
    }
}

fn base_env(bars: &[Bar]) -> HashMap<String, Vec<f64>> {
    let col = |f: fn(&Bar) -> f64| bars.iter().map(f).collect::<Vec<_>>();
    HashMap::from([
        ("O".to_owned(), col(|bar| bar.o)),
        ("H".to_owned(), col(|bar| bar.h)),
        ("L".to_owned(), col(|bar| bar.l)),
        ("C".to_owned(), col(|bar| bar.c)),
        ("V".to_owned(), col(|bar| bar.vol)),
    ])
}

fn eval(expr: &Expr, env: &HashMap<String, Vec<f64>>, len: usize) -> Result<Vec<f64>, String> {
    match expr {
        Expr::Num(v) => Ok(vec![*v; len]),
        Expr::Var(name) => env
            .get(name)
            .cloned()
            .ok_or_else(|| format!("未知变量 {name}")),
        Expr::Binary(op, a, b) => {
            let a = eval(a, env, len)?;
            let b = eval(b, env, len)?;
            Ok((0..len)
                .map(|i| match op {
                    Op::Add => a[i] + b[i],
                    Op::Sub => a[i] - b[i],
                    Op::Mul => a[i] * b[i],
                    Op::Div => safe_div(a[i], b[i]),
                    Op::Gt => f64::from(a[i] > b[i]),
                    Op::Lt => f64::from(a[i] < b[i]),
                    Op::Ge => f64::from(a[i] >= b[i]),
                    Op::Le => f64::from(a[i] <= b[i]),
                    Op::Eq => f64::from((a[i] - b[i]).abs() < 1e-12),
                    Op::Ne => f64::from((a[i] - b[i]).abs() >= 1e-12),
                })
                .collect())
        }
        Expr::Call(name, args) => call(name, args, env, len),
    }
}

fn call(
    name: &str,
    args: &[Expr],
    env: &HashMap<String, Vec<f64>>,
    len: usize,
) -> Result<Vec<f64>, String> {
    let series = |i: usize| eval(&args[i], env, len);
    match name {
        "IF" => {
            if args.len() != 3 {
                return Err("IF 需要 3 个参数".to_owned());
            }
            let cond = series(0)?;
            let yes = series(1)?;
            let no = series(2)?;
            Ok((0..len)
                .map(|i| if cond[i] > 0.0 { yes[i] } else { no[i] })
                .collect())
        }
        "MAX" | "MIN" => {
            if args.len() != 2 {
                return Err(format!("{name} 需要 2 个参数"));
            }
            let a = series(0)?;
            let b = series(1)?;
            Ok((0..len)
                .map(|i| {
                    if name == "MAX" {
                        a[i].max(b[i])
                    } else {
                        a[i].min(b[i])
                    }
                })
                .collect())
        }
        "ABS" => Ok(series(0)?.into_iter().map(f64::abs).collect()),
        "MA" | "EMA" | "REF" | "SUM" | "STD" | "HHV" | "LLV" => {
            if args.len() != 2 {
                return Err(format!("{name} 需要 2 个参数"));
            }
            let values = series(0)?;
            let raw_n = number_arg(&series(1)?);
            let n = raw_n.max(1) as usize;
            match name {
                "REF" => Ok((0..len)
                    .map(|i| i.checked_sub(n).map_or(f64::NAN, |k| values[k]))
                    .collect()),
                "MA" => Ok(rolling(&values, n, false, false)),
                "STD" => Ok(rolling(&values, n, true, false)),
                "SUM" if raw_n == 0 => {
                    let mut total = 0.0;
                    Ok(values
                        .into_iter()
                        .map(|value| {
                            if value.is_finite() {
                                total += value;
                            }
                            total
                        })
                        .collect())
                }
                "SUM" => Ok(rolling(&values, n, false, true)),
                "EMA" => Ok(ema(&values, n)),
                "HHV" => Ok(rolling(&values, n, false, false)
                    .iter()
                    .enumerate()
                    .map(|(i, _)| window_max(&values, i, n, f64::max))
                    .collect()),
                "LLV" => Ok((0..len)
                    .map(|i| window_max(&values, i, n, f64::min))
                    .collect()),
                _ => unreachable!(),
            }
        }
        "CROSS" => {
            if args.len() != 2 {
                return Err("CROSS 需要 2 个参数".to_owned());
            }
            let a = series(0)?;
            let b = series(1)?;
            Ok((0..len)
                .map(|i| f64::from(i > 0 && a[i] > b[i] && a[i - 1] <= b[i - 1]))
                .collect())
        }
        _ => Err(format!("未知函数 {name}")),
    }
}

fn number_arg(values: &[f64]) -> i64 {
    values
        .iter()
        .find(|v| v.is_finite())
        .copied()
        .unwrap_or(1.0)
        .round()
        .max(0.0) as i64
}

fn safe_div(a: f64, b: f64) -> f64 {
    if b.abs() < 1e-12 { f64::NAN } else { a / b }
}

fn rolling(values: &[f64], n: usize, square: bool, sum: bool) -> Vec<f64> {
    let n = n.max(1);
    (0..values.len())
        .map(|i| {
            if i + 1 < n {
                return f64::NAN;
            }
            let window = &values[i + 1 - n..=i];
            if window.iter().any(|v| !v.is_finite()) {
                return f64::NAN;
            }
            if sum {
                window.iter().sum()
            } else if square {
                let mean = window.iter().sum::<f64>() / n as f64;
                (window.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / n as f64).sqrt()
            } else {
                window.iter().sum::<f64>() / n as f64
            }
        })
        .collect()
}

fn ema(values: &[f64], n: usize) -> Vec<f64> {
    let k = 2.0 / (n.max(1) as f64 + 1.0);
    let mut out = vec![f64::NAN; values.len()];
    let mut prev = None;
    for (i, value) in values.iter().copied().enumerate() {
        if !value.is_finite() {
            continue;
        }
        prev = Some(match prev {
            None => value,
            Some(last) => last + k * (value - last),
        });
        out[i] = prev.unwrap_or(f64::NAN);
    }
    out
}

fn window_max(values: &[f64], i: usize, n: usize, pick: fn(f64, f64) -> f64) -> f64 {
    if i + 1 < n {
        return f64::NAN;
    }
    let window = &values[i + 1 - n..=i];
    if window.iter().any(|v| !v.is_finite()) {
        return f64::NAN;
    }
    window.iter().copied().reduce(pick).unwrap_or(f64::NAN)
}

fn draw_commands(
    draw: &DrawStmt,
    env: &HashMap<String, Vec<f64>>,
    bars: &[Bar],
    pal: &Palette,
    out: &mut Vec<DrawCmd>,
) -> Result<(), String> {
    match draw.kind {
        DrawKind::Candle => {
            out.extend(indicator::candles(
                bars,
                pal.candle_up,
                pal.candle_down,
                pal.candle_suspended,
            ));
        }
        DrawKind::Ohlc => out.extend(indicator::ohlc_bars(bars, pal.candle_up, pal.candle_down)),
        DrawKind::Volume => {
            out.extend(indicator::volume_bars(bars, pal.candle_up, pal.candle_down))
        }
        DrawKind::Zero => out.push(DrawCmd::HLine {
            p: 0.0,
            color: indicator::chart_zero(),
            w: 1,
        }),
        DrawKind::Line | DrawKind::Hist | DrawKind::HLine => {
            if draw.args.is_empty() || draw.args.len() > 2 {
                return Err("绘图命令需要数值和颜色".to_owned());
            }
            let values = eval(&draw.args[0], env, bars.len())?;
            let color = draw.args.get(1).map(|expr| color(expr, pal)).transpose()?;
            match draw.kind {
                DrawKind::Line => {
                    let color = color.filter(|c| *c != u32::MAX).unwrap_or(pal.text_up);
                    out.push(DrawCmd::Polyline {
                        pts: values
                            .iter()
                            .enumerate()
                            .filter(|(_, v)| v.is_finite())
                            .map(|(i, v)| (i, *v))
                            .collect(),
                        color,
                        w: 1,
                    });
                }
                DrawKind::Hist => {
                    for (i, value) in values.iter().copied().enumerate() {
                        if !value.is_finite() {
                            continue;
                        }
                        out.push(DrawCmd::Bar {
                            i,
                            v0: 0.0,
                            v1: value,
                            hollow: false,
                            color: color.filter(|c| *c != u32::MAX).unwrap_or(if value >= 0.0 {
                                pal.candle_up
                            } else {
                                pal.candle_down
                            }),
                        });
                    }
                }
                DrawKind::HLine => {
                    let Some(value) = values.into_iter().rev().find(|v| v.is_finite()) else {
                        return Ok(());
                    };
                    out.push(DrawCmd::HLine {
                        p: value,
                        color: color
                            .filter(|c| *c != u32::MAX)
                            .unwrap_or_else(indicator::chart_zero),
                        w: 1,
                    });
                }
                _ => {}
            }
        }
    }
    Ok(())
}

fn color(expr: &Expr, pal: &Palette) -> Result<u32, String> {
    match expr {
        Expr::Num(v) => Ok(*v as u32),
        Expr::Var(name) => match name.as_str() {
            "AUTO" => Ok(u32::MAX),
            "UP" => Ok(pal.candle_up),
            "DOWN" => Ok(pal.candle_down),
            "TEXT" => Ok(indicator::chart_text()),
            "ZERO" => Ok(indicator::chart_zero()),
            "DIF" => Ok(pal.dif),
            "DEA" => Ok(pal.dea),
            "K" => Ok(pal.kdj_k),
            "D" => Ok(pal.kdj_d),
            "J" => Ok(pal.kdj_j),
            "RSI1" => Ok(pal.rsi_1),
            "RSI2" => Ok(pal.rsi_2),
            "RSI3" => Ok(pal.rsi_3),
            "BOLL_MID" => Ok(pal.boll_mid),
            "BOLL_UP" => Ok(pal.boll_up),
            "BOLL_DOWN" => Ok(pal.boll_down),
            "OBV" => Ok(pal.obv),
            "CCI" => Ok(pal.cci),
            "BIAS1" => Ok(pal.bias[0]),
            "BIAS2" => Ok(pal.bias[1]),
            "BIAS3" => Ok(pal.bias[2]),
            "MA1" => Ok(pal.ma[0]),
            "MA2" => Ok(pal.ma[1]),
            "MA3" => Ok(pal.ma[2]),
            "MA4" => Ok(pal.ma[3]),
            "MA5" => Ok(pal.ma[4]),
            "MA6" => Ok(pal.ma[5]),
            "EMA1" => Ok(pal.ema[0]),
            "EMA2" => Ok(pal.ema[1]),
            "EMA3" => Ok(pal.ema[2]),
            "EMA4" => Ok(pal.ema[3]),
            "EMA5" => Ok(pal.ema[4]),
            "EMA6" => Ok(pal.ema[5]),
            _ => Err(format!("未知颜色 {name}")),
        },
        _ => Err("颜色必须是名称或 0xRRGGBB".to_owned()),
    }
}
