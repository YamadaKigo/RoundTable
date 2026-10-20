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
    Str(String),
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
    Pagoda,
    Close,
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

#[derive(Clone, Default)]
pub(crate) struct ScriptSettings {
    pub(crate) show_status: Option<bool>,
    pub(crate) show_time: Option<bool>,
    pub(crate) show_axis: Option<bool>,
    pub(crate) show_crosshair: Option<bool>,
}

#[derive(Clone)]
enum AxisSpec {
    Auto { zero: bool },
    Fixed { values: Vec<f64>, zero: bool },
}

#[derive(Clone)]
struct StatusStmt {
    label: Expr,
    value: Expr,
    color: Option<Expr>,
}

#[derive(Clone)]
pub(crate) struct ScriptStatus {
    label: String,
    values: Vec<f64>,
    color: u32,
}

#[derive(Clone)]
pub(crate) struct ScriptRender {
    pub(crate) cmds: Vec<DrawCmd>,
    pub(crate) settings: ScriptSettings,
    status: Vec<ScriptStatus>,
    axis: Option<AxisSpec>,
}

#[derive(Clone)]
pub(crate) struct ScriptProgram {
    statements: Vec<Statement>,
    settings: ScriptSettings,
    status: Vec<StatusStmt>,
    axis: Option<AxisSpec>,
}

#[derive(Clone, PartialEq, Debug)]
enum Token {
    Number(f64),
    Str(String),
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
        let mut settings = ScriptSettings::default();
        let mut status = Vec::new();
        let mut axis = None;
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
            match name.as_str() {
                "STATUS" | "STATUS_SHOW" | "SHOW_STATUS" => {
                    settings.show_status = Some(switch_arg(&name, &args)?);
                    continue;
                }
                "STATUS_HIDE" | "HIDE_STATUS" => {
                    settings.show_status = Some(false);
                    continue;
                }
                "TIME" | "TIME_SHOW" | "SHOW_TIME" => {
                    settings.show_time = Some(switch_arg(&name, &args)?);
                    continue;
                }
                "TIME_HIDE" | "HIDE_TIME" => {
                    settings.show_time = Some(false);
                    continue;
                }
                "AXIS" | "AXIS_SHOW" | "SHOW_AXIS" => {
                    let show = switch_arg(&name, &args)?;
                    settings.show_axis = Some(show);
                    if show {
                        axis.get_or_insert(AxisSpec::Auto { zero: false });
                    }
                    continue;
                }
                "AXIS_HIDE" | "HIDE_AXIS" => {
                    settings.show_axis = Some(false);
                    continue;
                }
                "AXIS_FIXED" => {
                    if args.is_empty() {
                        return Err("AXIS_FIXED 需要至少一个刻度".to_owned());
                    }
                    let values = args
                        .iter()
                        .map(number_expr)
                        .collect::<Result<Vec<_>, _>>()?;
                    axis = Some(AxisSpec::Fixed {
                        values,
                        zero: false,
                    });
                    settings.show_axis.get_or_insert(true);
                    continue;
                }
                "AXIS_DYNAMIC" | "AXIS_AUTO" => {
                    let zero = args.first().map(bool_expr).transpose()?.unwrap_or(false);
                    axis = Some(AxisSpec::Auto { zero });
                    settings.show_axis.get_or_insert(true);
                    continue;
                }
                "AXIS_ZERO" => {
                    axis = Some(AxisSpec::Auto { zero: true });
                    settings.show_axis.get_or_insert(true);
                    continue;
                }
                "CROSSHAIR" | "CROSSHAIR_SHOW" | "CROSS_SHOW" => {
                    settings.show_crosshair = Some(switch_arg(&name, &args)?);
                    continue;
                }
                "CROSSHAIR_HIDE" | "CROSS_HIDE" => {
                    settings.show_crosshair = Some(false);
                    continue;
                }
                "STATUS_FIELD" | "STATUS_VALUE" => {
                    if args.len() < 2 || args.len() > 3 {
                        return Err(
                            "STATUS_FIELD 需要 2 到 3 个参数：标签、数值、颜色；不支持位置参数"
                                .to_owned(),
                        );
                    }
                    status.push(StatusStmt {
                        label: args[0].clone(),
                        value: args[1].clone(),
                        color: args.get(2).cloned(),
                    });
                    continue;
                }
                _ => {}
            }
            let kind = draw_kind(&name).ok_or_else(|| format!("未知命令 {name}"))?;
            statements.push(Statement::Draw(DrawStmt { kind, args }));
        }
        Ok(Self {
            statements,
            settings,
            status,
            axis,
        })
    }

    #[allow(dead_code)]
    pub(crate) fn compute(&self, bars: &[Bar], pal: &Palette) -> Result<Vec<DrawCmd>, String> {
        Ok(self.render(bars, pal)?.cmds)
    }

    pub(crate) fn render(&self, bars: &[Bar], pal: &Palette) -> Result<ScriptRender, String> {
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
        let mut status = Vec::with_capacity(self.status.len());
        for item in &self.status {
            status.push(ScriptStatus {
                label: static_text(&item.label)?,
                values: eval(&item.value, &env, len)?,
                color: item
                    .color
                    .as_ref()
                    .map(|color| color_expr(color, pal))
                    .transpose()?
                    .filter(|color| *color != u32::MAX)
                    .unwrap_or_else(indicator::chart_text),
            });
        }
        Ok(ScriptRender {
            cmds: out,
            settings: self.settings.clone(),
            status,
            axis: self.axis.clone(),
        })
    }
}

impl ScriptRender {
    pub(crate) fn value_range(&self, lo: f64, hi: f64) -> (f64, f64) {
        match &self.axis {
            Some(AxisSpec::Fixed { values, zero }) => {
                let mut lo = values.iter().copied().reduce(f64::min).unwrap_or(lo);
                let mut hi = values.iter().copied().reduce(f64::max).unwrap_or(hi);
                if *zero {
                    lo = lo.min(0.0);
                    hi = hi.max(0.0);
                }
                (lo, hi)
            }
            Some(AxisSpec::Auto { zero: true }) => (lo.min(0.0), hi.max(0.0)),
            _ => (lo, hi),
        }
    }

    pub(crate) fn layout(&self, base: indicator::ChartLayout) -> indicator::ChartLayout {
        let status_rows = if self.settings.show_status == Some(true) {
            base.status_rows.max(2)
        } else {
            0
        };
        let time_rows = if self.settings.show_time == Some(true) {
            base.time_rows.max(2)
        } else {
            0
        };
        indicator::ChartLayout::new(status_rows, time_rows)
    }

    pub(crate) fn axis(&self, ctx: &indicator::ChartCtx) -> Option<Vec<DrawCmd>> {
        if self.settings.show_axis != Some(true) {
            return Some(Vec::new());
        }
        match &self.axis {
            None => Some(indicator::axis_ticks(
                ctx,
                3,
                indicator::chart_text(),
                indicator::chart_tick(),
                indicator::format_value,
            )),
            Some(AxisSpec::Fixed { values, zero }) => {
                let mut out = Vec::new();
                let mut values = values.clone();
                if *zero && !values.iter().any(|value| value.abs() < f64::EPSILON) {
                    values.push(0.0);
                }
                values.sort_by(f64::total_cmp);
                values.dedup_by(|a, b| (*a - *b).abs() < 1e-12);
                for value in values {
                    if !value.is_finite() || value < ctx.lo || value > ctx.hi {
                        continue;
                    }
                    out.push(DrawCmd::HLine {
                        p: value,
                        color: indicator::chart_tick(),
                        w: 1,
                    });
                    out.push(DrawCmd::AxisText {
                        p: value,
                        s: format_axis_value(value),
                        fg: indicator::chart_text(),
                        bg: 0,
                    });
                }
                Some(out)
            }
            Some(AxisSpec::Auto { zero }) => {
                let mut out = indicator::axis_ticks(
                    ctx,
                    3,
                    indicator::chart_text(),
                    indicator::chart_tick(),
                    format_axis_value,
                );
                if *zero && ctx.lo <= 0.0 && ctx.hi >= 0.0 {
                    out.push(DrawCmd::HLine {
                        p: 0.0,
                        color: indicator::chart_zero(),
                        w: 1,
                    });
                }
                Some(out)
            }
        }
    }

    pub(crate) fn time(&self, ctx: &indicator::ChartCtx) -> Option<Vec<DrawCmd>> {
        match self.settings.show_time {
            Some(true) => Some(indicator::time_labels(ctx, indicator::chart_text())),
            _ => Some(Vec::new()),
        }
    }

    pub(crate) fn status(&self, ctx: &indicator::ChartCtx) -> Option<Vec<DrawCmd>> {
        if self.settings.show_status != Some(true) {
            return Some(Vec::new());
        }
        if self.status.is_empty() {
            return Some(Vec::new());
        }
        let index = ctx.focus_bar()?;
        Some(
            self.status
                .iter()
                .map(|item| {
                    indicator::status_field(
                        &item.label,
                        format_status_value(item.values.get(index).copied()),
                        item.color,
                    )
                })
                .collect(),
        )
    }

    pub(crate) fn crosshair(&self, ctx: &indicator::ChartCtx) -> Vec<DrawCmd> {
        if self.settings.show_crosshair != Some(true) {
            return Vec::new();
        }
        indicator::crosshair(
            ctx,
            indicator::chart_cross(),
            indicator::chart_text(),
            indicator::chart_cross_bg(),
            indicator::format_value,
        )
    }
}

fn format_axis_value(value: f64) -> String {
    indicator::format_value(value)
}

fn format_status_value(value: Option<f64>) -> String {
    let Some(value) = value.filter(|value| value.is_finite()) else {
        return "--".to_owned();
    };
    indicator::format_value(value)
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
            let end = i + 7;
            let color_slot = matches!(
                out.last(),
                Some(Token::LParen | Token::Comma | Token::Colon | Token::Equal)
            );
            let is_color = color_slot
                && end <= chars.len()
                && chars[i + 1..end]
                    .iter()
                    .all(|digit| digit.is_ascii_hexdigit())
                && chars
                    .get(end)
                    .is_none_or(|next| !next.is_ascii_alphanumeric() && *next != '_');
            if is_color {
                out.push(Token::Str(chars[i..end].iter().collect()));
                i = end;
                continue;
            }
            while i < chars.len() && chars[i] != '\n' {
                i += 1;
            }
            continue;
        }
        if c == '"' {
            i += 1;
            let mut text = String::new();
            let mut closed = false;
            while i < chars.len() {
                match chars[i] {
                    '"' => {
                        closed = true;
                        i += 1;
                        break;
                    }
                    '\\' if i + 1 < chars.len() => {
                        i += 1;
                        text.push(match chars[i] {
                            'n' => '\n',
                            't' => '\t',
                            other => other,
                        });
                        i += 1;
                    }
                    other => {
                        text.push(other);
                        i += 1;
                    }
                }
            }
            if !closed {
                return Err("字符串缺少右引号".to_owned());
            }
            out.push(Token::Str(text));
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
            Token::Str(s) => Ok(Expr::Str(s)),
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

fn single_arg(name: &str, args: &[Expr]) -> Result<Expr, String> {
    if args.len() != 1 {
        return Err(format!("{name} 需要 1 个参数"));
    }
    Ok(args[0].clone())
}

fn switch_arg(name: &str, args: &[Expr]) -> Result<bool, String> {
    if args.is_empty() {
        Ok(true)
    } else {
        bool_expr(&single_arg(name, args)?)
    }
}

fn number_expr(expr: &Expr) -> Result<f64, String> {
    match expr {
        Expr::Num(value) => Ok(*value),
        Expr::Binary(Op::Sub, left, right) => {
            if number_expr(left)? == 0.0 {
                Ok(-number_expr(right)?)
            } else {
                Err("配置参数必须是常数".to_owned())
            }
        }
        _ => Err("配置参数必须是常数".to_owned()),
    }
}

fn bool_expr(expr: &Expr) -> Result<bool, String> {
    Ok(number_expr(expr)? != 0.0)
}

fn static_text(expr: &Expr) -> Result<String, String> {
    match expr {
        Expr::Str(text) => Ok(text.clone()),
        Expr::Var(name) => Ok(name.clone()),
        _ => Err("这里需要字符串或标签名".to_owned()),
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
        "PAGODA" | "RENKO" => Some(DrawKind::Pagoda),
        "CLOSE" | "CLOSELINE" => Some(DrawKind::Close),
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
        Expr::Str(_) => Err("字符串不能参与数值计算".to_owned()),
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
    let color_arg = |index: usize| {
        draw.args
            .get(index)
            .map(|expr| color_expr(expr, pal))
            .transpose()
    };
    match draw.kind {
        DrawKind::Candle => {
            if draw.args.len() > 3 {
                return Err("CANDLE 最多接受 3 个参数：上涨色、下跌色、线宽".to_owned());
            }
            let up = color_arg(0)?
                .filter(|color| *color != u32::MAX)
                .unwrap_or(pal.candle_up);
            let down = color_arg(1)?
                .filter(|color| *color != u32::MAX)
                .unwrap_or(pal.candle_down);
            let w = draw.args.get(2).map(line_width).transpose()?.unwrap_or(1);
            for (i, bar) in bars.iter().enumerate() {
                let color = if bar.vol <= 0.0 {
                    pal.candle_suspended
                } else if bar.c >= bar.o {
                    up
                } else {
                    down
                };
                if w == 1 {
                    out.push(DrawCmd::Candle {
                        i,
                        o: bar.o,
                        h: bar.h,
                        l: bar.l,
                        c: bar.c,
                        color,
                    });
                } else {
                    out.push(DrawCmd::StyledCandle {
                        i,
                        o: bar.o,
                        h: bar.h,
                        l: bar.l,
                        c: bar.c,
                        color,
                        w,
                    });
                }
            }
        }
        DrawKind::Ohlc => {
            if draw.args.len() > 3 {
                return Err("OHLC 最多接受 3 个参数：上涨色、下跌色、线宽".to_owned());
            }
            let up = color_arg(0)?
                .filter(|color| *color != u32::MAX)
                .unwrap_or(pal.candle_up);
            let down = color_arg(1)?
                .filter(|color| *color != u32::MAX)
                .unwrap_or(pal.candle_down);
            let w = draw.args.get(2).map(line_width).transpose()?.unwrap_or(1);
            for (i, bar) in bars.iter().enumerate() {
                let color = if bar.c >= bar.o { up } else { down };
                if w == 1 {
                    out.push(DrawCmd::Ohlc {
                        i,
                        o: bar.o,
                        h: bar.h,
                        l: bar.l,
                        c: bar.c,
                        color,
                    });
                } else {
                    out.push(DrawCmd::StyledOhlc {
                        i,
                        o: bar.o,
                        h: bar.h,
                        l: bar.l,
                        c: bar.c,
                        color,
                        w,
                    });
                }
            }
        }
        DrawKind::Pagoda => {
            if draw.args.len() > 3 {
                return Err("PAGODA 最多接受 3 个参数：上涨色、下跌色、线宽".to_owned());
            }
            let up = color_arg(0)?
                .filter(|color| *color != u32::MAX)
                .unwrap_or(pal.candle_up);
            let down = color_arg(1)?
                .filter(|color| *color != u32::MAX)
                .unwrap_or(pal.candle_down);
            let w = draw.args.get(2).map(line_width).transpose()?.unwrap_or(1);
            for (i, bar) in bars.iter().enumerate() {
                out.push(DrawCmd::Pagoda {
                    i,
                    o: bar.o,
                    h: bar.h,
                    l: bar.l,
                    c: bar.c,
                    color: if bar.c >= bar.o { up } else { down },
                    w,
                });
            }
        }
        DrawKind::Close => {
            if draw.args.len() > 3 {
                return Err("CLOSE 最多接受 3 个参数：上涨色、下跌色、线宽".to_owned());
            }
            let up = color_arg(0)?
                .filter(|color| *color != u32::MAX)
                .unwrap_or(pal.candle_up);
            let down = color_arg(1)?
                .filter(|color| *color != u32::MAX)
                .unwrap_or(pal.candle_down);
            let w = draw.args.get(2).map(line_width).transpose()?.unwrap_or(1);
            for i in 0..bars.len() {
                let (p0, p1) = if i == 0 {
                    (bars[i].c, bars[i].c)
                } else {
                    (bars[i - 1].c, bars[i].c)
                };
                out.push(DrawCmd::CloseLine {
                    i,
                    p0,
                    p1,
                    color: if p1 >= p0 { up } else { down },
                    w,
                });
            }
        }
        DrawKind::Volume => {
            if draw.args.is_empty() {
                out.extend(indicator::volume_bars(bars, pal.candle_up, pal.candle_down));
                return Ok(());
            }
            if draw.args.len() > 4 {
                return Err("VOL 最多接受 4 个参数：上涨色、下跌色、空心、线宽".to_owned());
            }
            let up = color_arg(0)?
                .filter(|color| *color != u32::MAX)
                .unwrap_or(pal.candle_up);
            let down = color_arg(1)?
                .filter(|color| *color != u32::MAX)
                .unwrap_or(pal.candle_down);
            let hollow = draw.args.get(2).map(bool_expr).transpose()?.unwrap_or(true);
            let w = draw.args.get(3).map(line_width).transpose()?.unwrap_or(1);
            for (i, bar) in bars.iter().enumerate() {
                let color = if bar.c >= bar.o { up } else { down };
                if w == 1 {
                    out.push(DrawCmd::Bar {
                        i,
                        v0: 0.0,
                        v1: bar.vol,
                        color,
                        hollow,
                    });
                } else {
                    out.push(DrawCmd::StyledBar {
                        i,
                        v0: 0.0,
                        v1: bar.vol,
                        color,
                        hollow,
                        w,
                    });
                }
            }
        }
        DrawKind::Zero => {
            if draw.args.len() > 2 {
                return Err("ZERO 最多接受 2 个参数：颜色、线宽".to_owned());
            }
            out.push(DrawCmd::HLine {
                p: 0.0,
                color: color_arg(0)?
                    .filter(|color| *color != u32::MAX)
                    .unwrap_or_else(indicator::chart_zero),
                w: draw.args.get(1).map(line_width).transpose()?.unwrap_or(1),
            });
        }
        DrawKind::Line => {
            if draw.args.is_empty() || draw.args.len() > 3 {
                return Err("LINE 需要数值，可附加颜色和线宽".to_owned());
            }
            let values = eval(&draw.args[0], env, bars.len())?;
            let color = color_arg(1)?
                .filter(|color| *color != u32::MAX)
                .unwrap_or(pal.text_up);
            let w = draw.args.get(2).map(line_width).transpose()?.unwrap_or(1);
            out.push(DrawCmd::Polyline {
                pts: values
                    .iter()
                    .enumerate()
                    .filter(|(_, value)| value.is_finite())
                    .map(|(i, value)| (i, *value))
                    .collect(),
                color,
                w,
            });
        }
        DrawKind::Hist => {
            if draw.args.is_empty() || draw.args.len() > 5 {
                return Err(
                    "HIST 需要数值，可附加颜色、空心、线宽或上涨色、下跌色、空心、线宽".to_owned(),
                );
            }
            let values = eval(&draw.args[0], env, bars.len())?;
            let separate = draw.args.len() == 5;
            let first = color_arg(1)?.filter(|color| *color != u32::MAX);
            let (up, down, color) = if separate {
                let up = first.unwrap_or(pal.candle_up);
                let down = color_arg(2)?
                    .filter(|color| *color != u32::MAX)
                    .unwrap_or(pal.candle_down);
                (up, down, None)
            } else {
                (pal.candle_up, pal.candle_down, first)
            };
            let hollow_index = if separate { 3 } else { 2 };
            let width_index = if separate { 4 } else { 3 };
            let hollow = draw
                .args
                .get(hollow_index)
                .map(bool_expr)
                .transpose()?
                .unwrap_or(false);
            let w = draw
                .args
                .get(width_index)
                .map(line_width)
                .transpose()?
                .unwrap_or(1);
            for (i, value) in values.iter().copied().enumerate() {
                if !value.is_finite() {
                    continue;
                }
                let color = color.unwrap_or(if value >= 0.0 { up } else { down });
                if w == 1 {
                    out.push(DrawCmd::Bar {
                        i,
                        v0: 0.0,
                        v1: value,
                        hollow,
                        color,
                    });
                } else {
                    out.push(DrawCmd::StyledBar {
                        i,
                        v0: 0.0,
                        v1: value,
                        hollow,
                        color,
                        w,
                    });
                }
            }
        }
        DrawKind::HLine => {
            if draw.args.is_empty() || draw.args.len() > 3 {
                return Err("HLINE 需要数值，可附加颜色和线宽".to_owned());
            }
            let values = eval(&draw.args[0], env, bars.len())?;
            let color = color_arg(1)?;
            let w = draw.args.get(2).map(line_width).transpose()?.unwrap_or(1);
            let Some(value) = values.into_iter().rev().find(|value| value.is_finite()) else {
                return Ok(());
            };
            out.push(DrawCmd::HLine {
                p: value,
                color: color
                    .filter(|color| *color != u32::MAX)
                    .unwrap_or_else(indicator::chart_zero),
                w,
            });
        }
    }
    Ok(())
}

fn line_width(expr: &Expr) -> Result<u32, String> {
    Ok(number_expr(expr)?.round().clamp(1.0, 16.0) as u32)
}

fn color_expr(expr: &Expr, pal: &Palette) -> Result<u32, String> {
    match expr {
        Expr::Num(value) => Ok(*value as u32 & 0x00FF_FFFF),
        Expr::Str(text) => parse_color_text(text),
        Expr::Var(name) => named_color(name, pal),
        Expr::Call(name, args) => match name.as_str() {
            "COLOR" | "CUSTOM_COLOR" => {
                let value = single_arg(name, args)?;
                match value {
                    Expr::Str(text) => parse_color_text(&text),
                    other => Ok(number_expr(&other)? as u32 & 0x00FF_FFFF),
                }
            }
            "SYSTEM_COLOR" | "SYSCOLOR" | "PALETTE" => {
                let name = static_text(&single_arg(name, args)?)?;
                named_color(&name, pal)
            }
            "RGB" => {
                if args.len() != 3 {
                    return Err("RGB 需要 3 个 0 到 255 的参数".to_owned());
                }
                let r = number_expr(&args[0])?.round().clamp(0.0, 255.0) as u32;
                let g = number_expr(&args[1])?.round().clamp(0.0, 255.0) as u32;
                let b = number_expr(&args[2])?.round().clamp(0.0, 255.0) as u32;
                Ok((r << 16) | (g << 8) | b)
            }
            _ => Err(format!("未知颜色函数 {name}")),
        },
        _ => Err("颜色必须是名称、0xRRGGBB 或颜色函数".to_owned()),
    }
}

fn parse_color_text(text: &str) -> Result<u32, String> {
    let value = text.trim().trim_start_matches('#');
    if value.len() != 6 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(format!("颜色 {text} 必须是 #RRGGBB"));
    }
    u32::from_str_radix(value, 16).map_err(|error| error.to_string())
}

fn named_color(name: &str, pal: &Palette) -> Result<u32, String> {
    match name {
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
    }
}
