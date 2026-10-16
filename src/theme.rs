// ===========================================================================
//  模块 theme：系统级配色表（皮肤 / 主题）
// ===========================================================================
//  全工程的颜色只允许从这儿取：界面代码一律走 theme()（当前皮肤），别处不许再写字面色值。
//  一套皮肤 = 一张「系统级配色表」（struct Theme）：UI 里所有元素的颜色都在里面，一个不落 -
//  以后要做皮肤自定义管理工具，改的就是这张表（每一项管哪儿见字段上的注释）。
//
//  当前提供两套：
//    - 经典（CLASSIC）：Win9x 灰色系，白底黑字，全部纯色；
//    - 暗色（DARK）   ：大智慧那种暗色风格，黑 / 灰底 + 红色线框（#CC0000）+ 白字，
//                        K 线涨色不变（红），跌色改成青色（#00FFFF）。
//  「图表区的颜色」是自定义的：这里把它做成「可指向系统级配色」- 见 Theme 里的图表区 / 涨跌
//  那几项；界面按系统级配色走（切皮肤就整体变），用户也可以在颜色面板里单独改 K 线涨跌色。
//
//  切换皮肤：set_theme(Theme)（面板上的「切换主题」按钮走 StockApp::switch_theme）。
//  图表区的 K 线涨跌色默认「指向系统级配色」（StockApp::chart_follows_theme = true），
//  切皮肤就跟着换；用户在颜色面板里自己定过色之后指向断开，想跟回去点「图表配色跟随系统」
//  （StockApp::follow_system_chart_colors）。
//  界面每帧都从 theme() 读色，切完下一帧整屏就变了。
//
// 【色值规矩（铁律）】每通道必须是「两个相同的十六进制数字」（0x00 / 0x11 / ... / 0xFF，
//  也就是 17 的整数倍）- 文件末尾对两套皮肤逐项做编译期自检，写错格式直接编译不过。
use eframe::egui;
use std::cell::Cell;

// ===========================================================================
//  一、经典皮肤的色值常量（也是「经典」那张表的定义；单元测试拿它们当参照色）
// ===========================================================================

// 经典 Win9x 灰色系，全部纯色：无渐变、无圆角、无阴影。
// 每个字节都是「重复的同一个十六进制数字」，严格满足 #aabbcc 规则（见开头铁律）。
pub(crate) const C_TITLE_BG: egui::Color32 = egui::Color32::from_rgb(0xCC, 0xCC, 0xCC);
pub(crate) const C_MENU_BG: egui::Color32 = egui::Color32::from_rgb(0xCC, 0xCC, 0xCC);
pub(crate) const C_WORK_BG: egui::Color32 = egui::Color32::from_rgb(0xFF, 0xFF, 0xFF);
// 主窗口的「桌面」底色（功能区那块底，几块子窗口浮在它上面）：取 Windows 常见窗口底色的灰度。
// #F0F0F0 不满足「每通道是同一个十六进制数字重复两遍」这条铁律，吸到最接近的 #EEEEEE。
pub(crate) const C_DESKTOP_BG: egui::Color32 = egui::Color32::from_rgb(0xEE, 0xEE, 0xEE);
pub(crate) const C_STATUS_BG: egui::Color32 = egui::Color32::from_rgb(0xCC, 0xCC, 0xCC);
pub(crate) const C_BTN_FACE: egui::Color32 = egui::Color32::from_rgb(0xCC, 0xCC, 0xCC);
pub(crate) const C_BTN_HOVER: egui::Color32 = egui::Color32::from_rgb(0xEE, 0xEE, 0xEE);
pub(crate) const C_CLOSE_HOVER: egui::Color32 = egui::Color32::from_rgb(0xFF, 0x00, 0x00);
pub(crate) const C_TEXT: egui::Color32 = egui::Color32::from_rgb(0x00, 0x00, 0x00);
pub(crate) const C_TEXT_WHITE: egui::Color32 = egui::Color32::from_rgb(0xFF, 0xFF, 0xFF);
pub(crate) const C_LINE: egui::Color32 = egui::Color32::from_rgb(0x99, 0x99, 0x99);
// 所有控件边框的默认色：统一 #999999（默认值；以后每个控件再各自配自己的边框色）。
// 边框只落在网格线格里，所以全界面任何一条边框都只有 1px，相邻控件共用同一条线。
pub(crate) const C_CTRL_BORDER: egui::Color32 = egui::Color32::from_rgb(0x99, 0x99, 0x99);
// UI 工程测试叠层（右栏「UI工程测试」按钮打开）：把 9px 网格线整个铺满窗口，用来核对各控件的
// 边框有没有落在网格线上 - 对齐的边框会被它盖住，没对齐的会露在它旁边。
// 网格线是灰色点状线（每 2px 一个点，见 grid 的 hline_dotted / vline_dotted）：和边框
//（经典皮肤 #999999 / 暗色皮肤 #CC0000）都分得开，铺在画面上也不抢眼。
pub(crate) const C_GRID_TEST: egui::Color32 = egui::Color32::from_rgb(0x66, 0x66, 0x66);
// UI 工程测试工具的三色（对齐 / 没对齐 / 鼠标格）：两套皮肤同色 - 调试叠层不跟着皮肤走。
pub(crate) const C_GRID_OK: egui::Color32 = egui::Color32::from_rgb(0x00, 0xAA, 0x00);
pub(crate) const C_GRID_BAD: egui::Color32 = egui::Color32::from_rgb(0xFF, 0x88, 0x00);
pub(crate) const C_GRID_CURSOR: egui::Color32 = egui::Color32::from_rgb(0x00, 0xCC, 0xCC);

// 图表：绘图区白底、右侧价格轴灰底；A 股习惯是涨红跌绿；刻度参考线用浅灰。
pub(crate) const C_CHART_BG: egui::Color32 = egui::Color32::from_rgb(0xFF, 0xFF, 0xFF);
pub(crate) const C_AXIS_BG: egui::Color32 = egui::Color32::from_rgb(0xCC, 0xCC, 0xCC);
pub(crate) const C_UP: egui::Color32 = egui::Color32::from_rgb(0xFF, 0x00, 0x00);
pub(crate) const C_DOWN: egui::Color32 = egui::Color32::from_rgb(0x00, 0x99, 0x00);
pub(crate) const C_TICK: egui::Color32 = egui::Color32::from_rgb(0xCC, 0xCC, 0xCC);

// 涨跌色的默认值：struct 里的 up_color / down_color 拿它们当初值（红涨绿跌，默认）。
// 真正画 K 线用的是那两个字段；C_UP / C_DOWN 从此只当默认值的来源和单元测试的参照色。
pub(crate) const UP_RGB_DEFAULT: (u8, u8, u8) = (C_UP.r(), C_UP.g(), C_UP.b());
pub(crate) const DOWN_RGB_DEFAULT: (u8, u8, u8) = (C_DOWN.r(), C_DOWN.g(), C_DOWN.b());

// 十字光标：线用中灰（白底上够醒目，又和 #999999 的轴边 / 分隔线分得开），
// 标签用浅灰底 + 黑边 + 黑字，和菜单 hover 高亮是同一套灰度。
pub(crate) const C_CROSS: egui::Color32 = egui::Color32::from_rgb(0x66, 0x66, 0x66);
pub(crate) const C_CROSS_LABEL: egui::Color32 = egui::Color32::from_rgb(0xEE, 0xEE, 0xEE);

// 两栏布局与控制面板：分割条灰、面板灰、按钮深灰硬边、按下态更暗。
pub(crate) const C_SPLITTER: egui::Color32 = egui::Color32::from_rgb(0xAA, 0xAA, 0xAA);
// 滚动条：槽和把手必须分得开（把手亮一档，一眼看得出滚到哪儿了）。
pub(crate) const C_SCROLL_TRACK: egui::Color32 = egui::Color32::from_rgb(0xAA, 0xAA, 0xAA);
pub(crate) const C_SCROLL_THUMB: egui::Color32 = egui::Color32::from_rgb(0xCC, 0xCC, 0xCC);
pub(crate) const C_PANEL_BG: egui::Color32 = egui::Color32::from_rgb(0xCC, 0xCC, 0xCC);
pub(crate) const C_BTN_PRESS: egui::Color32 = egui::Color32::from_rgb(0xAA, 0xAA, 0xAA);
// 文本编辑器里选区的高亮底色：浅灰，黑字压在上面照样看得清，又不会和控件边框的 #999999 混。
pub(crate) const C_SEL_BG: egui::Color32 = egui::Color32::from_rgb(0xCC, 0xCC, 0xCC);

// 源码编辑器里的语法着色（用户点名要的几个色）：关键字（let / fn / impl 等）蓝 #0000FF，
// 注释绿 #009900，字符串品红 #FF00FF；其余代码文字（变量名 / 数字 / 标点等）走默认文字色。
// 三个值都是「同一个十六进制数字重复两遍」，满足 #aabbcc 色值铁律。
pub(crate) const C_SYN_CODE: egui::Color32 = egui::Color32::from_rgb(0x00, 0x00, 0xFF);
pub(crate) const C_SYN_COMMENT: egui::Color32 = egui::Color32::from_rgb(0x00, 0x99, 0x00);
pub(crate) const C_SYN_STRING: egui::Color32 = egui::Color32::from_rgb(0xFF, 0x00, 0xFF);

// 指标区：0 轴那条参考线。DIF / DEA 两条线的颜色归这套皮肤统一管（见 Theme::dif / dea）。
pub(crate) const C_IND_ZERO: egui::Color32 = egui::Color32::from_rgb(0x99, 0x99, 0x99);

// MACD 两条线的经典配色（蓝 / 橙）。和 theme 里经典那张表的 dif / dea 同值。
pub(crate) const C_DIF: egui::Color32 = egui::Color32::from_rgb(0x33, 0x33, 0xCC);
pub(crate) const C_DEA: egui::Color32 = egui::Color32::from_rgb(0xCC, 0x99, 0x00);

// K 线上的叠加均线（MA / EMA 各 6 条，周期见 indicator::MA_PERIODS）：
// 经典皮肤是白底，所以走深色的 6 个色相；暗色皮肤是黑底，走亮色的那套。
/// 经典皮肤的 MA5 / MA10 / MA20 / MA60 / MA120 / MA250。
pub(crate) const C_MA_CLASSIC: [egui::Color32; 6] = [
    egui::Color32::from_rgb(0x33, 0x33, 0xCC),
    egui::Color32::from_rgb(0xCC, 0x00, 0x66),
    egui::Color32::from_rgb(0x00, 0x99, 0x33),
    egui::Color32::from_rgb(0x66, 0x00, 0xCC),
    egui::Color32::from_rgb(0x00, 0x99, 0x99),
    egui::Color32::from_rgb(0x66, 0x66, 0x66),
];
/// 经典皮肤的 EMA5 / EMA10 / EMA20 / EMA60 / EMA120 / EMA250。
pub(crate) const C_EMA_CLASSIC: [egui::Color32; 6] = [
    egui::Color32::from_rgb(0xFF, 0x00, 0xFF),
    egui::Color32::from_rgb(0xFF, 0x88, 0x00),
    egui::Color32::from_rgb(0x00, 0xCC, 0xCC),
    egui::Color32::from_rgb(0x88, 0xAA, 0x00),
    egui::Color32::from_rgb(0x88, 0x88, 0xFF),
    egui::Color32::from_rgb(0xAA, 0x00, 0xAA),
];
/// 暗色皮肤的 MA5 / MA10 / MA20 / MA60 / MA120 / MA250（传统那套：白 / 黄 / 紫 / 绿 / 青 / 灰）。
pub(crate) const C_MA_DARK: [egui::Color32; 6] = [
    egui::Color32::from_rgb(0xFF, 0xFF, 0xFF),
    egui::Color32::from_rgb(0xFF, 0xFF, 0x00),
    egui::Color32::from_rgb(0xFF, 0x00, 0xFF),
    egui::Color32::from_rgb(0x00, 0xFF, 0x00),
    egui::Color32::from_rgb(0x00, 0xFF, 0xFF),
    egui::Color32::from_rgb(0x88, 0x88, 0x88),
];
/// 暗色皮肤的 EMA5 / EMA10 / EMA20 / EMA60 / EMA120 / EMA250（和 MA 那套拉开色相）。
pub(crate) const C_EMA_DARK: [egui::Color32; 6] = [
    egui::Color32::from_rgb(0xFF, 0x88, 0x00),
    egui::Color32::from_rgb(0x88, 0xAA, 0xFF),
    egui::Color32::from_rgb(0xAA, 0xFF, 0x00),
    egui::Color32::from_rgb(0x00, 0xFF, 0x88),
    egui::Color32::from_rgb(0xFF, 0x00, 0xAA),
    egui::Color32::from_rgb(0xCC, 0xCC, 0xCC),
];

// 数字 / 文字涨跌色：红涨绿跌，和 K 线那套分开（暗色皮肤下 K 线跌色变青，这里不变）。
pub(crate) const C_TEXT_UP: egui::Color32 = egui::Color32::from_rgb(0xFF, 0x00, 0x00);
pub(crate) const C_TEXT_DOWN: egui::Color32 = egui::Color32::from_rgb(0x00, 0x99, 0x00);

// KDJ / RSI 那几条线的经典配色（白底上用深色那套：蓝 / 橙 / 品红）。
pub(crate) const C_KDJ_K: egui::Color32 = egui::Color32::from_rgb(0x33, 0x33, 0xCC);
pub(crate) const C_KDJ_D: egui::Color32 = egui::Color32::from_rgb(0xCC, 0x99, 0x00);
pub(crate) const C_KDJ_J: egui::Color32 = egui::Color32::from_rgb(0xFF, 0x00, 0xFF);
pub(crate) const C_RSI_1: egui::Color32 = egui::Color32::from_rgb(0x33, 0x33, 0xCC);
pub(crate) const C_RSI_2: egui::Color32 = egui::Color32::from_rgb(0xCC, 0x99, 0x00);
pub(crate) const C_RSI_3: egui::Color32 = egui::Color32::from_rgb(0xFF, 0x00, 0xFF);

// ===========================================================================
//  二、系统级配色表：一套皮肤的全部颜色
// ===========================================================================

/// 皮肤编号。按钮每点一次在几套之间轮换。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum ThemeId {
    Classic,
    Dark,
}

/// 一张系统级配色表 = 一套皮肤。字段顺序按「区域底色 -> 控件 -> 文字 -> 图表区 -> 涨跌 -> 调试」。
/// 想做皮肤自定义工具，就是给这张表的每个字段配一个编辑控件。
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) struct Theme {
    /// 自己是谁（按钮 / 状态栏回显用）。
    pub(crate) id: ThemeId,

    // ---- 区域底色 ----
    /// 标题栏 / Pane 标题栏 / 浮动窗标题栏。
    pub(crate) title_bg: egui::Color32,
    /// 菜单栏 / 下拉菜单面板。
    pub(crate) menu_bg: egui::Color32,
    /// 功能区底色（图表工作区、浮动面板本体）。
    pub(crate) work_bg: egui::Color32,
    /// 主窗口的默认底色（功能区那块「桌面」）：图表子窗口 / 按钮面板 / 浮动窗都浮在它上面。
    pub(crate) desktop_bg: egui::Color32,
    /// 状态栏。
    pub(crate) status_bg: egui::Color32,
    /// 右栏控制面板 / 源码窗行号区 / 下拉面板的行号区这类「面板底」。
    pub(crate) panel_bg: egui::Color32,
    /// 分割条 / 横向拖拽条本体。
    pub(crate) splitter: egui::Color32,
    /// 滚动条的槽（把手在里面滑的那条轨道）。
    pub(crate) scroll_track: egui::Color32,
    /// 滚动条的把手：比槽亮一档，滚到哪儿一眼看得出来。
    pub(crate) scroll_thumb: egui::Color32,

    // ---- 控件 ----
    /// 按钮常态底色。
    pub(crate) btn_face: egui::Color32,
    /// 按钮悬停底色。
    pub(crate) btn_hover: egui::Color32,
    /// 按钮按下底色。
    pub(crate) btn_press: egui::Color32,
    /// 文本编辑器里选区的高亮底色（文字颜色不变，所以要挑一个两种文字色都压得住的色）。
    pub(crate) sel_bg: egui::Color32,
    /// 关闭键悬停底色。
    pub(crate) close_hover: egui::Color32,
    /// 所有控件 / 窗口的 1px 线框色。
    pub(crate) ctrl_border: egui::Color32,
    /// 区域分隔线 / 状态栏与状态带里的点状虚线。
    pub(crate) line: egui::Color32,

    // ---- 文字 ----
    /// 默认文字色。
    pub(crate) text: egui::Color32,
    /// 彩底上的反白文字（关闭键的叉、色相把手）。
    pub(crate) text_white: egui::Color32,
    /// 源码编辑器里「关键字」的颜色（let / fn / impl 等，见 ui::editor 的着色）。
    /// 变量名 / 标点 / 数字这些不属于任何一类的代码文字走默认的 text 色。
    pub(crate) syn_code: egui::Color32,
    /// 源码编辑器里「注释」的颜色。
    pub(crate) syn_comment: egui::Color32,
    /// 源码编辑器里「字符串」的颜色。
    pub(crate) syn_string: egui::Color32,

    // ---- 图表区（可指向系统级配色；用户也可以在颜色面板里单独改 K 线涨跌色）----
    /// 绘图区底色。
    pub(crate) chart_bg: egui::Color32,
    /// 右侧纵轴底色。
    pub(crate) axis_bg: egui::Color32,
    /// 图表刻度参考线。
    pub(crate) tick: egui::Color32,
    /// 十字光标线。
    pub(crate) cross: egui::Color32,
    /// 十字光标数值 / 索引标签的底色（边框用 ctrl_border）。
    pub(crate) cross_label: egui::Color32,
    /// 指标 0 轴参考线。
    pub(crate) ind_zero: egui::Color32,

    // ---- 涨跌（K 线 / 柱子 与 数字 / 文字 两套，分开；都指向系统级配色）----
    /// K 线 / 柱子涨色。
    pub(crate) candle_up: egui::Color32,
    /// K 线 / 柱子跌色。
    pub(crate) candle_down: egui::Color32,
    /// 数字 / 文字涨色。
    pub(crate) text_up: egui::Color32,
    /// 数字 / 文字跌色。
    pub(crate) text_down: egui::Color32,
    /// MACD 的 DIF 线。
    pub(crate) dif: egui::Color32,
    /// MACD 的 DEA 线。
    pub(crate) dea: egui::Color32,
    /// KDJ 的 K / D / J 三条线。
    pub(crate) kdj_k: egui::Color32,
    pub(crate) kdj_d: egui::Color32,
    pub(crate) kdj_j: egui::Color32,
    /// RSI 的三条线（周期见 indicator::RSI_PERIODS）。
    pub(crate) rsi_1: egui::Color32,
    pub(crate) rsi_2: egui::Color32,
    pub(crate) rsi_3: egui::Color32,
    /// K 线上的叠加均线：MA / EMA 各 6 条（周期见 indicator::MA_PERIODS）。
    pub(crate) ma: [egui::Color32; 6],
    pub(crate) ema: [egui::Color32; 6],

    // ---- 调试叠层（UI工程测试工具用：见 ui::grid 那一套）----
    /// 9px UI工程测试线。
    pub(crate) grid_test: egui::Color32,
    /// UI工程测试：这块元素四条边都落在网格线上（对齐了）。
    pub(crate) grid_ok: egui::Color32,
    /// UI工程测试：这块元素有边没落在网格线上（要盯的就是它）。
    pub(crate) grid_bad: egui::Color32,
    /// UI工程测试：鼠标底下那一格 / 光标位置标记。
    pub(crate) grid_cursor: egui::Color32,
}

impl Theme {
    /// 经典（Win9x 灰）皮肤。
    pub(crate) const CLASSIC: Self = Self {
        id: ThemeId::Classic,
        title_bg: C_TITLE_BG,
        menu_bg: C_MENU_BG,
        work_bg: C_WORK_BG,
        desktop_bg: C_DESKTOP_BG,
        status_bg: C_STATUS_BG,
        panel_bg: C_PANEL_BG,
        splitter: C_SPLITTER,
        scroll_track: C_SCROLL_TRACK,
        scroll_thumb: C_SCROLL_THUMB,
        btn_face: C_BTN_FACE,
        btn_hover: C_BTN_HOVER,
        btn_press: C_BTN_PRESS,
        sel_bg: C_SEL_BG,
        close_hover: C_CLOSE_HOVER,
        ctrl_border: C_CTRL_BORDER,
        line: C_LINE,
        text: C_TEXT,
        text_white: C_TEXT_WHITE,
        syn_code: C_SYN_CODE,
        syn_comment: C_SYN_COMMENT,
        syn_string: C_SYN_STRING,
        chart_bg: C_CHART_BG,
        axis_bg: C_AXIS_BG,
        tick: C_TICK,
        cross: C_CROSS,
        cross_label: C_CROSS_LABEL,
        ind_zero: C_IND_ZERO,
        candle_up: C_UP,
        candle_down: C_DOWN,
        text_up: C_TEXT_UP,
        text_down: C_TEXT_DOWN,
        dif: C_DIF,
        dea: C_DEA,
        kdj_k: C_KDJ_K,
        kdj_d: C_KDJ_D,
        kdj_j: C_KDJ_J,
        rsi_1: C_RSI_1,
        rsi_2: C_RSI_2,
        rsi_3: C_RSI_3,
        ma: C_MA_CLASSIC,
        ema: C_EMA_CLASSIC,
        grid_test: C_GRID_TEST,
        grid_ok: C_GRID_OK,
        grid_bad: C_GRID_BAD,
        grid_cursor: C_GRID_CURSOR,
    };

    /// 暗色（大智慧那种）皮肤：黑 / 灰底、红色线框（#CC0000）、白字；
    /// K 线涨色不变（红），跌色改成青色（#00FFFF）；数字涨跌还是红涨绿跌。
    pub(crate) const DARK: Self = Self {
        id: ThemeId::Dark,
        title_bg: egui::Color32::from_rgb(0x33, 0x33, 0x33),
        menu_bg: egui::Color32::from_rgb(0x22, 0x22, 0x22),
        work_bg: egui::Color32::from_rgb(0x00, 0x00, 0x00),
        desktop_bg: egui::Color32::from_rgb(0x11, 0x11, 0x11),
        status_bg: egui::Color32::from_rgb(0x33, 0x33, 0x33),
        panel_bg: egui::Color32::from_rgb(0x22, 0x22, 0x22),
        splitter: egui::Color32::from_rgb(0x44, 0x44, 0x44),
        // 滚动条：槽用分割条那档灰，把手亮一档（#444444 的槽配 #444444 的把手是看不见的）。
        scroll_track: egui::Color32::from_rgb(0x44, 0x44, 0x44),
        scroll_thumb: egui::Color32::from_rgb(0x66, 0x66, 0x66),
        btn_face: egui::Color32::from_rgb(0x44, 0x44, 0x44),
        btn_hover: egui::Color32::from_rgb(0x55, 0x55, 0x55),
        btn_press: egui::Color32::from_rgb(0x33, 0x33, 0x33),
        // 选区：中灰，白字压在上面看得清（黑底上不刺眼，也和红边框分得开）。
        sel_bg: egui::Color32::from_rgb(0x66, 0x66, 0x66),
        close_hover: egui::Color32::from_rgb(0xCC, 0x00, 0x00),
        // 线框一律红色（用户指定）。
        ctrl_border: egui::Color32::from_rgb(0xCC, 0x00, 0x00),
        line: egui::Color32::from_rgb(0xCC, 0x00, 0x00),
        text: egui::Color32::from_rgb(0xFF, 0xFF, 0xFF),
        text_white: egui::Color32::from_rgb(0xFF, 0xFF, 0xFF),
        // 语法着色：黑底上把关键字 / 字符串提亮一档（名字蓝 / 字符串品红）；
        // 注释绿两套皮肤同色（用户点名 #009900），暗底上也照样看得清。
        syn_code: egui::Color32::from_rgb(0x88, 0x88, 0xFF),
        syn_comment: C_SYN_COMMENT,
        syn_string: egui::Color32::from_rgb(0xFF, 0x88, 0xFF),
        chart_bg: egui::Color32::from_rgb(0x00, 0x00, 0x00),
        axis_bg: egui::Color32::from_rgb(0x22, 0x22, 0x22),
        // 刻度参考线在黑底上用暗灰，才不至于糊一片红。
        tick: egui::Color32::from_rgb(0x33, 0x33, 0x33),
        cross: egui::Color32::from_rgb(0xCC, 0xCC, 0xCC),
        cross_label: egui::Color32::from_rgb(0x33, 0x33, 0x33),
        ind_zero: egui::Color32::from_rgb(0x66, 0x66, 0x66),
        candle_up: egui::Color32::from_rgb(0xFF, 0x00, 0x00),
        candle_down: egui::Color32::from_rgb(0x00, 0xFF, 0xFF),
        text_up: egui::Color32::from_rgb(0xFF, 0x00, 0x00),
        text_down: egui::Color32::from_rgb(0x00, 0x99, 0x00),
        dif: egui::Color32::from_rgb(0x33, 0x99, 0xFF),
        dea: egui::Color32::from_rgb(0xFF, 0xCC, 0x00),
        kdj_k: egui::Color32::from_rgb(0xFF, 0xFF, 0xFF),
        kdj_d: egui::Color32::from_rgb(0xFF, 0xFF, 0x00),
        kdj_j: egui::Color32::from_rgb(0xFF, 0x00, 0xFF),
        rsi_1: egui::Color32::from_rgb(0xFF, 0xFF, 0xFF),
        rsi_2: egui::Color32::from_rgb(0xFF, 0xFF, 0x00),
        rsi_3: egui::Color32::from_rgb(0xFF, 0x00, 0xFF),
        ma: C_MA_DARK,
        ema: C_EMA_DARK,
        grid_test: C_GRID_TEST,
        grid_ok: C_GRID_OK,
        grid_bad: C_GRID_BAD,
        grid_cursor: C_GRID_CURSOR,
    };

    /// 按编号取皮肤。
    pub(crate) const fn of(id: ThemeId) -> Self {
        match id {
            ThemeId::Classic => Self::CLASSIC,
            ThemeId::Dark => Self::DARK,
        }
    }

    /// 皮肤名字（按钮 / 状态栏回显）。
    pub(crate) fn name(self) -> &'static str {
        match self.id {
            ThemeId::Classic => "经典",
            ThemeId::Dark => "暗色",
        }
    }

    /// 下一套皮肤（按钮每点一次换一套）。
    pub(crate) fn next(self) -> Self {
        Self::of(match self.id {
            ThemeId::Classic => ThemeId::Dark,
            ThemeId::Dark => ThemeId::Classic,
        })
    }

    /// 这张表里的所有色值（编译期自检查每一套皮肤用）。
    pub(crate) const fn colors(&self) -> [egui::Color32; 55] {
        [
            self.title_bg, self.menu_bg, self.work_bg, self.desktop_bg, self.status_bg, self.panel_bg,
            self.splitter, self.scroll_track, self.scroll_thumb,
            self.btn_face, self.btn_hover, self.btn_press, self.sel_bg,
            self.close_hover,
            self.ctrl_border, self.line, self.text, self.text_white, self.syn_code,
            self.syn_comment, self.syn_string, self.chart_bg,
            self.axis_bg, self.tick, self.cross, self.cross_label, self.ind_zero,
            self.candle_up, self.candle_down, self.text_up, self.text_down, self.dif,
            self.dea, self.kdj_k, self.kdj_d, self.kdj_j, self.rsi_1, self.rsi_2,
            self.rsi_3, self.ma[0], self.ma[1], self.ma[2], self.ma[3], self.ma[4],
            self.ma[5], self.ema[0], self.ema[1], self.ema[2], self.ema[3], self.ema[4],
            self.ema[5], self.grid_test, self.grid_ok, self.grid_bad, self.grid_cursor,
        ]
    }
}

// ===========================================================================
//  三、当前皮肤（全工程取色的唯一入口）
// ===========================================================================

thread_local! {
    /// 当前皮肤。egui 是单线程的，且界面每帧都从这儿取色，所以放线程局部变量最省事。
    static CURRENT: Cell<Theme> = const { Cell::new(Theme::CLASSIC) };
}

/// 当前皮肤。界面里所有颜色都从它取（`theme().text` 之类）。
pub(crate) fn theme() -> Theme {
    CURRENT.with(|c| c.get())
}

/// 换皮肤。
pub(crate) fn set_theme(t: Theme) {
    CURRENT.with(|c| c.set(t));
}


/// Color32 -> 0xRRGGBB（交给指标层当 u32 色值用）。
pub(crate) const fn rgb32_u32(c: egui::Color32) -> u32 {
    ((c.r() as u32) << 16) | ((c.g() as u32) << 8) | c.b() as u32
}

/// 编译期自检：每个字节都必须是 0x00 / 0x11 / 0x22 ... 0xFF（都是 17 的整数倍），
/// 也就是「两个十六进制数字相同」。以后新加色值若写成 #808080 这类，这里直接报错。
pub(crate) const fn color_is_aabbcc(c: egui::Color32) -> bool {
    c.r() % 17 == 0 && c.g() % 17 == 0 && c.b() % 17 == 0
}

/// 一套皮肤的所有颜色都得过 #aabbcc 这条规矩。
const fn theme_is_aabbcc(t: Theme) -> bool {
    let cs = t.colors();
    let mut i = 0;
    while i < cs.len() {
        if !color_is_aabbcc(cs[i]) {
            return false;
        }
        i += 1;
    }
    true
}
const _: () = assert!(theme_is_aabbcc(Theme::CLASSIC));
const _: () = assert!(theme_is_aabbcc(Theme::DARK));

// 默认涨跌色必须真的是「红涨绿跌」（字段初值，点名要的）：写成编译期断言，以后改错立刻知道。
const _: () = assert!(
    UP_RGB_DEFAULT.0 == 0xFF
        && UP_RGB_DEFAULT.1 == 0x00
        && UP_RGB_DEFAULT.2 == 0x00
        && DOWN_RGB_DEFAULT.0 == 0x00
        && DOWN_RGB_DEFAULT.1 == 0x99
        && DOWN_RGB_DEFAULT.2 == 0x00
);
// 暗色皮肤的关键色值：红色线框、白字、K 线跌色变青。
const _: () = assert!(Theme::DARK.ctrl_border.r() == 0xCC && Theme::DARK.ctrl_border.g() == 0x00 && Theme::DARK.ctrl_border.b() == 0x00);
const _: () = assert!(Theme::DARK.text.r() == 0xFF && Theme::DARK.text.g() == 0xFF && Theme::DARK.text.b() == 0xFF);
const _: () = assert!(Theme::DARK.candle_down.r() == 0x00 && Theme::DARK.candle_down.g() == 0xFF && Theme::DARK.candle_down.b() == 0xFF);
const _: () = assert!(Theme::DARK.candle_up.r() == 0xFF && Theme::DARK.candle_up.g() == 0x00 && Theme::DARK.candle_up.b() == 0x00);
