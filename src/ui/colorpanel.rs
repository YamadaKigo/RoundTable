// ===========================================================================
//  模块 ui::colorpanel：涨跌颜色与自绘颜色面板
// ===========================================================================
//  颜色一律 (u8,u8,u8)；面板是主窗口内完全自绘的浮动面板，不用 egui 的 Window。
//  本文件 = 拆分前那份单文件里的「十五、涨跌颜色与自绘颜色面板」。
use crate::app::*;
use crate::grid::*;
use crate::theme::*;
use crate::ui::panel::*;
use eframe::egui;

// ===========================================================================
//  十五、涨跌颜色与自绘颜色面板
// ===========================================================================
//  颜色约定（重要，别处不要再写另一套）：
//    · 内部一律 (u8, u8, u8)：每通道 0-255 的整数；
//    · 显示 / 输入一律 #RRGGBB 六位十六进制（如 #2288FF）；三位简写（#28F）和
//      rgb() 形式一律不认；解析时十六进制大小写都收，显示时统一大写；
//    · 滑块拖动产生整数跳变：H 是 0..=360 的整数度，R / G / B / S / V 都是 0..=255
//      的整数，全程不往状态里存小数（内部换算用 f32 只是过一道，落回来的一定是整数）。
//    · 自定义颜色不设档位：任何 24bit 颜色都收（0..=255 随便取），界面配色表（第三节那些
//      C_* 常量）的 #aabbcc 规矩只管主题色，跟用户自己挑的涨跌色无关。
//  面板规格（主窗口内的自绘浮动面板，不用 egui::Window / Slider / color_picker 的任何外观）：
//    · 宽 224px（28 个 8px 内容格），高 312px（39 个内容格）；
//    · 最外圈那 1px 边框画在网格线格里，不属于本体（和主窗口外框同一套做法）；
//    · 内部自上而下 7 段，每段高度都是 8 的倍数；段与段之间那条 1px 线格为两个相邻段
//      共用的同一条线，所以各段相加正好 312，不会多出零头：
//        标题栏 24（可拖动）+ SV 调色板 128 + 色相条 16
//        + RGB 三行 48 + HSV 三行 48 + Hex 输入框 24 + 底部按钮 24 = 312
//    · 面板只管「调一个颜色」：编辑目标在打开时由外面的按钮定（设置涨色 / 设置跌色），
//      面板里不再有切换目标的色块 —— 这块面板将来说不定还要拿来调别的东西的颜色。
//    · 控件边框一律压在网格线格上：Hex 输入框那圈边框的上下两条边，正好落在「HSV 三行下沿」
//      和「底部按钮上沿」这两条横网格线上，所以它的矩形高 25px（3 个内容格 + 两端各 1px
//      线格），白底只填中间 23 行；边框压不在网格线上，就会和旁边的线格叠成 2px。
//    · 面板位置存 struct 字段（整数像素）；拖动按 1px 走，不对齐 8px 网格 —— 只有面板内部
//      各段相对面板左上角才是 8 的倍数。拖动只认标题栏那 24px。
//  交互一律走 ui.allocate_rect(rect, Sense::click() / drag())，外观全部 painter 手绘：
//  纯色矩形 + 1px 硬边，没有圆角、没有阴影、没有平滑渐变（渐变都用 1px 竖条拼）。

/// 面板宽 28 个内容格 = 224px；高 39 个内容格 = 312px。
pub(crate) const CP_COLS: i32 = 28;
pub(crate) const CP_W: f32 = CP_COLS as f32 * GRID;
pub(crate) const CP_H: f32 = 312.0;

/// 左右内边距各 1 格：内容列宽 = 224 - 16 = 208px（26 格）。
pub(crate) const CP_PAD: f32 = GRID;
pub(crate) const CP_CONTENT_W: f32 = CP_W - 2.0 * CP_PAD;

/// 自上而下各段的高度，全部是 8 的倍数。
pub(crate) const CP_TITLE_H: f32 = 24.0;
pub(crate) const CP_SV_H: f32 = 128.0;
pub(crate) const CP_HUE_H: f32 = 16.0;
pub(crate) const CP_ROW_H: f32 = 16.0;
pub(crate) const CP_HEX_H: f32 = 24.0;
pub(crate) const CP_BTN_H: f32 = 24.0;

/// 各段的纵向起点（相对面板左上角），同样都是 8 的倍数。
pub(crate) const CP_Y_SV: f32 = CP_TITLE_H;
pub(crate) const CP_Y_HUE: f32 = CP_Y_SV + CP_SV_H;
pub(crate) const CP_Y_RGB: f32 = CP_Y_HUE + CP_HUE_H;
pub(crate) const CP_Y_HSV: f32 = CP_Y_RGB + 3.0 * CP_ROW_H;
pub(crate) const CP_Y_HEX: f32 = CP_Y_HSV + 3.0 * CP_ROW_H;
pub(crate) const CP_Y_BTN: f32 = CP_Y_HEX + CP_HEX_H;

/// Hex 输入框那个矩形的真实高度：3 个内容格 + 两端各 1px 线格 = 24 + 1 = 25px。
/// 两端那 1px 就是网格线格本身（CP_Y_HEX 与 CP_Y_BTN 这两条横线），边框画上去正好压住它们。
pub(crate) const CP_HEX_BOX_H: f32 = CP_HEX_H + LINE_W;

/// SV 调色板：26 x 16 个 8x8 色块 = 416 块，正好铺满 208x128（不用平滑渐变）。
pub(crate) const SV_COLS: i32 = 26;
pub(crate) const SV_ROWS: i32 = 16;
pub(crate) const SV_BLOCK: f32 = GRID;

/// 色相条：208 根 1px 宽的竖条，铺满 208x16。
pub(crate) const HUE_BARS: i32 = 208;

/// 色相条上的把手：1 格宽（8px）、2 格高（16px）—— 和滑块把手同一档尺寸，都是 8 的倍数。
pub(crate) const CP_HUE_HANDLE_W: f32 = GRID;

/// 滑块一行里的横向分格：标签 2 格 + 间隔 1 格 + 轨道 19 格 + 间隔 1 格 + 数值 3 格
/// = 26 格 = 208px，正好等于内容列宽。
pub(crate) const CP_SLIDER_LABEL_W: f32 = 2.0 * GRID;
pub(crate) const CP_SLIDER_VALUE_W: f32 = 3.0 * GRID;
pub(crate) const CP_SLIDER_GAP: f32 = GRID;
pub(crate) const CP_TRACK_W: f32 = CP_CONTENT_W - CP_SLIDER_LABEL_W - CP_SLIDER_VALUE_W - 2.0 * CP_SLIDER_GAP;

/// 把手宽 1 格；把手能走的距离 = 轨道宽 - 把手宽（整数），比例乘它就得整数像素位置。
pub(crate) const CP_HANDLE_W: f32 = GRID;
pub(crate) const CP_HANDLE_TRAVEL: f32 = CP_TRACK_W - CP_HANDLE_W;

/// 两个色块 / 两个按钮平分内容列：各 13 格（104px）—— 内容列 26 格，一半正好 104px，
/// 中间那条 1px 网格线格就落在正中间那条网格线上（相对第 104 列），两半共用它：
/// 左半边的矩形得盖住这条线（104 + 1 = 105px），右半边从这条线开始（104px），
/// 这样右半边的右边框才压得住内容列最后一列（相对第 207 列，和 Hex 输入框的右边框同一列）。
pub(crate) const CP_HALF_W: f32 = CP_CONTENT_W / 2.0;
pub(crate) const CP_LEFT_W: f32 = CP_HALF_W + LINE_W;

/// Hex 输入框最多 7 个字符（# 加 6 位十六进制），多出来的字符直接不收。
pub(crate) const HEX_LEN_MAX: usize = 7;

/// 面板第一次打开时的位置（左上角，8 的倍数）；每帧还要按窗口夹一次。
pub(crate) const CP_DEFAULT_X: f32 = 448.0;
pub(crate) const CP_DEFAULT_Y: f32 = 96.0;

// 编译期自检：面板本体、内容列和里面每一段都必须站在 8px 栅格上。
const _: () = assert!((CP_W as i32) % 8 == 0 && (CP_H as i32) % 8 == 0);
const _: () = assert!(CP_H == CP_Y_BTN + CP_BTN_H);
const _: () = assert!((CP_CONTENT_W as i32) % 8 == 0 && (CP_HALF_W as i32) % 8 == 0);
// 左半边 + 右半边 - 中间那条共用线 = 内容列宽：两块合起来正好铺满，右边框落在最后一列上。
const _: () = assert!(CP_LEFT_W + CP_HALF_W - LINE_W == CP_CONTENT_W);
const _: () = assert!((CP_TRACK_W as i32) % 8 == 0 && (CP_HANDLE_TRAVEL as i32) % 8 == 0);
const _: () = assert!((CP_Y_SV as i32) % 8 == 0);
const _: () = assert!((CP_Y_HUE as i32) % 8 == 0 && (CP_Y_RGB as i32) % 8 == 0);
const _: () = assert!((CP_Y_HSV as i32) % 8 == 0 && (CP_Y_HEX as i32) % 8 == 0);
// Hex 输入框下边框压在 CP_Y_BTN 这条横网格线上（它同时也是底部按钮的顶边），所以它自己也必须
// 是 8 的倍数 —— 边框压不在网格线上，就会和旁边那条线格叠成 2px（那条红边就是这么来的）。
const _: () = assert!((CP_Y_BTN as i32) % 8 == 0);
const _: () = assert!(CP_Y_HEX + CP_HEX_BOX_H == CP_Y_BTN + LINE_W);
const _: () = assert!(SV_COLS as f32 * SV_BLOCK == CP_CONTENT_W);
const _: () = assert!(SV_ROWS as f32 * SV_BLOCK == CP_SV_H);
const _: () = assert!(HUE_BARS as f32 * LINE_W == CP_CONTENT_W);

// ---- 颜色换算：全是纯函数，进出都是整数 ----

/// HSV → RGB：h 是 0..=360 的整数度（360 当 0 用），s / v 是 0..=255 的整数。
/// 内部那点 f32 只用来过一道折线，返回的每个通道都 round 回整数。
pub(crate) fn hsv_to_rgb(h: u16, s: u8, v: u8) -> (u8, u8, u8) {
    let hf = (h % 360) as f32;
    let sf = s as f32 / 255.0;
    let vf = v as f32 / 255.0;
    let c = vf * sf;
    let x = c * (1.0 - ((hf / 60.0) % 2.0 - 1.0).abs());
    // 六段折线：色相落在哪 60 度里就取哪一段。
    let (r, g, b) = match (hf / 60.0) as u32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    let m = vf - c;
    let to = |f: f32| ((f + m) * 255.0).round().clamp(0.0, 255.0) as u8;
    (to(r), to(g), to(b))
}

/// RGB → HSV：返回 (h 0..=360, s 0..=255, v 0..=255)，全是整数。
pub(crate) fn rgb_to_hsv(r: u8, g: u8, b: u8) -> (u16, u8, u8) {
    let rf = r as f32 / 255.0;
    let gf = g as f32 / 255.0;
    let bf = b as f32 / 255.0;
    let max = rf.max(gf).max(bf);
    let min = rf.min(gf).min(bf);
    let d = max - min;
    // 色相：哪个通道最大就以谁为基准，算完再折回 0..360。
    let h = if d == 0.0 {
        0.0
    } else if max == rf {
        60.0 * (((gf - bf) / d) % 6.0)
    } else if max == gf {
        60.0 * ((bf - rf) / d + 2.0)
    } else {
        60.0 * ((rf - gf) / d + 4.0)
    };
    let h = if h < 0.0 { h + 360.0 } else { h };
    let s = if max == 0.0 { 0.0 } else { d / max };
    (
        (h.round() as u16).min(360),
        (s * 255.0).round() as u8,
        (max * 255.0).round() as u8,
    )
}

/// (u8,u8,u8) → Color32：只在交给 painter 画的时候转一道。
pub(crate) fn rgb32(rgb: (u8, u8, u8)) -> egui::Color32 {
    egui::Color32::from_rgb(rgb.0, rgb.1, rgb.2)
}

/// (u8,u8,u8) → #RRGGBB 文本（一律大写）。
pub(crate) fn hex_of(rgb: (u8, u8, u8)) -> String {
    format!("#{:02X}{:02X}{:02X}", rgb.0, rgb.1, rgb.2)
}

/// 单个十六进制数字（大小写都收）；别的字符一律 None。
pub(crate) fn hex_digit(c: u8) -> Option<u8> {
    match c {
        b'0'..=b'9' => Some(c - b'0'),
        b'a'..=b'f' => Some(c - b'a' + 10),
        b'A'..=b'F' => Some(c - b'A' + 10),
        _ => None,
    }
}

/// 严格解析 #RRGGBB：只认「# + 正好 6 位十六进制」，其余一律 None。
/// 三位简写（#28F）、rgb(34,136,255)、位数不对、前后带空格 —— 全都算非法。
pub(crate) fn parse_hex(s: &str) -> Option<(u8, u8, u8)> {
    let bytes = s.as_bytes();
    if bytes.len() != HEX_LEN_MAX || bytes[0] != b'#' {
        return None;
    }
    let mut rgb = [0u8; 3];
    for i in 0..3 {
        let hi = hex_digit(bytes[1 + i * 2])?;
        let lo = hex_digit(bytes[2 + i * 2])?;
        rgb[i] = hi * 16 + lo;
    }
    Some((rgb[0], rgb[1], rgb[2]))
}

/// 这块底色上该配黑字还是白字：按亮度挑，12px 宋体的笔画在任何底色上都看得清。
pub(crate) fn contrast_on(rgb: (u8, u8, u8)) -> egui::Color32 {
    let lum = 299 * rgb.0 as u32 + 587 * rgb.1 as u32 + 114 * rgb.2 as u32;
    if lum > 128_000 { C_TEXT } else { C_TEXT_WHITE }
}

/// 轨道上第 i 个 1px 竖条对应的整数值：0..max 均分到「轨道宽 - 1」段上。
pub(crate) fn bar_value(i: i32, max: i32) -> i32 {
    (i as f32 * max as f32 / (CP_TRACK_W as i32 - 1) as f32).round() as i32
}

/// 把手左边缘落在哪一列像素：轨道起点（本身就在整像素上）+ round(比例 x 可走距离)。
/// 可走距离 CP_HANDLE_TRAVEL = 轨道宽 - 把手宽 = 152 - 8 = 144，是整数常量；
/// round 出来的也是整数，两个整数相加必然还是整数 —— 屏幕上永远没有半个像素的把手。
pub(crate) fn handle_left(track_min_x: f32, ratio: f32) -> f32 {
    track_min_x + (ratio * CP_HANDLE_TRAVEL).round()
}

/// SV 调色板：第 col 列（0..25）对应的饱和度，26 列从 0 铺到 255。
pub(crate) fn s_of_col(col: i32) -> u8 {
    (col as f32 * 255.0 / (SV_COLS - 1) as f32).round() as u8
}

/// SV 调色板：第 row 行（0..15）对应的明度，最上一行 255、最下一行 0。
pub(crate) fn v_of_row(row: i32) -> u8 {
    ((SV_ROWS - 1 - row) as f32 * 255.0 / (SV_ROWS - 1) as f32).round() as u8
}

/// 反过来：当前饱和度落在第几列。
pub(crate) fn col_of_s(s: u8) -> i32 {
    (s as f32 * (SV_COLS - 1) as f32 / 255.0).round() as i32
}

/// 反过来：当前明度落在第几行。
pub(crate) fn row_of_v(v: u8) -> i32 {
    SV_ROWS - 1 - (v as f32 * (SV_ROWS - 1) as f32 / 255.0).round() as i32
}

/// 色相条上第 i 根 1px 竖条对应的色相角（0..=360）。
pub(crate) fn hue_of_bar(i: i32) -> u16 {
    (i as f32 * 360.0 / (HUE_BARS - 1) as f32).round() as u16
}

/// 反过来：当前色相角落在第几根竖条上。
pub(crate) fn bar_of_hue(h: u16) -> i32 {
    (h as f32 / 360.0 * (HUE_BARS - 1) as f32).round() as i32
}

/// 色相条把手（1 格宽 = 8px）的左边缘，相对色相条起点：以选中那根 1px 竖条为中心，
/// 再夹回色相条里（右边不能越出去）。GRID / 2 = 4、选中下标也是整数 —— 落点必然是整像素。
pub(crate) fn hue_handle_left(bar: i32) -> f32 {
    (bar as f32 - GRID / 2.0).clamp(0.0, HUE_BARS as f32 * LINE_W - CP_HUE_HANDLE_W)
}

// ---- 面板状态 ----

/// 面板正在编辑哪一路颜色。
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum ColorTarget {
    Up,
    Down,
}

impl ColorTarget {
    /// 这一路的中文名（预览区色块里那 12px 宋体字也用它）。
    pub(crate) fn name(self) -> &'static str {
        match self {
            ColorTarget::Up => "涨",
            ColorTarget::Down => "跌",
        }
    }
}

/// 面板里正在拖的东西：没在拖 / 拖面板 / 拖 SV 调色板 / 拖色相条。
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum ColorDrag {
    None,
    Panel,
    Sv,
    Hue,
}

/// 颜色面板的全部跨帧状态：开关、位置、编辑目标、当前 HSV 与 RGB、两路待定值、
/// Hex 文本与焦点、拖动会话。egui 是即时模式，这些东西不存字段就跨不过一帧。
/// RGB 和 HSV 六个数一起存是有意的：哪边刚被改过就以哪边为准，另一边界立刻跟着算 ——
/// 绝不让换算来回倒（倒一遍就会掉 1 个色阶，拖滑块时把手会左右抖）。
#[derive(Clone)]
pub(crate) struct ColorPanel {
    pub(crate) open: bool,
    /// 面板左上角：整数像素（拖动按 1px 走，不吸 8 的倍数）。
    pub(crate) pos: egui::Pos2,
    pub(crate) target: ColorTarget,
    /// 当前编辑中的颜色：HSV 侧（h 0..=360，s / v 0..=255）。
    pub(crate) h: u16,
    pub(crate) s: u8,
    pub(crate) v: u8,
    /// 当前编辑中的颜色：RGB 侧（和上面三个永远同步）。
    pub(crate) rgb: (u8, u8, u8),
    /// 两路颜色的待定值：只有点「应用」才写回 up_color / down_color，取消就整个丢掉。
    pub(crate) pending_up: (u8, u8, u8),
    pub(crate) pending_down: (u8, u8, u8),
    /// Hex 输入框：文本 + 是否聚焦 + 这份文本是否合法（非法只把边框变红，什么都不应用）。
    pub(crate) hex: String,
    pub(crate) hex_focus: bool,
    pub(crate) hex_valid: bool,
    /// 正在拖什么；拖面板时还要记下鼠标相对面板左上角的偏移（面板不会跳）。
    pub(crate) drag: ColorDrag,
    pub(crate) grab: egui::Vec2,
}

impl ColorPanel {
    pub(crate) fn new() -> Self {
        Self {
            open: false,
            pos: egui::pos2(CP_DEFAULT_X, CP_DEFAULT_Y),
            target: ColorTarget::Up,
            h: 0,
            s: 0,
            v: 0,
            rgb: (0, 0, 0),
            pending_up: UP_RGB_DEFAULT,
            pending_down: DOWN_RGB_DEFAULT,
            hex: String::new(),
            hex_focus: false,
            hex_valid: true,
            drag: ColorDrag::None,
            grab: egui::Vec2::ZERO,
        }
    }

    /// 改 H / S / V：RGB 侧立刻跟着重算（这一帧以 HSV 为准）。
    pub(crate) fn set_hsv(&mut self, h: u16, s: u8, v: u8) {
        self.h = h;
        self.s = s;
        self.v = v;
        self.rgb = hsv_to_rgb(h, s, v);
    }

    /// 改 R / G / B：HSV 侧立刻跟着重算（这一帧以 RGB 为准）。
    pub(crate) fn set_rgb(&mut self, rgb: (u8, u8, u8)) {
        self.rgb = rgb;
        let (h, s, v) = rgb_to_hsv(rgb.0, rgb.1, rgb.2);
        self.h = h;
        self.s = s;
        self.v = v;
    }

    /// 把一个颜色灌进编辑状态：RGB / HSV / Hex 文本三边一次对齐。
    pub(crate) fn load(&mut self, rgb: (u8, u8, u8)) {
        self.set_rgb(rgb);
        self.hex = hex_of(rgb);
        self.hex_valid = true;
    }

    /// 某一路上次留在面板里的待定值。
    pub(crate) fn pending_for(&self, target: ColorTarget) -> (u8, u8, u8) {
        match target {
            ColorTarget::Up => self.pending_up,
            ColorTarget::Down => self.pending_down,
        }
    }

    /// 把当前编辑中的颜色存回它自己那一路的待定值（切换编辑目标、点「应用」时用）。
    pub(crate) fn save_pending(&mut self) {
        let rgb = self.rgb;
        match self.target {
            ColorTarget::Up => self.pending_up = rgb,
            ColorTarget::Down => self.pending_down = rgb,
        }
    }

    /// 取消：一个字段都不写回（struct 里的颜色一直是打开时的值），顺手清掉焦点和拖动会话。
    pub(crate) fn close_cancelled(&mut self) {
        self.open = false;
        self.hex_focus = false;
        self.drag = ColorDrag::None;
    }

    /// 把面板左上角夹回窗口里：只保证整块都在窗口内、坐标落在整数像素上。
    /// 位置不吸 8 的倍数 —— 面板可以在窗口里 1px 1px 地随便挪（只有面板内部各段相对面板
    /// 左上角才是 8 的倍数）。
    pub(crate) fn clamp_pos(&mut self, layout: egui::Rect) {
        let max_x = (layout.max.x - CP_W).max(layout.min.x);
        let max_y = (layout.max.y - CP_H).max(layout.min.y);
        self.pos.x = self.pos.x.round().clamp(layout.min.x, max_x);
        self.pos.y = self.pos.y.round().clamp(layout.min.y, max_y);
    }
}

impl StockApp {
    /// 打开颜色面板：待定值从当前生效的两个颜色取，指定编辑目标，再把目标色灌进编辑状态。
    /// 每次打开都从零起步 —— 上一次没点「应用」就关掉的编辑不留痕。
    pub(crate) fn open_color_panel(&mut self, target: ColorTarget) {
        let (up, down) = (self.up_color, self.down_color);
        let cp = &mut self.color_panel;
        cp.open = true;
        cp.pending_up = up;
        cp.pending_down = down;
        cp.target = target;
        cp.hex_focus = false;
        cp.drag = ColorDrag::None;
        let rgb = cp.pending_for(target);
        cp.load(rgb);
        self.status_msg = format!("颜色面板：正在编辑{}色", target.name());
    }

    /// 涨跌颜色面板：主窗口里的一块自绘浮动面板，每帧最后画（压在窗口里所有东西之上）。
    /// 面板状态整份搬到本地变量上改，最后再写回字段 —— 中途要调 self.text(...)，那会借走整个 self。
    pub(crate) fn ui_color_panel(&mut self, ui: &mut egui::Ui, layout: egui::Rect) {
        let mut cp = self.color_panel.clone();
        if !cp.open {
            return;
        }
        // 位置每帧收口一次：拖过、或者窗口刚变过，都保证整块在窗口里（整数像素，不吸 8 的倍数）。
        cp.clamp_pos(layout);

        let painter = ui.painter().clone();
        let panel = egui::Rect::from_min_size(cp.pos, egui::vec2(CP_W, CP_H));
        let cx = panel.min.x + CP_PAD;
        // 第 y 段（相对面板顶部的偏移，一定是 8 的倍数）在内容列里的那条横带。
        let band = |y: f32, h: f32| {
            egui::Rect::from_min_size(egui::pos2(cx, panel.min.y + y), egui::vec2(CP_CONTENT_W, h))
        };

        // 0) 面板本体：先铺底色；再登记一整块「只吃事件、不画东西」的矩形 ——
        //    它排在窗口里所有控件之后（本函数是每帧最后跑的），底下的绘图区、控制面板都抢不到
        //    这块地方（在面板空白处按下不会把 K 线拖走）；面板自己的控件又排在它之后，照样点得到。
        //    底色用工作区白、标题栏用灰：和主窗口一套配色，标题栏才看得出是标题栏。
        painter.rect_filled(panel, egui::CornerRadius::ZERO, C_WORK_BG);
        ui.allocate_rect(panel, egui::Sense::click_and_drag());

        let pointer = ui.input(|i| i.pointer.latest_pos());
        let down = ui.input(|i| i.pointer.primary_down());
        // 开工前记一份颜色：收尾时比一下就知道这一帧有没有控件改过颜色（改过就要重写 Hex 文本）。
        let rgb_at_start = cp.rgb;

        // 1) 标题栏：横贯整个面板宽（224px，不是内容列那 208px），左文字、右 24x24 关闭键，
        //    中间那条留给拖动。
        let title = egui::Rect::from_min_size(panel.min, egui::vec2(CP_W, CP_TITLE_H));
        painter.rect_filled(title, egui::CornerRadius::ZERO, C_TITLE_BG);
        self.text(
            &painter,
            egui::pos2(title.min.x + TEXT_PAD_X, title.center().y),
            egui::Align2::LEFT_CENTER,
            "颜色设置",
            C_TEXT,
        );
        let close = egui::Rect::from_min_size(
            egui::pos2(panel.max.x - CP_TITLE_H, panel.min.y),
            egui::vec2(CP_TITLE_H, CP_TITLE_H),
        );
        let drag_bar = egui::Rect::from_min_max(title.min, egui::pos2(close.min.x, title.max.y));

        // 拖标题栏 = 挪面板：会话记在 cp.drag 里，松手清空；坐标按 1px 走、最后夹回窗口
        // （面板位置不对齐 8px 网格，只有面板内部各段相对它左上角才是 8 的倍数）。
        let drag_resp = ui.allocate_rect(drag_bar, egui::Sense::click_and_drag());
        if drag_resp.is_pointer_button_down_on() && cp.drag == ColorDrag::None {
            if let Some(p) = pointer {
                cp.drag = ColorDrag::Panel;
                cp.grab = p - panel.min;
            }
        }
        if cp.drag == ColorDrag::Panel {
            if let Some(p) = pointer {
                cp.pos = p - cp.grab;
                cp.clamp_pos(layout);
            }
        }

        // 关闭键：和主窗口标题栏那个关闭键同一套外观（hover 变红、叉叉变白）；点了等于「取消」。
        let close_resp = ui.allocate_rect(close, egui::Sense::click());
        let close_hot = close_resp.hovered();
        painter.rect_filled(
            close,
            egui::CornerRadius::ZERO,
            if close_hot { C_CLOSE_HOVER } else { C_BTN_FACE },
        );
        paint_cross(
            &painter,
            close.center(),
            5,
            if close_hot { C_TEXT_WHITE } else { C_TEXT },
        );
        if close_resp.clicked() {
            cp.close_cancelled();
            self.color_panel = cp;
            self.status_msg = "颜色面板：已取消，颜色没变".to_owned();
            return;
        }

        // 2) SV 调色板：26 x 16 个 8x8 色块拼出来（不是平滑渐变）；x 轴 = 饱和度、y 轴 = 明度，
        //    每块的颜色 = 当前色相 H 下对应的 HSV 转 RGB。
        let sv = band(CP_Y_SV, CP_SV_H);
        for row in 0..SV_ROWS {
            for col in 0..SV_COLS {
                let block = egui::Rect::from_min_size(
                    egui::pos2(
                        sv.min.x + col as f32 * SV_BLOCK,
                        sv.min.y + row as f32 * SV_BLOCK,
                    ),
                    egui::vec2(SV_BLOCK, SV_BLOCK),
                );
                painter.rect_filled(
                    block,
                    egui::CornerRadius::ZERO,
                    rgb32(hsv_to_rgb(cp.h, s_of_col(col), v_of_row(row))),
                );
            }
        }
        let sv_resp = ui.allocate_rect(sv, egui::Sense::click_and_drag());
        if sv_resp.is_pointer_button_down_on() && cp.drag == ColorDrag::None {
            cp.drag = ColorDrag::Sv;
        }
        if cp.drag == ColorDrag::Sv {
            if let Some(p) = pointer {
                // 鼠标压在哪一块上，S / V 就取那一块的整数值：块和数值一一对应，不产生小数。
                let col = (((p.x - sv.min.x) / SV_BLOCK).floor() as i32).clamp(0, SV_COLS - 1);
                let row = (((p.y - sv.min.y) / SV_BLOCK).floor() as i32).clamp(0, SV_ROWS - 1);
                cp.set_hsv(cp.h, s_of_col(col), v_of_row(row));
            }
        }
        // 当前位置的标记：选中的那一块套一圈对比色 1px 方框。
        // 调色板本身不描边 —— 26x16 块要一块不落地铺满 208x128，加边就吃掉最外那一圈色块了。
        let mark = egui::Rect::from_min_size(
            egui::pos2(
                sv.min.x + col_of_s(cp.s) as f32 * SV_BLOCK,
                sv.min.y + row_of_v(cp.v) as f32 * SV_BLOCK,
            ),
            egui::vec2(SV_BLOCK, SV_BLOCK),
        );
        border_hard(&painter, mark, contrast_on(cp.rgb));

        // 3) 色相条：208 根 1px 竖条拼出 0..360 的色相（同样不用平滑渐变）。
        let hue = band(CP_Y_HUE, CP_HUE_H);
        for i in 0..HUE_BARS {
            let bar = egui::Rect::from_min_size(
                egui::pos2(hue.min.x + i as f32 * LINE_W, hue.min.y),
                egui::vec2(LINE_W, CP_HUE_H),
            );
            painter.rect_filled(
                bar,
                egui::CornerRadius::ZERO,
                rgb32(hsv_to_rgb(hue_of_bar(i), 255, 255)),
            );
        }
        let hue_resp = ui.allocate_rect(hue, egui::Sense::click_and_drag());
        if hue_resp.is_pointer_button_down_on() && cp.drag == ColorDrag::None {
            cp.drag = ColorDrag::Hue;
        }
        if cp.drag == ColorDrag::Hue {
            if let Some(p) = pointer {
                let bar = ((p.x - hue.min.x).floor() as i32).clamp(0, HUE_BARS - 1);
                cp.set_hsv(hue_of_bar(bar), cp.s, cp.v);
            }
        }
        // 当前位置：1 格宽（8px）的白底黑边把手，压在色相条上（宽 1 格、高 2 格，都是 8 的倍数）。
        let hx = hue.min.x + hue_handle_left(bar_of_hue(cp.h));
        let hue_handle = egui::Rect::from_min_size(
            egui::pos2(hx, hue.min.y),
            egui::vec2(CP_HUE_HANDLE_W, CP_HUE_H),
        );
        painter.rect_filled(hue_handle, egui::CornerRadius::ZERO, C_TEXT_WHITE);
        border_hard(&painter, hue_handle, C_BORDER);

        // 4) RGB 三行：每行 208x16。轨道渐变 = 从 (0,G,B) 排到 (255,G,B)，G / B 取当前值；
        //    改 R / G / B 就走 set_rgb（HSV 侧跟着重算），Hex 文本在收尾处统一刷新。
        let row = |y: f32| band(y, CP_ROW_H);
        let (mut cr, mut cg, mut cb) = cp.rgb;
        if let Some(x) = self.color_slider(
            ui,
            &painter,
            row(CP_Y_RGB),
            "R",
            &cr.to_string(),
            cr as f32 / 255.0,
            255,
            |i| (bar_value(i, 255) as u8, cg, cb),
        ) {
            cp.set_rgb((x as u8, cg, cb));
        }
        (cr, cg, cb) = cp.rgb;
        if let Some(x) = self.color_slider(
            ui,
            &painter,
            row(CP_Y_RGB + CP_ROW_H),
            "G",
            &cg.to_string(),
            cg as f32 / 255.0,
            255,
            |i| (cr, bar_value(i, 255) as u8, cb),
        ) {
            cp.set_rgb((cr, x as u8, cb));
        }
        (cr, cg, cb) = cp.rgb;
        if let Some(x) = self.color_slider(
            ui,
            &painter,
            row(CP_Y_RGB + 2.0 * CP_ROW_H),
            "B",
            &cb.to_string(),
            cb as f32 / 255.0,
            255,
            |i| (cr, cg, bar_value(i, 255) as u8),
        ) {
            cp.set_rgb((cr, cg, x as u8));
        }

        // 5) HSV 三行：H 是 0..=360 的整数度，S / V 是 0..=255。
        //    H 的轨道是纯色相彩虹，S / V 的轨道按当前 H 和另一个通道排。
        let (mut ch, mut cs, mut cv) = (cp.h, cp.s, cp.v);
        if let Some(x) = self.color_slider(
            ui,
            &painter,
            row(CP_Y_HSV),
            "H",
            &ch.to_string(),
            ch as f32 / 360.0,
            360,
            |i| hsv_to_rgb(bar_value(i, 360) as u16, 255, 255),
        ) {
            cp.set_hsv(x as u16, cs, cv);
        }
        (ch, cs, cv) = (cp.h, cp.s, cp.v);
        if let Some(x) = self.color_slider(
            ui,
            &painter,
            row(CP_Y_HSV + CP_ROW_H),
            "S",
            &cs.to_string(),
            cs as f32 / 255.0,
            255,
            |i| hsv_to_rgb(ch, bar_value(i, 255) as u8, cv),
        ) {
            cp.set_hsv(ch, x as u8, cv);
        }
        (ch, cs, cv) = (cp.h, cp.s, cp.v);
        if let Some(x) = self.color_slider(
            ui,
            &painter,
            row(CP_Y_HSV + 2.0 * CP_ROW_H),
            "V",
            &cv.to_string(),
            cv as f32 / 255.0,
            255,
            |i| hsv_to_rgb(ch, cs, bar_value(i, 255) as u8),
        ) {
            cp.set_hsv(ch, cs, x as u8);
        }

        // 6) Hex 输入框：白底 + 1px 硬边；非法输入时边框变红（借主窗口关闭键那个红，
        //    色值仍然只出现在第三节的颜色表里）。
        //    矩形的上下两条边就是网格线格本身：上边压在 CP_Y_HEX、下边压在 CP_Y_BTN（它同时
        //    也是底部按钮的顶边），所以矩形高 25px（3 个内容格 + 两端线格），白底只填中间 23 行。
        //    那圈边框排在按钮和分隔线之后画（见第 10 步），否则不是被盖掉、就是叠成 2px。
        let hex_box = egui::Rect::from_min_size(
            egui::pos2(cx, panel.min.y + CP_Y_HEX),
            egui::vec2(CP_CONTENT_W, CP_HEX_BOX_H),
        );
        let hex_inner = egui::Rect::from_min_max(
            egui::pos2(hex_box.min.x + LINE_W, hex_box.min.y + LINE_W),
            egui::pos2(hex_box.max.x - LINE_W, hex_box.max.y - LINE_W),
        );
        let hex_resp = ui.allocate_rect(hex_box, egui::Sense::click());
        if hex_resp.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::Text);
        }
        if hex_resp.clicked() {
            cp.hex_focus = true;
        } else if ui.input(|i| i.pointer.primary_clicked()) && !hex_resp.hovered() {
            // 点在别处（包括面板里别的控件）就取消聚焦。
            cp.hex_focus = false;
        }
        // 聚焦时收键盘：只认可见字符、退格和整段粘贴（不做选区，退格就是删掉末尾那个字符）。
        let text_before = cp.hex.clone();
        if cp.hex_focus {
            for event in ui.input(|i| i.events.clone()) {
                match event {
                    egui::Event::Text(t) => {
                        for ch in t.chars().filter(|c| !c.is_control()) {
                            if cp.hex.chars().count() < HEX_LEN_MAX {
                                cp.hex.push(ch);
                            }
                        }
                    }
                    egui::Event::Key {
                        key: egui::Key::Backspace,
                        pressed: true,
                        ..
                    } => {
                        cp.hex.pop();
                    }
                    egui::Event::Paste(t) => {
                        cp.hex = t
                            .chars()
                            .filter(|c| !c.is_control())
                            .take(HEX_LEN_MAX)
                            .collect();
                    }
                    _ => {}
                }
            }
        }
        let hex_typed = cp.hex != text_before;
        // 文本合法就把 RGB 和 HSV 一起刷新（改 Hex → 两边都更新）；非法只红边，不应用任何东西。
        match parse_hex(&cp.hex) {
            Some(rgb) => {
                cp.hex_valid = true;
                if hex_typed {
                    cp.set_rgb(rgb);
                }
            }
            None => cp.hex_valid = false,
        }
        painter.rect_filled(hex_inner, egui::CornerRadius::ZERO, C_WORK_BG);
        self.text(
            &painter,
            egui::pos2(hex_inner.min.x + TEXT_PAD_X, hex_inner.center().y),
            egui::Align2::LEFT_CENTER,
            &cp.hex,
            C_TEXT,
        );
        // 聚焦时在文字末尾画一根 1px 竖线光标（整数像素，不闪也不糊）。
        if cp.hex_focus {
            let w = self.text_width(&cp.hex);
            vline_hard(
                &painter,
                hex_inner.min.x + TEXT_PAD_X + w,
                hex_inner.min.y + 4.0,
                hex_inner.max.y - 4.0,
                C_TEXT,
            );
        }

        // 7) 底部两个按钮：左 13 格 + 中间那条共用的线格、右 13 格，外观走全工程通用的
        //    draw_button —— 中间那条竖线落在正中间的网格线上，右边框和 Hex 输入框同一列。
        let btn_row = band(CP_Y_BTN, CP_BTN_H);
        let apply_rect = egui::Rect::from_min_size(btn_row.min, egui::vec2(CP_LEFT_W, CP_BTN_H));
        let cancel_rect = egui::Rect::from_min_size(
            egui::pos2(btn_row.min.x + CP_HALF_W, btn_row.min.y),
            egui::vec2(CP_HALF_W, CP_BTN_H),
        );
        let apply_text = ButtonContent::Text("应用".to_owned());
        if self.draw_button(ui, &painter, apply_rect, &apply_text, false) {
            // 应用 = 把当前编辑中的颜色写回它自己那一路（面板一次只编辑一个颜色，由打开它的按钮定），
            // 然后关面板。另一路一个字节都不动。
            cp.save_pending();
            let rgb = cp.pending_for(cp.target);
            match cp.target {
                ColorTarget::Up => self.up_color = rgb,
                ColorTarget::Down => self.down_color = rgb,
            }
            cp.close_cancelled();
            self.status_msg = format!("已应用：{}色 {}", cp.target.name(), hex_of(rgb));
        }
        let cancel_text = ButtonContent::Text("取消".to_owned());
        if self.draw_button(ui, &painter, cancel_rect, &cancel_text, false) {
            // 取消 = 放弃修改，什么都不写回（struct 里的还是打开时那两个颜色）。
            cp.close_cancelled();
            self.status_msg = "颜色面板：已取消，颜色没变".to_owned();
        }

        // 8) 段间分隔线：排在内容之后画，免得被色块和轨道盖掉（和功能区那几条线一个道理）。
        //    两条横网格线（CP_Y_HEX / CP_Y_BTN）照样铺满整个面板宽 —— 面板左右那两条 1 格
        //    内边距上也得有线；内容列上那一段随后被 Hex 输入框的边框压过去。
        for y in [
            CP_Y_SV,
            CP_Y_HUE,
            CP_Y_RGB,
            CP_Y_RGB + CP_ROW_H,
            CP_Y_RGB + 2.0 * CP_ROW_H,
            CP_Y_HSV,
            CP_Y_HSV + CP_ROW_H,
            CP_Y_HSV + 2.0 * CP_ROW_H,
            CP_Y_HEX,
            CP_Y_BTN,
        ] {
            hline_hard(&painter, panel.min.x, panel.max.x, panel.min.y + y, C_LINE);
        }

        // 9) Hex 输入框那圈 1px 硬边最后画：正好压在刚铺的那两条网格线上（内容列那一段），
        //     合法就是分隔线那个灰、非法就是关闭键那个红 —— 边框落在网格线格里面，不多占一行，
        //     也不会跟按钮的顶边叠成 2px（相邻控件共用一条线格，跟主窗口一套规矩，谁最后画是谁）。
        border_hard(
            &painter,
            hex_box,
            if cp.hex_valid {
                C_BTN_EDGE
            } else {
                C_CLOSE_HOVER
            },
        );

        // 10) 最外圈那 1px 硬边最后画：压在面板里所有东西之上（它占的是网格线格，不属于本体）。
        border_hard(&painter, panel, C_BORDER);

        // 11) 收尾：松手就清掉拖动会话；这一帧有控件改过颜色就把 Hex 文本重写一遍，
        //     让 RGB / HSV / Hex 三边始终一致（Hex 输入框自己改的不算 —— 那边以文本为准）。
        if !down {
            cp.drag = ColorDrag::None;
        }
        if cp.rgb != rgb_at_start && !hex_typed {
            cp.hex = hex_of(cp.rgb);
            cp.hex_valid = true;
        }
        self.color_panel = cp;
    }

    /// 一行自绘滑块：左标签（2 格）+ 渐变轨道（19 格）+ 把手 + 右数值（3 格），整行 208x16。
    /// ratio = 当前值在轨道上的比例（0..=1，由整数值算出来）；grad(i) = 轨道上第 i 个
    /// 1px 竖条的颜色。返回 Some(整数值) 表示这一帧鼠标正按在这条轨道上。
    pub(crate) fn color_slider(
        &mut self,
        ui: &mut egui::Ui,
        painter: &egui::Painter,
        row: egui::Rect,
        label: &str,
        value_text: &str,
        ratio: f32,
        max: i32,
        grad: impl Fn(i32) -> (u8, u8, u8),
    ) -> Option<i32> {
        // 轨道那一段才是可点的；标签和数值各占自己那几格（宽度都是 8 的倍数）。
        let track = egui::Rect::from_min_size(
            egui::pos2(row.min.x + CP_SLIDER_LABEL_W + CP_SLIDER_GAP, row.min.y),
            egui::vec2(CP_TRACK_W, CP_ROW_H),
        );
        self.text(
            painter,
            egui::pos2(row.min.x, row.center().y),
            egui::Align2::LEFT_CENTER,
            label,
            C_TEXT,
        );
        self.text(
            painter,
            egui::pos2(row.max.x, row.center().y),
            egui::Align2::RIGHT_CENTER,
            value_text,
            C_TEXT,
        );

        // 轨道：1px 竖条一条一条拼出该通道的渐变（不许平滑填充）。轨道本身不描边：
        // 整条轨道就是渐变本体，描边会吃掉最左 / 最右那两根竖条。
        for i in 0..CP_TRACK_W as i32 {
            let bar = egui::Rect::from_min_size(
                egui::pos2(track.min.x + i as f32, track.min.y),
                egui::vec2(LINE_W, CP_ROW_H),
            );
            painter.rect_filled(bar, egui::CornerRadius::ZERO, rgb32(grad(i)));
        }

        // 拖动：鼠标在轨道上按下就算数（按住不放、拖出轨道也继续跟），按 x 反算整数值。
        let resp = ui.allocate_rect(track, egui::Sense::click_and_drag());
        let drag_x = if resp.is_pointer_button_down_on() {
            ui.input(|i| i.pointer.latest_pos())
                .map(|p| p.x.clamp(track.min.x, track.min.x + CP_HANDLE_TRAVEL))
        } else {
            None
        };

        // 把手位置 = round(比例 x 轨道可用宽度)：轨道可用宽度 = 轨道宽 - 把手宽 = 152 - 8 = 144，
        // 是个整数；比例乘整数再 round 一次，落点必然落在整像素上（track.min.x 本身也是整数），
        // 屏幕上永远不会出现半个像素的把手。正在拖的时候直接用鼠标那一列，把手跟手不慢一帧。
        let hx = match drag_x {
            Some(x) => track.min.x + (x - track.min.x).round(),
            None => handle_left(track.min.x, ratio),
        };
        let handle = egui::Rect::from_min_size(
            egui::pos2(hx, track.min.y),
            egui::vec2(CP_HANDLE_W, CP_ROW_H),
        );
        painter.rect_filled(handle, egui::CornerRadius::ZERO, C_TEXT_WHITE);
        border_hard(painter, handle, C_BTN_BORDER);

        // 返回值也按同一个整数像素反算：把手停在哪一列，通道值就是那一列对应的整数。
        drag_x.map(|x| ((x.round() - track.min.x) / CP_HANDLE_TRAVEL * max as f32).round() as i32)
    }
}

