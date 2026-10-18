use crate::app::*;
use crate::grid::*;
use crate::ui::menu::*;
use eframe::egui;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(crate) enum ContextMenuKind {
    MainBlank,
    Editor,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum ContextAction {
    DemoOne,
    DemoTwo,
    AddChart,
    SwitchTheme,
    Cut,
    Copy,
    Paste,
    SelectAll,
    Undo,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct OpenContextMenu {
    pub(crate) kind: ContextMenuKind,
    pub(crate) anchor: egui::Pos2,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct ContextMenuState {
    open: Option<OpenContextMenu>,
    menu: MenuState,
}

impl ContextMenuState {
    pub(crate) fn open_main(&mut self, anchor: egui::Pos2) {
        self.open = Some(OpenContextMenu {
            kind: ContextMenuKind::MainBlank,
            anchor,
        });
        self.menu = MenuState {
            open: Some(0),
            path: Vec::new(),
        };
    }

    pub(crate) fn open_editor(&mut self, anchor: egui::Pos2) {
        self.open = Some(OpenContextMenu {
            kind: ContextMenuKind::Editor,
            anchor,
        });
        self.menu = MenuState {
            open: Some(0),
            path: Vec::new(),
        };
    }

    pub(crate) fn close(&mut self) {
        self.open = None;
        self.menu = MenuState::default();
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn open(&self) -> Option<OpenContextMenu> {
        self.open
    }
}

const MAIN_DEMO_ITEMS: &[MenuNode] = &[MenuNode::act("演示按钮一"), MenuNode::act("演示按钮二")];
const MAIN_ITEMS: &[MenuNode] = &[
    MenuNode::act("新增图表"),
    MenuNode::act("切换主题"),
    MenuNode::sub("演示项", MAIN_DEMO_ITEMS),
];

const EDITOR_ITEMS: &[MenuNode] = &[
    MenuNode::act("剪切"),
    MenuNode::act("复制"),
    MenuNode::act("粘贴"),
    MenuNode::act("全选"),
    MenuNode::act("撤销"),
];

fn items(kind: ContextMenuKind) -> &'static [MenuNode] {
    match kind {
        ContextMenuKind::MainBlank => MAIN_ITEMS,
        ContextMenuKind::Editor => EDITOR_ITEMS,
    }
}

fn action_of(label: &str) -> Option<ContextAction> {
    Some(match label {
        "新增图表" => ContextAction::AddChart,
        "切换主题" => ContextAction::SwitchTheme,
        "演示按钮一" => ContextAction::DemoOne,
        "演示按钮二" => ContextAction::DemoTwo,
        "剪切" => ContextAction::Cut,
        "复制" => ContextAction::Copy,
        "粘贴" => ContextAction::Paste,
        "全选" => ContextAction::SelectAll,
        "撤销" => ContextAction::Undo,
        _ => return None,
    })
}

pub(crate) fn menu_rect(
    app: &mut StockApp,
    kind: ContextMenuKind,
    anchor: egui::Pos2,
    layout: egui::Rect,
) -> egui::Rect {
    let rows = items(kind);
    let panel = MenuPanel::bounded(
        egui::Rect::from_min_size(anchor, egui::Vec2::ZERO),
        menu_panel_width(app, rows),
        rows.len(),
        layout,
    );
    panel.rect
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn item_rect(panel: egui::Rect, i: usize) -> egui::Rect {
    egui::Rect::from_min_size(
        egui::pos2(panel.min.x, panel.min.y + grid_line(i as i32 * TEXT_ROWS)),
        egui::vec2(panel.width(), cells(TEXT_ROWS)),
    )
}

pub(crate) fn ui_context_menu(
    app: &mut StockApp,
    ui: &mut egui::Ui,
    layout: egui::Rect,
) -> Option<ContextAction> {
    let menu = app.context_menu.open?;
    if ui.ctx().input(|i| i.key_pressed(egui::Key::Escape)) {
        app.context_menu.close();
        return None;
    }

    let rows = items(menu.kind);

    let anchor = egui::Rect::from_min_size(menu.anchor, egui::Vec2::ZERO);
    let keep = [egui::Rect::from_center_size(
        menu.anchor,
        egui::vec2(LINE_W, LINE_W),
    )];
    let mut state = app.context_menu.menu.clone();
    let picked = ui_menu_popup_persistent(app, ui, anchor, rows, &mut state, layout, &keep);
    if state.open.is_some() {
        app.context_menu.menu = state;
    } else {
        app.context_menu.close();
    }
    picked.and_then(action_of)
}
