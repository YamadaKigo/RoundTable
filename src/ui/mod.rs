// ===========================================================================
//  模块 ui：界面层
// ===========================================================================
//  每块界面一个文件，全部自绘、尺寸一律走 grid 里那套 cells() / grid_line()。
//
//  【UI 设计逻辑 - 改界面之前先读这条】界面是由一个个「控件区域」拼起来的：标题栏 / 菜单栏 /
//  快捷按钮栏 / 正文区 / 状态栏 / 面板里的每一段 ...，每一个控件区域的外圈都必须有横竖线划分，
//  线一律 1px、落在 9px 网格线里，绝不叠成 2px（铁律 7 / 9，见 main.rs 开头）。上下相邻的两段
//  共用中间那条 1px 网格线：各自的底色往里让开这条线，分隔线排在所有内容之后统一补画一遍，
//  谁都盖不掉（样板：ui::editor 的 paint_separators、ui::floatwin 的 end）。
//  跨窗口复用的「零件」各自单独成文件（同类型的东西全工程只有一份实现）：
//    icon      像素图标（16x16 PNG 读进来再自绘）
//    button    按钮（文字 / 图标 + 悬浮便签）
//    menu      菜单栏 + 下拉面板（主窗口、编辑器、图表上那些菜单共用）
//    floatwin  窗口外壳（窗口区里的子窗口 + 颜色面板共用；源码编辑窗也走它）
//    winmgr    窗口区（桌面）的窗口管理器：层级 / 自由位置尺寸 / 最小化 / 新增堆叠
//              窗口区里的每一块子窗口都归它管：图表窗 / 控制面板 / 源码编辑窗（同级）
//    textbuf   文本缓冲（行 + 光标 + 选区 + 撤销栈）
//    textbox   单行输入框
//    editor    多行文本编辑器（菜单栏 + 按钮栏 + 行号区 + 代码区）
pub(crate) mod button;
pub(crate) mod chart;
pub(crate) mod colorpanel;
pub(crate) mod editor;
pub(crate) mod floatwin;
pub(crate) mod icon;
pub(crate) mod menu;
pub(crate) mod overlaymenu;
pub(crate) mod panel;
pub(crate) mod panemenu;
pub(crate) mod source;
pub(crate) mod textbox;
pub(crate) mod textbuf;
pub(crate) mod titlebar;
pub(crate) mod winmgr;
