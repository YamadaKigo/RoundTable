// ===========================================================================
//  模块 ui::menu：菜单栏与下拉面板（全工程唯一一份）
// ===========================================================================
//  主窗口的菜单栏、源码编辑窗的菜单栏、图表上那些下拉面板（指标 / 叠加）全走这一份，
//  不许再各写一份 - 铁律：同类型的东西必须长一个样（菜单项宽 / 行高 / hover 高亮 /
//  边框 / 与邻居共用的那条 1px 网格线，一个都不能差）。
//
//  菜单栏（ui_menu_bar）：
//    - 左对齐：从区块左边那条网格线起、一项挨一项往右排（不右对齐、也不居中）；
//    - 每项宽 = 文字宽 + 左右各 1 格空档，吸到整数格（铁律：文字左右各留 1 格）；
//    - 相邻两项共用中间那条 1px 网格线（步进 = 宽 - 1px），高亮块碰不到邻居的边框；
//    - 项与项之间（含最后一项右边）一条 1px 实线竖线，落在共用那条网格线上；
//    - 下沿那条 1px 线 = 菜单栏和下一段共用的边界线，最后画（不然会被 hover 高亮啃掉一条）。
//  下拉面板（MenuPanel）：
//    - 上沿压在菜单栏 / 触发控件下沿那条网格线上（两段共用这 1px）；
//    - 每行 2 格（相邻两行共用中间那条网格线），1px 硬边框最后画；
//    - 两种对齐：down（左边和触发它的控件对齐）/ right（右边对齐，底下放不下就翻上去）。
//  下拉面板的统一入口是 ui_menu_popup：挂在锚点下沿、左边对齐、整块夹在当前窗口里、画在
//  窗口最上层（调用方必须排在窗口里所有内容之后调它），点面板外自动收起 - 主窗口和编辑器
//  的菜单栏下拉都只走它这一份。
//  鼠标一离开菜单栏和下拉面板也立刻收起（见 pointer_left）：不用点外面，也不用等下一帧。
use crate::app::*;
use crate::grid::*;
use crate::theme::*;
use eframe::egui;

/// 菜单里的一项：文字 +（可选）子菜单。
/// 没有子菜单的就是「能点的一项」；有子菜单的右边画一个箭头，指着它就会再开一块
///（一级一级往下都能开，见 ui_menu_popup）。
pub(crate) struct MenuNode {
    pub(crate) label: &'static str,
    pub(crate) children: &'static [MenuNode],
}

impl MenuNode {
    /// 能点的一项（没有子菜单）。
    pub(crate) const fn act(label: &'static str) -> Self {
        Self { label, children: &[] }
    }

    /// 带子菜单的一项。
    pub(crate) const fn sub(label: &'static str, children: &'static [MenuNode]) -> Self {
        Self { label, children }
    }

    pub(crate) fn has_children(&self) -> bool {
        !self.children.is_empty()
    }
}

/// 菜单栏的一项：标题 + 展开后要画的那棵树。
pub(crate) struct MenuTab {
    pub(crate) title: &'static str,
    pub(crate) items: &'static [MenuNode],
}

/// 一块菜单栏 / 下拉面板的状态：哪一项开着（None = 都收着）+ 子菜单开到了哪几行。
/// `path` 是「展开着的子菜单链」：从下拉面板那一行开始，每一级一个行下标
///（指针斜着往子菜单里挪的时候，靠它把已经开出来的那几块留住，不闪断）。
/// 菜单都是「同时只开一个」，所以主窗口、编辑器、图表上的菜单共用这一个小状态。
#[derive(Clone, Default)]
pub(crate) struct MenuState {
    pub(crate) open: Option<usize>,
    pub(crate) path: Vec<usize>,
}

/// 子菜单最多往下开几级（含最上面那块下拉面板）：示例树给到三级子菜单，这儿留一级富余。
pub(crate) const MENU_DEPTH_MAX: usize = 4;

// 主窗口菜单栏的那几棵树。动作现在只在状态栏回显（「布局」那两项是真办事的），
// 下面这些子菜单是**演示用**的示例项 - 用来把「项里有子菜单、子菜单里还能再有」这条规矩立住。
const FILE_RECENT_MORE: &[MenuNode] = &[
    MenuNode::act("三级示例 上证50"),
    MenuNode::act("三级示例 沪深300"),
];
const FILE_RECENT: &[MenuNode] = &[
    MenuNode::act("上证50.csv"),
    MenuNode::act("沪深300.csv"),
    MenuNode::sub("更多", FILE_RECENT_MORE),
];
const FILE_ITEMS: &[MenuNode] = &[
    MenuNode::act("新建"),
    MenuNode::act("打开"),
    MenuNode::sub("最近打开", FILE_RECENT),
    MenuNode::act("保存"),
    MenuNode::act("退出"),
];
const DATA_ITEMS: &[MenuNode] = &[
    MenuNode::act("抓取行情"),
    MenuNode::sub(
        "随机游走",
        &[
            MenuNode::act("按默认参数"),
            MenuNode::sub(
                "按参数",
                &[MenuNode::act("长度 512 根"), MenuNode::act("长度 1024 根")],
            ),
        ],
    ),
    MenuNode::act("导入CSV"),
    MenuNode::sub(
        "数据源设置",
        &[MenuNode::act("本地文件"), MenuNode::act("远程接口")],
    ),
];
const ANALYZE_ITEMS: &[MenuNode] = &[
    MenuNode::act("指标设置"),
    MenuNode::sub(
        "画线工具",
        &[
            MenuNode::act("趋势线"),
            MenuNode::act("水平线"),
            MenuNode::act("矩形"),
        ],
    ),
    MenuNode::act("回测"),
];
// 「布局」这几项管的是窗口：关掉了摆回来 / 新开一块（这两项是真办事的）。
const LAYOUT_ITEMS: &[MenuNode] = &[MenuNode::act("控制面板"), MenuNode::act("新增图表")];
const TOOL_ITEMS: &[MenuNode] = &[
    MenuNode::sub(
        "设置",
        &[
            MenuNode::act("界面"),
            MenuNode::sub(
                "配色",
                &[MenuNode::act("经典"), MenuNode::act("暗色")],
            ),
        ],
    ),
    MenuNode::act("日志"),
];
const HELP_ITEMS: &[MenuNode] = &[
    MenuNode::act("关于"),
    MenuNode::sub(
        "帮助主题",
        &[MenuNode::act("快捷键"), MenuNode::act("常见问题")],
    ),
];

/// 主窗口的菜单栏：顺序就是从左往右的顺序。
pub(crate) const TOP_TABS: [MenuTab; 6] = [
    MenuTab { title: "文件", items: FILE_ITEMS },
    MenuTab { title: "数据", items: DATA_ITEMS },
    MenuTab { title: "分析", items: ANALYZE_ITEMS },
    MenuTab { title: "布局", items: LAYOUT_ITEMS },
    MenuTab { title: "工具", items: TOOL_ITEMS },
    MenuTab { title: "帮助", items: HELP_ITEMS },
];

/// 菜单项宽度 = 文字宽 + 左右各 1 格（9px）内边距，向上吸到整数格（最少 3 格）。
pub(crate) fn tab_width(app: &mut StockApp, title: &str) -> f32 {
    snap_cells_up(app.text_width(title) + 2.0 * TEXT_PAD_X).max(cells(3))
}

/// 一块下拉面板的宽：默认 MENU_PANEL_W；哪一行的字比它宽就按那一行加宽（文字左右照样各留
/// 1 格空档，向上吸到整数格）。带子菜单的行右边还要一个箭头，再让出 1 格。
/// 铁律：文字绝不许顶到面板边框上 - 摆不下的那一项就把面板撑宽，而不是溢出边框。
/// 面板宽了照样整块夹在窗口里（见 MenuPanel::bounded / flyout）。
pub(crate) fn menu_panel_width(app: &mut StockApp, level: &[MenuNode]) -> f32 {
    let mut w = MENU_PANEL_W;
    for node in level {
        let arrow = if node.has_children() { PITCH } else { 0.0 };
        w = w.max(snap_cells_up(app.text_width(node.label) + 2.0 * TEXT_PAD_X + arrow));
    }
    w
}

/// 一排菜单项往哪边靠：主窗口 / 编辑器的菜单栏从左边排起；图表栏（Pane 标题栏）那几个
/// 按钮从右边排起（和窗口右边框对齐）。
#[derive(Clone, Copy, PartialEq)]
pub(crate) enum MenuAlign {
    Left,
    Right,
}

/// 一排菜单项的矩形：宽度由调用方按文字量好（tab_width），从 bar 的左边（Left）或右边（Right）
/// 一个挨一个排开。相邻两项共用中间那条网格线（步进 = 宽 - 1px）：各自尺寸照样是 cells(n)，
/// 中间永远只有 1px；返回顺序一律是从左往右。
pub(crate) fn menu_item_rects(bar: egui::Rect, widths: &[f32], align: MenuAlign) -> Vec<egui::Rect> {
    let item = |x: f32, w: f32| {
        egui::Rect::from_min_size(egui::pos2(x, bar.min.y), egui::vec2(w, bar.height()))
    };
    let mut rects = Vec::with_capacity(widths.len());
    match align {
        MenuAlign::Left => {
            let mut x = bar.min.x;
            for &w in widths {
                rects.push(item(x, w));
                x += w - LINE_W;
            }
        }
        MenuAlign::Right => {
            let mut x = bar.max.x;
            for &w in widths.iter().rev() {
                rects.push(item(x - w, w));
                x -= w - LINE_W;
            }
            rects.reverse();
        }
    }
    rects
}

/// 把一段坐标吸到 bounds 自己的网格线上（第 0 条 = bounds.min.x / min.y），再夹进
/// [lo, hi] 之间的整格位置。菜单面板一律过它 - 铁律：边框只许落在网格线里，落在格子中间
/// 就会和旁边的线叠成 2px。lo 必须落在网格线上（调用方给的窗口 / 区域左上角都是）。
fn snap_into(v: f32, lo: f32, hi: f32) -> f32 {
    let hi = hi.max(lo);
    let steps = ((hi - lo) / PITCH).floor();
    let k = ((v - lo) / PITCH).round().clamp(0.0, steps);
    lo + k * PITCH
}

/// 一块下拉面板的几何：位置 + 行数。行高 / 行距只有这一份定义，所有下拉面板都用它。
#[derive(Clone, Copy)]
pub(crate) struct MenuPanel {
    pub(crate) rect: egui::Rect,
    pub(crate) rows: usize,
}

impl MenuPanel {

    /// 挂在某个控件旁边、右边对齐：默认向下开（上沿压在控件下沿那条网格线上），
    /// 底下放不下就向上开（下沿压在控件上沿那条线上）- 边界同样只有 1px。
    pub(crate) fn right(anchor: egui::Rect, w: f32, rows: usize, window_bottom: f32) -> Self {
        let h = cells(MENU_ROW_ROWS * rows as i32);
        let x = anchor.max.x - w;
        let below = anchor.max.y - LINE_W;
        let y = if below + h > window_bottom {
            anchor.min.y + LINE_W - h
        } else {
            below
        };
        Self {
            rect: egui::Rect::from_min_size(egui::pos2(x, y), egui::vec2(w, h)),
            rows,
        }
    }

    /// 挂在锚点下沿、左边对齐（和触发它的菜单项左沿齐平），整块夹进 bounds 里：
    /// 右边放不下就向左挪、下边放不下就向上翻 - 面板永远不越出当前窗口的边。
    /// 这是菜单面板唯一的摆法：主窗口和编辑器菜单栏的下拉都走它（左对齐、不溢出）。
    pub(crate) fn bounded(anchor: egui::Rect, w: f32, rows: usize, bounds: egui::Rect) -> Self {
        let h = cells(MENU_ROW_ROWS * rows as i32);
        let x = snap_into(anchor.min.x, bounds.min.x, bounds.max.x - LINE_W - w);
        let below = anchor.max.y - LINE_W;
        let y_want = if below + h <= bounds.max.y - LINE_W {
            below
        } else {
            anchor.min.y + LINE_W - h
        };
        let y = snap_into(y_want, bounds.min.y, bounds.max.y - LINE_W - h);
        Self {
            rect: egui::Rect::from_min_size(egui::pos2(x, y), egui::vec2(w, h)),
            rows,
        }
    }

    /// 第 j 行：行高 2 格（相邻两行共用中间那条网格线，步进 = grid_line(2) = 18px）。
    pub(crate) fn row(&self, j: usize) -> egui::Rect {
        egui::Rect::from_min_size(
            egui::pos2(self.rect.min.x, self.rect.min.y + grid_line(j as i32 * MENU_ROW_ROWS)),
            egui::vec2(self.rect.width(), MENU_ROW_H),
        )
    }

    /// 子菜单那一块：挂在这一行（row）的右边 - 左沿落在两块共用的那条线上、上沿和这一行对齐；
    /// 右边放不下就翻到左边，整块夹进 bounds 里。行高 / 行距和父面板一模一样。
    pub(crate) fn flyout(row: egui::Rect, w: f32, rows: usize, bounds: egui::Rect) -> Self {
        let h = cells(MENU_ROW_ROWS * rows as i32);
        // 右开：左沿 = 这一行右边那条线（两块共用这条线）；右边放不下就左开（右沿压在父行左沿上）。
        let right_open = row.max.x - LINE_W;
        let x_want = if right_open + w <= bounds.max.x - LINE_W {
            right_open
        } else {
            row.min.x - w + LINE_W
        };
        // 上下和这一行对齐，再整块夹进 bounds（放不下就贴着上 / 下沿，绝不出窗口）。
        let y_want = row.min.y - LINE_W;
        Self {
            rect: egui::Rect::from_min_size(
                egui::pos2(
                    snap_into(x_want, bounds.min.x, (bounds.max.x - LINE_W - w).max(bounds.min.x)),
                    snap_into(y_want, bounds.min.y, (bounds.max.y - LINE_W - h).max(bounds.min.y)),
                ),
                egui::vec2(w, h),
            ),
            rows,
        }
    }

    /// 铺面板底色。
    pub(crate) fn paint_bg(&self, painter: &egui::Painter) {
        painter.rect_filled(self.rect, egui::CornerRadius::ZERO, theme().menu_bg);
    }

    /// 一条条目的 hover 高亮。
    pub(crate) fn paint_row_hover(&self, painter: &egui::Painter, row: egui::Rect) {
        painter.rect_filled(row, egui::CornerRadius::ZERO, theme().btn_hover);
    }

    /// 1px 硬边框：必须排在最后画（先画的话会被 hover 高亮和行底色啃掉一条像素）。
    pub(crate) fn paint_border(&self, painter: &egui::Painter) {
        border_hard(painter, self.rect, theme().ctrl_border);
    }

    /// 下拉面板的骨架：铺底 -> 逐行登记可点区域（悬停就高亮）-> 把这一行交给 draw_row 画内容 ->
    /// 1px 硬边压在最后。返回被点中的行下标（没点就是 None）。
    /// 三处下拉面板（菜单栏、指标切换、叠加）都走这一份 - 行高 / 高亮 / 边框 / 点击处理
    /// 只有这一处实现。项与项之间不画横线：一行就是一行，靠 hover 高亮分隔。
    pub(crate) fn ui_rows(
        &self,
        app: &mut StockApp,
        ui: &mut egui::Ui,
        painter: &egui::Painter,
        mut draw_row: impl FnMut(&mut StockApp, &egui::Painter, usize, egui::Rect),
    ) -> Option<usize> {
        self.paint_bg(painter);
        let mut clicked = None;
        for j in 0..self.rows {
            let row = self.row(j);
            app.probe(&format!("菜单项 {j}"), row);
            let resp = interact_rect(ui, row, egui::Sense::click());
            if resp.hovered() {
                self.paint_row_hover(painter, row);
            }
            draw_row(app, painter, j, row);
            if resp.clicked() {
                clicked = Some(j);
            }
        }
        self.paint_border(painter);
        clicked
    }
}

/// 一条菜单栏的规矩：同样的一排项，主窗口的菜单栏和图表栏有几处不一样。
#[derive(Clone, Copy)]
pub(crate) struct MenuBarStyle {
    /// hover 到别的项就换项（主窗口那种经典菜单栏：展开着一个的时候，
    /// 指着另一个就把它换出来）。图表栏不开这个 - 它的按钮是「工具」，
    /// 鼠标从按钮上滑过不许把展开着的菜单换掉（不然就成了「闪一下就没了」）。
    pub(crate) hover_switch: bool,
    /// 最左那一项左边也画一条竖线（图表栏：把左边那段文字和按钮隔开）。
    pub(crate) lead_divider: bool,
}

impl MenuBarStyle {
    /// 主窗口 / 编辑器那种经典菜单栏。
    pub(crate) const CLASSIC: Self = Self {
        hover_switch: true,
        lead_divider: false,
    };
    /// 图表栏（Pane 标题栏）：按钮往右靠、左边一段文字，只在点上换项。
    pub(crate) const PANE_BAR: Self = Self {
        hover_switch: false,
        lead_divider: true,
    };
}

/// 一排菜单项这一帧交回来的东西。
pub(crate) struct MenuBarOut {
    /// 每项的矩形（从左往右；下拉面板 / 菜单拿它当锚点）。
    pub(crate) rects: Vec<egui::Rect>,
    /// 这一帧被点中的项（None = 没点）。
    pub(crate) clicked: Option<usize>,
    /// 点 / hover 之后的展开状态（hover 换项）：None = 都收着。
    pub(crate) open: Option<usize>,
}

/// 菜单栏骨架：底色 -> 左端那段纯文字（图表栏的指标名；主窗口菜单栏没有）-> 逐项
///（点击开合 / hover 换项 / hover 高亮）-> 项间那条竖线 -> 下沿那条 1px 线。
/// 主窗口菜单栏、图表栏（Pane 标题栏）全走这一份 - 铁律：同类型的东西必须长一个样
///（底色 / 项宽 / 高亮 / 边线 / 开合与 hover 换项的规矩，全工程只有这一处实现）。
/// 只画菜单栏这一条，不管下拉面板：下拉面板要画在**当前窗口的最上层**，只能由调用方
/// 排在窗口里所有别的内容之后调 ui_menu_popup（见下）。
/// label 是左端那段文字（靠左排的那个内容格起）；项不管左右对齐，都跟它井水不犯河水。
/// style 见 MenuBarStyle：主窗口那种经典菜单栏 hover 就换项，图表栏只在点上换项。
pub(crate) fn menu_bar_ui(
    app: &mut StockApp,
    ui: &mut egui::Ui,
    bar: egui::Rect,
    titles: &[&str],
    align: MenuAlign,
    mut open: Option<usize>,
    label: Option<&str>,
    style: MenuBarStyle,
) -> MenuBarOut {
    let painter = ui.painter().clone();
    let widths: Vec<f32> = titles.iter().map(|t| tab_width(app, t)).collect();
    let rects = menu_item_rects(bar, &widths, align);
    // 菜单栏底色 = 皮肤里的菜单栏色。主窗口 / 图表栏都走这一份，
    // 所以它们一定同色（都是菜单栏，就不该长得不一样）。
    painter.rect_filled(bar, egui::CornerRadius::ZERO, theme().menu_bg);
    // 左端那段文字：落到内容格上（左边留 1 格空档），和其它区块的左对齐文字同一个口径。
    if let Some(text) = label {
        app.text(
            &painter,
            egui::pos2(text_left_x(bar.min.x, bar.min.x), bar.center().y),
            egui::Align2::LEFT_CENTER,
            text,
            theme().text,
        );
    }
    let mut hovered = vec![false; titles.len()];
    let mut clicked = None;
    for (i, rect) in rects.iter().enumerate() {
        // UI工程测试：菜单栏上的每一项也算一块元素（按菜单栏自己那套网格判对齐）。
        app.probe(titles[i], *rect);
        let resp = interact_rect(ui, *rect, egui::Sense::click());
        hovered[i] = resp.hovered();
        if resp.clicked() {
            clicked = Some(i);
            open = if open == Some(i) { None } else { Some(i) };
        } else if style.hover_switch && hovered[i] && open.is_some_and(|cur| cur != i) {
            open = Some(i);
        }
    }

    for (i, title) in titles.iter().enumerate() {
        if hovered[i] || open == Some(i) {
            painter.rect_filled(rects[i], egui::CornerRadius::ZERO, theme().btn_hover);
        }
        app.text(
            &painter,
            rects[i].center(),
            egui::Align2::CENTER_CENTER,
            title,
            theme().text,
        );
    }
    // 相邻两项之间一条实线竖线（最后一项右边那条也算），落在它俩共用的那条网格线上，
    // 上下各让开 1px 边框线。排在 hover 高亮之后画，鼠标压上去也看得见
    //（这是「项」的界线，不是高亮的一部分）。
    if style.lead_divider {
        // 最左那一项左边也来一条：图表栏靠它把左边那段文字和这排按钮隔开
        //（主窗口菜单栏最左那项贴着窗口边，那条线就是外框，不用再画）。
        vline_hard(
            &painter,
            rects[0].min.x,
            bar.min.y + LINE_W,
            bar.max.y - LINE_W,
            theme().line,
        );
    }
    for rect in &rects {
        vline_hard(
            &painter,
            rect.max.x - LINE_W,
            bar.min.y + LINE_W,
            bar.max.y - LINE_W,
            theme().line,
        );
    }
    // 下沿那条 1px 线：菜单栏和下一段共用的那条网格线，最后画。
    hline_hard(
        &painter,
        bar.min.x,
        bar.max.x,
        bar.max.y - LINE_W,
        theme().ctrl_border,
    );

    MenuBarOut { rects, clicked, open }
}

/// 菜单栏（主窗口 / 编辑器那一排下拉项）：按 tabs 的文字量左对齐排开，
/// 返回这一帧打开着的项下标和它的锚点矩形（没开就是 None），以及更新后的状态。
pub(crate) fn ui_menu_bar(
    app: &mut StockApp,
    ui: &mut egui::Ui,
    bar: egui::Rect,
    tabs: &[MenuTab],
    state: MenuState,
) -> (Option<(usize, egui::Rect)>, MenuState) {
    let titles: Vec<&str> = tabs.iter().map(|t| t.title).collect();
    let out = menu_bar_ui(
        app,
        ui,
        bar,
        &titles,
        MenuAlign::Left,
        state.open,
        None,
        MenuBarStyle::CLASSIC,
    );
    let switched = out.open != state.open;
    (
        out.open.map(|i| (i, out.rects[i])),
        MenuState {
            open: out.open,
            // 换了一项（或者收起来了）就把子菜单那条链清掉：新开的那一项不该带着上一项的链。
            path: if switched { Vec::new() } else { state.path },
        },
    )
}

/// 一整块菜单的下拉面板：挂在锚点下沿、左对齐、整块夹进 bounds 里（不越出当前窗口），
/// **必须排在窗口里所有别的内容之后调它** - 它就是当前窗口的最上层，盖得住菜单栏 / 按钮栏 /
/// 正文 / 分隔线（弹出菜单不会透出底下的东西）。点面板和 keep 以外的地方就把菜单收起来。
/// 子菜单：指针压在有子菜单的那一行上，就在那一行右边（放不下就翻到左边）再开一块，
/// 一级一级往下都能开（MENU_DEPTH_MAX 封顶；示例树给到三级）。
/// 返回这一帧被点中的**叶子**项的文字（没点就是 None）；点中叶子顺手把整块菜单收起来。
/// 主窗口和编辑器的菜单栏共用这一份 - 摆法 / 层次 / 子菜单 / 点外收起只有这一处实现。
pub(crate) fn ui_menu_popup(
    app: &mut StockApp,
    ui: &mut egui::Ui,
    anchor: egui::Rect,
    items: &'static [MenuNode],
    state: &mut MenuState,
    bounds: egui::Rect,
    keep: &[egui::Rect],
) -> Option<&'static str> {
    if state.open.is_none() {
        state.path.clear();
        return None;
    }
    // 1) 先把这一帧要画的几块面板算出来（链式：指针压在哪一行的子菜单上就再往下开一级）。
    //    指针不在某块面板里时，那一级沿用上一帧开着的行（state.path）-
    //    不然斜着往子菜单里挪的那一下会闪断。
    let pointer = ui.ctx().input(|i| i.pointer.hover_pos());
    let mut chain: Vec<(MenuPanel, &'static [MenuNode])> = Vec::new();
    let mut path: Vec<usize> = Vec::new();
    let mut row_anchor = anchor;
    let mut level = items;
    while !level.is_empty() && chain.len() < MENU_DEPTH_MAX {
        // 这一层的宽按这一层最宽的那一项量：字长了就把面板撑宽，绝不溢出边框。
        let w = menu_panel_width(app, level);
        let panel = if chain.is_empty() {
            MenuPanel::bounded(anchor, w, level.len(), bounds)
        } else {
            MenuPanel::flyout(row_anchor, w, level.len(), bounds)
        };
        let hovered = pointer.and_then(|p| (0..level.len()).find(|&j| panel.row(j).contains(p)));
        let next = match hovered {
            // 压在有子菜单的那一行上：把它的子菜单开出来。
            Some(j) if level[j].has_children() => Some(j),
            // 压在这一块里、但压的是没子菜单的行：子菜单收起来。
            Some(_) => None,
            // 指针不在这一块里：沿用上一帧开着的那个（指针正往子菜单里挪）。
            None => state
                .path
                .get(chain.len())
                .copied()
                .filter(|&j| j < level.len() && level[j].has_children()),
        };
        chain.push((panel, level));
        match next {
            Some(j) => {
                path.push(j);
                row_anchor = panel.row(j);
                level = level[j].children;
            }
            None => break,
        }
    }
    let mut keep_rects = keep.to_vec();
    keep_rects.extend(chain.iter().map(|(p, _)| p.rect));
    close_on_outside_click(ui, &keep_rects, state);
    // 鼠标一离开菜单栏（和这几块下拉面板）就把下拉收回去：不用点外面，也不用等下一帧。
    if state.open.is_some() && pointer_left(ui, &keep_rects) {
        state.open = None;
    }
    if state.open.is_none() {
        state.path.clear();
        return None;
    }
    // 2) 从外往里逐块画：文字 + 有子菜单的那一行右边一个箭头；点中的行记下来。
    let mut clicked: Option<(usize, usize)> = None;
    for (k, (panel, level)) in chain.iter().enumerate() {
        let painter = ui.painter().clone();
        let hit = panel.ui_rows(app, ui, &painter, |app, painter, j, row| {
            let node = &level[j];
            // 文字的吸附原点 = 这块下拉面板自己那套网格（面板左边那条线）。
            app.text(
                painter,
                egui::pos2(text_left_x(row.min.x, panel.rect.min.x), row.center().y),
                egui::Align2::LEFT_CENTER,
                node.label,
                theme().text,
            );
            if node.has_children() {
                paint_submenu_arrow(painter, row);
            }
        });
        if let Some(j) = hit {
            clicked = Some((k, j));
        }
    }
    let Some((k, j)) = clicked else {
        state.path = path;
        return None;
    };
    let node = &chain[k].1[j];
    if node.has_children() {
        // 点父项：把它的子菜单开出来（path 收到这一级），整块菜单不收起 - 下一帧就画出来了。
        path.truncate(k + 1);
        state.path = path;
        return None;
    }
    // 点中叶子项：办事（调用方看名字）+ 收菜单。
    state.open = None;
    state.path.clear();
    Some(node.label)
}

/// 带子菜单那一行右边的箭头：一个 3x5 的右向小三角（整像素硬边，和别的自绘符号一套）。
/// 贴在行的右沿、右边留 1 格空档，纵向取行正中。
fn paint_submenu_arrow(painter: &egui::Painter, row: egui::Rect) {
    let x = (row.max.x - PITCH + LINE_W).floor();
    let y = (row.center().y - 2.5).floor();
    for (i, w) in [1.0f32, 2.0, 3.0, 2.0, 1.0].into_iter().enumerate() {
        painter.rect_filled(
            egui::Rect::from_min_size(egui::pos2(x, y + i as f32), egui::vec2(w, LINE_W)),
            egui::CornerRadius::ZERO,
            theme().text,
        );
    }
}

/// 鼠标是不是已经不在 keep 里那几块上了（指针整个离开窗口也算）：菜单栏的下拉靠它
/// 「移出即收回」。指针完全不存在（没有鼠标的合成帧）时不算离开，免得没人碰它自己就关了。
pub(crate) fn pointer_left(ui: &egui::Ui, keep: &[egui::Rect]) -> bool {
    ui.ctx().input(|i| match i.pointer.hover_pos() {
        Some(p) => !keep.iter().any(|r| r.contains(p)),
        None => i.pointer.latest_pos().is_some(),
    })
}

/// 点「菜单栏和下拉面板以外」的地方就把菜单收起来。
pub(crate) fn close_on_outside_click(ui: &egui::Ui, keep: &[egui::Rect], state: &mut MenuState) {
    if state.open.is_none() {
        return;
    }
    let outside = ui.ctx().input(|i| {
        i.pointer.primary_clicked()
            && i.pointer
                .interact_pos()
                .is_some_and(|p| !keep.iter().any(|r| r.contains(p)))
    });
    if outside {
        state.open = None;
    }
}
