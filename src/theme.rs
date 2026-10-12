// ===========================================================================
//  模块 theme：颜色表
// ===========================================================================
//  一律 6 位 #RRGGBB 的硬编码色，文件末尾有断言逐个把关。
//  本文件 = 拆分前那份单文件里的「三、颜色表」。
use crate::indicator;
use eframe::egui;

// ===========================================================================
//  三、颜色表（一律 6 位 #RRGGBB）
// ===========================================================================

// 经典 Win9x 灰色系，全部纯色：无渐变、无圆角、无阴影。
// 每个字节都是「重复的同一个十六进制数字」，严格满足 #aabbcc 规则（见开头铁律第 1 条）。
pub(crate) const C_TITLE_BG: egui::Color32 = egui::Color32::from_rgb(0xCC, 0xCC, 0xCC);
pub(crate) const C_MENU_BG: egui::Color32 = egui::Color32::from_rgb(0xCC, 0xCC, 0xCC);
pub(crate) const C_WORK_BG: egui::Color32 = egui::Color32::from_rgb(0xFF, 0xFF, 0xFF);
pub(crate) const C_STATUS_BG: egui::Color32 = egui::Color32::from_rgb(0xCC, 0xCC, 0xCC);
pub(crate) const C_BTN_FACE: egui::Color32 = egui::Color32::from_rgb(0xCC, 0xCC, 0xCC);
pub(crate) const C_BTN_HOVER: egui::Color32 = egui::Color32::from_rgb(0xEE, 0xEE, 0xEE);
pub(crate) const C_CLOSE_HOVER: egui::Color32 = egui::Color32::from_rgb(0xFF, 0x00, 0x00);
pub(crate) const C_TEXT: egui::Color32 = egui::Color32::from_rgb(0x00, 0x00, 0x00);
pub(crate) const C_TEXT_WHITE: egui::Color32 = egui::Color32::from_rgb(0xFF, 0xFF, 0xFF);
pub(crate) const C_LINE: egui::Color32 = egui::Color32::from_rgb(0x99, 0x99, 0x99);
pub(crate) const C_BTN_EDGE: egui::Color32 = egui::Color32::from_rgb(0x99, 0x99, 0x99);
// 窗口最外圈那 1px 边框的颜色：纯黑（用户要求的默认值，窗口内所有硬边框都取它）。
pub(crate) const C_BORDER: egui::Color32 = egui::Color32::from_rgb(0x00, 0x00, 0x00);

// 图表：绘图区白底、右侧价格轴灰底；A 股习惯是涨红跌绿；刻度参考线用浅灰。
pub(crate) const C_CHART_BG: egui::Color32 = egui::Color32::from_rgb(0xFF, 0xFF, 0xFF);
pub(crate) const C_AXIS_BG: egui::Color32 = egui::Color32::from_rgb(0xCC, 0xCC, 0xCC);
pub(crate) const C_UP: egui::Color32 = egui::Color32::from_rgb(0xFF, 0x00, 0x00);
pub(crate) const C_DOWN: egui::Color32 = egui::Color32::from_rgb(0x00, 0x99, 0x00);
pub(crate) const C_TICK: egui::Color32 = egui::Color32::from_rgb(0xCC, 0xCC, 0xCC);

// 涨跌色的默认值：struct 里的 up_color / down_color 拿它们当初值（红涨绿跌，用户要求的默认）。
// 真正画 K 线用的是那两个字段；C_UP / C_DOWN 从此只当默认值的来源和单元测试的参照色。
pub(crate) const UP_RGB_DEFAULT: (u8, u8, u8) = (C_UP.r(), C_UP.g(), C_UP.b());
pub(crate) const DOWN_RGB_DEFAULT: (u8, u8, u8) = (C_DOWN.r(), C_DOWN.g(), C_DOWN.b());

// 十字光标：线用中灰（白底上够醒目，又和 #999999 的轴边 / 分隔线分得开），
// 标签用浅灰底 + 黑边 + 黑字，和菜单 hover 高亮是同一套灰度。
pub(crate) const C_CROSS: egui::Color32 = egui::Color32::from_rgb(0x66, 0x66, 0x66);
pub(crate) const C_CROSS_LABEL: egui::Color32 = egui::Color32::from_rgb(0xEE, 0xEE, 0xEE);

// 两栏布局与控制面板：分割条灰、面板灰、按钮深灰硬边、按下态更暗。
pub(crate) const C_SPLITTER: egui::Color32 = egui::Color32::from_rgb(0xAA, 0xAA, 0xAA);
pub(crate) const C_PANEL_BG: egui::Color32 = egui::Color32::from_rgb(0xCC, 0xCC, 0xCC);
pub(crate) const C_BTN_BORDER: egui::Color32 = egui::Color32::from_rgb(0x66, 0x66, 0x66);
pub(crate) const C_BTN_PRESS: egui::Color32 = egui::Color32::from_rgb(0xAA, 0xAA, 0xAA);

// 指标区：0 轴那条参考线。DIF / DEA 两条线的颜色归指标模块自己带（见 indicator::DIF_RGB），
// 因为「画什么颜色」是指标的一部分；这边只管画。
pub(crate) const C_IND_ZERO: egui::Color32 = egui::Color32::from_rgb(0x99, 0x99, 0x99);

// 编译期自检：每个字节都必须是 0x00 / 0x11 / 0x22 … 0xFF（都是 17 的整数倍），
// 也就是「两个十六进制数字相同」。以后新加色值若写成 #808080 这类，这里直接报错。
pub(crate) const fn color_is_aabbcc(c: egui::Color32) -> bool {
    c.r() % 17 == 0 && c.g() % 17 == 0 && c.b() % 17 == 0
}
const _: () = assert!(color_is_aabbcc(C_TITLE_BG));
const _: () = assert!(color_is_aabbcc(C_MENU_BG));
const _: () = assert!(color_is_aabbcc(C_WORK_BG));
const _: () = assert!(color_is_aabbcc(C_STATUS_BG));
const _: () = assert!(color_is_aabbcc(C_BTN_FACE));
const _: () = assert!(color_is_aabbcc(C_BTN_HOVER));
const _: () = assert!(color_is_aabbcc(C_CLOSE_HOVER));
const _: () = assert!(color_is_aabbcc(C_TEXT));
const _: () = assert!(color_is_aabbcc(C_TEXT_WHITE));
const _: () = assert!(color_is_aabbcc(C_LINE));
const _: () = assert!(color_is_aabbcc(C_BTN_EDGE));
const _: () = assert!(color_is_aabbcc(C_BORDER));
const _: () = assert!(color_is_aabbcc(C_CHART_BG));
const _: () = assert!(color_is_aabbcc(C_AXIS_BG));
const _: () = assert!(color_is_aabbcc(C_UP));
const _: () = assert!(color_is_aabbcc(C_DOWN));
// 默认涨跌色必须真的是「红涨绿跌」（字段初值，用户点名要的）：写成编译期断言，以后改错立刻知道。
const _: () = assert!(
    UP_RGB_DEFAULT.0 == 0xFF
        && UP_RGB_DEFAULT.1 == 0x00
        && UP_RGB_DEFAULT.2 == 0x00
        && DOWN_RGB_DEFAULT.0 == 0x00
        && DOWN_RGB_DEFAULT.1 == 0x99
        && DOWN_RGB_DEFAULT.2 == 0x00
);
const _: () = assert!(color_is_aabbcc(C_TICK));
const _: () = assert!(color_is_aabbcc(C_CROSS));
const _: () = assert!(color_is_aabbcc(C_CROSS_LABEL));
const _: () = assert!(color_is_aabbcc(C_SPLITTER));
const _: () = assert!(color_is_aabbcc(C_PANEL_BG));
const _: () = assert!(color_is_aabbcc(C_BTN_BORDER));
const _: () = assert!(color_is_aabbcc(C_BTN_PRESS));
const _: () = assert!(color_is_aabbcc(C_IND_ZERO));
// 指标模块自己带的两个线色也过一遍同一条规矩（那边不依赖 egui，所以只能逐通道查）。
const _: () = assert!(
    indicator::DIF_RGB.0 % 17 == 0
        && indicator::DIF_RGB.1 % 17 == 0
        && indicator::DIF_RGB.2 % 17 == 0
        && indicator::DEA_RGB.0 % 17 == 0
        && indicator::DEA_RGB.1 % 17 == 0
        && indicator::DEA_RGB.2 % 17 == 0
);

