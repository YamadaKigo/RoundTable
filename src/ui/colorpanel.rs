// ===========================================================================
//  模块 ui::colorpanel：涨跌颜色与自绘颜色面板
// ===========================================================================
//  颜色一律 (u8,u8,u8)；面板是主窗口内完全自绘的浮动面板，不用 egui 的 Window。
//  本文件 = 拆分前那份单文件里的「十五、涨跌颜色与自绘颜色面板」。
use crate::app::*;
use crate::grid::*;
use crate::theme::*;
use crate::ui::floatwin::*;
use crate::ui::button::*;
use crate::ui::textbox::*;
use eframe::egui;

// ===========================================================================
//  十五、涨跌颜色与自绘颜色面板
// ===========================================================================
//  颜色约定（重要，别处不要再写另一套）：
//    - 内部一律 (u8, u8, u8)：每通道 0-255 的整数；
//    - 显示 / 输入一律 #RRGGBB 六位十六进制（如 #2288FF）；三位简写（#28F）和
//      rgb() 形式一律不认；解析时十六进制大小写都收，显示时统一大写；
//    - 滑块拖动产生整数跳变：H 是 0..=360 的整数度，R / G / B / S / V 都是 0..=255
//      的整数，全程不往状态里存小数（内部换算用 f32 只是过一道，落回来的一定是整数）。
//    - 自定义颜色不设档位：任何 24bit 颜色都收（0..=255 随便取），界面配色表（第三节那些
//      C_* 常量）的 #aabbcc 规矩只管主题色，跟用户自己挑的涨跌色无关。
//  面板规格（主窗口内的自绘浮动面板，不用 egui::Window / Slider / color_picker 的任何外观）：
//    - 本体 26 格宽 x 36 格高 = cells(26) x cells(36) = 235x325；最外圈那 1px 是边框，
//      落在网格线格里（和主窗口外框同一套做法）；
//    - 内部自上而下 11 段，每段都是一个跨若干格的控件，相邻两段共用中间那条 1px 网格线：
//        标题栏 2 格 / SV 调色板 16 格 / 色相条 2 格 / RGB 三行各 2 格 / HSV 三行各 2 格
//        / Hex 输入框 2 格 / 底部按钮 2 格 = 36 格（每段都是一个文本行盒那么高，文字才踩在网格上）；
//      「每段内容 = 8 x 格数，段与段之间那条线另外占 1px」- 线不占内容，所以不会出现
//      「分隔线吃掉内容顶行」（旧版把线画在内容顶行上，SV 第一排因此成了 7px）。
//    - SV 调色板是 26 x 16 个 8x8 色块，块与块之间那 1px 就是共用的网格线（默认边框色）；
//      选中那块四周那圈线按底色反差取色：暗底白框、亮底黑框（#000000 / #FFFFFF）。
//    - 色相条横贯 26 格、是 233 根 1px 竖条；RGB / HSV 六行滑块按「标签 2 格 + 间隔 1 格 +
//      轨道 17 格 + 间隔 1 格 + 数值 5 格」分格，轨道和把手都贴在竖网格线上（把手 1 格宽）。
//      数值那 5 格里文字右对齐、右边正好留 1 格空档（铁律），"255" 这类最长也就是 3 格。
//    - 窗口外壳（底色 / 标题栏 / 标题栏下沿那条线 / 1px 外框 / 拖动 / 关闭键）走 ui::floatwin
//      的 Win：和源码编辑窗、窗口区里的子窗口同一套 - 1px 跟手地挪、整块夹在主窗口里，
//      只是颜色面板改了尺寸也没意义，所以不给缩放。
//    - 每一处边框都只落在网格线格上、只有 1px：相邻控件共用同一条线，绝不可能叠成 2px。
//    - 面板只管「调一个颜色」：编辑目标在打开时由外面的按钮定（设置涨色 / 设置跌色），
//      面板里不再有切换目标的色块 - 这块面板将来说不定还要拿来调别的东西的颜色。
//    - 面板位置存 struct 字段，自由像素（左上是默认值 50*9 / 11*9，拖动中 1px 1px 地挪）。
//      面板里所有元素都按「这块面板自己的左上角」那套 9px 网格排（grid 的吸附入口传 origin），
//      所以面板挪到哪一列像素上，里头的边框、文字与面板的相对位置都不变。
//      拖动只认标题栏那一段。
//  交互一律走 ui.allocate_rect(rect, Sense::click() / drag())，外观全部 painter 手绘：
//  纯色矩形 + 1px 硬边，没有圆角、没有阴影、没有平滑渐变（渐变都用 1px 竖条拼）。

/// 面板本体：26 格宽、36 格高 - cells(26) x cells(36) = 235 x 325。
/// 最外圈那 1px 是边框（落在网格线格里）；内部自上而下 11 段，每段都是一个跨若干格的控件，
/// 相邻两段共用中间那条 1px 网格线。段与段之间那条线不占内容，所以不会出现「分隔线吃掉内容
/// 顶行」（旧版把线画在内容顶行上，SV 第一排因此成了 7px）。
pub(crate) const CP_COLS: i32 = 26;
pub(crate) const CP_ROWS: i32 = 36;
pub(crate) const CP_W: f32 = cells(CP_COLS);
pub(crate) const CP_H: f32 = cells(CP_ROWS);

/// 左右边框之内的内容列宽：235 - 2 = 233px（26 个内容格 + 25 条内部线格）。
pub(crate) const CP_CONTENT_W: f32 = CP_W - 2.0 * LINE_W;

/// 内部各段占几格。
pub(crate) const CP_TITLE_ROWS: i32 = 2;
pub(crate) const CP_SV_ROWS: i32 = 16;
pub(crate) const CP_HUE_ROWS: i32 = 2;
pub(crate) const CP_SLIDER_ROWS: i32 = 2;
pub(crate) const CP_HEX_ROWS: i32 = 2;
pub(crate) const CP_BTN_ROWS: i32 = 2;

/// 各段上沿所在的横网格线序号（= 相对面板顶部的偏移 / 9）。
pub(crate) const CP_ROW_SV: i32 = CP_TITLE_ROWS;
pub(crate) const CP_ROW_HUE: i32 = CP_ROW_SV + CP_SV_ROWS;
pub(crate) const CP_ROW_RGB: i32 = CP_ROW_HUE + CP_HUE_ROWS;
pub(crate) const CP_ROW_HSV: i32 = CP_ROW_RGB + 3 * CP_SLIDER_ROWS;
pub(crate) const CP_ROW_HEX: i32 = CP_ROW_HSV + 3 * CP_SLIDER_ROWS;
pub(crate) const CP_ROW_BTN: i32 = CP_ROW_HEX + CP_HEX_ROWS;

/// 各段上沿相对面板顶部的像素偏移（全是 9 的倍数）。
pub(crate) const CP_Y_SV: f32 = grid_line(CP_ROW_SV);
pub(crate) const CP_Y_HUE: f32 = grid_line(CP_ROW_HUE);
pub(crate) const CP_Y_RGB: f32 = grid_line(CP_ROW_RGB);
pub(crate) const CP_Y_HSV: f32 = grid_line(CP_ROW_HSV);
pub(crate) const CP_Y_HEX: f32 = grid_line(CP_ROW_HEX);
pub(crate) const CP_Y_BTN: f32 = grid_line(CP_ROW_BTN);

/// 标题栏里的关闭键：跨 2 格 = cells(2) = 19x19（和标题栏一样高），贴着面板右上角。
/// 它现在由 ui::floatwin 的窗口外壳统一画（尺寸就是外壳那份 WIN_CLOSE_W，编译期对齐）。
pub(crate) const CP_CLOSE_W: f32 = cells(2);
const _: () = assert!(CP_CLOSE_W == WIN_CLOSE_W && CP_TITLE_ROWS == WIN_TITLE_ROWS);

/// 某一段（第 row 条横网格线起、跨 rows 格）的矩形。相邻两段共用边界那条网格线。
pub(crate) fn cp_band(panel: egui::Rect, row: i32, rows: i32) -> egui::Rect {
    egui::Rect::from_min_size(
        egui::pos2(panel.min.x, panel.min.y + grid_line(row)),
        egui::vec2(CP_W, cells(rows)),
    )
}

/// 某一段去掉最外圈那 1px 边框之后的内容区（内容高 = 8 x 格数）。
pub(crate) fn cp_inner(panel: egui::Rect, row: i32, rows: i32) -> egui::Rect {
    cp_band(panel, row, rows).shrink(LINE_W)
}

/// SV 调色板：26 x 16 个 8x8 色块；块与块之间那 1px 就是共用的网格线。
pub(crate) const SV_COLS: i32 = CP_COLS;
pub(crate) const SV_ROWS: i32 = CP_SV_ROWS;
pub(crate) const SV_BLOCK: f32 = GRID;

/// 色相条：横贯面板内容宽（26 格），1px 竖条拼出 0..360 的连续彩虹。
pub(crate) const HUE_BARS: i32 = CP_CONTENT_W as i32;

/// 滑块一行里的横向分格（单位 = 格，五段加起来正好是面板的 26 格）：
///   标签 2 格 + 间隔 1 格 + 轨道 17 格 + 间隔 1 格 + 数值 5 格。
/// 轨道从第 CP_TRACK_LINE 条竖网格线起、跨 CP_TRACK_COLS 格 - 它和把手都按网格线站队。
pub(crate) const CP_SLIDER_LABEL_COLS: i32 = 2;
pub(crate) const CP_SLIDER_GAP_COLS: i32 = 1;
/// 数值栏的格数：数值最长 3 位（"255" / "360"），按网格文字模型是 3 x 9 = 27px 宽（一个字符一格），
/// 铁律要求左右各留 1 格（18px），所以数值栏至少 45px - 吸到整数格就是 5 格（46px），正好。
pub(crate) const CP_SLIDER_VALUE_COLS: i32 = 5;
pub(crate) const CP_TRACK_COLS: i32 =
    CP_COLS - CP_SLIDER_LABEL_COLS - CP_SLIDER_VALUE_COLS - 2 * CP_SLIDER_GAP_COLS;
pub(crate) const CP_TRACK_LINE: i32 = CP_SLIDER_LABEL_COLS + CP_SLIDER_GAP_COLS;
pub(crate) const CP_TRACK_W: f32 = cells(CP_TRACK_COLS);
/// 轨道内容宽（去掉左右各 1px 边框）：9 * 轨道格数 - 1 - 1px 渐变竖条铺这么宽。
pub(crate) const CP_TRACK_INNER_W: i32 = 9 * CP_TRACK_COLS - 1;

/// 把手：1 格宽（cells(1) = 10px），在轨道里 1px 1px 地滑（铁律 4：滑块不受 9px 网格约束，
/// 网格管的是「排版对齐」，不是「不许挪一格不到的距离」）。
/// 可走距离 = 轨道宽 - 把手宽：把手左边缘从轨道左边那条线一路滑到右边那条线。
pub(crate) const CP_HANDLE_W: f32 = cells(1);
pub(crate) const CP_HANDLE_TRAVEL: f32 = CP_TRACK_W - CP_HANDLE_W;

/// 色相条把手：同样 1 格宽、贴着竖网格线停；整条色相条横跨 26 格，能停 26 个位置。
pub(crate) const HUE_STEPS: i32 = CP_COLS - 1;

/// 底部两个按钮平分面板宽：各 13 格（cells(13) = 118px），中间那条竖线两半共用。
pub(crate) const CP_HALF_COLS: i32 = CP_COLS / 2;
pub(crate) const CP_HALF_W: f32 = cells(CP_HALF_COLS);
pub(crate) const CP_LEFT_W: f32 = CP_HALF_W;

/// Hex 输入框最多 7 个字符（# 加 6 位十六进制），多出来的字符直接不收。
pub(crate) const HEX_LEN_MAX: usize = 7;

/// Hex 输入框收哪些字符：只要不是控制字符都收。收的时候就按「只认十六进制」去拦，
/// 人就看不出来自己哪儿打错了 - 让非法文本先进来，由 parse_hex 判红边。
pub(crate) fn hex_input_char(c: char) -> bool {
    !c.is_control()
}

/// 面板第一次打开时的位置（左上角）；每帧还要按窗口网格夹一次。
/// 450 = 50*9、99 = 11*9：左上角落在窗口网格线上，面板内部各段也就跟着落在网格线上。
pub(crate) const CP_DEFAULT_X: f32 = 450.0;
pub(crate) const CP_DEFAULT_Y: f32 = 99.0;

// 编译期自检：面板本体、每一段、内容列都站在同一套 9px 栅格上。
const _: () = assert!(CP_W == 235.0 && CP_H == 325.0);
const _: () = assert!((CP_W as i32 - 1) % 9 == 0 && (CP_H as i32 - 1) % 9 == 0);
const _: () = assert!(CP_ROW_BTN + CP_BTN_ROWS == CP_ROWS);
// 内容列 = 26 个内容格 + 25 条内部线格；SV 的 26 列正好铺满它。
const _: () = assert!(CP_CONTENT_W == 26.0 * GRID + 25.0 * LINE_W);
const _: () = assert!(SV_COLS as f32 * SV_BLOCK + (SV_COLS as f32 - 1.0) * LINE_W == CP_CONTENT_W);
const _: () = assert!(
    SV_ROWS as f32 * SV_BLOCK + (SV_ROWS as f32 - 1.0) * LINE_W == cells(CP_SV_ROWS) - 2.0 * LINE_W
);
// 左右两半各 cells(13)，共用中间那条线后正好等于面板宽。
const _: () = assert!((CP_HALF_W as i32 - 1) % 9 == 0);
const _: () = assert!(CP_LEFT_W + CP_HALF_W - LINE_W == CP_W);
// 滑块的横向分格必须正好铺满面板宽；轨道和把手都是整格（尺寸 = cells(n)）。
const _: () = assert!(
    CP_SLIDER_LABEL_COLS + CP_SLIDER_GAP_COLS + CP_TRACK_COLS + CP_SLIDER_GAP_COLS
        + CP_SLIDER_VALUE_COLS
        == CP_COLS
);
const _: () = assert!(CP_TRACK_W > CP_HANDLE_W);
const _: () = assert!((CP_TRACK_W as i32 - 1) % 9 == 0 && (CP_HANDLE_W as i32 - 1) % 9 == 0);
// 可走距离必须是整像素（把手左边缘永远落在整像素列上），而且得走得动。
const _: () = assert!(CP_HANDLE_TRAVEL > 0.0 && CP_HANDLE_TRAVEL % 1.0 == 0.0);
const _: () = assert!(HUE_STEPS > 0);
// 每一段的上沿都落在 9 的倍数上。
const _: () = assert!((CP_Y_SV as i32) % 9 == 0 && (CP_Y_HUE as i32) % 9 == 0);
const _: () = assert!((CP_Y_RGB as i32) % 9 == 0 && (CP_Y_HSV as i32) % 9 == 0);
const _: () = assert!((CP_Y_HEX as i32) % 9 == 0 && (CP_Y_BTN as i32) % 9 == 0);
const _: () = assert!(HUE_BARS as f32 == CP_CONTENT_W);
// 默认位置也得落在窗口网格线上（50*9 / 11*9）。
const _: () = assert!((CP_DEFAULT_X as i32) % 9 == 0 && (CP_DEFAULT_Y as i32) % 9 == 0);

// ---- 颜色换算：全是纯函数，进出都是整数 ----

/// HSV -> RGB：h 是 0..=360 的整数度（360 当 0 用），s / v 是 0..=255 的整数。
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

/// RGB -> HSV：返回 (h 0..=360, s 0..=255, v 0..=255)，全是整数。
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

/// 在一个颜色的底上画「选中框」该用什么色：暗底用白框、亮底用黑框。
/// 判据用亮度（Rec.601 加权：299R + 587G + 114B），阈值取中点 128（除以 1000 比较）。
pub(crate) fn contrast_on(rgb: (u8, u8, u8)) -> egui::Color32 {
    let luma = 299 * rgb.0 as u32 + 587 * rgb.1 as u32 + 114 * rgb.2 as u32;
    if luma < 128 * 1000 {
        theme().text_white
    } else {
        theme().text
    }
}
/// (u8,u8,u8) -> Color32：只在交给 painter 画的时候转一道。
pub(crate) fn rgb32(rgb: (u8, u8, u8)) -> egui::Color32 {
    egui::Color32::from_rgb(rgb.0, rgb.1, rgb.2)
}

/// (u8,u8,u8) -> #RRGGBB 文本（一律大写）。
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
/// 三位简写（#28F）、rgb(34,136,255)、位数不对、前后带空格 - 全都算非法。
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

/// 轨道上第 i 个 1px 竖条对应的整数值：0..max 均分到「轨道内容宽 - 1」段上。
pub(crate) fn bar_value(i: i32, max: i32) -> i32 {
    (i as f32 * max as f32 / (CP_TRACK_INNER_W - 1) as f32).round() as i32
}

/// 把手左边缘落在哪一列像素：轨道起点 + 比例 * 可走距离，落到整像素上（1px 1px 地滑，
/// 不吸网格 - 铁律 4）。轨道起点在竖网格线上，比例 0 / 1 时正好贴两端。
pub(crate) fn handle_left(track_min_x: f32, ratio: f32) -> f32 {
    track_min_x + (ratio.clamp(0.0, 1.0) * CP_HANDLE_TRAVEL).round()
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

/// 色相把手停在色相条的第几个格（0..=HUE_STEPS）：比例吸到最近的格。
pub(crate) fn hue_step(h: u16) -> i32 {
    (h as f32 / 360.0 * HUE_STEPS as f32).round().clamp(0.0, HUE_STEPS as f32) as i32
}

/// 某个格对应的色相角（0..=360）：首尾正好是 0 和 360。
pub(crate) fn hue_of_step(j: i32) -> u16 {
    (j as f32 * 360.0 / HUE_STEPS as f32).round() as u16
}

/// 色相把手（1 格宽 = cells(1) = 10px）的左边缘，相对色相条起点：整数个格 * 9 - 落在网格线上。
pub(crate) fn hue_handle_left(h: u16) -> f32 {
    grid_line(hue_step(h))
}

// ---- 面板状态 ----

/// 面板正在编辑哪一路颜色。
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum ColorTarget {
    Up,
    Down,
}

impl ColorTarget {
    /// 这一路的中文名（预览区色块里那 16px 点阵字也用它）。
    pub(crate) fn name(self) -> &'static str {
        match self {
            ColorTarget::Up => "涨",
            ColorTarget::Down => "跌",
        }
    }
}

/// 面板里正在拖的东西：没在拖 / 拖 SV 调色板 / 拖色相条。
/// （拖面板本身归窗口外壳管，见 ui::floatwin 的 Win::drag_title - 那不是面板内的控件。）
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum ColorDrag {
    None,
    Sv,
    Hue,
}

/// 颜色面板的全部跨帧状态：开关、位置、编辑目标、当前 HSV 与 RGB、两路待定值、
/// Hex 文本与焦点、拖动会话。egui 是即时模式，这些东西不存字段就跨不过一帧。
/// RGB 和 HSV 六个数一起存是有意的：哪边刚被改过就以哪边为准，另一边界立刻跟着算 -
/// 绝不让换算来回倒（倒一遍就会掉 1 个色阶，拖滑块时把手会左右抖）。
#[derive(Clone)]
pub(crate) struct ColorPanel {
    /// 窗口外壳：底色 / 标题栏 / 标题栏下沿那条线 / 1px 外框 / 1px 跟手拖动 / 关闭键，全在里面。
    pub(crate) win: Win,
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
    /// Hex 输入框：通用的自绘单行输入框（文本 / 光标 / 选区 / 输入法都在里面），外加
    /// 「这份文本是否合法」（非法只把边框变红，什么都不应用）。焦点归 egui 记（按 id），
    /// 所以不用再单独存一个 bool - 只有拿到焦点才会收键盘事件。
    pub(crate) hex: TextInput,
    pub(crate) hex_valid: bool,
    /// 正在拖面板里的哪个控件（拖窗口本身归 win 管）。
    pub(crate) drag: ColorDrag,
}

impl ColorPanel {
    pub(crate) fn new() -> Self {
        Self {
            // 颜色面板和别的窗口同一套外壳，只是改了尺寸也没意义 -> 不给缩放（resizable 保持 false）。
            win: Win::new(WIN_ID_COLOR, CP_DEFAULT_X, CP_DEFAULT_Y, CP_W, CP_H),
            target: ColorTarget::Up,
            h: 0,
            s: 0,
            v: 0,
            rgb: (0, 0, 0),
            pending_up: UP_RGB_DEFAULT,
            pending_down: DOWN_RGB_DEFAULT,
            hex: TextInput::new(HEX_LEN_MAX),
            hex_valid: true,
            drag: ColorDrag::None,
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
        self.hex.set_text(&hex_of(rgb));
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
        self.win.open = false;
        self.win.end_sessions();
        self.drag = ColorDrag::None;
    }
}

impl StockApp {
    /// 打开颜色面板：待定值从当前生效的两个颜色取，指定编辑目标，再把目标色灌进编辑状态。
    /// 每次打开都从零起步 - 上一次没点「应用」就关掉的编辑不留痕。
    pub(crate) fn open_color_panel(&mut self, target: ColorTarget) {
        let (up, down) = (self.up_color, self.down_color);
        let cp = &mut self.color_panel;
        cp.win.open = true;
        cp.pending_up = up;
        cp.pending_down = down;
        cp.target = target;
        cp.drag = ColorDrag::None;
        let rgb = cp.pending_for(target);
        cp.load(rgb);
        self.status_msg = format!("颜色面板：正在编辑{}色", target.name());
    }

    /// 涨跌颜色面板：主窗口里的一块自绘浮动面板，每帧最后画（压在窗口里所有东西之上）。
    /// 面板状态整份搬到本地变量上改，最后再写回字段 - 中途要调 self.text(...)，那会借走整个 self。
    /// work = 主窗口的窗口区（标题栏 / 菜单栏 / 状态栏之外的那块）：浮动面板也只能待在里面，
    /// 拖不出窗口区（见 ui() 里传的那个矩形）。
    pub(crate) fn ui_color_panel(&mut self, ui: &mut egui::Ui, work: egui::Rect) {
        let mut cp = self.color_panel.clone();
        if !cp.win.open {
            return;
        }

        // 0) 窗口外壳：底色 + 标题栏（左文字 / 右关闭键 / 下沿那条线）+ 1px 跟手拖动，
        //    全在 ui::floatwin 的 Win 里；它还把整块面板登记成「只吃事件、不画东西」的矩形
        //    （本函数是每帧最后跑的，底下的绘图区、控制面板都抢不到这块地方 -
        //    在面板空白处按下不会把 K 线拖走）。
        let out = cp.win.begin(self, ui, work, "颜色设置", WinKeys::CloseOnly);
        // UI工程测试：颜色面板整块报一笔（它自己左上角那套网格）。
        self.probe("颜色面板", cp.win.rect());
        if out.close {
            cp.close_cancelled();
            self.color_panel = cp;
            self.status_msg = "颜色面板：已取消，颜色没变".to_owned();
            return;
        }
        // 颜色面板不缩放，所以 begin 一定给得出整块面板。
        let Some(panel) = out.panel else {
            return;
        };

        let painter = ui.painter().clone();
        let pointer = ui.input(|i| i.pointer.latest_pos());
        let down = ui.input(|i| i.pointer.primary_down());
        // 开工前记一份颜色：收尾时比一下就知道这一帧有没有控件改过颜色（改过就要重写 Hex 文本）。
        let rgb_at_start = cp.rgb;

        // 2) SV 调色板：26 x 16 个 8x8 色块，块与块之间是共用网格线（默认边框色）；
        //    x 轴 = 饱和度、y 轴 = 明度，每块的颜色 = 当前色相 H 下对应的 HSV 转 RGB。
        let sv = cp_inner(panel, CP_ROW_SV, CP_SV_ROWS);
        for row in 0..SV_ROWS {
            for col in 0..SV_COLS {
                let block = egui::Rect::from_min_size(
                    egui::pos2(
                        sv.min.x + col as f32 * PITCH,
                        sv.min.y + row as f32 * PITCH,
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
        // 色块之间那些共用网格线：26 列共 27 条竖线、16 行共 17 条横线，颜色就是默认边框色。
        let sv_band = cp_band(panel, CP_ROW_SV, CP_SV_ROWS);
        for col in 0..=SV_COLS {
            vline_hard(
                &painter,
                sv.min.x - LINE_W + grid_line(col),
                sv_band.min.y,
                sv_band.max.y,
                theme().ctrl_border,
            );
        }
        for row in 0..=SV_ROWS {
            hline_hard(
                &painter,
                sv_band.min.x,
                sv_band.max.x,
                sv.min.y - LINE_W + grid_line(row),
                theme().ctrl_border,
            );
        }
        let sv_resp = ui.allocate_rect(sv, egui::Sense::click_and_drag());
        if sv_resp.is_pointer_button_down_on() && cp.drag == ColorDrag::None {
            cp.drag = ColorDrag::Sv;
        }
        if cp.drag == ColorDrag::Sv {
            if let Some(p) = pointer {
                // 鼠标压在哪一块上，S / V 就取那一块的整数值：块和数值一一对应，不产生小数。
                let col = (((p.x - sv.min.x) / PITCH).floor() as i32).clamp(0, SV_COLS - 1);
                let row = (((p.y - sv.min.y) / PITCH).floor() as i32).clamp(0, SV_ROWS - 1);
                cp.set_hsv(cp.h, s_of_col(col), v_of_row(row));
            }
        }
        // 当前位置：选中的那一块，四周那圈线按底色反差取色（暗底白、亮底黑，只有 1px）。
        let sel = egui::Rect::from_min_size(
            egui::pos2(
                sv.min.x - LINE_W + col_of_s(cp.s) as f32 * PITCH,
                sv.min.y - LINE_W + row_of_v(cp.v) as f32 * PITCH,
            ),
            egui::vec2(SV_BLOCK + 2.0 * LINE_W, SV_BLOCK + 2.0 * LINE_W),
        );

        // 3) 色相条：横贯面板内容宽（26 格），233 根 1px 竖条拼出 0..360 的连续彩虹。
        //    整条色相条就是一个跨 26 格的控件：左右两条边框落在面板外框上，上下两条落在段间网格线上。
        let hue_band = cp_band(panel, CP_ROW_HUE, CP_HUE_ROWS);
        let hue = hue_band.shrink(LINE_W);
        for i in 0..HUE_BARS {
            let bar = egui::Rect::from_min_size(
                egui::pos2(hue.min.x + i as f32 * LINE_W, hue.min.y),
                egui::vec2(LINE_W, hue.height()),
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
                // 压在哪一格上就取那一格的色相角：26 个格对应 0..360，永远是整数度。
                let step = (((p.x - hue.min.x) / PITCH).floor() as i32).clamp(0, HUE_STEPS);
                cp.set_hsv(hue_of_step(step), cp.s, cp.v);
            }
        }
        // 当前位置：1 格宽 x 2 格高的白底把手（cells(1) x cells(2)）。左 / 右两条边框落在竖网格线上，
        // 上下两条和色相条的段间网格线重合 - 都只有 1px，绝不会叠厚。
        let hue_handle = egui::Rect::from_min_size(
            egui::pos2(hue_band.min.x + hue_handle_left(cp.h), hue_band.min.y),
            egui::vec2(CP_HANDLE_W, hue_band.height()),
        );
        painter.rect_filled(hue_handle, egui::CornerRadius::ZERO, theme().text_white);
        border_hard(&painter, hue_handle, theme().ctrl_border);

        // 4) RGB 三行：轨道渐变 = 从 (0,G,B) 排到 (255,G,B)，G / B 取当前值；
        //    改 R / G / B 就走 set_rgb（HSV 侧跟着重算），Hex 文本在收尾处统一刷新。
        let (mut cr, mut cg, mut cb) = cp.rgb;
        if let Some(x) = self.color_slider(
            ui,
            &painter,
            cp_band(panel, CP_ROW_RGB, CP_SLIDER_ROWS),
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
            cp_band(panel, CP_ROW_RGB + CP_SLIDER_ROWS, CP_SLIDER_ROWS),
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
            cp_band(panel, CP_ROW_RGB + 2 * CP_SLIDER_ROWS, CP_SLIDER_ROWS),
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
            cp_band(panel, CP_ROW_HSV, CP_SLIDER_ROWS),
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
            cp_band(panel, CP_ROW_HSV + CP_SLIDER_ROWS, CP_SLIDER_ROWS),
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
            cp_band(panel, CP_ROW_HSV + 2 * CP_SLIDER_ROWS, CP_SLIDER_ROWS),
            "V",
            &cv.to_string(),
            cv as f32 / 255.0,
            255,
            |i| hsv_to_rgb(ch, cs, bar_value(i, 255) as u8),
        ) {
            cp.set_hsv(ch, cs, x as u8);
        }

        // 6) Hex 输入框：白底 + 1px 硬边；非法输入时边框变红（借主窗口关闭键那个红）。
        //    文字 / 光标 / 选区 / 输入法全交给通用的自绘单行输入框（ui::textbox）- 和源码
        //    编辑器同一套底子，省得再手搓一份。矩形的上下两条边就是段间那条网格线
        //    （CP_ROW_HEX / CP_ROW_BTN），左右两条是面板外框，所以框边由宿主自己画
        //    （style.edge = None）、永远只有 1px，不会和旁边那条线叠成 2px。
        let hex_band = cp_band(panel, CP_ROW_HEX, CP_HEX_ROWS);
        let hex_inner = hex_band.shrink(LINE_W);
        let hex_typed = cp.hex.ui(
            self,
            ui,
            hex_band,
            egui::Id::new("rt_color_hex"),
            &hex_input_char,
            TextInputStyle {
                bg: theme().work_bg,
                edge: None,
            },
        );
        // 文本合法就把 RGB 和 HSV 一起刷新（改 Hex -> 两边都更新）；非法只红边，不应用任何东西。
        match parse_hex(&cp.hex.text()) {
            Some(rgb) => {
                cp.hex_valid = true;
                if hex_typed {
                    cp.set_rgb(rgb);
                }
            }
            None => cp.hex_valid = false,
        }

        // 7) 底部两个按钮：左 13 格 + 中间那条共用的线格、右 13 格，外观走全工程通用的
        //    draw_button - 中间那条竖线落在正中间的网格线上。
        let btn_band = cp_band(panel, CP_ROW_BTN, CP_BTN_ROWS);
        let apply_rect =
            egui::Rect::from_min_size(btn_band.min, egui::vec2(CP_LEFT_W, btn_band.height()));
        let cancel_rect = egui::Rect::from_min_size(
            egui::pos2(btn_band.min.x + CP_HALF_W - LINE_W, btn_band.min.y),
            egui::vec2(CP_HALF_W, btn_band.height()),
        );
        let apply_text = ButtonContent::Text("应用".to_owned());
        if self.draw_button(ui, &painter, apply_rect, &apply_text, false, "") {
            // 应用 = 把当前编辑中的颜色写回它自己那一路（面板一次只编辑一个颜色，由打开它的按钮定），
            // 然后关面板。另一路一个字节都不动。
            // 用户自己定过色就不再「指向系统级配色」：以后切皮肤不动这两个颜色
            // （想再跟回去，点控制面板上的「图表配色跟随系统」）。
            cp.save_pending();
            let rgb = cp.pending_for(cp.target);
            match cp.target {
                ColorTarget::Up => self.up_color = rgb,
                ColorTarget::Down => self.down_color = rgb,
            }
            self.chart_follows_theme = false;
            cp.close_cancelled();
            self.status_msg = format!("已应用：{}色 {}（图表配色改为自定义）", cp.target.name(), hex_of(rgb));
        }
        let cancel_text = ButtonContent::Text("取消".to_owned());
        if self.draw_button(ui, &painter, cancel_rect, &cancel_text, false, "") {
            // 取消 = 放弃修改，什么都不写回（struct 里的还是打开时那两个颜色）。
            cp.close_cancelled();
            self.status_msg = "颜色面板：已取消，颜色没变".to_owned();
        }

        // 8) 段间共用网格线 + 面板外框：排在内容之后画，免得被色块和轨道盖掉。
        //    这 10 条横线就是 11 段的边界（相邻两段共用同一条），每条都只有 1px。
        for row in [
            CP_ROW_SV,
            CP_ROW_HUE,
            CP_ROW_RGB,
            CP_ROW_RGB + CP_SLIDER_ROWS,
            CP_ROW_RGB + 2 * CP_SLIDER_ROWS,
            CP_ROW_HSV,
            CP_ROW_HSV + CP_SLIDER_ROWS,
            CP_ROW_HSV + 2 * CP_SLIDER_ROWS,
            CP_ROW_HEX,
            CP_ROW_BTN,
        ] {
            hline_hard(
                &painter,
                panel.min.x,
                panel.max.x,
                panel.min.y + grid_line(row),
                theme().ctrl_border,
            );
        }
        // 外壳后半：颜色面板不缩放，所以这儿只有标题栏下沿那条线和最外圈那 1px 硬边框。
        cp.win.end(ui);

        // 9) SV 当前位置：选中的那一块，四周那圈线改成对比色（排在段线之后，才盖得住）。
        border_hard(&painter, sel, contrast_on(cp.rgb));

        // 10) Hex 输入框的上下两条边最后画：正好压在段间网格线上（合法就是默认边框色、
        //      非法就是关闭键那个红），只覆盖内容列那一段，不会跟面板外框叠成 2px。
        let hex_edge = if cp.hex_valid { theme().ctrl_border } else { theme().close_hover };
        hline_hard(
            &painter,
            hex_inner.min.x,
            hex_inner.max.x,
            hex_band.min.y,
            hex_edge,
        );
        hline_hard(
            &painter,
            hex_inner.min.x,
            hex_inner.max.x,
            hex_band.max.y - LINE_W,
            hex_edge,
        );

        // 11) 收尾：松手就清掉面板里的拖动会话（拖窗口本身由 Win::update_geometry 按鼠标状态收）。
        //     这一帧有控件改过颜色就把 Hex 文本重写一遍，让 RGB / HSV / Hex 三边始终一致
        //     （Hex 输入框自己改的不算 - 那边以文本为准）。
        if !down {
            cp.drag = ColorDrag::None;
        }
        if cp.rgb != rgb_at_start && !hex_typed {
            cp.hex.set_text(&hex_of(cp.rgb));
            cp.hex_valid = true;
        }
        self.color_panel = cp;
    }

    /// 一行自绘滑块：左标签（2 格）+ 渐变轨道（20 格）+ 把手（1 格）+ 右数值（2 格），横向分格
    /// 加起来正好是面板的 26 格。整行是 cells(2) = 19px、内容高 17px（上下各 1px 是它和邻居
    /// 共用的网格线）。ratio = 当前值在轨道上的比例（0..=1，由整数值算出来）；
    /// grad(i) = 轨道内容里第 i 个 1px 竖条的颜色。返回 Some(整数值) 表示这一帧鼠标正按在轨道上。
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
        // 轨道：从第 CP_TRACK_LINE 条竖网格线起、跨 CP_TRACK_COLS 格 - 左右两条边框落在网格线上。
        let track = egui::Rect::from_min_size(
            egui::pos2(row.min.x + grid_line(CP_TRACK_LINE), row.min.y),
            egui::vec2(CP_TRACK_W, row.height()),
        );
        self.text(
            painter,
            egui::pos2(text_left_x(row.min.x, row.min.x), row.center().y),
            egui::Align2::LEFT_CENTER,
            label,
            theme().text,
        );
        // 数值：最右边那 CP_SLIDER_VALUE_COLS 格，在自己那一栏里居中显示。
        let value_rect = egui::Rect::from_min_size(
            egui::pos2(
                row.min.x + grid_line(CP_TRACK_LINE + CP_TRACK_COLS + CP_SLIDER_GAP_COLS),
                row.min.y,
            ),
            egui::vec2(cells(CP_SLIDER_VALUE_COLS), row.height()),
        );
        self.text(
            painter,
            value_rect.center(),
            egui::Align2::CENTER_CENTER,
            value_text,
            theme().text,
        );

        // 轨道内容：1px 竖条一条一条拼出该通道的渐变（不许平滑填充）。内容高 17px、上下各 1px
        // 就是轨道和它上下邻居共用的网格线。
        let inner = track.shrink(LINE_W);
        for i in 0..CP_TRACK_INNER_W {
            let bar = egui::Rect::from_min_size(
                egui::pos2(inner.min.x + i as f32, inner.min.y),
                egui::vec2(LINE_W, inner.height()),
            );
            painter.rect_filled(bar, egui::CornerRadius::ZERO, rgb32(grad(i)));
        }

        // 拖动：鼠标在轨道上按下就算数（按住不放、拖出轨道也继续跟），按 x 反算把手停在哪个像素 -
        // 1px 1px 地滑（铁律 4），不吸 9px 网格，所以把手能停在任意一列像素上。
        let resp = ui.allocate_rect(track, egui::Sense::click_and_drag());
        let drag_x = if resp.is_pointer_button_down_on() {
            ui.input(|i| i.pointer.latest_pos()).map(|p| {
                (p.x - track.min.x).round().clamp(0.0, CP_HANDLE_TRAVEL)
            })
        } else {
            None
        };

        // 把手左边缘 = 轨道起点 + 比例 * 可走距离（整像素）。正在拖就跟手走，否则按当前值的比例摆。
        let hx = match drag_x {
            Some(x) => track.min.x + x,
            None => handle_left(track.min.x, ratio),
        };
        let handle = egui::Rect::from_min_size(
            egui::pos2(hx, row.min.y),
            egui::vec2(CP_HANDLE_W, row.height()),
        );
        painter.rect_filled(handle, egui::CornerRadius::ZERO, theme().text_white);
        border_hard(painter, handle, theme().ctrl_border);

        // 轨道左右两条竖边框：落在网格线上；把手滑到两端时和它重合（同位即同一条线，不会叠厚）。
        vline_hard(painter, track.min.x, track.min.y, track.max.y, theme().ctrl_border);
        vline_hard(
            painter,
            track.max.x - LINE_W,
            track.min.y,
            track.max.y,
            theme().ctrl_border,
        );

        // 返回值按同一个像素反算：把手停在那一列，通道值就是那一列对应的整数。
        drag_x.map(|x| (x / CP_HANDLE_TRAVEL * max as f32).round() as i32)
    }
}
