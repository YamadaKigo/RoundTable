use crate::application::app::*;
use crate::domain::layout::*;
use crate::engine::kit;
use crate::ui::menu::*;

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
    OpenConsole,
    Cut,
    Copy,
    Paste,
    SelectAll,
    Undo,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct OpenContextMenu {
    pub(crate) kind: ContextMenuKind,
    pub(crate) anchor: kit::Pos2,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct ContextMenuState {
    open: Option<OpenContextMenu>,
    menu: MenuState,
}

impl ContextMenuState {
    pub(crate) fn open_main(&mut self, anchor: kit::Pos2) {
        self.open = Some(OpenContextMenu {
            kind: ContextMenuKind::MainBlank,
            anchor,
        });
        self.menu = MenuState {
            open: Some(0),
            path: Vec::new(),
        };
    }

    pub(crate) fn open_editor(&mut self, anchor: kit::Pos2) {
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

    pub(crate) fn open(&self) -> Option<OpenContextMenu> {
        self.open
    }
}

const MAIN_DEMO_ITEMS: &[MenuNode] = &[
    MenuNode::act("Demo Action One"),
    MenuNode::act("Demo Action Two"),
];
const MAIN_ITEMS: &[MenuNode] = &[
    MenuNode::act("New Chart"),
    MenuNode::act("OpenDebug Console"),
    MenuNode::sub("Demo Actions", MAIN_DEMO_ITEMS),
];

const EDITOR_ITEMS: &[MenuNode] = &[
    MenuNode::act("Cut"),
    MenuNode::act("Copy"),
    MenuNode::act("Paste"),
    MenuNode::act("Select All"),
    MenuNode::act("Undo"),
];

fn items(kind: ContextMenuKind) -> &'static [MenuNode] {
    match kind {
        ContextMenuKind::MainBlank => MAIN_ITEMS,
        ContextMenuKind::Editor => EDITOR_ITEMS,
    }
}

fn action_of(label: &str) -> Option<ContextAction> {
    Some(match label {
        "New Chart" => ContextAction::AddChart,
        "OpenDebug Console" => ContextAction::OpenConsole,
        "Demo Action One" => ContextAction::DemoOne,
        "Demo Action Two" => ContextAction::DemoTwo,
        "Cut" => ContextAction::Cut,
        "Copy" => ContextAction::Copy,
        "Paste" => ContextAction::Paste,
        "Select All" => ContextAction::SelectAll,
        "Undo" => ContextAction::Undo,
        _ => return None,
    })
}

pub(crate) fn menu_rect(
    app: &mut AppState,
    kind: ContextMenuKind,
    anchor: kit::Pos2,
    layout: kit::Rect,
) -> kit::Rect {
    let rows = items(kind);
    let panel = MenuPanel::bounded(
        kit::Rect::from_min_size(anchor, kit::Vec2::ZERO),
        menu_panel_width(app, rows),
        rows.len(),
        layout,
    );
    panel.rect
}

#[cfg(test)]
pub(crate) fn item_rect(panel: kit::Rect, i: usize) -> kit::Rect {
    kit::Rect::from_min_size(
        kit::pos2(panel.min.x, panel.min.y + grid_line(i as i32 * TEXT_ROWS)),
        kit::vec2(panel.width(), cells(TEXT_ROWS)),
    )
}

pub(crate) fn ui_context_menu(
    app: &mut AppState,
    ui: &mut kit::Ui,
    layout: kit::Rect,
) -> Option<ContextAction> {
    let menu = app.shell.context_menu.open?;
    if ui.ctx().input(|i| i.key_pressed(kit::Key::Escape)) {
        app.shell.context_menu.close();
        return None;
    }

    let rows = items(menu.kind);

    let anchor = kit::Rect::from_min_size(menu.anchor, kit::Vec2::ZERO);
    let keep = [kit::Rect::from_center_size(
        menu.anchor,
        kit::vec2(LINE_W, LINE_W),
    )];
    let mut state = app.shell.context_menu.menu.clone();
    let picked = ui_menu_popup_persistent(app, ui, anchor, rows, &mut state, layout, &keep);
    if state.open.is_some() {
        app.shell.context_menu.menu = state;
    } else {
        app.shell.context_menu.close();
    }
    picked.and_then(action_of)
}
