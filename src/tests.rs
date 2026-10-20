#![allow(
    clippy::assertions_on_constants,
    clippy::field_reassign_with_default,
    clippy::len_zero,
    clippy::mem_replace_with_default,
    clippy::needless_range_loop,
    clippy::unnecessary_cast
)]

use super::*;

use crate::ui::button::*;
use crate::ui::editor::*;
use crate::ui::icon::*;
use crate::ui::menu::*;
use crate::ui::panel::*;
use crate::ui::textbox::*;
use crate::ui::textbuf::*;

use crate::theme::*;
use crate::ui::chart::*;
use crate::ui::colorpanel::*;
use crate::ui::contextmenu::*;
use crate::ui::floatwin::*;
use crate::ui::source::*;
use crate::ui::titlebar::*;
use crate::ui::winmgr::*;

use crate::font::*;

const CANVAS_W: usize = 40;
const CANVAS_H: usize = 120;

const LEFT: f32 = 20.0;
const BODY_TOP: f32 = 40.5;
const BODY_BOTTOM: f32 = 50.5;
const WICK_TOP: f32 = 20.5;
const WICK_BOTTOM: f32 = 70.5;

const WICK_ROW: usize = 30;
const BODY_ROW: usize = 45;

fn render_candle(scale: u32, rising: bool) -> Vec<Vec<Option<egui::Color32>>> {
    let ctx = egui::Context::default();
    configure_tessellation(&ctx);
    configure_zero_spacing(&ctx);
    ctx.set_pixels_per_point(1.0);

    set_theme(Theme::CLASSIC);

    let mut raw = egui::RawInput::default();
    raw.screen_rect = Some(egui::Rect::from_min_size(
        egui::Pos2::ZERO,
        egui::vec2(CANVAS_W as f32, CANVAS_H as f32),
    ));

    let bar_w = snap_scale(scale) as f32;
    let color = if rising { C_UP } else { C_DOWN };
    let mut out = ctx.run_ui(raw, |ui| {
        paint_candle(
            ui.painter(),
            LEFT,
            bar_w,
            rising,
            color,
            BODY_TOP,
            BODY_BOTTOM,
            WICK_TOP,
            WICK_BOTTOM,
        );
    });

    let shapes = std::mem::take(&mut out.shapes);
    absorb_textures(&mut out.textures_delta);
    rasterize(&ctx.tessellate(shapes, 1.0))
}

fn edge(a: egui::Pos2, b: egui::Pos2, c: egui::Pos2) -> f32 {
    (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x)
}

fn blank_app() -> StockApp {
    set_theme(Theme::CLASSIC);
    let palette = indicator::Palette::default();
    let chart_world = ChartWorld::new(StockApp::build_panes(palette), palette);
    let mut app = StockApp {
        chart_world,
        kline_h: KLINE_H_DEFAULT,
        vol_h: VOL_H_DEFAULT,
        ind_h: IND_H_DEFAULT,
        up_color: UP_RGB_DEFAULT,
        down_color: DOWN_RGB_DEFAULT,
        chart_follows_theme: true,
        hwnd: 0,
        client_cover_ready: false,

        icons: IconSet::load_or_create(),

        symbols: SymbolSet::load(),

        cursors: Cursors::load(),
        pointer: crate::pointer::PointerManager::new(crate::pointer::PointerFeel::default()),
        tooltip: None,
        menu: MenuState::default(),
        status_msg: String::new(),
        frames: 0,
        last_frame_time: 0.0,
        caret_blink_epoch: 0.0,
        frame_interval_ms: 0.0,
        frame_ms: 0.0,
        fps: 0.0,
        text_engine: None,
        hsplit_grab: None,
        vsplit_grab: None,
        split_x: CHART_PANE_W,
        win_mgr: WinMgr::with_default_windows_and_chart(),
        work_rect: egui::Rect::NOTHING,
        panel_buttons: Vec::new(),
        panel_press: None,
        color_panel: ColorPanel::new(),
        win_silent: false,
        context_menu: crate::ui::contextmenu::ContextMenuState::default(),
        win_cmd: None,
        pane_menu: None,
        pane_menu_anchor: egui::Rect::NOTHING,
        pane_menu_bar: egui::Rect::NOTHING,
        source: SourceView::new(),
        console: crate::ui::console::DebugConsole::new(),
        debug_windows: crate::app::DebugWindowSnapshot::default(),
    };

    app.bars.replace(Vec::new());
    app.panes[0].view.visible = 0;
    app.adopt_theme_candle_colors(theme());
    app.sync_pane_palette();
    app
}

fn blank_app_on(t: Theme) -> StockApp {
    set_theme(t);
    let mut app = blank_app();
    set_theme(t);
    app.adopt_theme_candle_colors(t);
    app.sync_pane_palette();
    app
}

fn color_panel_app() -> StockApp {
    let mut app = blank_app();
    app.open_color_panel(ColorTarget::Up);
    app.color_panel.load((34, 136, 255));
    app
}

fn color_index(app: &StockApp) -> usize {
    app.win_mgr.color_index().expect("颜色面板没打开")
}

fn load_long_indicator_script(app: &mut StockApp) {
    let mut text = String::new();
    for i in 0..80 {
        text.push_str(&format!("# test line {i}\n"));
    }
    text.push_str("LINE(C,TEXT);");
    app.source.editor.load(&text);
    app.source.last_text = text;
}

fn cursor_painted_at(
    app: &mut StockApp,
    grid: &[Vec<Option<egui::Color32>>],
    tip: egui::Pos2,
    kind: CursorKind,
) -> bool {
    let rgba = app.cursors.rgba(kind);
    let (x0, y0) = (tip.x.round() as i32, tip.y.round() as i32);
    rgba.chunks_exact(4).enumerate().any(|(k, p)| {
        if p[3] == 0 {
            return false;
        }
        let x = x0 + (k % ICON_PX) as i32;
        let y = y0 + (k / ICON_PX) as i32;
        grid.get(y as usize)
            .and_then(|row| row.get(x as usize))
            .is_some_and(|c| *c == Some(egui::Color32::from_rgb(p[0], p[1], p[2])))
    })
}

fn render_panel(app: &mut StockApp) -> Vec<Vec<Option<egui::Color32>>> {
    let ctx = egui::Context::default();
    configure_tessellation(&ctx);
    configure_zero_spacing(&ctx);
    ctx.set_pixels_per_point(1.0);

    if let Some(engine) = HardText::new(&ctx) {
        app.text_engine = Some(engine);
    }
    if app.panel_buttons.is_empty() {
        app.build_panel_buttons();
    }
    let mut raw = egui::RawInput::default();
    raw.screen_rect = Some(egui::Rect::from_min_size(
        egui::Pos2::ZERO,
        egui::vec2(WINDOW_W, WINDOW_H),
    ));
    raw.events = vec![egui::Event::PointerMoved(egui::pos2(5.0, 5.0))];
    let mut out = ctx.run_ui(raw, |ui| {
        app.draw_frame(ui);
    });
    let shapes = std::mem::take(&mut out.shapes);
    absorb_textures(&mut out.textures_delta);
    rasterize_into(
        &ctx.tessellate(shapes, 1.0),
        WINDOW_W as usize,
        WINDOW_H as usize,
    )
}

fn rasterize(prims: &[egui::ClippedPrimitive]) -> Vec<Vec<Option<egui::Color32>>> {
    rasterize_into(prims, CANVAS_W, CANVAS_H)
}

thread_local! {


    static DUMP_TEXTURES: std::cell::RefCell<
        std::collections::HashMap<egui::TextureId, egui::ColorImage>,
    > = std::cell::RefCell::new(std::collections::HashMap::new());
}

fn absorb_textures(delta: &mut egui::TexturesDelta) {
    DUMP_TEXTURES.with(|cell| {
        let mut store = cell.borrow_mut();
        for (id, list) in &delta.set {
            for img in list {
                let egui::ImageData::Color(src) = &img.image;
                match img.pos {
                    None => {
                        store.insert(*id, (**src).clone());
                    }
                    Some(p) => {
                        if let Some(dst) = store.get_mut(id) {
                            for y in 0..src.size[1] {
                                for x in 0..src.size[0] {
                                    let (dx, dy) = (p[0] + x, p[1] + y);
                                    if dx < dst.size[0] && dy < dst.size[1] {
                                        dst[(dx, dy)] = src[(x, y)];
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        for id in &delta.free {
            store.remove(id);
        }

        delta.set.clear();
        delta.free.clear();
    });
}

fn sample_texture(id: egui::TextureId, uv: egui::Pos2) -> Option<egui::Color32> {
    DUMP_TEXTURES.with(|cell| {
        cell.borrow().get(&id).map(|img| {
            let sx =
                ((uv.x * img.size[0] as f32) as isize).rem_euclid(img.size[0] as isize) as usize;
            let sy =
                ((uv.y * img.size[1] as f32) as isize).rem_euclid(img.size[1] as isize) as usize;
            img[(sx, sy)]
        })
    })
}

fn rasterize_into(
    prims: &[egui::ClippedPrimitive],
    w: usize,
    h: usize,
) -> Vec<Vec<Option<egui::Color32>>> {
    let mut grid = vec![vec![None; w]; h];
    for prim in prims {
        let egui::epaint::Primitive::Mesh(mesh) = &prim.primitive else {
            continue;
        };
        for tri in mesh.indices.chunks_exact(3) {
            let [a, b, c] = [
                mesh.vertices[tri[0] as usize],
                mesh.vertices[tri[1] as usize],
                mesh.vertices[tri[2] as usize],
            ];
            let area = edge(a.pos, b.pos, c.pos);
            if area == 0.0 {
                continue;
            }
            let x0 = a.pos.x.min(b.pos.x).min(c.pos.x).floor().max(0.0) as usize;
            let x1 = (a.pos.x.max(b.pos.x).max(c.pos.x).ceil() as usize).min(w);
            let y0 = a.pos.y.min(b.pos.y).min(c.pos.y).floor().max(0.0) as usize;
            let y1 = (a.pos.y.max(b.pos.y).max(c.pos.y).ceil() as usize).min(h);
            for y in y0..y1 {
                for x in x0..x1 {
                    let p = egui::pos2(x as f32 + 0.5, y as f32 + 0.5);
                    let d = [
                        edge(b.pos, c.pos, p),
                        edge(c.pos, a.pos, p),
                        edge(a.pos, b.pos, p),
                    ];
                    let (neg, pos) = (d.iter().any(|v| *v < 0.0), d.iter().any(|v| *v > 0.0));
                    if !(neg && pos) {
                        let solid = match sample_texture(
                            mesh.texture_id,
                            egui::pos2(
                                (a.uv.x * d[0] + b.uv.x * d[1] + c.uv.x * d[2]) / area,
                                (a.uv.y * d[0] + b.uv.y * d[1] + c.uv.y * d[2]) / area,
                            ),
                        ) {
                            None => true,
                            Some(px) => px.a() > 0,
                        };
                        if solid {
                            grid[y][x] = Some(a.color);
                        }
                    }
                }
            }
        }
    }
    grid
}

fn cols_of(grid: &[Vec<Option<egui::Color32>>], row: usize, color: egui::Color32) -> Vec<usize> {
    (0..CANVAS_W)
        .filter(|&x| grid[row][x] == Some(color))
        .collect()
}

fn is_title_face(c: Option<egui::Color32>) -> bool {
    c == Some(C_TITLE_BG) || c == Some(C_MENU_BG) || c == Some(theme().title_fill)
}

fn ascii(grid: &[Vec<Option<egui::Color32>>], rows: std::ops::Range<usize>) -> String {
    let mut out = String::new();
    for row in grid.iter().take(rows.end).skip(rows.start) {
        for cell in row {
            out.push(match cell {
                None => '.',
                Some(c) if *c == C_UP => 'R',
                Some(c) if *c == C_DOWN => 'G',
                Some(_) => 'w',
            });
        }
        out.push('\n');
    }
    out
}

fn assert_centered(label: &str, body: &[usize], wick: usize, art: String) {
    assert_eq!(
        wick - body[0],
        body[body.len() - 1] - wick,
        "{label}\n{art}"
    );
}

#[test]
fn wick_is_centered_in_the_body_for_every_scale() {
    for scale in (SCALE_MIN..=SCALE_MAX).step_by(2) {
        let grid = render_candle(scale, false);
        let art = ascii(&grid, 28..52);
        println!("下跌实心 scale={scale}\n{art}");

        let body = cols_of(&grid, BODY_ROW, C_DOWN);
        let wick = cols_of(&grid, WICK_ROW, C_DOWN);
        let label = format!("实体 {scale}px 里影线没在正中");
        assert_eq!(body.len(), snap_scale(scale) as usize, "{label}\n{art}");
        assert_eq!(wick.len(), 1, "影线不是 1px 宽\n{art}");
        assert_centered(&label, &body, wick[0], art);
    }
}

#[test]
fn hollow_body_keeps_the_wick_in_the_middle() {
    for scale in (SCALE_MIN..=SCALE_MAX).step_by(2) {
        let grid = render_candle(scale, true);
        let art = ascii(&grid, 28..52);
        println!("上涨空心 scale={scale}\n{art}");

        let edges = cols_of(&grid, BODY_ROW, C_UP);
        let inner = cols_of(&grid, BODY_ROW, C_CHART_BG);
        let wick = cols_of(&grid, WICK_ROW, C_UP);
        let label = format!("空心实体 {scale}px 里影线没在正中");
        assert_eq!(edges.len(), 2, "空心实体的竖边不是两根\n{art}");
        assert_eq!(
            inner.len(),
            snap_scale(scale) as usize - 2,
            "{label}\n{art}"
        );
        assert_eq!(wick.len(), 1, "影线不是 1px 宽\n{art}");
        let axis = edges[0] + edges[1];
        assert_eq!(axis, inner[0] + inner[inner.len() - 1], "{label}\n{art}");
        assert_eq!(wick[0] * 2, axis, "{label}\n{art}");
    }
}

#[test]
fn color_panel_sits_on_the_grid() {
    assert_eq!(CP_W as i32, 235, "面板宽");
    assert_eq!(CP_H as i32, 271, "面板高");
    assert_eq!((CP_W as i32 - 1) % 9, 0, "面板宽必须是 cells(n)（9n+1）");
    assert_eq!((CP_H as i32 - 1) % 9, 0, "面板高必须是 cells(n)（9n+1）");
    assert_eq!(CP_CONTENT_W as i32, 233, "内容列宽");

    assert_eq!(
        CP_TITLE_ROWS + CP_SV_ROWS + CP_HUE_ROWS + 3 * CP_SLIDER_ROWS + CP_HEX_ROWS + CP_BTN_ROWS,
        CP_ROWS,
        "内部各段格数加起来不是面板总格数"
    );
    assert_eq!(SV_COLS * SV_ROWS, 416, "SV 调色板应该是 26x16 块");
    assert_eq!(SV_BLOCK as i32, 8, "每块应该是 8x8");

    assert_eq!(
        SV_COLS as f32 * SV_BLOCK + (SV_COLS - 1) as f32 * LINE_W,
        CP_CONTENT_W
    );
    assert_eq!(
        SV_ROWS as f32 * SV_BLOCK + (SV_ROWS - 1) as f32 * LINE_W,
        cells(CP_SV_ROWS) - 2.0 * LINE_W
    );

    assert_eq!(HUE_BARS as f32, CP_CONTENT_W);

    assert_eq!(
        CP_SLIDER_LABEL_COLS
            + CP_SLIDER_GAP_COLS
            + CP_TRACK_COLS
            + CP_SLIDER_GAP_COLS
            + CP_SLIDER_VALUE_COLS,
        CP_COLS
    );
    assert_eq!((CP_TRACK_W as i32 - 1) % 9, 0, "轨道宽必须是 cells(n)");
    assert_eq!((CP_HANDLE_W as i32 - 1) % 9, 0, "把手宽必须是 cells(n)");

    assert_eq!(CP_HALF_W as i32, 118, "底部两个按钮各占一半");
    assert_eq!(CP_LEFT_W as i32, 118, "左半边");
    assert_eq!(CP_LEFT_W + CP_HALF_W - LINE_W, CP_W);

    assert_eq!(
        (CP_Y_BTN - CP_Y_HEX) as i32,
        CP_HEX_ROWS * 9,
        "Hex 段高不是 2 格"
    );
    assert_eq!(CP_Y_HEX as i32, 234, "Hex 段上沿");
    assert_eq!(CP_Y_BTN as i32, 252, "底部按钮段上沿");
}

#[test]
fn color_selection_cross_is_two_pixels_and_centered() {
    let cell = egui::Rect::from_min_size(egui::pos2(100.0, 200.0), egui::vec2(SV_BLOCK, SV_BLOCK));
    let center = cell.center();
    let [horizontal, vertical] = selection_cross_rects(center);
    assert_eq!(horizontal.size(), egui::vec2(6.0, 2.0));
    assert_eq!(vertical.size(), egui::vec2(2.0, 6.0));
    assert_eq!(horizontal.center(), center);
    assert_eq!(vertical.center(), center);
    assert!(cell.contains_rect(horizontal));
    assert!(cell.contains_rect(vertical));
    assert_eq!(horizontal.min.x - cell.min.x, 1.0);
    assert_eq!(cell.max.x - horizontal.max.x, 1.0);
    assert_eq!(vertical.min.y - cell.min.y, 1.0);
    assert_eq!(cell.max.y - vertical.max.y, 1.0);
    assert_eq!(contrast_on((0, 0, 0)), egui::Color32::WHITE);
    assert_eq!(contrast_on((255, 255, 255)), egui::Color32::BLACK);
    assert_eq!(contrast_on((127, 127, 127)), egui::Color32::WHITE);
    assert_eq!(contrast_on((128, 128, 128)), egui::Color32::BLACK);
}

#[test]
fn hex_text_is_strict() {
    assert_eq!(parse_hex("2288FF"), Some((0x22, 0x88, 0xFF)));
    assert_eq!(parse_hex("2288ff"), Some((0x22, 0x88, 0xFF)));

    assert_eq!(parse_hex("#2288FF"), None);
    assert_eq!(hex_of((0x22, 0x88, 0xFF)), "#2288FF");
    assert_eq!(hex_digits((0x22, 0x88, 0xFF)), "2288FF");
    for bad in [
        "#28F",
        "rgb(34,136,255)",
        "#2288F",
        "#2288FFF",
        "#GG88FF",
        "#",
        "",
        "2288F",
        "2288FFF",
        "GG88FF",
    ] {
        assert_eq!(parse_hex(bad), None, "{bad} 竟然被当成合法颜色了");
    }
}

#[test]
fn color_conversions_are_sane() {
    assert_eq!(hsv_to_rgb(0, 255, 255), (255, 0, 0));
    assert_eq!(hsv_to_rgb(120, 255, 255), (0, 255, 0));
    assert_eq!(hsv_to_rgb(240, 255, 255), (0, 0, 255));
    assert_eq!(hsv_to_rgb(0, 0, 255), (255, 255, 255));
    assert_eq!(hsv_to_rgb(0, 0, 0), (0, 0, 0));
    assert_eq!(rgb_to_hsv(255, 0, 0), (0, 255, 255));
    assert_eq!(rgb_to_hsv(0, 0, 0), (0, 0, 0));
    let near = |a: u8, b: u8| (a as i32 - b as i32).abs() <= 2;
    for r in (0..=255).step_by(17) {
        for g in (0..=255).step_by(17) {
            for b in (0..=255).step_by(17) {
                let (h, s, v) = rgb_to_hsv(r, g, b);
                let back = hsv_to_rgb(h, s, v);
                assert!(
                    near(back.0, r) && near(back.1, g) && near(back.2, b),
                    "rgb({r},{g},{b}) 绕 HSV 一圈变成 {back:?}，中间是 ({h},{s},{v})"
                );
            }
        }
    }
}

#[test]
fn palette_index_maps_hit_both_ends() {
    assert_eq!(s_of_col(0), 0);
    assert_eq!(s_of_col(SV_COLS - 1), 255);
    assert_eq!(v_of_row(0), 255);
    assert_eq!(v_of_row(SV_ROWS - 1), 0);
    assert_eq!(col_of_s(0), 0);
    assert_eq!(col_of_s(255), SV_COLS - 1);
    assert_eq!(row_of_v(255), 0);
    assert_eq!(row_of_v(0), SV_ROWS - 1);
    assert_eq!(hue_of_bar(0), 0);
    assert_eq!(hue_of_bar(HUE_BARS - 1), 360);
    assert_eq!(hue_handle_left(0), 0.0);
    assert_eq!(hue_handle_left(360), HUE_TRAVEL);
    assert_eq!(hue_at_x(0.0), 0);
    assert_eq!(hue_at_x(HUE_TRAVEL), 360);
    assert_eq!(hue_handle_left(180).fract(), 0.0);
    assert_eq!(bar_value(0, 255), 0);
    assert_eq!(bar_value(CP_TRACK_INNER_W - 1, 255), 255);
    assert_eq!(bar_value(CP_TRACK_INNER_W - 1, 360), 360);

    assert_eq!(handle_left(0.0, 0.0), 0.0);
    assert_eq!(handle_left(0.0, 1.0), CP_HANDLE_TRAVEL);
    assert_eq!(handle_left(0.0, 0.5).fract(), 0.0, "把手落在了小数像素上");
}

#[test]
fn color_panel_paints_blocks_and_bars() {
    let mut app = color_panel_app();
    let grid = render_panel(&mut app);

    let panel = app.win_mgr.color_rect().expect("颜色面板不存在");
    let px = panel.min.x as usize;
    let py = panel.min.y as usize;
    let (w, h) = (CP_W as usize, CP_H as usize);

    assert_eq!(grid[py][px], Some(C_CTRL_BORDER), "面板左上角应该是边框");
    assert_eq!(
        grid[py][px + w - 1],
        Some(C_CTRL_BORDER),
        "面板右上角应该是边框"
    );
    assert_eq!(
        grid[py + h - 1][px + w - 1],
        Some(C_CTRL_BORDER),
        "面板右下角应该是边框"
    );

    assert_ne!(
        grid[py + 100][px + w],
        Some(C_CTRL_BORDER),
        "面板右边多画了一条边框"
    );
    assert_ne!(
        grid[py + h][px + 100],
        Some(C_CTRL_BORDER),
        "面板下边多画了一条边框"
    );

    assert!(is_title_face(grid[py + 4][px + 2]), "标题栏底色");
    assert_eq!(
        grid[py + CP_Y_RGB as usize + 8][px + 20],
        Some(C_WORK_BG),
        "滑块行空白处应该是内容底色"
    );

    let sx = px + LINE_W as usize;
    let sy = py + CP_Y_SV as usize + LINE_W as usize;
    let picked = (
        col_of_s(app.color_panel.s) as usize,
        row_of_v(app.color_panel.v) as usize,
    );
    for row in 0..SV_ROWS {
        for col in 0..SV_COLS {
            if (col as usize, row as usize) == picked {
                continue;
            }
            let want = rgb32(hsv_to_rgb(app.color_panel.h, s_of_col(col), v_of_row(row)));
            let x = sx + col as usize * PITCH as usize + 4;
            let y = sy + row as usize * PITCH as usize + 4;
            assert_eq!(grid[y][x], Some(want), "SV 块 ({col}, {row}) 的颜色不对");
        }
    }

    let sel_x = sx + col_of_s(app.color_panel.s) as usize * PITCH as usize;
    let sel_y = sy + row_of_v(app.color_panel.v) as usize * PITCH as usize;
    let (cx, cy) = (sel_x + 4, sel_y + 4);
    let mark = contrast_on(app.color_panel.rgb);
    for dx in 2..=6 {
        assert_eq!(
            grid[cy][sel_x + dx],
            Some(mark),
            "颜色十字星的横向笔画缺了 ({dx})"
        );
        assert_eq!(
            grid[sel_y + dx][cx],
            Some(mark),
            "颜色十字星的纵向笔画缺了 ({dx})"
        );
    }
    assert_ne!(grid[sel_y + 1][sel_x + 1], Some(mark), "十字星不该铺满整格");

    for col in 0..SV_COLS - 1 {
        let x = sx + (col as usize + 1) * PITCH as usize - 1;
        assert_eq!(
            grid[sy + 4][x],
            Some(C_CTRL_BORDER),
            "第 {col} 块和第 {} 块之间那条网格线不见了",
            col + 1
        );
    }

    let hx = px + LINE_W as usize;
    let hy = py + CP_Y_HUE as usize + LINE_W as usize;
    let handle_x0 = px + hue_handle_left(app.color_panel.h) as usize;
    let covered = handle_x0..handle_x0 + CP_HANDLE_W as usize;
    for i in 0..HUE_BARS as usize {
        let x = hx + i;
        if covered.contains(&x) {
            continue;
        }
        let want = rgb32(hsv_to_rgb(hue_of_bar(i as i32), 255, 255));
        assert_eq!(grid[hy + 8][x], Some(want), "第 {i} 根色相条的颜色不对");
    }
}

#[test]
fn slider_handles_land_on_whole_pixels() {
    let track_x = CP_DEFAULT_X + grid_line(CP_TRACK_LINE);
    assert!(track_x.fract() == 0.0, "轨道起点本身就得在整像素上");
    for max in [255, 360] {
        for v in 0..=max {
            let hx = handle_left(track_x, v as f32 / max as f32);
            assert!(hx.fract() == 0.0, "值 {v}/{max} 的把手落在小数像素 {hx} 上");
            assert!(
                hx >= track_x && hx <= track_x + CP_HANDLE_TRAVEL,
                "值 {v}/{max} 的把手跑出轨道可走范围了：{hx}"
            );
        }
    }
}

#[test]
fn hex_box_frame_sits_on_grid_lines() {
    let mut app = color_panel_app();
    let grid = render_panel(&mut app);
    let cp_rect = app.win_mgr.color_rect().expect("颜色面板不存在");
    let px = cp_rect.min.x as usize;
    let py = cp_rect.min.y as usize;
    let x = px + LINE_W as usize + 20;
    let top = py + CP_Y_HEX as usize;
    let bottom = py + CP_Y_BTN as usize;
    assert_eq!(grid[top][x], Some(C_CTRL_BORDER), "上边框没压在网格线上");
    assert_eq!(grid[bottom][x], Some(C_CTRL_BORDER), "下边框没压在网格线上");
    assert_eq!(grid[top - 1][x], Some(C_WORK_BG), "上边框上面还多了一条线");
    assert_eq!(
        grid[bottom - 1][x],
        Some(C_WORK_BG),
        "下边框上面还多了一条线"
    );
    assert_eq!(
        grid[bottom + 1][x],
        Some(C_BTN_FACE),
        "下边框下面还多了一条线"
    );

    let hex_rect = hex_box(cp_rect);
    let label_row = py + CP_Y_HEX as usize + 9;
    let label_ink = (px + LINE_W as usize..hex_rect.min.x as usize)
        .filter(|&cx| (label_row - 8..label_row + 8).any(|cy| grid[cy][cx] == Some(C_TEXT)))
        .count();
    assert!(label_ink > 0, "前缀 '#' 没画在输入框外面那一栏里");

    let box_line = hex_rect.min.x as usize;
    assert_eq!(
        grid[label_row][box_line],
        Some(C_CTRL_BORDER),
        "输入框左边那条线没画出来"
    );
    assert_ne!(
        grid[label_row][box_line - 1],
        Some(C_CTRL_BORDER),
        "输入框左边那条线叠成 2px 了"
    );

    let mut app = color_panel_app();
    app.color_panel.hex.set_text("2288F");

    let _ = render_panel(&mut app);
    let grid = render_panel(&mut app);
    assert_eq!(grid[top][x], Some(C_CTRL_BORDER), "非法时上边框不该变色");
    assert_eq!(grid[bottom][x], Some(C_CTRL_BORDER), "非法时下边框不该变色");
    assert_eq!(
        grid[bottom - 1][x],
        Some(C_WORK_BG),
        "非法时下边框上面还有一条线"
    );

    let hex_row = py + CP_Y_HEX as usize + 9;
    let hex_rect = hex_box(cp_rect);
    let (inner_l, inner_r) = (
        hex_rect.min.x as usize + LINE_W as usize,
        hex_rect.max.x as usize - LINE_W as usize,
    );
    let ink = (inner_l..inner_r)
        .filter(|&cx| (hex_row - 8..hex_row + 8).any(|cy| grid[cy][cx] == Some(C_UP)))
        .count();
    assert!(ink > 0, "非法时文字没变红");
}

#[test]
fn panel_right_edges_line_up() {
    let mut app = color_panel_app();
    let grid = render_panel(&mut app);

    let panel = app.win_mgr.color_rect().expect("颜色面板不存在");
    let px = panel.min.x as usize;
    let py = panel.min.y as usize;
    let (w, _h) = (CP_W as usize, CP_H as usize);
    let inner_l = px + LINE_W as usize;
    let inner_r = px + w - 1 - LINE_W as usize;

    let hex_row = py + CP_Y_HEX as usize + 8;
    let btn_row = py + CP_Y_BTN as usize + 8;
    let hue_row = py + CP_Y_HUE as usize + 8;

    assert_eq!(
        grid[hex_row][px],
        Some(C_CTRL_BORDER),
        "Hex 输入框左边框不对"
    );
    assert_eq!(
        grid[hex_row][px + w - 1],
        Some(C_CTRL_BORDER),
        "Hex 输入框右边框不对"
    );
    assert_eq!(
        grid[hex_row][inner_l],
        Some(C_WORK_BG),
        "Hex 输入框左边多了一条线"
    );
    assert_eq!(
        grid[hex_row][inner_r],
        Some(C_WORK_BG),
        "Hex 输入框右边多了一条线"
    );

    assert_ne!(
        grid[hue_row][inner_r],
        Some(C_WORK_BG),
        "色相条右边留了白边"
    );
    assert_eq!(
        grid[hue_row][px + w - 1],
        Some(C_CTRL_BORDER),
        "色相条右边框不对"
    );

    let mid = px + CP_HALF_W as usize - LINE_W as usize;
    assert_eq!((mid - px) % 9, 0, "中间那条共用的竖线没落在 9px 网格线上");
    assert_eq!(
        grid[btn_row][mid],
        Some(C_CTRL_BORDER),
        "中间那条竖线不见了"
    );
    assert_eq!(
        grid[btn_row][mid - 1],
        Some(C_BTN_FACE),
        "中间那条竖线不止 1px"
    );
    assert_eq!(
        grid[btn_row][mid + 1],
        Some(C_BTN_FACE),
        "中间那条竖线不止 1px"
    );

    assert_eq!(grid[btn_row][px], Some(C_CTRL_BORDER), "应用按钮左边框不对");
    assert_eq!(
        grid[btn_row][px + w - 1],
        Some(C_CTRL_BORDER),
        "取消按钮右边框不对"
    );
}

fn left_work() -> egui::Rect {
    egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(CHART_PANE_W, cells(WORK_ROWS)))
}

fn render_left(app: &mut StockApp) -> (Vec<Vec<Option<egui::Color32>>>, [egui::Rect; 5]) {
    render_left_in(app, left_work(), WINDOW_H as usize)
}

fn render_left_in(
    app: &mut StockApp,
    work: egui::Rect,
    canvas_h: usize,
) -> (Vec<Vec<Option<egui::Color32>>>, [egui::Rect; 5]) {
    app.clamp_left_heights(work);
    let [kline, bar, volume, bar2, indicator] = StockApp::left_rows(work, app.kline_h, app.vol_h);
    let ctx = egui::Context::default();
    configure_tessellation(&ctx);
    ctx.set_pixels_per_point(1.0);

    if let Some(engine) = HardText::new(&ctx) {
        app.text_engine = Some(engine);
    }
    let mut raw = egui::RawInput::default();
    raw.screen_rect = Some(egui::Rect::from_min_size(
        egui::Pos2::ZERO,
        egui::vec2(WINDOW_W, canvas_h as f32),
    ));
    let mut out = ctx.run_ui(raw, |ui| {
        app.ui_chart(ui, PANE_CANDLE, kline);
        app.ui_chart(ui, PANE_VOLUME, volume);
        app.ui_chart(ui, PANE_MACD, indicator);
        app.ui_hsplitter(ui, bar, work, LeftSplit::Volume);
        app.ui_hsplitter(ui, bar2, work, LeftSplit::Indicator);
    });
    let shapes = std::mem::take(&mut out.shapes);
    absorb_textures(&mut out.textures_delta);
    (
        rasterize_into(&ctx.tessellate(shapes, 1.0), WINDOW_W as usize, canvas_h),
        [kline, bar, volume, bar2, indicator],
    )
}

fn color_cols(
    grid: &[Vec<Option<egui::Color32>>],
    x0: usize,
    x1: usize,
    y0: usize,
    y1: usize,
    color: egui::Color32,
) -> Vec<usize> {
    (x0..x1)
        .filter(|&x| (y0..y1).any(|y| grid[y][x] == Some(color)))
        .collect()
}

#[test]
fn volume_bars_align_with_candles() {
    let bars: Vec<data::random_walk::Bar> = (0..48)
        .map(|i| data::random_walk::Bar {
            o: 100.0,
            h: 101.0,
            l: 89.0,
            c: 90.0,
            vol: 1.0e7 + (i % 7) as f64 * 1.0e6,
            date: data::random_walk::DATE_START.add_days(i as i32),
        })
        .collect();
    let mut app = blank_app();
    app.replace_bars(bars);
    for pane in &mut app.panes {
        pane.view.scale = SCALE_MIN;
    }

    let (grid, [kline, _bar, volume, _bar2, _ind]) = render_left(&mut app);

    let down = rgb32(DOWN_RGB_DEFAULT);
    let (_, _, kmain, _) = chart_rows(kline, app.panes[PANE_CANDLE].indicator.layout());
    let (kplot, _) = frame_rects(kmain);
    let (vplot, _) = frame_rects(volume);
    let k_cols = color_cols(
        &grid,
        kplot.min.x as usize,
        kplot.max.x as usize,
        kplot.min.y as usize,
        kplot.max.y as usize,
        down,
    );
    let v_cols = color_cols(
        &grid,
        vplot.min.x as usize,
        vplot.max.x as usize,
        volume.min.y as usize,
        volume.max.y as usize,
        down,
    );
    assert!(!k_cols.is_empty(), "蜡烛一根都没画出来");
    assert_eq!(k_cols, v_cols, "量柱和蜡烛占的像素列对不上（没严格对齐）");
}

#[test]
fn left_column_splits_on_the_grid() {
    assert_eq!(
        KLINE_H_DEFAULT as i32,
        cells(KLINE_ROWS) as i32,
        "图表1 默认高度应按 KLINE_ROWS 计算"
    );
    assert_eq!(VOL_H_DEFAULT as i32, 190, "图表2 默认 21 格 = cells(21)");
    assert_eq!(IND_H_DEFAULT as i32, 190, "图表3 默认 21 格 = cells(21)");
    assert_eq!(
        PANE_H_MIN as i32, 199,
        "三块图表统一下限 22 格 = cells(22)（比原来多的是图表窗自己的状态栏 2 格）"
    );
    assert_eq!(HSPLIT_H as i32, 10, "横向拖拽条 1 格 = cells(1) = 10px");
    assert_eq!(
        LEFT_FIXED_H as i32, 16,
        "固定开销 = 两条拖拽条 - 四条共用线"
    );

    let work = egui::Rect::from_min_size(
        egui::pos2(0.0, 54.0),
        egui::vec2(CHART_PANE_W, cells(WORK_ROWS)),
    );
    let mut app = blank_app();

    for (want_vol, want_ind) in [
        (VOL_H_DEFAULT, IND_H_DEFAULT),
        (0.0, 0.0),
        (10000.0, 10000.0),
        (300.0, 300.0),
        (PANE_H_MIN, 5000.0),
        (5000.0, PANE_H_MIN),
    ] {
        app.vol_h = want_vol;
        app.ind_h = want_ind;
        app.clamp_left_heights(work);
        let [kline, bar, vol, bar2, ind] = StockApp::left_rows(work, app.kline_h, app.vol_h);

        for h in [app.kline_h, app.vol_h, app.ind_h] {
            assert_eq!((h as i32 - 1) % 9, 0, "块高 {h} 不是整数格 cells(n)");
            assert!(h >= PANE_H_MIN, "块高 {h} 低于统一最小高度");
        }
        assert_eq!(bar.height(), HSPLIT_H, "上面那条拖拽条高度不是 1 格");
        assert_eq!(bar2.height(), HSPLIT_H, "下面那条拖拽条高度不是 1 格");

        assert_eq!(kline.height(), app.kline_h, "图表1 高度和收口值对不上");
        assert_eq!(vol.height(), app.vol_h, "图表2 高度和收口值对不上");
        assert_eq!(ind.height(), app.ind_h, "图表3 高度和收口值对不上");

        assert_eq!(kline.min.y, work.min.y);
        assert_eq!(bar.min.y, kline.max.y - LINE_W);
        assert_eq!(vol.min.y, bar.max.y - LINE_W);
        assert_eq!(bar2.min.y, vol.max.y - LINE_W);
        assert_eq!(ind.min.y, bar2.max.y - LINE_W);

        assert_eq!(ind.max.y, work.max.y, "图表3 没铺到左栏底部");
        assert_eq!(
            kline.height() + LEFT_FIXED_H + vol.height() + ind.height(),
            work.height(),
            "五块加起来不等于左栏高度"
        );
        for r in [kline, bar, vol, bar2, ind] {
            assert!(r.min.x.fract() == 0.0 && r.min.y.fract() == 0.0);
        }
    }

    let mut app = blank_app();
    app.clamp_left_heights(work);

    assert_eq!(app.vol_h, PANE_H_MIN);
    assert_eq!(app.ind_h, PANE_H_MIN);
    assert_eq!(
        app.kline_h,
        cells(WORK_ROWS) - LEFT_FIXED_H - 2.0 * PANE_H_MIN,
        "图表1 拿剩下的全部高度"
    );
}

#[test]
fn left_column_lines_are_1px() {
    let mut app = blank_app();
    let (grid, [kline, bar, volume, bar2, indicator]) = render_left(&mut app);
    let x = 100usize;
    let px = |y: usize| grid[y][x];

    let layout = app.panes[PANE_CANDLE].indicator.layout();
    let (title, status, main, time) = chart_rows(kline, layout);
    let (info, time) = (status.unwrap(), time.unwrap());
    assert_eq!(
        px(title.max.y as usize - 1),
        Some(C_LINE),
        "标题栏下沿那条线不对"
    );
    assert_eq!(
        px(title.max.y as usize - 2),
        Some(C_MENU_BG),
        "标题栏下沿多画了一行"
    );
    assert_eq!(
        px(main.min.y as usize - 1),
        Some(C_LINE),
        "状态带下沿那条线不对"
    );
    assert_eq!(
        px(main.min.y as usize - 2),
        Some(C_WORK_BG),
        "状态带下沿多画了一行"
    );
    assert_eq!(
        px(main.min.y as usize),
        Some(C_CHART_BG),
        "绘图区没紧跟着状态带"
    );
    assert_eq!(
        px(time.min.y as usize),
        Some(C_LINE),
        "时间带上沿那条线不对"
    );
    assert_eq!(
        px(time.min.y as usize + 1),
        Some(C_WORK_BG),
        "时间带没紧跟着那条线"
    );
    assert_eq!(
        info.min.y,
        title.max.y - LINE_W,
        "状态带和标题栏没共用那条线"
    );

    let cbar = chart_bar(kline);
    assert_eq!(
        time.max.y,
        cbar.min.y + LINE_W,
        "时间带和状态栏没共用那条线"
    );
    assert_eq!(cbar.max.y, kline.max.y, "图表窗状态栏没铺到图表底边");
    assert_eq!(
        px(cbar.min.y as usize),
        Some(C_LINE),
        "状态栏上沿那条线不对"
    );
    assert_eq!(
        px(cbar.min.y as usize + 1),
        Some(C_STATUS_BG),
        "状态栏没紧跟着那条线"
    );

    let b1 = bar.min.y as usize;
    assert_eq!(bar.height() as usize, HSPLIT_H as usize, "拖拽条不是 1 格");

    assert_ne!(px(b1 - 1), Some(C_LINE), "图表1 下沿叠出了第二条线");
    assert_eq!(
        px(b1),
        Some(C_CTRL_BORDER),
        "图表1 / 拖拽条共用的上边线不对"
    );
    for r in 1..(HSPLIT_H as usize - 1) {
        assert_eq!(px(b1 + r), Some(C_SPLITTER), "拖拽条第 {r} 行不是本体色");
    }
    assert_eq!(
        px(b1 + HSPLIT_H as usize - 1),
        Some(C_CTRL_BORDER),
        "拖拽条下沿那条线不对"
    );
    assert_eq!(
        volume.min.y as usize,
        b1 + HSPLIT_H as usize - 1,
        "图表2 和拖拽条没共用那条线"
    );
    assert_eq!(
        px(volume.min.y as usize + 1),
        Some(C_MENU_BG),
        "图表2 的标题栏没紧跟着拖拽条"
    );

    let b2 = bar2.min.y as usize;
    assert_eq!(
        bar2.height() as usize,
        HSPLIT_H as usize,
        "下面那条拖拽条不是 1 格"
    );

    assert_ne!(px(b2 - 1), Some(C_LINE), "图表2 下沿叠出了第二条线");
    assert_eq!(
        px(b2),
        Some(C_CTRL_BORDER),
        "图表2 / 拖拽条共用的上边线不对"
    );
    for r in 1..(HSPLIT_H as usize - 1) {
        assert_eq!(
            px(b2 + r),
            Some(C_SPLITTER),
            "下面那条拖拽条第 {r} 行不是本体色"
        );
    }
    assert_eq!(
        px(b2 + HSPLIT_H as usize - 1),
        Some(C_CTRL_BORDER),
        "下面那条拖拽条下沿不对"
    );
    assert_eq!(
        indicator.min.y as usize,
        b2 + HSPLIT_H as usize - 1,
        "图表3 和拖拽条没共用那条线"
    );
    assert_eq!(
        px(indicator.min.y as usize + 1),
        Some(C_MENU_BG),
        "图表3 的标题栏没紧跟着拖拽条"
    );
}

#[test]
fn splitters_sit_on_the_grid() {
    assert_eq!(SPLITTER_W as i32, 10, "竖分割条 1 格 = cells(1)");
    assert_eq!(HSPLIT_H as i32, 10, "横拖拽条 1 格 = cells(1)");
    let work = left_work();

    let [left, splitter, panel] = StockApp::work_columns(work, CHART_PANE_W);
    assert_eq!(
        left.max.x - LINE_W,
        splitter.min.x,
        "左栏和分割条没共用那条线"
    );
    assert_eq!(
        splitter.max.x - LINE_W,
        panel.min.x,
        "分割条和右栏没共用那条线"
    );
    assert_eq!(panel.max.x, work.max.x, "右栏没铺到功能区右边");
    assert_eq!(splitter.min.x % 9.0, 0.0, "分割条左边不在竖网格线上");
    assert_eq!(
        (splitter.max.x - LINE_W) % 9.0,
        0.0,
        "分割条右边不在竖网格线上"
    );

    let mut app = blank_app();
    for want in [-1000.0, 0.0, 1000.0, 5000.0] {
        app.split_x = want;
        app.clamp_split(work);
        assert_eq!((app.split_x as i32 - 1) % 9, 0, "左栏宽度不是整数格");
        assert!(app.split_x >= PANE_MIN_W, "左栏低于最小宽度");
        assert!(
            app.split_x <= StockApp::split_max_x(work),
            "左栏超过最大宽度"
        );
        let [_l, _s, panel] = StockApp::work_columns(work, app.split_x);
        assert!(panel.width() >= PANE_MIN_W - LINE_W, "右栏低于最小宽度");
    }
}

#[test]
fn timeline_labels_never_crowd() {
    let mut app = blank_app();
    for scale in [SCALE_MIN, 5, 9, SCALE_MAX] {
        let advance = bar_advance(scale);
        let step = app.time_step(advance);
        let gap = step as f32 * advance;
        assert!(
            gap >= TIME_LABEL_GAP_MIN,
            "scale={scale} 的标注间距只有 {gap}px"
        );
        assert!(
            gap < TIME_LABEL_GAP_MIN + advance,
            "scale={scale} 的标注间隔多留了一格：{gap}px"
        );
    }
}

#[test]
fn status_fields_sit_on_the_grid() {
    use crate::indicator::candle::{INFO_LABELS, INFO_TEMPLATES};

    let ctx = egui::Context::default();
    let mut app = blank_app();
    app.text_engine = HardText::new(&ctx);
    if app.text_engine.is_none() {
        return;
    }
    let mut sx = 0.0;
    let mut fit = 0;
    for i in 0..INFO_LABELS.len() {
        let w = snap_cells_up(
            app.text_width(INFO_LABELS[i])
                + GRID
                + app.text_width(INFO_TEMPLATES[i])
                + 2.0 * TEXT_PAD_X,
        );
        assert_eq!(
            (w as i32 - 1) % 9,
            0,
            "第 {i} 项「{}」宽 {w} 不是整数格 cells(n)",
            INFO_LABELS[i]
        );
        assert!(w >= cells(1), "第 {i} 项比一格还窄");
        if sx + w > CHART_PANE_W {
            break;
        }
        sx += w - LINE_W;
        fit += 1;
    }

    assert_eq!(fit, 6, "默认左栏一排放得下的项数变了：{fit} 项");
    assert!(sx + LINE_W <= CHART_PANE_W, "状态带排出来的宽度超过了左栏");

    for scale in [SCALE_MIN, 5, SCALE_MAX] {
        let advance = bar_advance(scale);
        assert!(app.time_step(advance) as f32 * advance >= TIME_LABEL_GAP_MIN);
    }
}

#[test]
fn volume_text_uses_wan_and_yi() {
    use crate::indicator::volume::fmt_volume;
    assert_eq!(fmt_volume(2.0e12), "2兆");
    assert_eq!(fmt_volume(1.5e8), "2亿");
    assert_eq!(fmt_volume(1.0e8), "1亿");
    assert_eq!(fmt_volume(9.99e7), "9990万");
    assert_eq!(fmt_volume(2.3456e7), "2346万");
    assert_eq!(fmt_volume(1.0e4), "1.0万");
    assert_eq!(fmt_volume(9999.0), "9999");
    assert_eq!(fmt_volume(0.0), "0");

    for (v, s) in [(9.999e7, "9999万"), (1.0e12, "1兆"), (9.9e5, "99.0万")] {
        assert!(
            crate::font::text_width(&fmt_volume(v)) <= VALUE_COLS_MAX * 9,
            "{s} 比纵轴留给数值的 7 格还宽"
        );
    }
}

#[test]
fn bar_dates_skip_weekends() {
    let bars = data::random_walk::generate(BAR_SEED, BAR_COUNT, BAR_START);
    assert_eq!(bars[0].date, data::random_walk::Date::new(2024, 1, 2));

    assert_eq!(bars[3].date, data::random_walk::Date::new(2024, 1, 5));
    assert_eq!(bars[4].date, data::random_walk::Date::new(2024, 1, 8));
    assert_eq!(bars[5].date, data::random_walk::Date::new(2024, 1, 9));
    for w in bars.windows(2) {
        assert!(w[0].date < w[1].date, "日期没有严格向后走");
    }
    assert_ne!(
        bars[0].date.y,
        bars[BAR_COUNT - 1].date.y,
        "512 根应该跨过一个新年"
    );
    use crate::indicator::volume::{WAN, YI};
    let max = bars.iter().fold(0.0f64, |a, b| a.max(b.vol));
    assert!(max > YI, "最大成交量 {max} 没上亿，刻度里就只剩「万」了");
    assert!(
        bars.iter()
            .all(|b| b.vol >= 0.0 && b.h >= b.o.max(b.c) && b.l <= b.o.min(b.c))
    );
    assert!(bars.iter().filter(|b| b.vol > WAN).count() > bars.len() / 2);
}

#[test]
fn random_walk_models_a_share_auctions_limits_and_suspensions() {
    use crate::data::random_walk::{BarState, bar_state, limit_prices};

    let bars = data::random_walk::generate(BAR_SEED, BAR_COUNT, BAR_START);
    let mut gaps = 0;
    let mut limit_up = 0;
    let mut limit_down = 0;
    let mut suspended = 0;
    for (i, bar) in bars.iter().enumerate() {
        for v in [bar.o, bar.h, bar.l, bar.c] {
            assert!(
                ((v * 100.0).round() - v * 100.0).abs() < 1e-8,
                "价格没有落在 0.01 的报价单位上：{v}"
            );
        }
        if i > 0 {
            let prev = bars[i - 1].c;
            if (bar.o - prev).abs() > 0.005 {
                gaps += 1;
            }
            let (up, down) = limit_prices(prev);
            if bar.vol > 0.0 {
                assert!(bar.h <= up + 1e-9 && bar.l >= down - 1e-9);
                assert!(bar.o <= up + 1e-9 && bar.o >= down - 1e-9);
                assert!(bar.c <= up + 1e-9 && bar.c >= down - 1e-9);
            }
        }
        match bar_state(&bars, i) {
            BarState::LimitUp => {
                limit_up += 1;
                let prev = bars[i - 1].c;
                assert!((bar.c - limit_prices(prev).0).abs() < 1e-9);
            }
            BarState::LimitDown => {
                limit_down += 1;
                let prev = bars[i - 1].c;
                assert!((bar.c - limit_prices(prev).1).abs() < 1e-9);
            }
            BarState::Suspended => {
                suspended += 1;
                let prev = bars[i - 1].c;
                assert_eq!(bar.vol, 0.0);
                assert_eq!((bar.o, bar.h, bar.l, bar.c), (prev, prev, prev, prev));
            }
            BarState::Normal => {}
        }
    }
    assert!(
        gaps > bars.len() / 4,
        "开盘价几乎全是昨收，没有集合竞价跳空"
    );
    assert!(limit_up > 0, "512 根里没有模拟出涨停");
    assert!(limit_down > 0, "512 根里没有模拟出跌停");
    assert!(suspended > 0, "512 根里没有模拟出停牌");
}

#[test]
fn window_layout_follows_the_client_rect() {
    let base = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(WINDOW_W, WINDOW_H));
    let a = split_areas(base);
    assert_eq!(a[0].min.y, 0.0);
    assert_eq!(a[0].max.y, TITLEBAR_H);
    assert_eq!(a[1].min.y, grid_line(MENU_LINE));
    assert_eq!(a[2].min.y, grid_line(WORK_LINE));
    assert_eq!(a[2].height(), cells(WORK_ROWS), "默认尺寸下功能区高度变了");
    assert_eq!(a[4].min.y, WINDOW_H - STATUS_H);
    assert_eq!(a[4].max.y, WINDOW_H);
    assert_eq!(a[3].height(), TASKBAR_H);
    assert_eq!(a[3].min.y, a[4].min.y - TASKBAR_H + LINE_W);

    assert_eq!(a[2].max.y, a[3].min.y + LINE_W);

    let big = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(WINDOW_W, WINDOW_H + 180.0));
    let b = split_areas(big);
    assert_eq!(b[1].min.y, a[1].min.y);
    assert_eq!(b[2].min.y, a[2].min.y);
    assert_eq!(b[4].max.y, big.max.y, "状态栏没贴着底");
    assert_eq!(b[2].max.y, b[3].min.y + LINE_W);
    assert_eq!(
        b[2].height() - a[2].height(),
        180.0,
        "多出来的高度没给功能区"
    );

    let mut base_app = blank_app();
    base_app.clamp_left_heights(a[2]);
    let mut big_app = blank_app();
    big_app.clamp_left_heights(b[2]);

    assert_eq!(base_app.vol_h, PANE_H_MIN, "图表2 的高度下限是 22 格");
    assert_eq!(base_app.ind_h, PANE_H_MIN, "图表3 的高度下限是 22 格");
    assert_eq!(
        base_app.kline_h,
        cells(WORK_ROWS) - LEFT_FIXED_H - 2.0 * PANE_H_MIN,
        "默认窗口下图表1 应该拿剩下的全部高度"
    );
    assert_eq!((big_app.kline_h as i32 - 1) % 9, 0, "图表1 高度不是整数格");
    assert_eq!(
        big_app.vol_h, PANE_H_MIN,
        "图表2 不该被窗口高度改掉（下限 22 格）"
    );
    assert_eq!(
        big_app.ind_h, PANE_H_MIN,
        "图表3 不该被窗口高度改掉（下限 22 格）"
    );
    assert!(
        big_app.kline_h > base_app.kline_h,
        "窗口拉高后图表1 没跟着变高：{} -> {}",
        base_app.kline_h,
        big_app.kline_h
    );
    let [_k, _bar, _vol, _bar2, ind] = StockApp::left_rows(b[2], big_app.kline_h, big_app.vol_h);
    assert_eq!(ind.max.y, b[2].max.y, "图表3 没铺到左栏底部");
}

#[test]
fn volume_bars_stretch_with_the_pane() {
    let bars: Vec<data::random_walk::Bar> = (0..48)
        .map(|i| data::random_walk::Bar {
            o: 100.0,
            h: 101.0,
            l: 89.0,
            c: 90.0,
            vol: 1.0e7 + (i % 7) as f64 * 1.0e6,
            date: data::random_walk::DATE_START.add_days(i as i32),
        })
        .collect();
    let down = rgb32(DOWN_RGB_DEFAULT);

    let count_down = |grid: &Vec<Vec<Option<egui::Color32>>>, r: egui::Rect| -> usize {
        (r.min.y as usize..r.max.y as usize)
            .map(|y| {
                (r.min.x as usize..r.max.x as usize)
                    .filter(|&x| grid[y][x] == Some(down))
                    .count()
            })
            .sum()
    };

    let mut app = blank_app();
    app.replace_bars(bars);
    for pane in &mut app.panes {
        pane.view.scale = SCALE_MIN;
    }

    let work = left_work();

    app.ind_h = PANE_H_MIN;
    app.vol_h = PANE_H_MIN;
    let (grid_short, [_, _, vol_short, _, _]) = render_left_in(&mut app, work, WINDOW_H as usize);
    let short_px = count_down(&grid_short, vol_short);

    app.vol_h = cells(40);
    let (grid_tall, [_, _, vol_tall, _, ind_tall]) =
        render_left_in(&mut app, work, WINDOW_H as usize);
    let tall_px = count_down(&grid_tall, vol_tall);

    assert_eq!(ind_tall.max.y, work.max.y, "指标区下沿没贴住左栏底部");
    assert!(
        vol_tall.max.y < work.max.y,
        "量区下面还有指标区，不该顶到左栏底"
    );
    assert!(vol_tall.min.y < vol_short.min.y, "量区上沿没跟着上移");
    assert!(
        vol_tall.height() > vol_short.height() + 120.0,
        "量区没被拉高：{} -> {}",
        vol_short.height(),
        vol_tall.height()
    );
    assert!(tall_px > short_px, "量柱没跟着量区一起拉伸");

    let (_, _, vmain, _) = chart_rows(vol_tall, app.panes[PANE_VOLUME].chart_layout());
    let (vplot, _) = frame_rects(vmain);
    let x = vplot.min.x as usize + 6 * bar_advance(SCALE_MIN) as usize;
    let top = vmain.min.y as usize + CHART_PAD as usize;
    let base = vmain.max.y as usize - CHART_PAD as usize;

    let rows: Vec<usize> = (vol_tall.min.y as usize..vol_tall.max.y as usize)
        .filter(|&y| grid_tall[y][x] == Some(down))
        .collect();
    assert!(!rows.is_empty(), "最高的那根量柱没画出来");
    assert!(
        rows[0] == top || rows[0] == top + 1,
        "最高的那根没顶到上边距：第一行 {}，期望 {}",
        rows[0],
        top
    );

    assert_eq!(
        rows[rows.len() - 1],
        base,
        "量柱没落到下边距（量区底 - 8px，0 值那条线上）"
    );
}

#[test]
fn axis_column_is_gray_only_beside_the_plot() {
    let mut app = blank_app();
    let (grid, [kline, _bar, _vol, _bar2, _ind]) = render_left(&mut app);
    let layout = app.panes[PANE_CANDLE].indicator.layout();
    let (_, status, main, time) = chart_rows(kline, layout);
    let info = status.unwrap();
    let tband = time.unwrap();
    let (_plot, axis) = frame_rects(main);
    assert!(info.min.y < main.min.y, "状态带和绘图区之间没留线格");

    for y in main.min.y as usize..main.max.y as usize {
        for x in axis.min.x as usize..axis.max.x as usize {
            assert_ne!(
                grid[y][x],
                Some(C_WORK_BG),
                "价格轴列 ({x}, {y}) 露出了白底：绘图区里的轴灰没铺满"
            );
        }
    }
    let probe_x = axis.min.x as usize + 4;

    for y in info.min.y as usize + 1..main.min.y as usize - 1 {
        assert_eq!(grid[y][probe_x], Some(C_WORK_BG), "状态带右侧露出了灰底");
    }

    for y in tband.min.y as usize + 1..tband.max.y as usize - 1 {
        assert_eq!(
            grid[y][probe_x],
            Some(C_WORK_BG),
            "时间带右侧露出了灰底（不该有灰色区域） y={y}"
        );
    }
}

#[test]
fn builtin_indicators_carry_their_own_source() {
    let builtins = indicator::builtin();
    assert_eq!(
        builtins.len(),
        10,
        "内置指标应该是蜡烛 / 美国线 / 成交量 / MACD / KDJ / RSI / BOLL / OBV / CCI / BIAS 十个"
    );

    let want = [
        "CANDLE(", "OHLC(", "VOL(UP", "DIF:", "RSV:", "RSI1:", "BOLL_UP", "OBVALUE:", "CCIV:",
        "BIAS1:",
    ];
    let bars = data::random_walk::generate(BAR_SEED, 64, BAR_START);
    for (i, ind) in builtins.iter().enumerate() {
        let src = indicator::script_source(ind.name());
        assert!(!src.is_empty(), "{} 没加载指标代码", ind.name());
        assert!(src.contains(want[i]), "{} 的指标脚本不对", ind.name());
        assert_eq!(
            src.lines().next(),
            Some(format!("# {}", ind.name()).as_str()),
            "{} 的指标代码首行不是指标名称注释",
            ind.name()
        );
        assert!(
            src.contains("STATUS_SHOW(1);"),
            "{} 没有状态栏配置",
            ind.name()
        );
        assert!(
            src.contains("TIME_SHOW(1);"),
            "{} 没有时间轴配置",
            ind.name()
        );
        assert!(src.contains("AXIS_"), "{} 没有纵轴配置", ind.name());
        let program = crate::script::ScriptProgram::parse(&src).expect("指标脚本应能解析");
        let cmds = program
            .compute(&bars, &indicator::Palette::default())
            .expect("指标脚本应能计算");
        assert!(!cmds.is_empty(), "{} 没画出内容", ind.name());
        let path = indicator::script_path(ind.name());
        assert!(
            path.is_file(),
            "{} 的脚本文件不存在：{}",
            ind.name(),
            path.display()
        );
        assert!(
            path.ends_with(indicator::script_file(ind.name())),
            "{} 的脚本文件名不对",
            ind.name()
        );
    }
    assert_eq!(
        (PANE_CANDLE, PANE_VOLUME, PANE_MACD),
        (0, 1, 2),
        "三个面板槽位对不上"
    );
}

#[test]
fn pane_range_scans_only_visible_data_cmds() {
    use crate::indicator::DrawCmd;
    let cmds = vec![
        DrawCmd::Candle {
            i: 0,
            o: 1.0,
            h: 3.0,
            l: 0.5,
            c: 2.0,
            color: 0x000000,
        },
        DrawCmd::Candle {
            i: 1,
            o: 2.0,
            h: 9.0,
            l: 1.0,
            c: 3.0,
            color: 0x000000,
        },
        DrawCmd::Bar {
            i: 1,
            v0: -50.0,
            v1: 100.0,
            color: 0x000000,
            hollow: false,
        },
        DrawCmd::HLine {
            p: 999.0,
            color: 0x000000,
            w: 1,
        },
        DrawCmd::Text {
            i: 1,
            p: -999.0,
            s: "x".to_owned(),
            color: 0x000000,
        },
    ];

    assert_eq!(
        pane_range(&cmds, 0, 1),
        (0.5, 3.0),
        "只该扫到第一根蜡烛的高低"
    );

    assert_eq!(
        pane_range(&cmds, 0, 2),
        (-50.0, 100.0),
        "柱子的极值和蜡烛的极值没一起算进来"
    );

    assert_eq!(pane_range(&cmds, 5, 9), (0.0, 0.0), "空区间该返回 (0, 0)");
}

#[test]
fn pane_compute_runs_only_when_dirty() {
    let mut app = blank_app();
    app.replace_bars(data::random_walk::generate(BAR_SEED, 8, BAR_START));
    let pane = PANE_CANDLE;

    assert!(app.panes[pane].dirty, "新面板应该一开始就是脏的");
    app.ensure_pane_computed(pane);
    assert!(!app.panes[pane].cmds.is_empty(), "首帧应该算出了指令");
    assert!(!app.panes[pane].dirty, "算完应该清脏");

    let ptr = app.panes[pane].cmds.as_ptr();
    let len = app.panes[pane].cmds.len();
    app.ensure_pane_computed(pane);
    assert_eq!(app.panes[pane].cmds.as_ptr(), ptr, "干净时不该重算");
    assert_eq!(app.panes[pane].cmds.len(), len);

    app.mark_panes_dirty();
    assert!(
        app.panes.iter().all(|p| p.dirty),
        "数据换代应该把所有面板置脏"
    );
    app.ensure_pane_computed(pane);
    assert!(!app.panes[pane].dirty);

    app.up_color = (0x11, 0x22, 0x33);
    app.down_color = (0x44, 0x55, 0x66);
    app.sync_pane_palette();
    assert!(app.panes[pane].dirty, "换色应该置脏");
    app.ensure_pane_computed(pane);
    let (up, down) = (
        indicator::rgb_u32(app.up_color),
        indicator::rgb_u32(app.down_color),
    );
    let mut candles = 0;
    for cmd in &app.panes[pane].cmds {
        if let crate::indicator::DrawCmd::Candle { color, .. } = cmd {
            assert!(*color == up || *color == down, "蜡烛没换成新的涨跌色");
            candles += 1;
        }
    }
    assert_eq!(candles, app.bars.len(), "每根 K 线都该有一根蜡烛指令");
}

#[test]
fn chart_rows_share_lines_and_snap_to_the_grid() {
    let work = left_work();
    let [kline, _bar, volume, _bar2, indicator] =
        StockApp::left_rows(work, KLINE_H_DEFAULT, VOL_H_DEFAULT);

    let on_grid = |y: f32, area: egui::Rect| (y - area.min.y) % 9.0 == 0.0;

    let (title, status, main, time) = chart_rows(kline, crate::indicator::ChartLayout::new(2, 2));
    let (status, time) = (status.unwrap(), time.unwrap());
    assert_eq!(title.height(), cells(PANE_TITLE_ROWS), "标题栏不是 2 格");
    assert_eq!(
        title.max.y - LINE_W,
        status.min.y,
        "标题栏和状态带没共用那条线"
    );
    assert_eq!(status.height(), cells(2), "状态带不是 2 格");
    assert_eq!(status.max.y, main.min.y, "状态带和绘图区没共用那条线");
    assert_eq!(main.max.y, time.min.y, "绘图区和时间带没共用那条线");
    assert_eq!(time.height(), cells(2), "时间带不是 2 格");

    assert!(on_grid(title.min.y, kline), "标题栏上沿没落在网格线上");
    assert!(on_grid(status.min.y, kline), "状态带上沿没落在网格线上");
    assert!(
        on_grid(main.min.y - LINE_W, kline),
        "绘图区上沿没落在网格线上"
    );
    assert!(on_grid(time.min.y, kline), "时间带上沿没落在网格线上");

    assert_eq!(
        time.max.y,
        chart_bar(kline).min.y + LINE_W,
        "时间带和状态栏没共用那条线"
    );
    assert_eq!(
        chart_bar(kline).max.y,
        kline.max.y,
        "图表窗状态栏没铺到面板底边"
    );
    assert!(
        on_grid(chart_bar(kline).min.y, kline),
        "状态栏上沿没落在网格线上"
    );
    println!(
        "图表1 {}px = 标题栏 cells(2) + 状态带 cells(2) + 绘图区 {} + 时间带 cells(2)",
        kline.height(),
        main.height(),
    );

    let (vtitle, vstatus, vmain, vtime) = chart_rows(volume, crate::indicator::ChartLayout::NONE);
    assert!(vstatus.is_none() && vtime.is_none());
    assert_eq!(vtitle.max.y, vmain.min.y, "标题栏和绘图区没共用那条线");
    assert_eq!(
        vmain.max.y,
        chart_bar(volume).min.y,
        "绘图区和状态栏没共用那条线"
    );
    assert_eq!(
        chart_bar(volume).max.y,
        volume.max.y,
        "图表窗状态栏没铺到面板底边"
    );
    println!(
        "图表2 {}px = 标题栏 cells(2) + 绘图区 {}",
        volume.height(),
        vmain.height(),
    );

    let (_, mstatus, mmain, mtime) = chart_rows(indicator, chart_rows_layout_with_status());
    assert!(mstatus.is_some() && mtime.is_none());
    assert_eq!(
        mmain.max.y,
        chart_bar(indicator).min.y,
        "绘图区和状态栏没共用那条线"
    );
    assert_eq!(
        chart_bar(indicator).max.y,
        indicator.max.y,
        "图表窗状态栏没铺到面板底边"
    );
    println!(
        "图表3 {}px = 标题栏 + 状态带 + 绘图区 {}px",
        indicator.height(),
        mmain.height()
    );
}

fn chart_rows_layout_with_status() -> crate::indicator::ChartLayout {
    crate::indicator::ChartLayout::with_status(2)
}

fn pane_btn_widths() -> Vec<f32> {
    let w = |s: &str| snap_cells_up(crate::font::text_width(s) as f32 + 2.0 * TEXT_PAD_X);
    vec![w("切换"), w("代码")]
}

#[test]
fn pane_titlebar_holds_two_buttons_on_the_grid() {
    let work = left_work();
    let [kline, _bar, _vol, _bar2, _ind] =
        StockApp::left_rows(work, KLINE_H_DEFAULT, VOL_H_DEFAULT);
    let (title, _info, _main, _time) = chart_rows(kline, crate::indicator::ChartLayout::new(2, 2));
    let buttons = StockApp::pane_button_rects(title, &pane_btn_widths());

    assert_eq!(pane_btn_widths(), vec![cells(6), cells(6)]);
    assert_eq!(buttons[0].width(), cells(6));
    assert_eq!(buttons[0].height(), PANE_BTN_H);
    assert_eq!(
        buttons[1].min.x,
        buttons[0].max.x - LINE_W,
        "两个按钮没共用中间那条线"
    );
    assert_eq!(
        buttons[1].max.x, kline.max.x,
        "最右边的按钮没贴着 Pane 右边框"
    );
    for b in &buttons {
        for x in [b.min.x, b.max.x - LINE_W] {
            assert_eq!((x - kline.min.x) % 9.0, 0.0, "按钮竖边框 {x} 不在网格线上");
        }
        assert_eq!(b.min.y, kline.min.y, "按钮上边框没压在 Pane 顶边上");
        assert_eq!(b.max.y, title.max.y, "按钮下边框没压在标题栏下沿");
    }
    println!(
        "标题栏 {}px，两个按钮各 {}x{}",
        title.height(),
        buttons[0].width(),
        PANE_BTN_H
    );
}

#[test]
fn the_window_title_seam_is_a_single_line() {
    let win = egui::Rect::from_min_size(egui::pos2(13.0, 5.0), egui::vec2(443.0, 233.0));
    assert!(win.width() % 9.0 != 1.0, "这块窗口得是「非整格」的");

    let seam = win.min.y + grid_line(WIN_TITLE_ROWS);
    assert_eq!(
        seam,
        win.min.y + WIN_TITLE_H - LINE_W,
        "标题栏下沿那一行算不对"
    );

    let body = ManagedWin::body(win);
    assert_eq!(body.min.y, seam, "Pane 标题栏没压在窗口标题栏下沿那条线上");
    let (title, _, _, _) = chart_rows(body, crate::indicator::ChartLayout::new(2, 2));
    assert_eq!(title.min.y, seam);
    assert_eq!(PANE_TITLE_H, WIN_TITLE_H, "Pane 标题栏和窗口标题栏得一样高");
    assert_eq!(PANE_BTN_H, WIN_TITLE_H, "那几个按钮和标题栏得一样高");
    for b in StockApp::pane_button_rects(title, &pane_btn_widths()) {
        assert_eq!(b.min.y, seam, "按钮上边框没和标题栏下沿重合（会错成 2px）");
        assert_eq!(
            b.max.y - LINE_W,
            title.max.y - LINE_W,
            "按钮下边框没和 Pane 标题栏下沿重合"
        );
    }

    assert!(
        seam - LINE_W > win.min.y,
        "那条线贴着窗口顶边，上下没留出面"
    );
    assert!(
        seam + LINE_W < title.max.y - LINE_W,
        "那条线上下没留出面，成了 2px"
    );
}

#[test]
fn pane_menu_lists_builtins_and_swaps_the_indicator() {
    let names = indicator::builtin_names();
    assert_eq!(
        names.join(","),
        "蜡烛图,美国线,成交量,MACD(12,26,9),KDJ(9,3,3),RSI(6,12,24),BOLL(20,2),OBV,CCI(14),BIAS(6,12,24)",
        "内置指标的名字 / 顺序变了"
    );
    let mut app = blank_app();
    let work = left_work();
    let [kline, _bar, _vol, _bar2, _ind] =
        StockApp::left_rows(work, KLINE_H_DEFAULT, VOL_H_DEFAULT);
    let (title, _info, _main, _time) = chart_rows(kline, crate::indicator::ChartLayout::new(2, 2));
    let buttons = StockApp::pane_button_rects(title, &pane_btn_widths());
    app.pane_menu = Some(PANE_CANDLE);
    app.pane_menu_anchor = buttons[0];
    let panel = app.pane_menu_rect(names.len(), WINDOW_H).unwrap();
    assert_eq!(
        panel.max.x, buttons[0].max.x,
        "面板右边缘没和「切换」按钮对齐"
    );
    assert_eq!(
        panel.min.y,
        buttons[0].max.y - LINE_W,
        "面板上沿没压在标题栏下沿那条线上"
    );
    assert_eq!(
        panel.height(),
        cells(PANE_MENU_ROW_ROWS * names.len() as i32)
    );
    assert_eq!(
        (panel.max.x - PANE_MENU_W - kline.min.x) % 9.0,
        0.0,
        "面板左边不在网格线上"
    );

    assert_eq!(app.panes[PANE_CANDLE].indicator.name(), "蜡烛图");
    app.set_pane_indicator(PANE_CANDLE, 2);
    assert_eq!(app.panes[PANE_CANDLE].indicator.name(), "成交量");
    assert!(app.panes[PANE_CANDLE].dirty, "换指标之后没置脏");
    app.replace_bars(data::random_walk::generate(BAR_SEED, 8, BAR_START));
    app.ensure_pane_computed(PANE_CANDLE);
    assert!(
        app.panes[PANE_CANDLE]
            .cmds
            .iter()
            .any(|c| matches!(c, crate::indicator::DrawCmd::Bar { .. })),
        "换过指标之后算出来的不是成交量柱"
    );
    assert!(
        !app.panes[PANE_CANDLE]
            .cmds
            .iter()
            .any(|c| matches!(c, crate::indicator::DrawCmd::Candle { .. }))
    );
}

#[test]
fn pane_menu_flips_up_when_it_would_overflow_the_window() {
    let mut app = blank_app();
    let work = left_work();
    let [kline, _b0, _vol, _b1, ind] = StockApp::left_rows(work, KLINE_H_DEFAULT, VOL_H_DEFAULT);
    let names = indicator::builtin_names();
    let h = cells(PANE_MENU_ROW_ROWS * names.len() as i32);
    let w = |s: &str| snap_cells_up(crate::font::text_width(s) as f32 + 2.0 * TEXT_PAD_X);
    let widths = [w("切换"), w("代码")];

    let (title, _, _, _) = chart_rows(kline, crate::indicator::ChartLayout::new(2, 2));
    app.pane_menu_anchor = StockApp::pane_button_rects(title, &widths)[0];
    let panel = app.pane_menu_rect(names.len(), WINDOW_H).unwrap();
    assert_eq!(
        panel.min.y,
        app.pane_menu_anchor.max.y - LINE_W,
        "没压在标题栏下沿那条线上"
    );
    assert_eq!((panel.min.y) % PITCH, 0.0, "面板上沿不在网格线上");
    assert_eq!((panel.max.y - LINE_W) % PITCH, 0.0, "面板下沿不在网格线上");
    assert_eq!(panel.height(), h);

    let (title, _, _, _) = chart_rows(ind, crate::indicator::ChartLayout::new(2, 2));
    app.pane_menu_anchor = StockApp::pane_button_rects(title, &widths)[0];
    let anchor = app.pane_menu_anchor;
    let panel = app.pane_menu_rect(names.len(), anchor.max.y).unwrap();
    assert_eq!(
        panel.max.y - LINE_W,
        anchor.min.y,
        "翻上去之后没压在标题栏上沿那条线上"
    );
    assert_eq!(panel.height(), h);
    assert_eq!((panel.min.y) % PITCH, 0.0, "翻上去之后面板上沿不在网格线上");
    assert_eq!(
        (panel.max.y - LINE_W) % PITCH,
        0.0,
        "翻上去之后面板下沿不在网格线上"
    );
    assert!(panel.max.y <= anchor.max.y, "翻上去之后盖住了按钮那一行");
}

#[test]
fn pane_title_holds_switch_and_code_buttons_on_the_grid() {
    let work = left_work();
    let [kline, _b0, _vol, _b1, _ind] = StockApp::left_rows(work, KLINE_H_DEFAULT, VOL_H_DEFAULT);
    let (title, _, _, _) = chart_rows(kline, crate::indicator::ChartLayout::new(2, 2));
    let w = |s: &str| snap_cells_up(crate::font::text_width(s) as f32 + 2.0 * TEXT_PAD_X);
    let widths = vec![w("切换"), w("代码")];
    assert_eq!(
        widths,
        vec![cells(6), cells(6)],
        "两个按钮各按 2 个汉字 + 左右 1 格量"
    );
    let buttons = StockApp::pane_button_rects(title, &widths);
    assert_eq!(buttons.len(), 2);
    assert_eq!(
        buttons[1].max.x, kline.max.x,
        "最右边的「代码」没贴着 Pane 右边框"
    );
    assert_eq!(
        buttons[1].min.x,
        buttons[0].max.x - LINE_W,
        "「代码」和「切换」没共用中间那条线"
    );
    for (i, b) in buttons.iter().enumerate() {
        for x in [b.min.x, b.max.x - LINE_W] {
            assert_eq!(
                (x - kline.min.x) % 9.0,
                0.0,
                "按钮 {i} 的竖边框不在网格线上"
            );
        }
        assert_eq!(b.height(), PANE_BTN_H);
    }
}

#[test]
fn pane_title_has_a_round_table_button_left_of_the_action_buttons() {
    let work = left_work();
    let [kline, _b0, _vol, _b1, _ind] = StockApp::left_rows(work, KLINE_H_DEFAULT, VOL_H_DEFAULT);
    let (title, _, _, _) = chart_rows(kline, crate::indicator::ChartLayout::new(2, 2));
    let width = |text: &str| snap_cells_up(crate::font::text_width(text) as f32 + 2.0 * TEXT_PAD_X);
    let widths = [
        width(StockApp::PANE_LEAD),
        width(StockApp::PANE_TABS[0]),
        width(StockApp::PANE_TABS[1]),
    ];
    let rects = menu_item_rects(title, &widths, MenuAlign::Right);
    assert_eq!(rects.len(), 3);
    assert_eq!(rects[0].max.x, rects[1].min.x + LINE_W);
    assert_eq!(rects[1].max.x, rects[2].min.x + LINE_W);
    assert_eq!(rects[2].max.x, title.max.x);
    assert_eq!(rects[0].width(), width(StockApp::PANE_LEAD));
}

#[test]
fn indicator_script_edits_apply_without_compilation() {
    let mut app = blank_app();
    app.replace_bars(data::random_walk::generate(BAR_SEED, 64, BAR_START));
    app.open_source_window(PANE_CANDLE);
    app.apply_script_source("LINE(C,TEXT);".to_owned());
    app.ensure_pane_computed(PANE_CANDLE);
    assert!(app.panes[PANE_CANDLE].script_error.is_none());
    assert!(matches!(
        app.panes[PANE_CANDLE].cmds.first(),
        Some(crate::indicator::DrawCmd::Polyline { .. })
    ));

    app.apply_script_source("LINE(C,TEXT);\nBROKEN(".to_owned());
    app.ensure_pane_computed(PANE_CANDLE);
    assert!(app.panes[PANE_CANDLE].script_error.is_some());
    assert!(matches!(
        app.panes[PANE_CANDLE].cmds.first(),
        Some(crate::indicator::DrawCmd::Polyline { .. })
    ));
}

#[test]
fn indicator_script_functions_compute_series() {
    let bars = data::random_walk::generate(BAR_SEED, 8, BAR_START);
    let program =
        crate::script::ScriptProgram::parse("X: MA(C,2);\nLINE(X,MA1);").expect("脚本应能解析");
    let cmds = program
        .compute(&bars, &indicator::Palette::default())
        .expect("脚本应能计算");
    let Some(crate::indicator::DrawCmd::Polyline { pts, .. }) = cmds.first() else {
        panic!("LINE 没生成折线");
    };
    assert_eq!(pts.len(), bars.len() - 1);
}

#[test]
fn indicator_script_customizes_bands_axes_styles_and_status() {
    use crate::indicator::{ChartCtx, DrawCmd};

    let bars = data::random_walk::generate(BAR_SEED, 64, BAR_START);
    let source = concat!(
        "STATUS(1);\n",
        "TIME(1);\n",
        "AXIS_FIXED(0,20,50,80,100);\n",
        "CANDLE(SYSTEM_COLOR(UP),COLOR(0x00FF00),2);\n",
        "LINE(C,COLOR(\"#112233\"),3);\n",
        "HIST(C-O,AUTO,1,2);\n",
        "STATUS_FIELD(\"C\",C,SYSTEM_COLOR(TEXT),\"0.00\");\n",
    );
    let program = crate::script::ScriptProgram::parse(source).expect("自定义脚本应能解析");
    let render = program
        .render(&bars, &indicator::Palette::default())
        .expect("自定义脚本应能计算");

    assert_eq!(render.settings.show_status, Some(true));
    assert_eq!(render.settings.show_time, Some(true));
    assert_eq!(render.settings.show_axis, Some(true));
    assert_eq!(
        render.layout(crate::indicator::ChartLayout::NONE),
        crate::indicator::ChartLayout::new(2, 2)
    );
    assert!(render.cmds.iter().any(|cmd| matches!(
        cmd,
        DrawCmd::StyledCandle {
            color: 0xFF0000,
            w: 2,
            ..
        }
    )));
    assert!(render.cmds.iter().any(|cmd| matches!(
        cmd,
        DrawCmd::Polyline {
            color: 0x112233,
            w: 3,
            ..
        }
    )));
    assert!(render.cmds.iter().any(|cmd| matches!(
        cmd,
        DrawCmd::StyledBar {
            hollow: true,
            w: 2,
            ..
        }
    )));

    let ctx = ChartCtx {
        bars: &bars,
        cmds: &render.cmds,
        from: 0,
        to: bars.len(),
        visible: bars.len(),
        advance: 1.0,
        time_step: 5,
        lo: 0.0,
        hi: 100.0,
        cursor_bar: None,
        cursor_index: None,
        cursor_value: None,
    };
    assert_eq!(render.value_range(10.0, 90.0), (0.0, 100.0));
    let axis = render.axis(&ctx).expect("固定纵轴应有指令");
    assert_eq!(
        axis.iter()
            .filter(|cmd| matches!(cmd, DrawCmd::AxisText { .. }))
            .count(),
        5
    );
    let status = render.status(&ctx).expect("自定义状态栏应有指令");
    assert!(matches!(
        status.as_slice(),
        [DrawCmd::StatusField { label, .. }] if label == "C"
    ));
    assert!(render.time(&ctx).is_some_and(|cmds| !cmds.is_empty()));
}

#[test]
fn indicator_script_can_hide_all_three_bands() {
    let bars = data::random_walk::generate(BAR_SEED, 32, BAR_START);
    let source = "STATUS(0);\nTIME(0);\nAXIS(0);\nLINE(C,TEXT);\n";
    let program = crate::script::ScriptProgram::parse(source).expect("开关脚本应能解析");
    let render = program
        .render(&bars, &indicator::Palette::default())
        .expect("开关脚本应能计算");
    assert_eq!(
        render.layout(crate::indicator::ChartLayout::new(2, 2)),
        crate::indicator::ChartLayout::NONE
    );
    assert_eq!(render.settings.show_axis, Some(false));
}

#[test]
fn indicator_script_accepts_rrggbb_literals_and_display_switches() {
    use crate::indicator::DrawCmd;

    let bars = data::random_walk::generate(BAR_SEED, 32, BAR_START);
    let source = concat!(
        "# comment\n",
        "STATUS_SHOW(1);\n",
        "TIME_HIDE();\n",
        "AXIS_FIXED(0,50,100);\n",
        "LINE(C,#112233,2);\n",
        "HLINE(50,#FF0000,1);\n",
        "STATUS_FIELD(\"C\",C,#445566,\"0.00\");\n",
    );
    let program =
        crate::script::ScriptProgram::parse(source).expect("RRGGBB literals should parse");
    let render = program
        .render(&bars, &indicator::Palette::default())
        .expect("script should render");
    assert_eq!(render.settings.show_status, Some(true));
    assert_eq!(render.settings.show_time, Some(false));
    assert!(render.cmds.iter().any(|cmd| matches!(
        cmd,
        DrawCmd::Polyline {
            color: 0x112233,
            w: 2,
            ..
        }
    )));
    assert!(render.cmds.iter().any(|cmd| matches!(
        cmd,
        DrawCmd::HLine {
            color: 0xFF0000,
            ..
        }
    )));
    let ctx = crate::indicator::ChartCtx {
        bars: &bars,
        cmds: &render.cmds,
        from: 0,
        to: bars.len(),
        visible: bars.len(),
        advance: 1.0,
        time_step: 5,
        lo: 0.0,
        hi: 100.0,
        cursor_bar: None,
        cursor_index: None,
        cursor_value: None,
    };
    let status = render.status(&ctx).expect("status fields");
    assert!(matches!(
        status.as_slice(),
        [DrawCmd::StatusField {
            color: 0x445566,
            ..
        }]
    ));
}

#[test]
fn explicit_display_hide_wins_over_later_field_declarations() {
    let bars = data::random_walk::generate(BAR_SEED, 8, BAR_START);
    let source = concat!(
        "STATUS_HIDE();\n",
        "AXIS_HIDE();\n",
        "STATUS_FIELD(\"C\",C,#112233,\"0.00\");\n",
        "AXIS_FIXED(0,50,100);\n",
        "LINE(C,#445566);\n",
    );
    let program = crate::script::ScriptProgram::parse(source).expect("script should parse");
    let render = program
        .render(&bars, &indicator::Palette::default())
        .expect("script should render");
    assert_eq!(render.settings.show_status, Some(false));
    assert_eq!(render.settings.show_axis, Some(false));
}

#[test]
fn indicator_script_draws_pagoda_and_close_lines() {
    use crate::indicator::DrawCmd;

    let bars = data::random_walk::generate(BAR_SEED, 16, BAR_START);
    let program = crate::script::ScriptProgram::parse("PAGODA();\nCLOSE();\n")
        .expect("宝塔线和收盘线应能解析");
    let cmds = program
        .compute(&bars, &indicator::Palette::default())
        .expect("宝塔线和收盘线应能计算");
    assert_eq!(
        cmds.iter()
            .filter(|cmd| matches!(cmd, DrawCmd::Pagoda { .. }))
            .count(),
        bars.len()
    );
    assert_eq!(
        cmds.iter()
            .filter(|cmd| matches!(cmd, DrawCmd::CloseLine { .. }))
            .count(),
        bars.len()
    );
}

#[test]
fn indicator_script_customizes_zero_axis_and_two_sided_bars() {
    use crate::indicator::DrawCmd;

    let bars = data::random_walk::generate(BAR_SEED, 32, BAR_START);
    let program =
        crate::script::ScriptProgram::parse("ZERO(COLOR(0x112233),2);\nHIST(C-O,UP,DOWN,1,3);\n")
            .expect("双侧柱状图应能解析");
    let render = program
        .render(&bars, &indicator::Palette::default())
        .expect("双侧柱状图应能计算");
    assert!(render.cmds.iter().any(|cmd| matches!(
        cmd,
        DrawCmd::HLine {
            color: 0x112233,
            w: 2,
            ..
        }
    )));
    let palette = indicator::Palette::default();
    let styled: Vec<(u32, bool, u32)> = render
        .cmds
        .iter()
        .filter_map(|cmd| match cmd {
            DrawCmd::StyledBar {
                color, hollow, w, ..
            } => Some((*color, *hollow, *w)),
            _ => None,
        })
        .collect();
    assert_eq!(styled.len(), 32);
    assert!(
        styled
            .iter()
            .all(|(_, hollow, width)| *hollow && *width == 3)
    );
    assert!(
        styled
            .iter()
            .all(|(color, _, _)| [palette.candle_up, palette.candle_down].contains(color))
    );
}

#[test]
fn every_builtin_indicator_script_renders_without_errors() {
    let bars = data::random_walk::generate(BAR_SEED, 96, BAR_START);
    let palette = indicator::Palette::default();
    for name in indicator::builtin_names() {
        let source = indicator::script_source(&name);
        let program = crate::script::ScriptProgram::parse(&source)
            .unwrap_or_else(|error| panic!("{name} 脚本解析失败：{error}"));
        let render = program
            .render(&bars, &palette)
            .unwrap_or_else(|error| panic!("{name} 脚本计算失败：{error}"));
        assert!(!render.cmds.is_empty(), "{name} 脚本没有输出绘图命令");
    }
}

#[test]
fn zero_indicator_periods_fall_back_to_one_instead_of_infinity() {
    use crate::indicator::DrawCmd;
    use crate::indicator::{Indicator, kdj::Kdj, macd::Macd};

    let bars = data::random_walk::generate(BAR_SEED, 32, BAR_START);
    let macd = Macd::new(0, 0, 0).compute(&bars);
    let kdj = Kdj::new(0, 0, 0).compute(&bars);
    for commands in [&macd, &kdj] {
        for command in commands {
            if let DrawCmd::Polyline { pts, .. } = command {
                assert!(pts.iter().all(|(_, value)| value.is_finite()));
            }
        }
    }
}

#[test]
fn indicator_code_window_shows_the_script() {
    let ctx = egui::Context::default();
    configure_tessellation(&ctx);
    ctx.set_pixels_per_point(1.0);
    let mut app = blank_app();
    app.text_engine = HardText::new(&ctx);
    app.replace_bars(data::random_walk::generate(BAR_SEED, 64, BAR_START));
    app.open_source_window(PANE_CANDLE);
    assert!(app.win_mgr.source_open() && app.source.pane == PANE_CANDLE);
    let visible = SRC_VISIBLE_LINES;
    let total = app.source.editor.buf.lines.len();
    assert_eq!(
        visible, 27,
        "正文区一屏应该放得下 27 行（54 格 / 每行 2 格）"
    );
    assert!(total <= visible, "示例指标脚本应完整显示在一屏内");
    assert_eq!(app.source.editor.scroll, 0, "刚打开应该从第 1 行开始");
    assert_eq!(app.source.editor.max_scroll(), 0);

    let joined = app.source.editor.buf.lines.join("\n");
    assert!(joined.contains("CANDLE("), "指标代码里没有蜡烛绘制命令");
    assert!(joined.contains("MA(C,5)"), "指标代码里没有均线函数");
    assert!(
        !joined.contains("impl Indicator"),
        "编辑器还在显示 Rust 源码"
    );
    assert!(!joined.contains('\t'), "Tab 没换成 4 个空格");
    assert!(!joined.contains('\r'), "行尾的换行符没清掉");
    for name in ["成交量", "MACD(12,26,9)"] {
        assert!(!indicator::script_source(name).is_empty());
    }

    app.source.editor.scroll = 9999;
    let layout = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(WINDOW_W, WINDOW_H));
    let mut raw = egui::RawInput::default();
    raw.screen_rect = Some(layout);
    let mut out = ctx.run_ui(raw, |ui| {
        app.draw_frame(ui);
    });
    let shapes = std::mem::take(&mut out.shapes);
    absorb_textures(&mut out.textures_delta);
    let grid = rasterize_into(
        &ctx.tessellate(shapes, 1.0),
        WINDOW_W as usize,
        WINDOW_H as usize,
    );
    assert_eq!(app.source.editor.scroll, 0, "短脚本不应产生滚动");
    let src = app
        .win_mgr
        .wins
        .iter()
        .position(|w| w.kind == WinKind::Source)
        .expect("源码窗没建出来");
    let win = app.win_mgr.wins[src].w.rect();
    let (x0, y0) = (win.min.x as usize, win.min.y as usize);
    let (px1, py1) = (win.max.x as usize - 1, win.max.y as usize - 1);
    assert_eq!(grid[y0][x0 + 300], Some(C_CTRL_BORDER), "面板上边框不对");
    assert_eq!(grid[py1][x0 + 300], Some(C_CTRL_BORDER), "面板下边框不对");
    assert_eq!(grid[y0 + 200][x0], Some(C_CTRL_BORDER), "面板左边框不对");
    assert_eq!(grid[y0 + 200][px1], Some(C_CTRL_BORDER), "面板右边框不对");
    assert!(is_title_face(grid[y0 + 4][x0 + 300]), "标题栏底色不对");
    let gutter_line = x0 + grid_line(SRC_GUTTER_COLS) as usize;
    assert_eq!(
        grid[y0 + 200][gutter_line],
        Some(C_CTRL_BORDER),
        "行号区右边那条线不对"
    );
    assert_eq!(
        grid[y0 + 200][gutter_line - 2],
        Some(C_PANEL_BG),
        "行号区底色不对"
    );

    assert_eq!(
        grid[y0 + SRC_BODY_TOP as usize][x0 + 300],
        Some(C_CTRL_BORDER),
        "标题栏下沿那条线不对"
    );

    let empty_y = y0 + SRC_BODY_TOP as usize + SRC_VISIBLE_LINES * SRC_ROW_PITCH as usize + 1;
    assert_eq!(grid[empty_y][x0 + 300], Some(C_WORK_BG), "代码区底色不对");
}

#[test]
fn source_window_moves_by_pixels_and_keeps_its_own_grid() {
    assert_eq!(SRC_W as i32, 721, "面板宽 = cells(80)");
    assert_eq!(SRC_H as i32, 550, "面板高 = cells(61)");
    assert_eq!(SRC_VISIBLE_LINES, 27, "一屏 27 行 = 54 格 / 每行 2 格");
    let layout = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(WINDOW_W, WINDOW_H));
    let mut app = blank_app();
    app.open_source_window(PANE_CANDLE);
    let i = app
        .win_mgr
        .wins
        .iter()
        .position(|w| w.kind == WinKind::Source)
        .expect("源码编辑窗没建出来");
    app.source.editor.buf.lines = (0..100).map(|k| format!("{k}")).collect();
    assert_eq!(app.source.editor.max_scroll(), 100 - 27);

    assert_eq!(
        app.win_mgr.wins[i].w.pos,
        egui::Pos2::ZERO,
        "源码窗还没摆进工作区前应停在默认原点"
    );

    app.win_mgr.wins[i].w.pos = egui::pos2(203.4, 97.6);
    app.win_mgr.wins[i].w.clamp_into(layout);
    assert_eq!(
        app.win_mgr.wins[i].w.pos,
        egui::pos2(203.4, 97.6),
        "窗口位置被吸网格了"
    );

    app.win_mgr.wins[i].w.pos = egui::pos2(-40.0, WINDOW_H + 40.0);
    app.win_mgr.wins[i].w.clamp_into(layout);
    assert_eq!(app.win_mgr.wins[i].w.pos, egui::pos2(0.0, WINDOW_H - SRC_H));

    app.win_mgr.wins[i].w.size = egui::vec2(WINDOW_W + 500.0, WINDOW_H + 500.0);
    app.win_mgr.wins[i].w.clamp_into(layout);
    assert_eq!(app.win_mgr.wins[i].w.size, egui::vec2(WINDOW_W, WINDOW_H));

    app.win_mgr.wins[i].w.pos = egui::pos2(203.4, 97.6);
    for k in 0..9 {
        let band = egui::Rect::from_min_size(
            app.win_mgr.wins[i].w.pos + egui::vec2(k as f32, 0.0),
            egui::vec2(SRC_W, WIN_TITLE_H),
        );
        assert_eq!(
            text_left_x(band.min.x, band.min.x) - band.min.x,
            TEXT_INSET_X,
            "窗口里的文字没跟着窗口自己那套网格走"
        );
    }
}

#[test]
fn source_window_text_keeps_a_cell_on_both_sides() {
    let gutter_line = grid_line(SRC_GUTTER_COLS);
    let bar_line = grid_line(SRC_COLS - 1);

    let num_w = text_width("999") as f32;
    let num_left = text_right_x(gutter_line, 0.0) - num_w;
    assert_eq!(
        ((num_left - LINE_W) as i32) % PITCH as i32,
        0,
        "行号左边没落在内容格上"
    );
    assert!(num_left - LINE_W >= GRID, "行号左边只剩不到 1 格空档");

    let code_x = text_left_x(gutter_line, 0.0);
    assert_eq!(
        ((code_x - LINE_W) as i32) % PITCH as i32,
        0,
        "代码起点没落在内容格上"
    );
    assert_eq!(
        (code_x - gutter_line as f32) as i32,
        (LINE_W + PITCH) as i32
    );
    let code_w = bar_line - GRID - code_x;
    assert_eq!((code_w as i32) % PITCH as i32, 0, "代码区宽度不是整数格");

    assert_eq!(bar_line as i32 - (code_x + code_w) as i32, GRID as i32);
}

#[test]
fn the_same_indicator_renders_the_same_in_any_chart() {
    let render = |app: &mut StockApp, pane: usize, area: egui::Rect| {
        let ctx = egui::Context::default();
        configure_tessellation(&ctx);
        ctx.set_pixels_per_point(1.0);
        let mut raw = egui::RawInput::default();
        raw.screen_rect = Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(area.max.x, area.max.y),
        ));
        let mut out = ctx.run_ui(raw, |ui| {
            app.ui_chart(ui, pane, area);
        });
        let shapes = std::mem::take(&mut out.shapes);
        absorb_textures(&mut out.textures_delta);
        rasterize_into(
            &ctx.tessellate(shapes, 1.0),
            area.max.x as usize,
            area.max.y as usize,
        )
    };

    let mut app = blank_app();
    app.replace_bars(data::random_walk::generate(BAR_SEED, 64, BAR_START));

    app.set_pane_indicator(PANE_MACD, 0);
    let area = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(CHART_PANE_W, 300.0));
    let a = render(&mut app, PANE_CANDLE, area);
    let b = render(&mut app, PANE_MACD, area);
    assert_eq!(a, b, "同一个指标挂到两块图表上，画出来的像素不一样");
}

#[test]
fn crosshair_stays_in_the_hovered_chart_only() {
    let ctx = egui::Context::default();
    configure_tessellation(&ctx);
    ctx.set_pixels_per_point(1.0);
    let mut app = blank_app();
    app.replace_bars(data::random_walk::generate(BAR_SEED, 64, BAR_START));

    let work = left_work();
    app.clamp_left_heights(work);
    let [kline, _bar1, volume, _bar2, indicator] =
        StockApp::left_rows(work, app.kline_h, app.vol_h);
    let areas = [kline, volume, indicator];
    let mut plots = [egui::Rect::NOTHING; 3];
    for (i, area) in areas.iter().enumerate() {
        let layout = app.panes[i].indicator.layout();
        let (_, _, main, _) = chart_rows(*area, layout);
        plots[i] = frame_rects(main).0;
        app.panes[i].height = main.height() as i32;
    }
    for (pane, plot) in plots.iter().enumerate() {
        app.clamp_view(pane, plot.width());
    }
    for i in 0..app.panes.len() {
        app.ensure_pane_computed(i);
    }

    let target = plots[1].center();
    assert!(app.win_mgr.focus_pane(PANE_VOLUME));
    let mut raw = egui::RawInput::default();
    raw.screen_rect = Some(egui::Rect::from_min_size(
        egui::Pos2::ZERO,
        egui::vec2(WINDOW_W, WINDOW_H),
    ));
    raw.events = vec![egui::Event::PointerMoved(target)];
    let mut out = ctx.run_ui(raw, |ui| {
        app.update_crosshair(ui, &plots);
    });
    absorb_textures(&mut out.textures_delta);
    let cross = app.panes[PANE_VOLUME]
        .view
        .crosshair
        .expect("鼠标在图上应该有十字光标");
    assert_eq!(cross.pane, PANE_VOLUME, "十字光标该落在鼠标压着的那一块");
    assert!(app.panes[PANE_CANDLE].view.crosshair.is_none());
    assert!(app.panes[PANE_MACD].view.crosshair.is_none());

    let from = app.panes[PANE_VOLUME].view.offset;
    let advance = bar_advance(app.panes[PANE_VOLUME].view.scale);
    let x =
        px_center(plots[PANE_VOLUME].min.x + (cross.fpos - from as f64) as f32 * advance) as usize;
    let (grid, [k, _b, v, _b2, m]) = render_left(&mut app);
    let vlines = |area: egui::Rect, pane: usize| -> usize {
        let layout = app.panes[pane].indicator.layout();
        let (_, _, main, _) = chart_rows(area, layout);
        let plot = frame_rects(main).0;
        (plot.min.y as usize..plot.max.y as usize)
            .filter(|&y| grid[y][x] == Some(C_CROSS))
            .count()
    };
    assert!(vlines(v, PANE_VOLUME) > 0, "悬停那一块没画竖线");
    assert_eq!(
        vlines(k, PANE_CANDLE),
        0,
        "没悬停的块不该有竖线（同步出现）"
    );
    assert_eq!(vlines(m, PANE_MACD), 0, "没悬停的块不该有竖线（同步出现）");

    let width = |area: egui::Rect, pane: usize| -> (usize, usize, usize) {
        let layout = app.panes[pane].indicator.layout();
        let (_, _, main, _) = chart_rows(area, layout);
        let plot = frame_rects(main).0;
        let mut widest = 0;
        for y in plot.min.y as usize..plot.max.y as usize {
            let n = (plot.min.x as usize..plot.max.x as usize)
                .filter(|&xx| grid[y][xx] == Some(C_CROSS))
                .count();
            widest = widest.max(n);
        }
        (widest, plot.min.y as usize, plot.max.y as usize)
    };

    assert!(width(v, PANE_VOLUME).0 > 100, "悬停那一块没有横线");
    assert!(width(k, PANE_CANDLE).0 <= 1, "没悬停的块不该有横线");
    assert!(width(m, PANE_MACD).0 <= 1, "没悬停的块不该有横线");
}

#[test]
fn text_advances_on_the_grid() {
    for c in ['0', '9', 'A', 'z', '.', ',', '%', '+', '-', '/', ' '] {
        assert_eq!(char_advance(c), 9.0, "{c:?} 该是 ASCII（1 个内容格）");
    }
    for c in ['圆', '桌', '万', '亿', '：', '，', '（'] {
        assert_eq!(char_advance(c), 18.0, "{c:?} 该是中文（2 个内容格）");
    }

    assert_eq!(text_advance("1234"), 36.0);
    assert_eq!(text_advance("圆桌"), 36.0);
    assert_eq!(text_advance("123") % 9.0, 0.0, "ASCII 的步进本身就是整格");

    assert_eq!(text_width(""), 0);
    assert_eq!(text_width("1234"), 36);
    assert_eq!(text_width("123"), 27);
    assert_eq!(text_width("12.5%"), 45);
    assert_eq!(text_width("圆桌"), 36);
    assert_eq!(text_width("圆桌 2024"), 81);
    for s in [
        "0",
        "1234.56",
        "圆桌",
        "-1200.5万",
        "MACD(12,26,9)",
        "2026-10-14",
    ] {
        assert_eq!(text_width(s) % 9, 0, "{s} 的宽度不是 9 的整数倍");
    }
}

#[test]
fn every_glyph_fits_its_advance() {
    let ctx = egui::Context::default();
    configure_tessellation(&ctx);
    ctx.set_pixels_per_point(1.0);
    let mut app = blank_app();
    app.text_engine = HardText::new(&ctx);
    if app.text_engine.is_none() {
        return;
    }

    let probe = |app: &mut StockApp, s: &str, w: usize| -> Option<(usize, usize)> {
        let mut raw = egui::RawInput::default();
        raw.screen_rect = Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(w as f32, 24.0),
        ));
        let color = theme().text;
        let mut out = ctx.run_ui(raw, |ui| {
            let painter = ui.painter().clone();
            app.text(
                &painter,
                egui::pos2(TEXT_INSET_X, 12.0),
                egui::Align2::LEFT_CENTER,
                s,
                color,
            );
        });
        let shapes = std::mem::take(&mut out.shapes);
        absorb_textures(&mut out.textures_delta);
        let grid = rasterize_into(&ctx.tessellate(shapes, 1.0), w, 24);
        let mut lo = usize::MAX;
        let mut hi = 0usize;
        for row in grid.iter().take(24) {
            for (x, cell) in row.iter().enumerate() {
                if cell.is_some() {
                    lo = lo.min(x);
                    hi = hi.max(x);
                }
            }
        }
        (lo != usize::MAX).then_some((lo, hi))
    };

    for c in "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz.,%+-*/()".chars() {
        let s = c.to_string();
        let (lo, hi) = probe(&mut app, &s, 64).unwrap_or_else(|| panic!("{c:?} 没画出墨迹"));
        let ink = (hi - lo + 1) as f32;
        assert!(
            ink <= ADV_ASCII,
            "{c:?} 的墨迹 {ink}px 比步进 {ADV_ASCII}px 还宽，字会互相压住"
        );

        assert!(
            (lo as f32) < TEXT_INSET_X + ADV_ASCII && hi as f32 >= TEXT_INSET_X,
            "{c:?} 的墨迹落在 {lo} 到 {hi}，没踩着内容格起点 {TEXT_INSET_X}"
        );
    }

    for c in ['圆', '桌', '万', '亿', '：', '，'] {
        let s = c.to_string();
        let (lo, hi) = probe(&mut app, &s, 64).unwrap_or_else(|| panic!("{c:?} 没画出墨迹"));
        let ink = (hi - lo + 1) as f32;
        assert!(
            ink <= ADV_WIDE,
            "{c:?} 的墨迹 {ink}px 比步进 {ADV_WIDE}px 还宽"
        );
    }

    let s = "0123456789";
    let (lo, hi) = probe(&mut app, s, 128).expect("整串没画出墨迹");

    assert!(
        lo as f32 >= TEXT_INSET_X && lo as f32 <= TEXT_INSET_X + 2.0,
        "整串没从内容格起点附近起笔：{lo}"
    );
    assert!(
        (hi as f32) < TEXT_INSET_X + text_advance(s),
        "整串墨迹排到 {hi}，超出了 text_advance() = {}px 的宽度",
        text_advance(s)
    );
}

#[test]
fn allow_only_chinese_and_ascii_in_sources() {
    fn walk(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
        for e in std::fs::read_dir(dir).expect("读不到源码目录") {
            let p = e.expect("目录项").path();
            if p.is_dir() {
                walk(&p, out);
            } else if p.extension().is_some_and(|x| x == "rs") {
                out.push(p);
            }
        }
    }
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = Vec::new();
    walk(&root, &mut files);
    assert!(files.len() >= 15, "源码文件数不对：{}", files.len());
    for p in files {
        let text = std::fs::read_to_string(&p).expect("源码读成 UTF-8 失败");
        for (i, c) in text.chars().enumerate() {
            let ascii_ok = c.is_ascii() && (c == '\t' || c == '\n' || c == '\r' || !c.is_control());
            assert!(
                ascii_ok || is_wide_char(c),
                "{} 第 {} 个字符 {c:?}（U+{:04X}）既不是中文也不是 ASCII",
                p.display(),
                i,
                c as u32
            );
        }
    }
}

#[test]
fn the_default_theme_is_dark() {
    let id = std::thread::spawn(|| theme().id)
        .join()
        .expect("读默认皮肤的线程炸了");
    assert_eq!(id, ThemeId::Dark, "开机默认皮肤不是深色");
    assert_eq!(
        rgb32_u32(Theme::DARK.chart_bg),
        0x000000,
        "深色的图表底该是黑色"
    );

    let app = blank_app_on(Theme::DARK);
    let dark = app.current_palette();
    set_theme(Theme::CLASSIC);
    assert!(app.current_palette() == dark, "浅色图表配色应跟随深色");
    assert_eq!(rgb32_u32(Theme::DARK.candle_down), 0x00FFFF);
    assert_eq!(rgb32_u32(Theme::DARK.chart_bg), 0x000000);
    set_theme(Theme::DARK);
}

#[test]
fn startup_candle_colors_follow_the_default_dark_skin() {
    assert_eq!(
        Theme::DARK.candle_down,
        egui::Color32::from_rgb(0x00, 0xFF, 0xFF)
    );
    let app = blank_app_on(Theme::DARK);
    assert!(app.chart_follows_theme, "默认该指向系统级配色");
    assert_eq!(app.up_color, (0xFF, 0x00, 0x00), "深色开机：涨色是红");
    assert_eq!(app.down_color, (0x00, 0xFF, 0xFF), "深色开机：跌色是青");

    assert_eq!(app.current_palette().candle_down, 0x00FFFF);
    assert!(
        app.panes_palette == app.current_palette(),
        "开机灌进指标的那套色该和当前皮肤那套一样"
    );
    set_theme(Theme::DARK);
}

#[test]
fn theme_switches_the_whole_palette() {
    set_theme(Theme::CLASSIC);
    assert_eq!(theme().id, ThemeId::Classic);
    assert_eq!(rgb32_u32(theme().ctrl_border), 0x999999);
    assert_eq!(rgb32_u32(theme().text), 0x000000);

    set_theme(Theme::DARK);
    assert_eq!(theme().id, ThemeId::Dark);
    assert_eq!(rgb32_u32(theme().ctrl_border), 0xFF0000, "深色线框该是红色");
    assert_eq!(rgb32_u32(theme().line), 0xFF0000, "深色分隔线也该是红色");
    assert_eq!(rgb32_u32(theme().text), 0xFFFFFF, "深色文字该反相成白色");
    assert_eq!(rgb32_u32(theme().candle_up), 0xFF0000, "K 线涨色跟随浅色");
    assert_eq!(rgb32_u32(theme().candle_down), 0x00FFFF, "K 线跌色使用青色");
    assert_eq!(rgb32_u32(theme().text_down), 0x009900, "数字跌色跟随浅色");
    assert_eq!(rgb32_u32(theme().text_up), 0xFF0000, "数字涨色跟随浅色");
    assert_eq!(
        rgb32_u32(theme().chart_bg),
        0x000000,
        "深色图表背景该是黑色"
    );
    assert_eq!(
        rgb32_u32(theme().symbol),
        0xFFFFFF,
        "深色主题的符号色应跟随文字"
    );

    set_theme(Theme::CLASSIC);
    assert_eq!(theme().id, ThemeId::Classic);
}

#[test]
fn theme_button_switches_skin_and_candle_palette() {
    set_theme(Theme::CLASSIC);
    let mut app = blank_app();
    app.replace_bars(data::random_walk::generate(BAR_SEED, 8, BAR_START));
    app.sync_pane_palette();
    assert_eq!(app.down_color, DOWN_RGB_DEFAULT, "默认跌色是绿");

    app.switch_theme();
    assert_eq!(theme().id, ThemeId::Dark);
    assert_eq!(app.up_color, (0xFF, 0x00, 0x00));
    assert_eq!(app.down_color, (0x00, 0xFF, 0xFF));
    assert!(app.panes.iter().all(|p| p.dirty), "换皮肤该让图表重算");

    app.switch_theme();
    assert_eq!(theme().id, ThemeId::Classic);
    assert_eq!(app.down_color, DOWN_RGB_DEFAULT);
    set_theme(Theme::CLASSIC);
}

#[test]
fn chart_colors_point_at_the_system_palette_until_customized() {
    set_theme(Theme::CLASSIC);
    let mut app = blank_app();
    assert!(app.chart_follows_theme, "默认该指向系统级配色");

    app.switch_theme();
    assert_eq!(theme().id, ThemeId::Dark);
    assert_eq!(app.up_color, (0xFF, 0x00, 0x00));
    assert_eq!(app.down_color, (0x00, 0xFF, 0xFF));

    app.chart_follows_theme = false;
    app.up_color = (0x11, 0x22, 0x33);
    app.down_color = (0x44, 0x55, 0x66);
    app.switch_theme();
    assert_eq!(theme().id, ThemeId::Classic);
    assert_eq!(
        app.up_color,
        (0x11, 0x22, 0x33),
        "自定义过的颜色不该被切皮肤改掉"
    );
    assert_eq!(
        app.down_color,
        (0x44, 0x55, 0x66),
        "自定义过的颜色不该被切皮肤改掉"
    );

    app.follow_system_chart_colors();
    assert!(app.chart_follows_theme);
    let t = theme();
    assert_eq!(
        app.up_color,
        (t.candle_up.r(), t.candle_up.g(), t.candle_up.b())
    );
    assert_eq!(
        app.down_color,
        (t.candle_down.r(), t.candle_down.g(), t.candle_down.b())
    );
    assert!(app.panes.iter().all(|p| p.dirty), "配色变了该让图表重算");
    set_theme(Theme::CLASSIC);
}

#[test]
fn every_text_box_keeps_one_cell_of_padding() {
    let ctx = egui::Context::default();
    configure_tessellation(&ctx);
    ctx.set_pixels_per_point(1.0);
    let mut app = blank_app();
    app.text_engine = HardText::new(&ctx);
    if app.text_engine.is_none() {
        return;
    }
    let min_pad = 2.0 * GRID;

    app.build_panel_buttons();
    assert!(app.panel_buttons.len() >= 10, "面板按钮不该只有这么几个");
    let buttons: Vec<(String, bool, f32)> = app
        .panel_buttons
        .iter()
        .map(|b| {
            (
                b.button.content.label(),
                matches!(
                    &b.button.content,
                    ButtonContent::IconText(..) | ButtonContent::Icon(..)
                ),
                b.button.w,
            )
        })
        .collect();
    for (text, has_icon, w) in buttons {
        let inner = app.text_width(&text) + if has_icon { ICON_W + ICON_GAP } else { 0.0 };
        assert!(
            w - inner >= min_pad,
            "按钮「{text}」左右空档不足：盒宽 {w}、内容宽 {inner}"
        );
    }

    for title in ["文件", "数据", "分析", "布局", "工具", "帮助"] {
        let w = tab_width(&mut app, title);
        assert!(
            w - app.text_width(title) >= min_pad,
            "菜单「{title}」左右空档不足"
        );
    }

    for seg in app.status_segments() {
        let w = snap_cells_up(app.text_width(&seg) + 2.0 * TEXT_PAD_X);
        assert!(
            w - app.text_width(&seg) >= min_pad,
            "状态栏项「{seg}」空档不足"
        );
    }

    let value_w = app.text_width("255");
    assert!(
        cells(CP_SLIDER_VALUE_COLS) - value_w >= min_pad,
        "滑块数值栏放不下三位数：栏宽 {}、数值宽 {value_w}",
        cells(CP_SLIDER_VALUE_COLS)
    );

    for text in ["切换", "代码"] {
        let content = ButtonContent::Text(text.to_owned());
        let w = app.button_width(&content);
        assert!(
            w - app.text_width(text) >= min_pad,
            "Pane 标题栏按钮「{text}」左右空档不足：盒宽 {w}"
        );
        assert_eq!(w, snap_cells_up(app.text_width(text) + 2.0 * TEXT_PAD_X));
        assert!(w >= PANE_BTN_W, "Pane 标题栏按钮比下限还窄");
    }

    for s in ["12.34", "123"] {
        assert!(
            app.cross_label_w(s) - app.text_width(s) >= min_pad,
            "浮动标签「{s}」空档不足"
        );
    }
}

#[test]
fn text_starts_on_a_content_cell() {
    assert_eq!(TEXT_INSET_X, LINE_W + PITCH);
    assert_eq!(((TEXT_INSET_X as i32 - LINE_W as i32) % PITCH as i32), 0);
    assert!(
        TEXT_INSET_X > TEXT_PAD_X,
        "落点要比算宽度用的大 1px（让开线格）"
    );

    assert_eq!(
        ((ICON_W + ICON_GAP) as i32) % 9,
        0,
        "图标区 + 间隔不是整数格"
    );

    let ctx = egui::Context::default();
    configure_tessellation(&ctx);
    ctx.set_pixels_per_point(1.0);
    let mut app = blank_app();
    app.text_engine = HardText::new(&ctx);
    if app.text_engine.is_none() {
        return;
    }

    for s in [
        "重新生成",
        "图表配色跟随系统",
        "打印当前状态",
        "12.34",
        "255",
        "360",
    ] {
        let text = app.text_width(s);
        let box_w = snap_cells_up(text + 2.0 * TEXT_PAD_X);
        let offset = ((box_w - text) / 2.0).round();
        assert_eq!(
            offset, TEXT_INSET_X,
            "「{s}」在 cells 盒里居中之后没落在内容格起点（偏移 {offset}）"
        );
    }
}

#[test]
fn right_aligned_and_floating_labels_land_on_content_cells() {
    assert_eq!(text_left_x(0.0, 0.0), TEXT_INSET_X);
    assert_eq!(
        text_left_x(LINE_W, 0.0),
        TEXT_INSET_X,
        "shrink 过 1px 的内框也得吸到同一条线"
    );
    assert_eq!(text_left_x(PITCH, 0.0), PITCH + TEXT_INSET_X);
    assert_eq!(text_right_x(9.0 * 89.0, 0.0), 9.0 * 89.0 - GRID);

    let ox = 203.4;
    assert_eq!(text_left_x(ox, ox) - ox, TEXT_INSET_X);
    assert_eq!(
        text_left_x(ox + 3.0 * PITCH, ox) - ox,
        3.0 * PITCH + TEXT_INSET_X
    );
    assert_eq!(text_right_x(ox + 2.0 * PITCH, ox) - ox, 2.0 * PITCH - GRID);

    let ctx = egui::Context::default();
    configure_tessellation(&ctx);
    ctx.set_pixels_per_point(1.0);
    let mut app = blank_app();
    app.text_engine = HardText::new(&ctx);
    if app.text_engine.is_none() {
        return;
    }
    app.replace_bars(data::random_walk::generate(BAR_SEED, BAR_COUNT, BAR_START));

    let work = left_work();
    app.clamp_left_heights(work);
    let [kline, _bar, volume, _bar2, indicator] = StockApp::left_rows(work, app.kline_h, app.vol_h);
    for area in [kline, volume, indicator] {
        let (_, _, main, _) = chart_rows(area, app.panes[0].indicator.layout());
        let (plot, axis) = frame_rects(main);
        let axis_line = (axis.max.x - LINE_W) as i32;
        assert_eq!(axis_line % PITCH as i32, 0, "轴的右边框不在竖网格线上");

        let tick = StockApp::axis_tick_x(axis) as i32;
        assert_eq!(
            (tick - LINE_W as i32) % PITCH as i32,
            0,
            "轴刻度没压在内容格上：{tick}"
        );
        assert_eq!(
            axis_line - tick,
            GRID as i32,
            "轴刻度和轴的右边框之间的空档不是 1 格"
        );

        for s in ["20.00", "1234", "255"] {
            let w = app.cross_label_w(s);
            let x0 = StockApp::cross_axis_label_x(axis, w) as i32;
            assert_eq!(x0 % PITCH as i32, 0, "数值牌左边不在网格线上：{x0}");
            assert!(x0 >= axis.min.x as i32, "数值牌跑到轴左边去了：{x0}");
            assert!(
                x0 + w as i32 <= axis_line - GRID as i32,
                "数值牌顶到轴的右边框上了：{x0}+{w}"
            );
        }

        for (cx, s) in [
            (plot.min.x, "123"),
            (plot.center().x, "12"),
            (plot.max.x, "1234"),
        ] {
            let w = app.cross_label_w(s);
            let x0 = StockApp::cross_time_label_x(plot, cx, w, area.min.x) as i32;
            assert_eq!(x0 % PITCH as i32, 0, "索引牌左边不在网格线上：{x0}");
            assert!(
                x0 >= plot.min.x as i32 - LINE_W as i32,
                "索引牌跑出绘图区左边：{x0}"
            );
            assert!(x0 + w as i32 <= plot.max.x as i32, "索引牌顶出绘图区右边");
        }
    }

    let panel = egui::Rect::from_min_size(
        egui::pos2(CP_DEFAULT_X, CP_DEFAULT_Y),
        egui::vec2(CP_W, CP_H),
    );
    let hex_band = cp_band(panel, CP_ROW_HEX, CP_HEX_ROWS);
    let hex_box = hex_box(panel);

    let prefix_x = text_left_x(hex_band.min.x, panel.min.x);
    assert_eq!(
        prefix_x - hex_band.min.x,
        TEXT_INSET_X,
        "前缀 '#' 左边不是 1 格空档：{prefix_x}"
    );
    assert!(
        prefix_x + app.text_width(HEX_PREFIX) <= hex_box.min.x,
        "前缀 '#' 顶进输入框里了"
    );

    let digits_x = text_left_x(hex_box.min.x, hex_box.min.x);
    assert_eq!(
        digits_x - hex_box.min.x,
        TEXT_INSET_X,
        "Hex 数字左边不是 1 格空档：{digits_x}"
    );
    assert_eq!(
        (hex_box.min.x - panel.min.x - grid_line(CP_TRACK_LINE)) % PITCH,
        0.0,
        "Hex 输入框左边那条线没跟滑块的轨道左沿对齐"
    );
}

#[test]
fn axis_ticks_fit_inside_the_axis_column() {
    let ctx = egui::Context::default();
    let mut app = blank_app();
    app.text_engine = HardText::new(&ctx);
    if app.text_engine.is_none() {
        return;
    }
    app.replace_bars(data::random_walk::generate(BAR_SEED, BAR_COUNT, BAR_START));

    let hi = app.bars.iter().map(|b| b.h).fold(f64::MIN, f64::max);
    let lo = app.bars.iter().map(|b| b.l).fold(f64::MAX, f64::min);
    for v in [hi, lo] {
        let s = format!("{v:.2}");
        let w = app.text_width(&s);
        assert!(
            w + 2.0 * GRID <= AXIS_W,
            "纵轴刻度「{s}」装不下：轴宽 {AXIS_W}、文字宽 {w}"
        );
    }
}

#[test]
fn splitter_drag_does_not_touch_the_chart() {
    let ctx = egui::Context::default();
    configure_tessellation(&ctx);
    ctx.set_pixels_per_point(1.0);
    let mut app = blank_app();
    app.replace_bars(data::random_walk::generate(BAR_SEED, 64, BAR_START));

    let work = left_work();
    app.clamp_left_heights(work);
    let [kline, bar1, volume, bar2, indicator] = StockApp::left_rows(work, app.kline_h, app.vol_h);
    let bars = [bar1, bar2, egui::Rect::NOTHING];
    let areas = [kline, volume, indicator];
    let mut plots = [egui::Rect::NOTHING; 3];
    for (i, area) in areas.iter().enumerate() {
        let l = app.panes[i].indicator.layout();
        let (_, _, main, _) = chart_rows(*area, l);
        plots[i] = frame_rects(main).0;
        app.panes[i].height = main.height() as i32;
    }
    for (pane, plot) in plots.iter().enumerate() {
        app.clamp_view(pane, plot.width());
    }
    let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(WINDOW_W, WINDOW_H));
    let target = plots[1].center();
    assert!(app.win_mgr.focus_pane(PANE_VOLUME));

    let press = |pos: egui::Pos2| {
        vec![
            egui::Event::PointerMoved(pos),
            egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: egui::Modifiers::default(),
            },
        ]
    };

    let mut raw = egui::RawInput::default();
    raw.screen_rect = Some(screen);
    raw.events = press(target);
    let mut out = ctx.run_ui(raw, |ui| {
        app.chart_interact(ui, &plots, &bars);
        app.update_crosshair(ui, &plots);
    });
    absorb_textures(&mut out.textures_delta);
    assert!(
        app.panes[PANE_VOLUME].view.pan.is_some(),
        "绘图区按下该能平移"
    );
    assert!(
        app.panes[PANE_VOLUME].view.crosshair.is_some(),
        "鼠标在图上该有十字光标"
    );
    app.panes[PANE_VOLUME].view.pan = None;

    app.hsplit_grab = Some(5.0);
    let mut raw = egui::RawInput::default();
    raw.screen_rect = Some(screen);
    raw.events = press(target);
    let mut out = ctx.run_ui(raw, |ui| {
        app.chart_interact(ui, &plots, &bars);
        app.update_crosshair(ui, &plots);
    });
    absorb_textures(&mut out.textures_delta);
    assert!(
        app.panes[PANE_VOLUME].view.pan.is_none(),
        "拖拖拽条时不该把图表拖走"
    );
    assert!(
        app.panes[PANE_VOLUME].view.crosshair.is_none(),
        "拖拖拽条时不该响应图表区的十字光标"
    );

    let before = (
        app.panes[PANE_VOLUME].view.scale,
        app.panes[PANE_VOLUME].view.offset,
    );
    let mut raw = egui::RawInput::default();
    raw.screen_rect = Some(screen);
    raw.events = vec![
        egui::Event::PointerMoved(target),
        egui::Event::MouseWheel {
            unit: egui::MouseWheelUnit::Line,
            delta: egui::vec2(0.0, 3.0),
            phase: egui::TouchPhase::Move,
            modifiers: egui::Modifiers::default(),
        },
    ];
    let mut out = ctx.run_ui(raw, |ui| {
        app.chart_interact(ui, &plots, &bars);
        app.update_crosshair(ui, &plots);
    });
    absorb_textures(&mut out.textures_delta);
    assert_eq!(
        (
            app.panes[PANE_VOLUME].view.scale,
            app.panes[PANE_VOLUME].view.offset
        ),
        before,
        "拖拖拽条时滚轮不该缩放 / 平移 K 线"
    );
    assert!(
        app.panes[PANE_VOLUME].view.crosshair.is_none(),
        "拖拖拽条时滚轮扫过绘图区也不该冒十字线"
    );
    app.hsplit_grab = None;

    app.hsplit_grab = None;
    let mut raw = egui::RawInput::default();
    raw.screen_rect = Some(screen);
    raw.events = vec![egui::Event::PointerMoved(target)];
    let mut out = ctx.run_ui(raw, |ui| {
        app.update_crosshair(ui, &plots);
    });
    absorb_textures(&mut out.textures_delta);
    assert!(
        app.panes[PANE_VOLUME].view.crosshair.is_some(),
        "松开拖拽条之后十字光标该恢复"
    );

    app.open_color_panel(ColorTarget::Up);
    let color = color_index(&app);
    app.win_mgr.wins[color].w.set_dragging(true);
    let mut raw = egui::RawInput::default();
    raw.screen_rect = Some(screen);
    raw.events = vec![egui::Event::PointerMoved(target)];
    let mut out = ctx.run_ui(raw, |ui| {
        app.update_crosshair(ui, &plots);
    });
    absorb_textures(&mut out.textures_delta);
    assert!(
        app.panes[PANE_VOLUME].view.crosshair.is_none(),
        "拖浮动窗口时不该响应图表区的十字光标"
    );
    app.win_mgr.wins[color].w.set_dragging(false);

    let win_i = app
        .win_mgr
        .wins
        .iter()
        .position(|w| w.kind == WinKind::Chart)
        .expect("窗口区里没有图表窗");
    app.win_mgr.wins[win_i].w.set_dragging(true);
    let mut raw = egui::RawInput::default();
    raw.screen_rect = Some(screen);
    raw.events = vec![egui::Event::PointerMoved(target)];
    let mut out = ctx.run_ui(raw, |ui| {
        app.update_crosshair(ui, &plots);
    });
    absorb_textures(&mut out.textures_delta);
    assert!(
        app.panes[PANE_VOLUME].view.crosshair.is_none(),
        "拖子窗口时不该响应图表区的十字光标"
    );
    app.win_mgr.wins[win_i].w.set_dragging(false);
    let mut raw = egui::RawInput::default();
    raw.screen_rect = Some(screen);
    raw.events = vec![egui::Event::PointerMoved(target)];
    let mut out = ctx.run_ui(raw, |ui| {
        app.update_crosshair(ui, &plots);
    });
    absorb_textures(&mut out.textures_delta);
    assert!(
        app.panes[PANE_VOLUME].view.crosshair.is_some(),
        "松开浮动窗口之后十字光标该恢复"
    );
}

#[test]
fn hover_focus_keeps_overlapping_chart_crosshairs_in_the_front_window() {
    let ctx = egui::Context::default();
    configure_tessellation(&ctx);
    ctx.set_pixels_per_point(1.0);
    let mut app = blank_app();
    app.text_engine = HardText::new(&ctx);
    app.replace_bars(data::random_walk::generate(BAR_SEED, 64, BAR_START));
    let frame = |app: &mut StockApp, pos: Option<egui::Pos2>| {
        let events = pos.map_or_else(Vec::new, |p| vec![egui::Event::PointerMoved(p)]);
        let _ = render_events_in(&ctx, app, events);
    };
    frame(&mut app, None);
    app.add_chart_window();
    frame(&mut app, None);
    let work = app.work_rect;

    let lower = app
        .win_mgr
        .wins
        .iter()
        .position(|w| w.kind == WinKind::Chart && w.pane == PANE_CANDLE)
        .expect("底层图表窗");
    let upper = app.win_mgr.wins.len() - 1;
    let lower_pane = app.win_mgr.wins[lower].pane;
    let upper_pane = app.win_mgr.wins[upper].pane;
    let lower_id = app.win_mgr.wins[lower].w.id;
    let upper_id = app.win_mgr.wins[upper].w.id;
    app.win_mgr.wins[lower].w.pos = work.min;
    app.win_mgr.wins[lower].w.size = egui::vec2(cells(80), cells(50));
    app.win_mgr.wins[upper].w.pos = work.min + egui::vec2(cells(45), cells(45));
    app.win_mgr.wins[upper].w.size = egui::vec2(cells(45), cells(28));
    frame(&mut app, None);

    let lower_body = ManagedWin::body(app.win_mgr.wins[lower].w.rect());
    let lower_layout = app.panes[lower_pane].indicator.layout();
    let (_, _, lower_main, _) = chart_rows(lower_body, lower_layout);
    let lower_point = frame_rects(lower_main).0.center();
    frame(&mut app, Some(lower_point));
    assert_eq!(
        app.win_mgr.live_index().map(|i| app.win_mgr.wins[i].w.id),
        Some(lower_id),
        "悬停底层图表可见区域没有转移焦点"
    );
    assert!(
        app.panes[lower_pane].view.crosshair.is_some(),
        "底层图表可见区域悬停时没有十字光标"
    );
    assert!(app.panes[upper_pane].view.crosshair.is_none());

    let upper = app
        .win_mgr
        .wins
        .iter()
        .position(|w| w.w.id == upper_id)
        .expect("顶层图表窗");
    let upper_body = ManagedWin::body(app.win_mgr.wins[upper].w.rect());
    let upper_layout = app.panes[upper_pane].indicator.layout();
    let (_, _, upper_main, _) = chart_rows(upper_body, upper_layout);
    let upper_point = frame_rects(upper_main).0.center();
    frame(&mut app, Some(upper_point));
    assert_eq!(
        app.win_mgr.live_index().map(|i| app.win_mgr.wins[i].w.id),
        Some(upper_id),
        "悬停顶层图表没有转移焦点"
    );
    assert!(
        app.panes[upper_pane].view.crosshair.is_some(),
        "顶层图表悬停时没有十字光标"
    );
    assert!(
        app.panes[lower_pane].view.crosshair.is_none(),
        "顶层图表悬停时底层图表仍保留了十字光标"
    );
}

#[test]
fn chart_status_band_has_solid_dividers_on_the_grid() {
    let ctx = egui::Context::default();
    configure_tessellation(&ctx);
    ctx.set_pixels_per_point(1.0);
    let mut app = blank_app();
    app.text_engine = HardText::new(&ctx);
    if app.text_engine.is_none() {
        return;
    }
    app.replace_bars(data::random_walk::generate(BAR_SEED, 64, BAR_START));
    let work = left_work();
    app.clamp_left_heights(work);
    let [kline, _bar, _vol, _bar2, _ind] = StockApp::left_rows(work, app.kline_h, app.vol_h);
    let layout = app.panes[PANE_CANDLE].indicator.layout();
    let (_, status, _, _) = chart_rows(kline, layout);
    let band = status.unwrap();

    let mut want = Vec::new();
    let mut sx = band.min.x;
    let mut first = true;
    let fields = [
        ("开", "0.00"),
        ("高", "0.00"),
        ("低", "0.00"),
        ("收", "0.00"),
    ];
    for (label, tmpl) in fields {
        let w =
            snap_cells_up(app.text_width(label) + GRID + app.text_width(tmpl) + 2.0 * TEXT_PAD_X);
        if sx + w > band.max.x {
            break;
        }
        if !first {
            want.push(sx - band.min.x);
        }
        sx += w - LINE_W;
        first = false;
    }
    if !first {
        want.push(sx - band.min.x);
    }
    assert!(want.len() >= 2, "状态带至少要有几条分割线才测得出");

    let mut raw = egui::RawInput::default();
    raw.screen_rect = Some(egui::Rect::from_min_size(
        egui::Pos2::ZERO,
        egui::vec2(WINDOW_W, WINDOW_H),
    ));
    let mut out = ctx.run_ui(raw, |ui| {
        app.ui_chart(ui, PANE_CANDLE, kline);
    });
    let shapes = std::mem::take(&mut out.shapes);
    absorb_textures(&mut out.textures_delta);
    let grid = rasterize_into(
        &ctx.tessellate(shapes, 1.0),
        WINDOW_W as usize,
        WINDOW_H as usize,
    );
    let rows = band.min.y as usize + 1..band.max.y as usize - 1;
    for dx in want {
        assert_eq!(dx % 9.0, 0.0, "状态带分割线没落在竖网格线上");
        let col = (band.min.x + dx) as usize;
        let lit = rows
            .clone()
            .filter(|&y| grid[y][col] == Some(C_LINE))
            .count();
        assert!(lit > 0, "状态带分割线 {col} 没画出来");
        assert_eq!(lit, rows.len(), "状态带分割线 {col} 不是完整实线");
    }
}

#[test]
fn status_bar_has_solid_dividers_on_the_grid() {
    let ctx = egui::Context::default();
    configure_tessellation(&ctx);
    ctx.set_pixels_per_point(1.0);
    let mut app = blank_app();
    app.text_engine = HardText::new(&ctx);
    if app.text_engine.is_none() {
        return;
    }
    app.fps = 60.4;
    app.status_msg = "状态栏测试".to_owned();
    app.replace_bars(data::random_walk::generate(BAR_SEED, 80, BAR_START));
    app.panes[0].view.visible = 80;
    let layout = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(WINDOW_W, WINDOW_H));
    let areas = split_areas(layout);
    let status = areas[4];

    let mut want = Vec::new();
    let mut sx = status.min.x;
    let mut first = true;
    for seg in app.status_segments() {
        let w = snap_cells_up(app.text_width(&seg) + 2.0 * TEXT_PAD_X);
        if sx + w > status.max.x {
            break;
        }
        if !first {
            want.push(sx - status.min.x);
        }
        sx += w - LINE_W;
        first = false;
    }
    if !first {
        want.push(sx - status.min.x);
    }
    assert!(want.len() >= 1, "状态栏至少要有几条分割线才测得出");

    let mut raw = egui::RawInput::default();
    raw.screen_rect = Some(layout);
    let mut out = ctx.run_ui(raw, |ui| {
        app.paint_skeleton(ui.painter(), layout, areas);
    });
    let shapes = std::mem::take(&mut out.shapes);
    absorb_textures(&mut out.textures_delta);
    let grid = rasterize_into(
        &ctx.tessellate(shapes, 1.0),
        WINDOW_W as usize,
        WINDOW_H as usize,
    );
    let rows = status.min.y as usize + 1..status.max.y as usize - 1;
    for dx in want {
        assert_eq!(dx % 9.0, 0.0, "状态栏分割线没落在竖网格线上");
        let col = (status.min.x + dx) as usize;
        let lit = rows
            .clone()
            .filter(|&y| grid[y][col] == Some(C_LINE))
            .count();
        assert!(lit > 0, "状态栏分割线 {col} 没画出来");
        assert_eq!(lit, rows.len(), "状态栏分割线 {col} 不是完整实线");
    }
}

#[test]
fn panes_render_at_the_minimum_height() {
    let ctx = egui::Context::default();
    configure_tessellation(&ctx);
    ctx.set_pixels_per_point(1.0);
    let mut app = blank_app();
    app.replace_bars(data::random_walk::generate(BAR_SEED, 64, BAR_START));
    app.text_engine = HardText::new(&ctx);

    let work = egui::Rect::from_min_size(
        egui::pos2(0.0, 54.0),
        egui::vec2(CHART_PANE_W, cells(LEFT_WORK_MIN_ROWS)),
    );
    app.ind_h = PANE_H_MIN;
    app.vol_h = PANE_H_MIN;
    app.clamp_left_heights(work);
    assert_eq!(app.kline_h, PANE_H_MIN);
    assert_eq!(app.vol_h, PANE_H_MIN);
    assert_eq!(app.ind_h, PANE_H_MIN);

    let [kline, _bar, volume, _bar2, indicator] = StockApp::left_rows(work, app.kline_h, app.vol_h);
    let areas = [kline, volume, indicator];
    let mut plots = [egui::Rect::NOTHING; 3];
    for (i, area) in areas.iter().enumerate() {
        let layout = app.panes[i].indicator.layout();
        let (_, _, main, _) = chart_rows(*area, layout);
        plots[i] = frame_rects(main).0;
        app.panes[i].height = main.height() as i32;
        assert!(plots[i].height() > 0.0, "第 {} 块绘图区高度非正", i + 1);
    }
    for (pane, plot) in plots.iter().enumerate() {
        app.clamp_view(pane, plot.width());
    }
    for i in 0..app.panes.len() {
        app.ensure_pane_computed(i);
    }

    let target = plots[2].center();
    assert!(app.win_mgr.focus_pane(PANE_MACD));
    let mut raw = egui::RawInput::default();
    raw.screen_rect = Some(egui::Rect::from_min_size(
        egui::Pos2::ZERO,
        egui::vec2(WINDOW_W, WINDOW_H),
    ));
    raw.events = vec![egui::Event::PointerMoved(target)];
    let mut out = ctx.run_ui(raw, |ui| {
        app.update_crosshair(ui, &plots);
        for (i, area) in areas.iter().enumerate() {
            app.ui_chart(ui, i, *area);
        }
    });
    let _ = ctx.tessellate(std::mem::take(&mut out.shapes), 1.0);
    absorb_textures(&mut out.textures_delta);
    assert!(
        app.panes[PANE_MACD].view.crosshair.is_some(),
        "最小高度下十字光标没建起来"
    );
}

#[test]
fn panes_render_at_the_minimum_width() {
    let ctx = egui::Context::default();
    configure_tessellation(&ctx);
    ctx.set_pixels_per_point(1.0);
    let mut app = blank_app();
    app.replace_bars(data::random_walk::generate(BAR_SEED, 64, BAR_START));
    app.text_engine = HardText::new(&ctx);

    let work =
        egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(PANE_MIN_W, cells(WORK_ROWS)));
    app.clamp_left_heights(work);
    let [kline, _bar, volume, _bar2, indicator] = StockApp::left_rows(work, app.kline_h, app.vol_h);
    let areas = [kline, volume, indicator];

    let layout = app.panes[PANE_CANDLE].indicator.layout();
    let (title, _, _, _) = chart_rows(kline, layout);
    let buttons = StockApp::pane_button_rects(title, &pane_btn_widths());
    assert_eq!(buttons[1].max.x, kline.max.x, "最右按钮没贴着右边框");
    assert!(buttons[0].min.x >= kline.min.x, "按钮超出左栏");

    let mut raw = egui::RawInput::default();
    raw.screen_rect = Some(egui::Rect::from_min_size(
        egui::Pos2::ZERO,
        egui::vec2(WINDOW_W, WINDOW_H),
    ));
    let mut out = ctx.run_ui(raw, |ui| {
        for (i, area) in areas.iter().enumerate() {
            app.ui_chart(ui, i, *area);
        }
    });
    let _ = ctx.tessellate(std::mem::take(&mut out.shapes), 1.0);
    absorb_textures(&mut out.textures_delta);
}

fn render_window(app: &mut StockApp) -> Vec<Vec<Option<egui::Color32>>> {
    render_window_at(app, None)
}

fn render_window_at(
    app: &mut StockApp,
    pointer: Option<egui::Pos2>,
) -> Vec<Vec<Option<egui::Color32>>> {
    if app.panel_buttons.is_empty() {
        app.build_panel_buttons();
    }
    let ctx = egui::Context::default();
    configure_tessellation(&ctx);
    ctx.set_pixels_per_point(1.0);

    if let Some(engine) = HardText::new(&ctx) {
        app.text_engine = Some(engine);
    }
    let layout = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(WINDOW_W, WINDOW_H));
    let mut raw = egui::RawInput::default();
    raw.screen_rect = Some(layout);

    if let Some(p) = pointer {
        raw.events = vec![egui::Event::PointerMoved(p)];
    }
    let mut out = ctx.run_ui(raw, |ui| {
        let painter = ui.painter().clone();
        let areas = split_areas(layout);
        app.paint_skeleton(&painter, layout, areas);
        app.ui_titlebar(ui, areas[0]);
        let menubar_state = app.menu.clone();
        let (_, menubar_state) = ui_menu_bar(app, ui, areas[1], &TOP_TABS, menubar_state);
        app.menu = menubar_state;

        app.work_rect = areas[2];
        let mut mgr = app.win_mgr.take();
        mgr.plan(app, ui, areas[2], areas[3]);
        app.win_mgr = mgr;
        let plots = app.win_mgr.plots.clone();
        app.chart_interact(ui, &plots, &[]);
        for (pane, plot) in plots.iter().enumerate() {
            if plot.is_positive() {
                app.clamp_view(pane, plot.width());
            }
        }
        app.update_crosshair(ui, &plots);
        let mut mgr = app.win_mgr.take();
        mgr.draw(app, ui, areas[2], areas[3]);
        app.win_mgr = mgr;
        app.run_win_cmd();
        app.paint_separators(&painter, layout, areas);
        border_hard(&painter, layout, C_CTRL_BORDER);

        app.ui_pane_menu(ui);
    });
    let shapes = std::mem::take(&mut out.shapes);
    absorb_textures(&mut out.textures_delta);
    let n = shapes.len();
    let t = std::time::Instant::now();
    let prim = ctx.tessellate(shapes, 1.0);
    println!(
        "[计时] 一帧形状 {n} 个 / 图元 {} 个：构建 + 镶嵌 {:?}",
        prim.len(),
        t.elapsed()
    );
    rasterize_into(&prim, WINDOW_W as usize, WINDOW_H as usize)
}

fn write_bmp(path: &str, grid: &[Vec<Option<egui::Color32>>]) {
    let h = grid.len();
    let w = grid.first().map_or(0, |r| r.len());
    let row_pad = (4 - (w * 3) % 4) % 4;
    let row_size = w * 3 + row_pad;
    let pixel_bytes = row_size * h;
    let file_size = 54 + pixel_bytes;
    let mut out = Vec::with_capacity(file_size);
    out.extend_from_slice(b"BM");
    out.extend_from_slice(&(file_size as u32).to_le_bytes());
    out.extend_from_slice(&0u32.to_le_bytes());
    out.extend_from_slice(&54u32.to_le_bytes());
    out.extend_from_slice(&40u32.to_le_bytes());
    out.extend_from_slice(&(w as i32).to_le_bytes());
    out.extend_from_slice(&(h as i32).to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&24u16.to_le_bytes());
    out.extend_from_slice(&0u32.to_le_bytes());
    out.extend_from_slice(&(pixel_bytes as u32).to_le_bytes());
    out.extend_from_slice(&2835u32.to_le_bytes());
    out.extend_from_slice(&2835u32.to_le_bytes());
    out.extend_from_slice(&0u32.to_le_bytes());
    out.extend_from_slice(&0u32.to_le_bytes());
    for y in (0..h).rev() {
        for px in &grid[y] {
            let (r, g, b) = match px {
                Some(c) => (c.r(), c.g(), c.b()),
                None => (255, 255, 255),
            };
            out.extend_from_slice(&[b, g, r]);
        }
        out.extend(std::iter::repeat_n(0u8, row_pad));
    }
    std::fs::write(path, out).expect("写 BMP 失败");
}

#[test]
fn right_anchored_labels_follow_the_window_edge_by_one_pixel() {
    let axis = egui::Rect::from_min_size(egui::pos2(700.0, 40.0), egui::vec2(302.0, 400.0));
    let wider = egui::Rect::from_min_size(axis.min, axis.size() + egui::vec2(1.0, 0.0));

    let tick = StockApp::axis_tick_x(axis);
    assert_eq!(
        StockApp::axis_tick_x(wider) - tick,
        1.0,
        "窗口宽 1px，纵轴刻度数字没跟着挪 1px"
    );

    let w = cells(4);
    let tag = StockApp::cross_axis_label_x(axis, w);
    assert_eq!(
        StockApp::cross_axis_label_x(wider, w) - tag,
        1.0,
        "窗口宽 1px，纵轴里那块数值牌没跟着挪 1px"
    );

    let plot = egui::Rect::from_min_size(egui::pos2(0.0, 40.0), egui::vec2(690.0, 400.0));
    let wider_plot = egui::Rect::from_min_size(plot.min, plot.size() + egui::vec2(1.0, 0.0));
    let time = StockApp::cross_time_label_x(plot, plot.max.x, w, plot.min.x);
    assert_eq!(
        StockApp::cross_time_label_x(wider_plot, wider_plot.max.x, w, wider_plot.min.x) - time,
        1.0,
        "窗口宽 1px，时间带那块索引牌没跟着挪 1px"
    );
}

#[test]
fn a_window_is_one_piece_while_being_dragged() {
    let ctx = egui::Context::default();
    configure_tessellation(&ctx);
    ctx.set_pixels_per_point(1.0);
    let mut app = blank_app();
    app.text_engine = HardText::new(&ctx);
    app.replace_bars(data::random_walk::generate(BAR_SEED, BAR_COUNT, BAR_START));
    app.fps = 60.0;
    app.panes[0].view.visible = visible_bars(CHART_PANE_W - LINE_W - AXIS_W, BAR_SCALE_DEFAULT);
    if app.panel_buttons.is_empty() {
        app.build_panel_buttons();
    }
    let work = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(cells(120), cells(70)));
    app.work_rect = work;

    win_frame(&mut app, &ctx, work, vec![]);
    let i = app
        .win_mgr
        .wins
        .iter()
        .position(|w| w.kind == WinKind::Chart)
        .unwrap();
    app.win_mgr.wins[i].w.size = egui::vec2(cells(40), cells(24));
    app.win_mgr.wins[i].w.pos = work.min + egui::vec2(200.0, 200.0);
    let before = app.win_mgr.wins[i].rect();
    win_frame(&mut app, &ctx, work, vec![]);

    let handle = free_band(before, 0, WIN_TITLE_ROWS).center();
    let press = vec![
        egui::Event::PointerMoved(handle),
        egui::Event::PointerButton {
            pos: handle,
            button: egui::PointerButton::Primary,
            pressed: true,
            modifiers: egui::Modifiers::NONE,
        },
    ];
    let _ = win_frame(&mut app, &ctx, work, press);
    let moved = handle + egui::vec2(3.0, 3.0);
    let dragged = win_frame(&mut app, &ctx, work, vec![egui::Event::PointerMoved(moved)]);
    let after = app.win_mgr.wins[i].rect();

    let want = (egui::vec2(3.0, 3.0) * WINDOW_DRAG_SPEED).round();
    assert_eq!(after.min, before.min + want, "拖动那一帧没跟手");

    app.win_mgr.wins[i].w.end_sessions();
    let reference = win_frame(&mut app, &ctx, work, vec![]);

    let (x, y) = (after.min.x as usize, after.min.y as usize);
    let (w, h) = (after.width() as usize, after.height() as usize);
    let mut diff = 0;
    for dy in 0..h {
        for dx in 0..w {
            if dragged[y + dy][x + dx] != reference[y + dy][x + dx] {
                diff += 1;
            }
        }
    }
    assert_eq!(
        diff, 0,
        "拖动的这一帧里窗口不是铁板一块（{diff} 个像素和静止的同一位置对不上）"
    );
}

#[test]
fn system_pointer_drag_reverses_immediately_after_hitting_the_bottom() {
    let ctx = egui::Context::default();
    configure_tessellation(&ctx);
    ctx.set_pixels_per_point(1.0);
    let mut app = blank_app();
    app.pointer.system = true;
    app.text_engine = HardText::new(&ctx);
    app.replace_bars(data::random_walk::generate(BAR_SEED, BAR_COUNT, BAR_START));
    if app.panel_buttons.is_empty() {
        app.build_panel_buttons();
    }
    let work = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(cells(120), cells(70)));
    app.work_rect = work;
    win_frame(&mut app, &ctx, work, vec![]);
    let i = app
        .win_mgr
        .wins
        .iter()
        .position(|w| w.kind == WinKind::Chart)
        .unwrap();
    app.win_mgr.wins[i].w.size = egui::vec2(cells(40), cells(24));
    app.win_mgr.wins[i].w.pos = work.min + egui::vec2(180.0, 120.0);
    win_frame(&mut app, &ctx, work, vec![]);
    let before = app.win_mgr.wins[i].w.rect();
    let handle = free_band(before, 0, WIN_TITLE_ROWS).center();
    let press = vec![
        egui::Event::PointerMoved(handle),
        egui::Event::PointerButton {
            pos: handle,
            button: egui::PointerButton::Primary,
            pressed: true,
            modifiers: egui::Modifiers::NONE,
        },
    ];
    win_frame(&mut app, &ctx, work, press);
    let far = egui::pos2(handle.x, work.max.y + 240.0);
    win_frame(&mut app, &ctx, work, vec![egui::Event::PointerMoved(far)]);
    let bottom = app.win_mgr.wins[i].w.rect();
    assert_eq!(bottom.max.y, work.max.y, "窗口没有贴到窗口区下沿");
    win_frame(
        &mut app,
        &ctx,
        work,
        vec![egui::Event::PointerMoved(far - egui::vec2(0.0, 1.0))],
    );
    assert!(
        app.win_mgr.wins[i].w.rect().min.y < bottom.min.y,
        "指针从下边界外回移 1px，窗口没有立刻跟手（仍在吃死区）"
    );
}

#[test]
#[ignore]
fn dump_full_window_for_visual_check() {
    let mut app = blank_app();
    app.replace_bars(data::random_walk::generate(BAR_SEED, BAR_COUNT, BAR_START));
    let ctx = egui::Context::default();
    configure_tessellation(&ctx);
    ctx.set_pixels_per_point(1.0);
    app.text_engine = HardText::new(&ctx);
    app.fps = 60.0;
    app.panes[0].view.visible = visible_bars(CHART_PANE_W - LINE_W - AXIS_W, BAR_SCALE_DEFAULT);
    let grid = render_window(&mut app);
    write_bmp("target/ui_full.bmp", &grid);

    set_theme(Theme::DARK);
    app.up_color = (
        Theme::DARK.candle_up.r(),
        Theme::DARK.candle_up.g(),
        Theme::DARK.candle_up.b(),
    );
    app.down_color = (
        Theme::DARK.candle_down.r(),
        Theme::DARK.candle_down.g(),
        Theme::DARK.candle_down.b(),
    );
    app.mark_panes_dirty();
    let dark = render_window(&mut app);
    write_bmp("target/ui_full_dark.bmp", &dark);

    set_theme(Theme::CLASSIC);
    app.up_color = UP_RGB_DEFAULT;
    app.down_color = DOWN_RGB_DEFAULT;
    app.mark_panes_dirty();

    let layout = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(WINDOW_W, WINDOW_H));

    let chart = ManagedWin::body(StockApp::child_areas(split_areas(layout)[2])[0]);
    let (_, _, main, _) = chart_rows(chart, app.panes[PANE_CANDLE].indicator.layout());
    let pos = frame_rects(main).0.center();
    let cross = render_window_at(&mut app, Some(pos));
    write_bmp("target/ui_crosshair.bmp", &cross);

    let work = split_areas(egui::Rect::from_min_size(
        egui::Pos2::ZERO,
        egui::vec2(WINDOW_W, WINDOW_H),
    ))[2];
    app.work_rect = work;
    for _ in 0..3 {
        app.add_chart_window();
    }
    let chart_i = app
        .win_mgr
        .wins
        .iter()
        .position(|w| w.kind == WinKind::Chart)
        .unwrap();
    app.win_mgr.wins[chart_i].w.pos = egui::pos2(work.min.x + 5.0, work.min.y + 5.0);
    app.win_mgr.wins[chart_i].w.size = egui::vec2(500.0, 300.0);

    let last = app.win_mgr.wins.len() - 1;
    app.win_mgr.wins[last].minimized = true;
    let wins = render_window(&mut app);
    write_bmp("target/ui_windows.bmp", &wins);
    set_theme(Theme::CLASSIC);
}

fn win_frame(
    app: &mut StockApp,
    ctx: &egui::Context,
    work: egui::Rect,
    events: Vec<egui::Event>,
) -> Vec<Vec<Option<egui::Color32>>> {
    let mut raw = egui::RawInput::default();
    raw.screen_rect = Some(work);
    raw.events = events;
    app.work_rect = work;
    configure_zero_spacing(ctx);
    let taskbar = egui::Rect::from_min_size(work.min, egui::vec2(work.width(), WIN_TITLE_H));
    let mut out = ctx.run_ui(raw, |ui| {
        let mut mgr = app.win_mgr.take();
        mgr.plan(app, ui, work, taskbar);
        app.win_mgr = mgr;
        let mut mgr = app.win_mgr.take();
        mgr.draw(app, ui, work, taskbar);
        app.win_mgr = mgr;

        app.run_win_cmd();
    });
    absorb_textures(&mut out.textures_delta);
    rasterize_into(
        &ctx.tessellate(std::mem::take(&mut out.shapes), 1.0),
        WINDOW_W as usize,
        WINDOW_H as usize,
    )
}

fn win_frame_cursor(
    app: &mut StockApp,
    ctx: &egui::Context,
    work: egui::Rect,
    events: Vec<egui::Event>,
) -> egui::CursorIcon {
    let mut raw = egui::RawInput::default();
    raw.screen_rect = Some(work);
    raw.events = events;
    app.work_rect = work;
    configure_zero_spacing(ctx);
    let taskbar = egui::Rect::from_min_size(work.min, egui::vec2(work.width(), WIN_TITLE_H));
    let out = ctx.run_ui(raw, |ui| {
        let mut mgr = app.win_mgr.take();
        mgr.plan(app, ui, work, taskbar);
        app.win_mgr = mgr;
        let mut mgr = app.win_mgr.take();
        mgr.draw(app, ui, work, taskbar);
        app.win_mgr = mgr;
        app.run_win_cmd();
    });
    let mut out = out;
    absorb_textures(&mut out.textures_delta);
    out.platform_output.cursor_icon
}

#[test]
fn window_move_and_resize_are_free_and_stop_at_the_minimum() {
    let ctx = egui::Context::default();
    configure_tessellation(&ctx);
    ctx.set_pixels_per_point(1.0);
    let mut app = blank_app();
    if let Some(panel) = app
        .win_mgr
        .wins
        .iter()
        .position(|w| w.kind == WinKind::Panel)
    {
        app.win_mgr.set_pin(panel, true);
        app.win_mgr.wins[panel].exclusive = false;
    }
    app.text_engine = HardText::new(&ctx);
    if app.panel_buttons.is_empty() {
        app.build_panel_buttons();
    }
    let work = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(cells(120), cells(60)));
    let min = WinMgr::chart_min_size();

    let chart_i = |app: &StockApp| {
        app.win_mgr
            .wins
            .iter()
            .position(|w| w.kind == WinKind::Chart)
            .expect("图表窗不见了")
    };
    let chart = |app: &StockApp| app.win_mgr.wins[chart_i(app)].rect();
    let frame = |app: &mut StockApp, events: Vec<egui::Event>| win_frame(app, &ctx, work, events);
    let press = |pos: egui::Pos2| {
        vec![
            egui::Event::PointerMoved(pos),
            egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: egui::Modifiers::NONE,
            },
        ]
    };
    let drag = |pos: egui::Pos2| vec![egui::Event::PointerMoved(pos)];
    let release = |pos: egui::Pos2| {
        vec![egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: egui::Modifiers::NONE,
        }]
    };

    frame(&mut app, vec![]);
    let first = chart(&app);
    assert_eq!(
        first,
        egui::Rect::from_center_size(work.center(), egui::vec2(NEW_WIN_W, NEW_WIN_H)),
        "图表窗没在窗口区正中间出生"
    );
    assert!(
        app.win_mgr
            .wins
            .iter()
            .any(|w| w.kind == WinKind::Panel && w.rect().max == work.max),
        "按钮面板没摆到右下角"
    );

    let edge = egui::pos2(first.max.x - 1.0, first.center().y);
    frame(&mut app, press(edge));
    frame(
        &mut app,
        drag(egui::pos2(first.min.x + 10.0, first.center().y)),
    );
    let shrunk = chart(&app);
    assert_eq!(shrunk.width(), min.x, "缩过头了：最小尺寸没托住");
    assert_eq!(shrunk.min.x, first.min.x, "拖右边不该动左边");
    assert!(shrunk.width() >= min.x);
    frame(&mut app, release(egui::pos2(work.max.x, work.max.y)));

    let bottom = egui::pos2(shrunk.center().x, shrunk.max.y - 1.0);
    frame(&mut app, press(bottom));
    frame(&mut app, drag(egui::pos2(bottom.x, shrunk.min.y + 10.0)));
    let tall = chart(&app);
    assert_eq!(tall.height(), min.y, "缩过头了：最小高度没托住");
    assert_eq!(tall.min.y, shrunk.min.y, "拖下边不该动上边");
    assert!(tall.max.y <= work.max.y, "窗口越出窗口区了");
    frame(&mut app, release(egui::pos2(work.max.x, work.max.y)));
    assert!(
        !app.win_mgr.wins[chart_i(&app)].w.resizing() && !app.win_mgr.drag_active(),
        "松手之后会话没结束"
    );

    let corner = chart(&app);
    let grab = corner.max - egui::vec2(LINE_W, LINE_W);
    frame(&mut app, press(grab));
    frame(&mut app, drag(grab + egui::vec2(90.0, 40.0)));
    let grown = chart(&app);
    assert_eq!(grown.min, corner.min, "拖右下角不该动左上角");
    assert_eq!(
        grown.width(),
        corner.width() + 90.0,
        "拖右下角没做到 1px 跟手"
    );
    assert_eq!(
        grown.height(),
        corner.height() + 40.0,
        "拖右下角没做到 1px 跟手"
    );
    frame(&mut app, release(egui::pos2(work.max.x, work.max.y)));

    frame(&mut app, press(egui::pos2(grown.min.x, grown.center().y)));
    frame(
        &mut app,
        drag(egui::pos2(grown.min.x + 60.0, grown.center().y)),
    );
    let from_left = chart(&app);
    assert_eq!(from_left.max.x, grown.max.x, "拖左边不该动右边");
    assert_eq!(from_left.min.x, grown.min.x + 60.0, "拖左边没做到 1px 跟手");
    assert_eq!(
        from_left.width(),
        grown.width() - 60.0,
        "拖左边没做到 1px 跟手"
    );
    frame(
        &mut app,
        drag(egui::pos2(work.min.x - 300.0, grown.center().y)),
    );
    assert_eq!(chart(&app).min.x, work.min.x, "往左拖没贴到窗口区左边框");
    frame(&mut app, release(egui::pos2(work.max.x, work.max.y)));

    for probe in 0..3 {
        let cur = chart(&app);
        let p = match probe {
            0 => egui::pos2(cur.center().x, cur.min.y + LINE_W),
            1 => cur.min + egui::vec2(LINE_W, LINE_W),
            _ => egui::pos2(cur.max.x - LINE_W, cur.min.y + LINE_W),
        };
        frame(&mut app, press(p));
        frame(&mut app, drag(p + egui::vec2(0.0, 40.0)));
        assert_eq!(
            chart(&app).size(),
            cur.size(),
            "上边 / 上边那两个角不该能拽着改尺寸（{p:?}）"
        );

        if probe < 2 {
            assert_eq!(
                chart(&app).min,
                cur.min + (egui::vec2(0.0, 40.0) * WINDOW_DRAG_SPEED).round(),
                "上边那一按该是拖窗口（{p:?}）"
            );
        }
        frame(&mut app, release(egui::pos2(work.max.x, work.max.y)));
    }

    for probe in 0..2 {
        let cur = chart(&app);
        let title_band = free_band(cur, 0, WIN_TITLE_ROWS);
        let p = if probe == 0 {
            egui::pos2(cur.min.x, title_band.center().y)
        } else {
            egui::pos2(cur.max.x - LINE_W, title_band.center().y)
        };
        frame(&mut app, press(p));
        frame(&mut app, drag(p + egui::vec2(-30.0, 0.0)));
        assert_eq!(
            chart(&app).size(),
            cur.size(),
            "标题栏那一段的左右边不该能拽着改尺寸（{p:?}）"
        );
        frame(&mut app, release(egui::pos2(work.max.x, work.max.y)));
    }

    let before_br = chart(&app);
    let br = before_br.max - egui::vec2(LINE_W, LINE_W);
    frame(&mut app, press(br));
    frame(&mut app, drag(br + egui::vec2(40.0, 20.0)));
    let grown_br = chart(&app);
    assert_eq!(grown_br.min, before_br.min, "拖右下角不该动左上角");
    assert_eq!(
        grown_br.width(),
        before_br.width() + 40.0,
        "拖右下角没做到 1px 跟手"
    );
    assert_eq!(
        grown_br.height(),
        before_br.height() + 20.0,
        "拖右下角没做到 1px 跟手"
    );
    frame(&mut app, release(egui::pos2(work.max.x, work.max.y)));

    let before = chart(&app);
    let handle = free_band(before, 0, WIN_TITLE_ROWS).center();
    let walk = egui::vec2(37.0, 23.0);
    frame(&mut app, press(handle));
    assert_eq!(
        chart_i(&app),
        app.win_mgr.wins.len() - 1,
        "点标题栏没把窗口抬到最上层"
    );
    frame(&mut app, drag(handle + walk));
    let moved = chart(&app);

    assert_eq!(
        moved.min,
        before.min + (walk * WINDOW_DRAG_SPEED).round(),
        "拖标题栏没跟手"
    );
    assert_eq!(moved.size(), before.size(), "拖窗口不该改尺寸");

    let drag_y = handle.y + walk.y;
    frame(&mut app, drag(egui::pos2(work.max.x + 300.0, drag_y)));
    let at_edge = chart(&app);
    assert_eq!(
        at_edge.max.x, work.max.x,
        "往右拖出窗口区了 / 右边框没贴到窗口区右边框"
    );
    let grab = handle - before.min;
    let pin = app.pointer.pinned.expect("拖标题栏时该把指针钉在抓点上");
    assert_eq!(
        pin,
        at_edge.min + grab,
        "指针没钉在按下那一刻抓住的那一格上"
    );
    frame(&mut app, drag(egui::pos2(pin.x - 20.0, pin.y)));
    let pulled = chart(&app);
    assert!(pulled.max.x < work.max.x, "往回拉之后窗口还钉在边缘上没动");
    assert_eq!(
        pulled.min.x,
        at_edge.min.x - (20.0 * WINDOW_DRAG_SPEED).round(),
        "往回拉没跟手"
    );

    let last_pin = app.pointer.pinned.expect("往回拉的那一帧指针还该钉着");
    let want = egui::pos2(pin.x - 60.0, pin.y);
    let step = last_pin.x - want.x;
    frame(&mut app, drag(want));
    assert_eq!(
        chart(&app).min.x,
        pulled.min.x - (step * WINDOW_DRAG_SPEED).round(),
        "往回拉没跟手（中间夹了一段空距离）"
    );
    frame(&mut app, release(moved.center()));

    let now = chart(&app);
    frame(&mut app, press(free_band(now, 0, WIN_TITLE_ROWS).center()));
    frame(
        &mut app,
        drag(egui::pos2(work.min.x - 500.0, work.min.y - 500.0)),
    );
    assert_eq!(chart(&app).min, work.min, "窗口拖出窗口区了");
    frame(&mut app, release(moved.center()));

    let at = chart(&app);
    frame(&mut app, press(egui::pos2(at.center().x, at.max.y - 1.0)));
    frame(
        &mut app,
        drag(egui::pos2(at.center().x, work.max.y + 300.0)),
    );
    assert_eq!(chart(&app).max.y, work.max.y, "下边没贴到窗口区下边框");
    frame(&mut app, release(egui::pos2(work.max.x, work.max.y)));

    if let Some(color) = app.win_mgr.color_index() {
        assert!(!app.win_mgr.wins[color].w.resizable, "颜色面板不该能改尺寸");
    }
    assert_eq!(
        app.win_mgr
            .wins
            .iter()
            .find(|w| w.kind == WinKind::Panel)
            .map(|w| w.w.min_size),
        Some(app.panel_min_size()),
        "按钮面板的尺寸下限不对"
    );

    let src = app.win_mgr.show_source(work);
    assert!(app.win_mgr.wins[src].w.resizable);
    assert_eq!(app.win_mgr.wins[src].w.min_size, egui::vec2(SRC_W, SRC_H));
}

#[test]
fn a_press_on_the_one_pixel_edge_drags_right_away() {
    let ctx = egui::Context::default();
    configure_tessellation(&ctx);
    ctx.set_pixels_per_point(1.0);
    let mut app = blank_app();
    app.text_engine = HardText::new(&ctx);
    if app.panel_buttons.is_empty() {
        app.build_panel_buttons();
    }
    let work = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(cells(120), cells(70)));
    app.work_rect = work;
    let frame = |app: &mut StockApp, events: Vec<egui::Event>| win_frame(app, &ctx, work, events);
    let drag = |pos: egui::Pos2| vec![egui::Event::PointerMoved(pos)];
    let release = |pos: egui::Pos2| {
        vec![egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: egui::Modifiers::NONE,
        }]
    };

    let press_and_drag = |from: egui::Pos2, to: egui::Pos2| {
        vec![
            egui::Event::PointerMoved(from),
            egui::Event::PointerButton {
                pos: from,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: egui::Modifiers::NONE,
            },
            egui::Event::PointerMoved(to),
        ]
    };

    frame(&mut app, vec![]);
    let i = app
        .win_mgr
        .wins
        .iter()
        .position(|w| w.kind == WinKind::Chart)
        .expect("图表窗没建出来");
    app.win_mgr.wins[i].w.size = egui::vec2(cells(40), cells(24));
    app.win_mgr.wins[i].w.pos = work.min + egui::vec2(200.0, 200.0);
    frame(&mut app, vec![]);
    let start = app.win_mgr.wins[i].rect();

    let grab = egui::pos2(start.max.x - LINE_W, start.center().y);
    frame(&mut app, press_and_drag(grab, grab + egui::vec2(40.0, 0.0)));
    let grown = app.win_mgr.wins[i].rect();
    assert_eq!(
        grown.width(),
        start.width() + 40.0,
        "按住就往外拖没跟手（这一按被当成没抓住边了）"
    );
    assert_eq!(grown.min.x, start.min.x, "拖右边不该动左边");
    assert_eq!(grown.min.y, start.min.y, "拖右边不该动上边");

    frame(&mut app, drag(grab + egui::vec2(90.0, 0.0)));
    assert_eq!(
        app.win_mgr.wins[i].rect().width(),
        start.width() + 90.0,
        "第二帧没跟手"
    );
    frame(&mut app, release(grab + egui::vec2(90.0, 0.0)));
    assert!(
        !app.win_mgr.wins[i].w.resizing() && !app.win_mgr.drag_active(),
        "松手之后会话没结束"
    );

    let at = app.win_mgr.wins[i].rect();
    let grab = egui::pos2(at.max.x - LINE_W, at.center().y);
    frame(&mut app, press_and_drag(grab, grab - egui::vec2(50.0, 0.0)));
    let shrunk = app.win_mgr.wins[i].rect();
    assert_eq!(shrunk.width(), at.width() - 50.0, "按住往里拖没跟手");
    assert_eq!(shrunk.min.x, at.min.x, "拖右边不该动左边");
    frame(&mut app, release(grab));

    let at = app.win_mgr.wins[i].rect();
    let grab = egui::pos2(at.max.x - LINE_W, at.center().y);
    frame(
        &mut app,
        vec![
            egui::Event::PointerMoved(grab),
            egui::Event::PointerButton {
                pos: grab,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: egui::Modifiers::NONE,
            },
        ],
    );
    frame(&mut app, drag(grab + egui::vec2(30.0, 0.0)));
    assert_eq!(
        app.win_mgr.wins[i].rect().width(),
        at.width() + 30.0,
        "按住停一帧再拖没跟手"
    );
    frame(&mut app, release(grab + egui::vec2(30.0, 0.0)));

    let at = app.win_mgr.wins[i].rect();
    let grab = egui::pos2(at.center().x, at.max.y - LINE_W);
    frame(&mut app, press_and_drag(grab, grab + egui::vec2(0.0, 20.0)));
    let taller = app.win_mgr.wins[i].rect();
    assert_eq!(
        taller.height(),
        at.height() + 20.0,
        "下边外半截按住就往下拖没跟手"
    );
    assert_eq!(taller.min.y, at.min.y, "拖下边不该动上边");
    frame(&mut app, release(grab + egui::vec2(0.0, 20.0)));
}

#[test]
fn window_contents_travel_with_the_window_by_one_pixel() {
    let ctx = egui::Context::default();
    configure_tessellation(&ctx);
    ctx.set_pixels_per_point(1.0);
    let mut app = blank_app();
    app.text_engine = HardText::new(&ctx);
    app.replace_bars(data::random_walk::generate(BAR_SEED, BAR_COUNT, BAR_START));
    if app.panel_buttons.is_empty() {
        app.build_panel_buttons();
    }
    let layout = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(WINDOW_W, WINDOW_H));
    let work = split_areas(layout)[2];
    app.work_rect = work;

    win_frame(&mut app, &ctx, work, vec![]);
    let i = app
        .win_mgr
        .wins
        .iter()
        .position(|w| w.kind == WinKind::Chart)
        .unwrap();

    app.win_mgr.wins[i].w.size = egui::vec2(NEW_WIN_W, NEW_WIN_H);
    app.win_mgr.wins[i].w.pos = work.min + egui::vec2(200.0, 300.0);
    let start = app.win_mgr.wins[i].rect().min;
    let size = app.win_mgr.wins[i].w.size;
    let a = render_window(&mut app);

    app.win_mgr.wins[i].w.pos = start + egui::vec2(1.0, 1.0);
    let b = render_window(&mut app);

    let (x, y) = (start.x as usize, start.y as usize);
    let (w, h) = (size.x as usize, size.y as usize);
    assert!(
        x + w + 1 < WINDOW_W as usize && y + h + 1 < WINDOW_H as usize,
        "窗口别贴边"
    );
    let mut diff = 0;
    for dy in 0..h {
        for dx in 0..w {
            if a[y + dy][x + dx] != b[y + dy + 1][x + dx + 1] {
                diff += 1;
            }
        }
    }
    assert_eq!(
        diff, 0,
        "窗口挪 1px 之后里头的元素没跟着一起挪（{diff} 个像素对不上）"
    );
}

#[test]
fn window_overflow_lands_in_the_elastic_bands() {
    let win = egui::Rect::from_min_size(egui::pos2(7.0, 4.0), egui::vec2(900.0, 500.0));
    assert!(
        win.width() % 9.0 != 1.0 && win.height() % 9.0 != 1.0,
        "这块窗口得是「非整格」的"
    );

    let title = free_band(win, 0, WIN_TITLE_ROWS);
    assert_eq!(title.height(), WIN_TITLE_H, "窗口标题栏高度被零头改了");
    assert_eq!(title.min.y, win.min.y);
    let body = ManagedWin::body(win);
    let layout = indicator::ChartLayout::new(2, 2);
    let (bar, status, main, time) = chart_rows(body, layout);
    let (status, time) = (status.unwrap(), time.unwrap());
    assert_eq!(bar.height(), PANE_TITLE_H, "图表栏高度被零头改了");
    assert_eq!(status.height(), cells(2), "状态带高度被零头改了");
    assert_eq!(time.height(), cells(2), "时间带高度被零头改了");

    assert_eq!(
        time.max.y,
        chart_bar(body).min.y + LINE_W,
        "时间带和状态栏没共用那条线"
    );
    assert_eq!(
        chart_bar(body).max.y,
        win.max.y,
        "图表窗状态栏没钉死在窗口下沿"
    );
    assert_eq!(main.max.y, time.min.y, "绘图区和时间带没共用那条线");
    assert_eq!(status.max.y, main.min.y, "状态带和绘图区没共用那条线");

    let snap = egui::Rect::from_min_size(
        win.min,
        egui::vec2(snap_cells_up(win.width()), snap_cells_up(win.height())),
    );
    let (_, _, main_snap, _) = chart_rows(ManagedWin::body(snap), layout);
    assert_eq!(
        main_snap.height() - main.height(),
        snap.height() - win.height(),
        "竖向零头没落在绘图区（弹性段）里"
    );

    let (plot, axis) = frame_rects(main);
    assert_eq!(axis.width(), AXIS_W, "纵轴宽度被零头改了");
    assert_eq!(axis.max.x, win.max.x, "纵轴没钉死在窗口右边");
    assert_eq!(plot.min.x, win.min.x + LINE_W);
    let (plot_snap, _) = frame_rects(main_snap);
    assert_eq!(
        plot_snap.width() - plot.width(),
        snap.width() - win.width(),
        "横向零头没落在绘图区（弹性段）里"
    );
    let keys = sys_button_rects(title);
    for &k in &keys {
        assert_eq!(
            k.size(),
            egui::vec2(SYS_BTN_W, SYS_BTN_H),
            "右边那几个键被零头改了尺寸"
        );
    }
    assert_eq!(keys[3].max.x, title.max.x, "关闭键没钉在窗口右边");
    assert_eq!(keys[3].min.y, win.min.y, "关闭键没压在窗口标题栏上");

    assert_eq!(keys[1].min.x, keys[0].max.x - LINE_W, "最小化没挨着置顶");
    assert_eq!(keys[2].min.x, keys[1].max.x - LINE_W, "最大化没挨着最小化");
    assert_eq!(keys[3].min.x, keys[2].max.x - LINE_W, "关闭没挨着最大化");
}

#[test]
fn window_title_keys_raise_minimize_and_close() {
    let ctx = egui::Context::default();
    configure_tessellation(&ctx);
    ctx.set_pixels_per_point(1.0);
    let mut app = blank_app();
    app.text_engine = HardText::new(&ctx);
    if app.panel_buttons.is_empty() {
        app.build_panel_buttons();
    }
    let work = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(cells(120), cells(60)));
    let chart_i = |app: &StockApp| {
        app.win_mgr
            .wins
            .iter()
            .position(|w| w.kind == WinKind::Chart)
            .expect("图表窗不见了")
    };
    let click = |pos: egui::Pos2| {
        vec![
            egui::Event::PointerMoved(pos),
            egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: egui::Modifiers::NONE,
            },
        ]
    };
    let release = |pos: egui::Pos2| {
        vec![egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: egui::Modifiers::NONE,
        }]
    };

    win_frame(&mut app, &ctx, work, vec![]);
    app.add_chart_window();
    win_frame(&mut app, &ctx, work, vec![]);
    let new_id = app.win_mgr.wins.last().unwrap().w.id;
    assert!(
        app.win_mgr.wins.last().unwrap().kind == WinKind::Chart,
        "新增的不是图表窗"
    );

    let old = chart_i(&app);
    let old_id = app.win_mgr.wins[old].w.id;
    assert!(old_id != new_id);
    let new_i = app
        .win_mgr
        .wins
        .iter()
        .position(|w| w.w.id == new_id)
        .unwrap();
    app.win_mgr.wins[new_i].w.pos = work.max - app.win_mgr.wins[new_i].w.size;
    win_frame(&mut app, &ctx, work, vec![]);
    let title = free_band(app.win_mgr.wins[old].rect(), 0, WIN_TITLE_ROWS);
    let keys = sys_button_rects(title);
    let handle = egui::pos2((title.min.x + keys[0].min.x) / 2.0, title.center().y);
    win_frame(&mut app, &ctx, work, click(handle));
    win_frame(&mut app, &ctx, work, release(handle));
    assert_eq!(
        app.win_mgr.wins.last().unwrap().w.id,
        old_id,
        "点标题栏没抬到最上层"
    );

    let i = app
        .win_mgr
        .wins
        .iter()
        .position(|w| w.w.id == old_id)
        .unwrap();
    let before = app.win_mgr.wins[i].rect();
    let min_key = sys_button_rects(free_band(before, 0, WIN_TITLE_ROWS))[1].center();
    win_frame(&mut app, &ctx, work, click(min_key));
    win_frame(&mut app, &ctx, work, release(min_key));
    let i = app
        .win_mgr
        .wins
        .iter()
        .position(|w| w.w.id == old_id)
        .unwrap();
    assert!(app.win_mgr.wins[i].minimized, "点最小化键没最小化");
    assert_eq!(
        app.win_mgr.rect_of(i).min,
        app.win_mgr.taskbar_items_rect().min,
        "最小化之后没堆到左上角"
    );
    let title = app.win_mgr.wins[i].title.clone();
    assert_eq!(
        app.win_mgr.rect_of(i).size(),
        egui::vec2(WinMgr::mini_width(&mut app, &title), WIN_TITLE_H)
    );

    win_frame(&mut app, &ctx, work, vec![]);
    let mini = app.win_mgr.rect_of(i);
    let restore_key = sys_button_rects_n(free_band(mini, 0, WIN_TITLE_ROWS), 3)[1].center();
    win_frame(&mut app, &ctx, work, click(restore_key));
    win_frame(&mut app, &ctx, work, release(restore_key));
    let i = app
        .win_mgr
        .wins
        .iter()
        .position(|w| w.w.id == old_id)
        .unwrap();
    assert!(!app.win_mgr.wins[i].minimized, "再点一次没还原");
    assert_eq!(
        app.win_mgr.wins[i].rect(),
        before,
        "还原之后没回到原位原尺寸"
    );

    win_frame(&mut app, &ctx, work, vec![]);
    let close_key = sys_button_rects(free_band(before, 0, WIN_TITLE_ROWS))[3].center();
    let was = app.win_mgr.chart_count();
    win_frame(&mut app, &ctx, work, click(close_key));
    win_frame(&mut app, &ctx, work, release(close_key));
    let i = app
        .win_mgr
        .wins
        .iter()
        .position(|w| w.w.id == old_id)
        .unwrap();
    assert!(!app.win_mgr.wins[i].w.open, "点关闭键没把窗口关掉");
    assert_eq!(app.win_mgr.chart_count(), was - 1, "关掉之后图表窗数没减");

    let panes_before = app.panes.len();
    let closed_pane = app.win_mgr.wins[i].pane;
    app.add_chart_window();
    assert_eq!(
        app.panes.len(),
        panes_before,
        "关掉的窗口留下的 Pane 槽没复用"
    );
    assert_eq!(app.win_mgr.wins.last().unwrap().pane, closed_pane);
}

#[test]
fn fresh_chart_title_bar_gets_the_move_cursor() {
    let ctx = egui::Context::default();
    configure_tessellation(&ctx);
    ctx.set_pixels_per_point(1.0);
    let mut app = blank_app();
    app.text_engine = HardText::new(&ctx);
    app.replace_bars(data::random_walk::generate(BAR_SEED, 64, BAR_START));
    let frame = |app: &mut StockApp, events: Vec<egui::Event>| {
        let _ = render_events_in(&ctx, app, events);
    };
    frame(&mut app, vec![]);
    app.add_chart_window();
    frame(&mut app, vec![]);

    let chart = app
        .win_mgr
        .wins
        .iter()
        .rposition(|window| window.kind == WinKind::Chart)
        .expect("新增图表窗不存在");
    let title = app.win_mgr.wins[chart].w.title_band();
    let keys = sys_button_rects(title);
    let handle = egui::pos2((title.min.x + keys[0].min.x) / 2.0, title.center().y);
    frame(&mut app, vec![egui::Event::PointerMoved(handle)]);
    assert_eq!(
        app.cursor_kind(&ctx),
        CursorKind::Move,
        "新建图表后标题栏没有移动光标"
    );
}

#[test]
fn window_manager_cascades_minimizes_and_caps() {
    let work = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(cells(120), cells(60)));
    let mut app = blank_app();
    app.work_rect = work;
    app.win_mgr.work = work;
    app.win_mgr.taskbar =
        egui::Rect::from_min_size(work.min, egui::vec2(work.width(), WIN_TITLE_H));
    if app.panel_buttons.is_empty() {
        app.build_panel_buttons();
    }
    let docked = StockApp::child_areas(work);
    let chart = |app: &StockApp| {
        app.win_mgr
            .wins
            .iter()
            .position(|w| w.kind == WinKind::Chart)
            .unwrap()
    };
    let panel = app
        .win_mgr
        .wins
        .iter()
        .position(|w| w.kind == WinKind::Panel)
        .unwrap();
    app.win_mgr.dock(chart(&app), docked[0]);
    app.win_mgr.dock(panel, docked[1]);

    app.add_chart_window();
    assert_eq!(app.win_mgr.chart_count(), 2);
    let centered = egui::Rect::from_center_size(work.center(), egui::vec2(NEW_WIN_W, NEW_WIN_H));
    let a = app.win_mgr.wins.last().unwrap().rect();
    assert_eq!(a, centered, "新窗口没有居中出生");
    assert_eq!(a.size(), egui::vec2(NEW_WIN_W, NEW_WIN_H));
    app.add_chart_window();
    let b = app.win_mgr.wins.last().unwrap().rect();
    assert_eq!(b, centered);
    app.add_chart_window();
    let c = app.win_mgr.wins.last().unwrap().rect();
    assert_eq!(c, centered);

    assert_eq!(app.win_mgr.chart_panes().len(), 4);
    assert_eq!(
        app.win_mgr
            .chart_panes()
            .iter()
            .collect::<std::collections::HashSet<_>>()
            .len(),
        4,
        "两块图表窗共用了同一个 Pane"
    );

    for _ in 0..4 {
        app.add_chart_window();
    }
    assert_eq!(app.win_mgr.chart_count(), MAX_CHART_WINS);
    app.add_chart_window();
    assert_eq!(
        app.win_mgr.chart_count(),
        MAX_CHART_WINS,
        "超过上限还往里加"
    );
    assert!(
        app.status_msg.contains("最多"),
        "到上限了没回显一句：{}",
        app.status_msg
    );

    let bottom = chart(&app);
    let bottom_id = app.win_mgr.wins[bottom].w.id;
    app.win_mgr.activate(bottom);
    assert_eq!(
        app.win_mgr.wins.last().unwrap().w.id,
        bottom_id,
        "激活之后没排到最上层"
    );

    let i = chart(&app);
    let before = app.win_mgr.wins[i].rect();

    let title = app.win_mgr.wins[i].title.clone();
    let w = WinMgr::mini_width(&mut app, &title);
    app.win_mgr.minimize(i, w);
    let mini = app.win_mgr.rect_of(i);
    assert_eq!(mini.min, work.min, "最小化的窗口没堆在窗口区左上角");
    assert_eq!(
        mini.size(),
        egui::vec2(MINI_W, WIN_TITLE_H),
        "最小化之后不是一条标题栏"
    );
    let j = app
        .win_mgr
        .wins
        .iter()
        .position(|w| w.kind == WinKind::Panel)
        .unwrap();
    let title = app.win_mgr.wins[j].title.clone();
    let w = WinMgr::mini_width(&mut app, &title);
    app.win_mgr.minimize(j, w);
    let second = app.win_mgr.rect_of(j);
    assert_eq!(
        second.min.x,
        mini.max.x - LINE_W,
        "第二块最小化的窗口没接在任务栏右侧"
    );
    assert_eq!(second.min.y, mini.min.y);

    let need = app.text_width(&title) + 2.0 * TEXT_PAD_X + 2.0 * SYS_BTN_W - LINE_W;
    assert!(
        second.width() >= need,
        "最小化那条太窄：标题 {title:?} 的整串文字放不下"
    );
    app.win_mgr.restore(i);
    assert_eq!(
        app.win_mgr.wins[i].rect(),
        before,
        "还原之后没回到原位原尺寸"
    );
}

#[test]
fn panel_add_chart_button_opens_a_window() {
    let ctx = egui::Context::default();
    configure_tessellation(&ctx);
    ctx.set_pixels_per_point(1.0);
    let mut app = blank_app();
    app.text_engine = HardText::new(&ctx);
    app.replace_bars(data::random_walk::generate(BAR_SEED, BAR_COUNT, BAR_START));
    app.build_panel_buttons();
    let work = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(cells(120), cells(60)));
    let click = |pos: egui::Pos2| {
        vec![
            egui::Event::PointerMoved(pos),
            egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: egui::Modifiers::NONE,
            },
        ]
    };
    let release = |pos: egui::Pos2| {
        vec![egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: egui::Modifiers::NONE,
        }]
    };
    win_frame(&mut app, &ctx, work, vec![]);

    let k = app
        .panel_buttons
        .iter()
        .position(|b| b.action == PanelAction::AddChart)
        .expect("控制面板里没有「新增图表」按钮");
    let panel_i = app
        .win_mgr
        .wins
        .iter()
        .position(|w| w.kind == WinKind::Panel)
        .expect("控制面板不见了");
    let panel = app.win_mgr.wins[panel_i].rect();
    let body = ManagedWin::body(panel);
    let rect = egui::Rect::from_min_size(
        egui::pos2(
            body.min.x + grid_line(1),
            body.min.y + grid_line(1) + BTN_STEP * k as f32,
        ),
        egui::vec2(app.panel_buttons[k].button.w, BTN_H),
    );

    let before = app.win_mgr.chart_count();
    let p = rect.center();

    win_frame(&mut app, &ctx, work, vec![egui::Event::PointerMoved(p)]);
    win_frame(&mut app, &ctx, work, click(p));

    let panel_front = app
        .win_mgr
        .wins
        .iter()
        .position(|w| w.kind == WinKind::Panel)
        .expect("控制面板不见了");
    assert_eq!(
        app.win_mgr.live_index(),
        Some(panel_front),
        "点一下没把控制面板抬到最前面"
    );
    win_frame(&mut app, &ctx, work, release(p));
    assert_eq!(
        app.win_mgr.chart_count(),
        before + 1,
        "点一下没做到「既激活又按下按钮」（控制面板上的「新增图表」）"
    );
}

#[test]
fn control_panel_lists_only_the_current_actions() {
    let ctx = egui::Context::default();
    configure_tessellation(&ctx);
    ctx.set_pixels_per_point(1.0);
    let mut app = blank_app();
    app.text_engine = HardText::new(&ctx);
    app.build_panel_buttons();
    let labels: Vec<String> = app
        .panel_buttons
        .iter()
        .map(|b| b.button.content.label())
        .collect();
    assert_eq!(labels.len(), PANEL_BTN_COUNT as usize);
    for removed in [
        "放大",
        "缩小",
        "鼠标速度 +",
        "鼠标速度 -",
        "鼠标加速 +",
        "鼠标加速 -",
        "UI工程测试",
    ] {
        assert!(
            !labels.iter().any(|label| label == removed),
            "控制面板仍保留「{removed}」按钮"
        );
    }
    assert!(labels.iter().any(|label| label == "鼠标轨迹"));

    assert!(!app.pointer.show_trail);
    app.run_action(PanelAction::TogglePointerTrail);
    assert!(app.pointer.show_trail);
    assert!(app.status_msg.contains("鼠标轨迹：开"));
    app.run_action(PanelAction::TogglePointerTrail);
    assert!(!app.pointer.show_trail);
    assert!(app.status_msg.contains("鼠标轨迹：关"));
}

#[test]
fn pointer_mode_button_switches_to_the_native_cursor() {
    let ctx = egui::Context::default();
    configure_tessellation(&ctx);
    ctx.set_pixels_per_point(1.0);
    let mut app = blank_app();
    app.text_engine = HardText::new(&ctx);
    app.replace_bars(data::random_walk::generate(BAR_SEED, 64, BAR_START));
    app.build_panel_buttons();
    assert!(
        app.panel_buttons.iter().any(|b| {
            b.button.content.label() == "切换鼠标控制" && b.action == PanelAction::TogglePointerMode
        }),
        "控制面板缺少「切换鼠标控制」按钮"
    );

    let at = egui::pos2(420.0, 300.0);
    let (_, icon, image) =
        render_pinned_events_in(&ctx, &mut app, vec![egui::Event::PointerMoved(at)]);
    assert_eq!(icon, egui::CursorIcon::None, "软件模式应藏掉系统指针");
    assert!(image.is_none(), "软件模式不该给系统交自定义光标位图");

    app.run_action(PanelAction::TogglePointerMode);
    assert!(app.pointer.system);
    assert!(app.status_msg.contains("系统鼠标"));
    let (_, icon, image) =
        render_pinned_events_in(&ctx, &mut app, vec![egui::Event::PointerMoved(at)]);
    assert_eq!(
        icon,
        egui::CursorIcon::Default,
        "系统模式应让系统指针原样显示"
    );
    assert!(image.is_none(), "系统模式不该再交自定义光标位图");

    app.run_action(PanelAction::TogglePointerMode);
    assert!(!app.pointer.system);
    assert!(app.status_msg.contains("软件鼠标"));
}

#[test]
fn settings_button_opens_console_and_console_commands_work() {
    let ctx = egui::Context::default();
    configure_tessellation(&ctx);
    ctx.set_pixels_per_point(1.0);
    let mut app = blank_app();
    app.text_engine = HardText::new(&ctx);
    app.build_panel_buttons();
    assert!(
        app.panel_buttons
            .iter()
            .any(|b| b.button.content.label() == "打开调试控制台"
                && b.action == PanelAction::OpenConsole),
        "控制面板缺少「打开调试控制台」按钮"
    );

    app.run_action(PanelAction::OpenConsole);
    app.run_win_cmd();
    assert!(
        app.win_mgr.console_open(),
        "「打开调试控制台」没有打开调试控制台"
    );
    assert!(
        app.win_mgr
            .wins
            .iter()
            .any(|w| w.kind == WinKind::Console && !w.minimized),
        "调试控制台窗口没有正常摆出来"
    );
    let _ = render_events_in(&ctx, &mut app, vec![]);

    let mut console = std::mem::replace(&mut app.console, crate::ui::console::DebugConsole::new());
    console.run_command(&mut app, "trail on", 1.0);
    console.run_command(&mut app, "status", 1.0);
    app.console = console;

    assert!(app.pointer.show_trail);
    assert!(app.console.log_contains("pointer pos="));
}

#[test]
fn console_footer_is_pinned_and_buttons_share_lines() {
    let ctx = egui::Context::default();
    configure_tessellation(&ctx);
    ctx.set_pixels_per_point(1.0);
    let mut app = blank_app();
    app.text_engine = HardText::new(&ctx);
    let widths = [
        app.button_width(&ButtonContent::Text("执行".to_owned())),
        app.button_width(&ButtonContent::Text("清空".to_owned())),
        app.button_width(&ButtonContent::Text("状态".to_owned())),
    ];

    let rect = egui::Rect::from_min_size(egui::pos2(200.0, 150.0), egui::vec2(700.0, 333.0));
    let layout = crate::ui::console::console_footer_layout(rect, widths);
    assert_eq!(layout.buttons[2].max, rect.max, "按钮组没有贴住右下角");
    assert_eq!(layout.input.min.y, layout.buttons[0].min.y);
    assert_eq!(
        layout.input.height(),
        layout.buttons[0].height(),
        "输入框和按钮没有同高"
    );
    assert_eq!(layout.input.min.x, rect.min.x + grid_line(1));
    assert_eq!(
        layout.input.max.x,
        layout.buttons[0].min.x + LINE_W,
        "输入框和按钮没有共用同一条网格线"
    );
    for pair in layout.buttons.windows(2) {
        assert_eq!(
            pair[0].max.x - LINE_W,
            pair[1].min.x,
            "相邻按钮没有共用同一条边框线"
        );
    }
}

#[test]
fn console_opens_at_the_default_spawn_and_the_opening_click_cannot_drag_it() {
    let ctx = egui::Context::default();
    configure_tessellation(&ctx);
    ctx.set_pixels_per_point(1.0);
    let mut app = blank_app();
    app.text_engine = HardText::new(&ctx);
    let work = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(WINDOW_W, WINDOW_H));
    let work = split_areas(work)[2];
    app.work_rect = work;
    win_frame(&mut app, &ctx, work, vec![]);
    app.open_console_window();

    let i = app
        .win_mgr
        .wins
        .iter()
        .position(|w| w.kind == WinKind::Console)
        .expect("控制台没建出来");
    let rect = app.win_mgr.wins[i].w.rect();
    assert!(
        rect.min.x >= work.min.x
            && rect.min.y >= work.min.y
            && rect.max.x <= work.max.x
            && rect.max.y <= work.max.y,
        "控制台初始位置跑出工作区：{rect:?} / {work:?}"
    );
    assert!(
        (rect.center().x - work.center().x).abs() <= 1.0
            && (rect.center().y - work.center().y).abs() <= 1.0,
        "控制台默认位置不在窗口区正中间"
    );

    let title = free_band(rect, 0, WIN_TITLE_ROWS).center();
    win_frame(
        &mut app,
        &ctx,
        work,
        vec![
            egui::Event::PointerMoved(title),
            egui::Event::PointerButton {
                pos: title,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: egui::Modifiers::NONE,
            },
        ],
    );
    win_frame(
        &mut app,
        &ctx,
        work,
        vec![egui::Event::PointerMoved(title + egui::vec2(60.0, 40.0))],
    );
    assert_eq!(
        app.win_mgr.wins[i].w.rect().min,
        rect.min,
        "打开控制台的那一下点击把窗口拖跑了"
    );

    win_frame(
        &mut app,
        &ctx,
        work,
        vec![egui::Event::PointerButton {
            pos: title + egui::vec2(60.0, 40.0),
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: egui::Modifiers::NONE,
        }],
    );
    win_frame(
        &mut app,
        &ctx,
        work,
        vec![
            egui::Event::PointerMoved(title),
            egui::Event::PointerButton {
                pos: title,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: egui::Modifiers::NONE,
            },
        ],
    );
    let walk = egui::vec2(30.0, 20.0);
    win_frame(
        &mut app,
        &ctx,
        work,
        vec![egui::Event::PointerMoved(title + walk)],
    );
    let moved_by = (walk * WINDOW_DRAG_SPEED).round();
    assert_eq!(
        app.win_mgr.wins[i].w.rect().min,
        egui::pos2(
            (rect.min.x + moved_by.x).round(),
            (rect.min.y + moved_by.y).round()
        ),
        "松手之后正常拖标题栏仍然不动"
    );
}

#[test]
fn pointer_trail_overlay_draws_recent_motion() {
    let ctx = egui::Context::default();
    configure_tessellation(&ctx);
    ctx.set_pixels_per_point(1.0);
    let mut app = blank_app();
    app.text_engine = HardText::new(&ctx);
    app.replace_bars(data::random_walk::generate(BAR_SEED, 64, BAR_START));

    app.pointer = crate::pointer::PointerManager::new(crate::pointer::PointerFeel {
        base_speed: 1.0,
        acceleration: 0.0,
        ..crate::pointer::PointerFeel::default()
    });
    let pos = egui::pos2(420.0, 300.0);
    let start = pos - egui::vec2(60.0, 0.0);
    let bounds = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(WINDOW_W, WINDOW_H));
    app.pointer.reset(start, 0.0);
    for i in 1..=6 {
        app.pointer
            .advance(&[egui::vec2(10.0, 0.0)], i as f64 * 0.02, bounds);
    }
    app.pointer.show_trail = true;
    let has_trail = |grid: &Vec<Vec<Option<egui::Color32>>>| {
        grid.iter().flatten().any(|c| {
            matches!(
                c,
                Some(c) if *c == C_POINTER_TRAIL || *c == C_POINTER_TRAIL_HOT
            )
        })
    };
    let grid = render_events_in(&ctx, &mut app, vec![egui::Event::PointerMoved(pos)]);
    assert!(has_trail(&grid), "打开「鼠标轨迹」后没有画出轨迹");

    app.pointer.show_trail = false;
    let grid = render_events_in(&ctx, &mut app, vec![egui::Event::PointerMoved(pos)]);
    assert!(!has_trail(&grid), "关掉「鼠标轨迹」后调试色还在");
}

#[test]
fn menu_popup_has_dividers_and_three_level_submenus() {
    let ctx = egui::Context::default();
    configure_tessellation(&ctx);
    ctx.set_pixels_per_point(1.0);
    let mut app = blank_app();
    app.text_engine = HardText::new(&ctx);
    let layout = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(WINDOW_W, WINDOW_H));
    let areas = split_areas(layout);

    let bar_rects = {
        let mut widths = Vec::new();
        for t in TOP_TABS.iter() {
            widths.push(tab_width(&mut app, t.title));
        }
        menu_item_rects(areas[1], &widths, MenuAlign::Left)
    };
    let tab = bar_rects[0];
    app.menu.open = Some(0);
    let grid = render_overlays_at(&mut app, Some(tab.center()));

    let panel = MenuPanel::bounded(
        tab,
        menu_panel_width(&mut app, TOP_TABS[0].items),
        TOP_TABS[0].items.len(),
        layout,
    );
    for j in 0..panel.rows.saturating_sub(1) {
        let seam = (panel.row(j).max.y - LINE_W) as usize;
        for col in [panel.rect.min.x + 3.0, panel.rect.max.x - 4.0] {
            assert_eq!(
                grid[seam][col as usize],
                Some(C_LINE),
                "弹出菜单第 {j} 项下面缺少横向分隔线"
            );
        }
    }

    app.text_engine = HardText::new(&ctx);
    let recent = TOP_TABS[0]
        .items
        .iter()
        .position(|n| n.label == "最近打开")
        .expect("「最近打开」不见了");

    let frame = |app: &mut StockApp, events: Vec<egui::Event>| {
        let mut raw = egui::RawInput::default();
        raw.screen_rect = Some(layout);
        raw.events = events;
        let mut out = ctx.run_ui(raw, |ui| {
            let painter = ui.painter().clone();
            app.paint_skeleton(&painter, layout, areas);
            app.ui_titlebar(ui, areas[0]);
            let state = app.menu.clone();
            let (open_anchor, state) = ui_menu_bar(app, ui, areas[1], &TOP_TABS, state);
            app.menu = state;
            if let Some((idx, anchor)) = open_anchor {
                let items = TOP_TABS[idx].items;
                let mut state = app.menu.clone();
                if let Some(label) =
                    ui_menu_popup(app, ui, anchor, items, &mut state, layout, &[areas[1]])
                {
                    app.status_msg = format!("你点击了：{label}");
                }
                app.menu = state;
            }
        });

        absorb_textures(&mut out.textures_delta);
    };
    let hover = |pos: egui::Pos2| vec![egui::Event::PointerMoved(pos)];
    let click = |pos: egui::Pos2| {
        vec![
            egui::Event::PointerMoved(pos),
            egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: egui::Modifiers::NONE,
            },
        ]
    };
    let release = |pos: egui::Pos2| {
        vec![egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: egui::Modifiers::NONE,
        }]
    };

    let row2 = panel.row(recent).center();
    frame(&mut app, hover(row2));
    let sub2 = MenuPanel::flyout(panel.row(recent), MENU_PANEL_W, 3, layout);
    assert_eq!(
        app.menu.path,
        vec![recent],
        "压住「最近打开」没开第二级子菜单"
    );
    assert_eq!(
        sub2.rect.min.x,
        panel.row(recent).max.x - LINE_W,
        "子菜单没挂在父行右边"
    );
    let more = TOP_TABS[0].items[recent]
        .children
        .iter()
        .position(|n| n.label == "更多")
        .expect("「更多」不见了");

    let row3 = sub2.row(more).center();
    frame(&mut app, hover(row3));
    assert_eq!(
        app.menu.path,
        vec![recent, more],
        "压住「更多」没开第三级子菜单"
    );

    let sub3 = MenuPanel::flyout(sub2.row(more), MENU_PANEL_W, 2, layout);
    let leaf = sub3.row(0).center();
    frame(&mut app, click(leaf));
    frame(&mut app, release(leaf));
    assert_eq!(
        app.status_msg, "你点击了：三级示例 上证50",
        "三级子菜单没点到底"
    );
    assert_eq!(app.menu.open, None, "点中叶子项之后菜单没收起来");
}

#[test]
fn window_maximize_key_fills_the_work_area_and_restores() {
    let ctx = egui::Context::default();
    configure_tessellation(&ctx);
    ctx.set_pixels_per_point(1.0);
    let mut app = blank_app();
    if let Some(panel) = app
        .win_mgr
        .wins
        .iter()
        .position(|w| w.kind == WinKind::Panel)
    {
        app.win_mgr.toggle_pin(panel);
        app.win_mgr.wins[panel].exclusive = false;
    }
    app.text_engine = HardText::new(&ctx);
    if app.panel_buttons.is_empty() {
        app.build_panel_buttons();
    }
    let work = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(cells(120), cells(60)));
    let press = |pos: egui::Pos2| {
        vec![
            egui::Event::PointerMoved(pos),
            egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: egui::Modifiers::NONE,
            },
        ]
    };
    let drag = |pos: egui::Pos2| vec![egui::Event::PointerMoved(pos)];
    let release = |pos: egui::Pos2| {
        vec![egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: egui::Modifiers::NONE,
        }]
    };
    win_frame(&mut app, &ctx, work, vec![]);
    let i = app.win_mgr.live_index().expect("窗口区里一块窗口都没有");
    let before = app.win_mgr.wins[i].rect();
    assert_ne!(before, work, "这块窗口本来不该是铺满的");

    let key = sys_button_rects(free_band(before, 0, WIN_TITLE_ROWS))[2].center();
    win_frame(&mut app, &ctx, work, press(key));
    win_frame(&mut app, &ctx, work, release(key));
    assert!(app.win_mgr.wins[i].w.maximized, "点最大化键没最大化");
    assert_eq!(app.win_mgr.wins[i].rect(), work, "最大化之后没铺满窗口区");

    let edge = egui::pos2(work.max.x - 1.0, work.center().y);
    win_frame(&mut app, &ctx, work, press(edge));
    win_frame(
        &mut app,
        &ctx,
        work,
        drag(egui::pos2(work.center().x, work.center().y)),
    );
    assert_eq!(app.win_mgr.wins[i].rect(), work, "最大化中还能拉边缩放");
    win_frame(&mut app, &ctx, work, release(edge));

    let key = sys_button_rects(free_band(work, 0, WIN_TITLE_ROWS))[2].center();
    win_frame(&mut app, &ctx, work, press(key));
    win_frame(&mut app, &ctx, work, release(key));
    assert!(!app.win_mgr.wins[i].w.maximized, "再点一下没还原");
    assert_eq!(
        app.win_mgr.wins[i].rect(),
        before,
        "还原之后没回到原位原尺寸"
    );
}

#[test]
fn maximized_window_fills_work_and_dragging_title_restores_it() {
    let ctx = egui::Context::default();
    configure_tessellation(&ctx);
    ctx.set_pixels_per_point(1.0);
    let mut app = blank_app();
    if let Some(panel) = app
        .win_mgr
        .wins
        .iter()
        .position(|window| window.kind == WinKind::Panel)
    {
        app.win_mgr.toggle_pin(panel);
        app.win_mgr.wins[panel].exclusive = false;
    }
    app.text_engine = HardText::new(&ctx);
    if app.panel_buttons.is_empty() {
        app.build_panel_buttons();
    }
    let work = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(cells(120), cells(60)));
    let press = |pos: egui::Pos2| {
        vec![
            egui::Event::PointerMoved(pos),
            egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: egui::Modifiers::NONE,
            },
        ]
    };
    let drag = |pos: egui::Pos2| vec![egui::Event::PointerMoved(pos)];
    let release = |pos: egui::Pos2| {
        vec![egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: egui::Modifiers::NONE,
        }]
    };
    win_frame(&mut app, &ctx, work, vec![]);
    let i = app.win_mgr.live_index().expect("no live window");
    let before = app.win_mgr.wins[i].rect();
    let blocker = app
        .win_mgr
        .wins
        .iter()
        .position(|window| window.kind != WinKind::Chart)
        .unwrap_or(i);
    app.win_mgr.wins[blocker].w.pos = work.center() - egui::vec2(80.0, 60.0);
    app.win_mgr.wins[blocker].w.size = egui::vec2(160.0, 120.0);
    app.win_mgr.wins[blocker].exclusive = true;
    win_frame(&mut app, &ctx, work, vec![]);

    let key = sys_button_rects(free_band(before, 0, WIN_TITLE_ROWS))[2].center();
    win_frame(&mut app, &ctx, work, press(key));
    win_frame(&mut app, &ctx, work, release(key));
    assert!(app.win_mgr.wins[i].w.maximized);
    assert_eq!(app.win_mgr.wins[i].rect(), work);

    let edge = egui::pos2(work.max.x - 1.0, work.center().y);
    win_frame(&mut app, &ctx, work, press(edge));
    win_frame(
        &mut app,
        &ctx,
        work,
        drag(egui::pos2(work.center().x, work.center().y)),
    );
    assert_eq!(app.win_mgr.wins[i].rect(), work);
    win_frame(&mut app, &ctx, work, release(edge));

    let title = free_band(work, 0, WIN_TITLE_ROWS).center();
    win_frame(&mut app, &ctx, work, press(title));
    win_frame(&mut app, &ctx, work, drag(title + egui::vec2(40.0, 0.0)));
    assert!(app.win_mgr.wins[i].w.maximized);
    assert_eq!(app.win_mgr.wins[i].rect(), work);
    win_frame(&mut app, &ctx, work, release(title));

    win_frame(&mut app, &ctx, work, press(title));
    win_frame(&mut app, &ctx, work, release(title));
    assert!(!app.win_mgr.wins[i].w.maximized);
    assert_eq!(app.win_mgr.wins[i].rect(), before);
}

#[test]
fn pane_menus_stay_open_after_the_click() {
    let ctx = egui::Context::default();
    configure_tessellation(&ctx);
    ctx.set_pixels_per_point(1.0);
    let mut app = blank_app();
    app.text_engine = HardText::new(&ctx);
    app.replace_bars(data::random_walk::generate(BAR_SEED, BAR_COUNT, BAR_START));
    if app.panel_buttons.is_empty() {
        app.build_panel_buttons();
    }
    let work = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(cells(120), cells(60)));
    let press = |pos: egui::Pos2| {
        vec![
            egui::Event::PointerMoved(pos),
            egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: egui::Modifiers::NONE,
            },
        ]
    };
    let release = |pos: egui::Pos2| {
        vec![egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: egui::Modifiers::NONE,
        }]
    };
    win_frame(&mut app, &ctx, work, vec![]);

    let i = app.win_mgr.live_index().expect("窗口区里一块窗口都没有");
    let pane = app.win_mgr.wins[i].pane;
    let body = ManagedWin::body(app.win_mgr.rect_of(i));
    let (title, _, _, _) = chart_rows(body, app.panes[pane].indicator.layout());
    let tabs = ["切换", "代码"];
    let widths: Vec<f32> = tabs.iter().map(|t| tab_width(&mut app, t)).collect();
    let buttons = StockApp::pane_button_rects(title, &widths);

    let p = buttons[0].center();
    win_frame(&mut app, &ctx, work, press(p));
    win_frame(&mut app, &ctx, work, release(p));
    assert_eq!(app.pane_menu, Some(pane), "点「切换」没开出指标面板");
    for _ in 0..3 {
        win_frame(&mut app, &ctx, work, vec![]);
    }
    assert_eq!(app.pane_menu, Some(pane), "指标面板闪一下就自己收回了");

    let panel = app
        .pane_menu_rect(indicator::builtin_names().len(), work.max.y)
        .expect("面板没算出来");
    win_frame(
        &mut app,
        &ctx,
        work,
        vec![egui::Event::PointerMoved(panel.center())],
    );
    assert_eq!(app.pane_menu, Some(pane), "指针移进面板就把面板收了");
}

#[test]
#[ignore]
fn dump_panels_for_visual_check() {
    let ctx = egui::Context::default();
    configure_tessellation(&ctx);
    ctx.set_pixels_per_point(1.0);

    let mut app = color_panel_app();
    app.text_engine = HardText::new(&ctx);
    let grid = render_panel(&mut app);
    write_bmp("target/ui_color_panel.bmp", &grid);

    let mut app2 = blank_app();
    app2.text_engine = HardText::new(&ctx);
    app2.replace_bars(data::random_walk::generate(BAR_SEED, BAR_COUNT, BAR_START));
    app2.open_source_window(PANE_MACD);
    let grid2 = render_overlays(&mut app2);
    write_bmp("target/ui_source.bmp", &grid2);
}

#[test]
#[ignore]
fn dump_ui_tweaks_for_visual_check() {
    let ctx = egui::Context::default();
    configure_tessellation(&ctx);
    ctx.set_pixels_per_point(1.0);

    let mut app = blank_app();
    app.text_engine = HardText::new(&ctx);
    app.replace_bars(data::random_walk::generate(BAR_SEED, BAR_COUNT, BAR_START));
    app.fps = 60.0;
    app.menu.open = Some(0);
    app.menu.path = vec![2, 2];
    let grid = render_overlays(&mut app);
    write_bmp("target/ui_menu_wide.bmp", &grid);

    set_theme(Theme::DARK);
    app.menu.open = None;
    app.menu.path.clear();
    app.open_source_window(PANE_MACD);
    app.source.editor.scroll = 20;
    let grid = render_overlays(&mut app);
    write_bmp("target/ui_dark_editor.bmp", &grid);
    set_theme(Theme::CLASSIC);

    let mut app = blank_app();
    app.text_engine = HardText::new(&ctx);
    app.replace_bars(data::random_walk::generate(BAR_SEED, BAR_COUNT, BAR_START));
    app.open_source_window(PANE_CANDLE);
    render_overlays(&mut app);
    let work = app.work_rect;
    let find = |app: &StockApp, kind: WinKind| {
        app.win_mgr
            .wins
            .iter()
            .position(|w| w.kind == kind)
            .expect("窗口不见了")
    };
    let chart_i = find(&app, WinKind::Chart);
    let src_i = find(&app, WinKind::Source);
    let mini_w = WinMgr::mini_width(&mut app, WIN_CHART_TITLE);
    app.win_mgr.minimize(chart_i, mini_w);
    app.win_mgr.wins[src_i].w.toggle_maximize(work);
    let grid = render_overlays(&mut app);
    write_bmp("target/ui_window_keys.bmp", &grid);

    let mut console_app = blank_app();
    console_app.text_engine = HardText::new(&ctx);
    console_app.replace_bars(data::random_walk::generate(BAR_SEED, BAR_COUNT, BAR_START));
    let layout = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(WINDOW_W, WINDOW_H));
    console_app.work_rect = split_areas(layout)[2];
    console_app.open_console_window();
    let console = render_window(&mut console_app);
    write_bmp("target/ui_console.bmp", &console);
}

#[test]
#[ignore]
fn dump_new_indicators_for_visual_check() {
    let ctx = egui::Context::default();
    configure_tessellation(&ctx);
    ctx.set_pixels_per_point(1.0);
    let mut app = blank_app();
    app.text_engine = HardText::new(&ctx);
    app.replace_bars(data::random_walk::generate(BAR_SEED, BAR_COUNT, BAR_START));
    app.fps = 60.0;
    app.panes[0].view.visible = visible_bars(CHART_PANE_W - LINE_W - AXIS_W, BAR_SCALE_DEFAULT);

    app.set_pane_indicator(PANE_CANDLE, 1);
    app.set_pane_indicator(PANE_VOLUME, 4);
    app.set_pane_indicator(PANE_MACD, 5);
    let grid = render_window(&mut app);
    write_bmp("target/ui_indicators.bmp", &grid);

    for (index, name) in [(6, "boll"), (7, "obv"), (8, "cci"), (9, "bias")] {
        app.set_pane_indicator(PANE_CANDLE, index);
        let shot = render_window(&mut app);
        write_bmp(&format!("target/ui_ind_{name}.bmp"), &shot);
    }

    app.set_pane_indicator(PANE_CANDLE, 0);
    app.set_pane_indicator(PANE_VOLUME, 2);
    app.set_pane_indicator(PANE_MACD, 3);
    let grid2 = render_window(&mut app);
    write_bmp("target/ui_overlay.bmp", &grid2);
    set_theme(Theme::DARK);
    app.up_color = (
        Theme::DARK.candle_up.r(),
        Theme::DARK.candle_up.g(),
        Theme::DARK.candle_up.b(),
    );
    app.down_color = (
        Theme::DARK.candle_down.r(),
        Theme::DARK.candle_down.g(),
        Theme::DARK.candle_down.b(),
    );
    app.mark_panes_dirty();
    app.set_pane_indicator(PANE_VOLUME, 4);
    app.set_pane_indicator(PANE_MACD, 5);
    let grid3 = render_window(&mut app);
    write_bmp("target/ui_indicators_dark.bmp", &grid3);
    set_theme(Theme::CLASSIC);
}

fn render_overlays(app: &mut StockApp) -> Vec<Vec<Option<egui::Color32>>> {
    render_overlays_at(app, None)
}

fn render_overlays_at(
    app: &mut StockApp,
    pointer: Option<egui::Pos2>,
) -> Vec<Vec<Option<egui::Color32>>> {
    let events = match pointer {
        Some(p) => vec![egui::Event::PointerMoved(p)],
        None => Vec::new(),
    };
    render_overlays_events(app, events)
}

fn render_overlays_events(
    app: &mut StockApp,
    events: Vec<egui::Event>,
) -> Vec<Vec<Option<egui::Color32>>> {
    let ctx = egui::Context::default();
    configure_tessellation(&ctx);
    ctx.set_pixels_per_point(1.0);
    render_events_in(&ctx, app, events)
}

fn render_events_in(
    ctx: &egui::Context,
    app: &mut StockApp,
    events: Vec<egui::Event>,
) -> Vec<Vec<Option<egui::Color32>>> {
    if app.text_engine.is_none() {
        app.text_engine = HardText::new(ctx);
    }
    if app.panel_buttons.is_empty() {
        app.build_panel_buttons();
    }
    configure_zero_spacing(ctx);
    let layout = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(WINDOW_W, WINDOW_H));
    let mut raw = egui::RawInput::default();
    raw.screen_rect = Some(layout);
    raw.events = events;
    let mut out = ctx.run_ui(raw, |ui| {
        app.draw_frame(ui);
    });
    let shapes = std::mem::take(&mut out.shapes);
    absorb_textures(&mut out.textures_delta);
    rasterize_into(
        &ctx.tessellate(shapes, 1.0),
        WINDOW_W as usize,
        WINDOW_H as usize,
    )
}

fn render_pinned_events_in(
    ctx: &egui::Context,
    app: &mut StockApp,
    events: Vec<egui::Event>,
) -> (
    Vec<Vec<Option<egui::Color32>>>,
    egui::CursorIcon,
    Option<egui::CustomCursorImage>,
) {
    if app.text_engine.is_none() {
        app.text_engine = HardText::new(ctx);
    }
    if app.panel_buttons.is_empty() {
        app.build_panel_buttons();
    }
    let layout = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(WINDOW_W, WINDOW_H));
    let mut raw = egui::RawInput::default();
    raw.screen_rect = Some(layout);
    raw.events = events;
    let mut out = ctx.run_ui(raw, |ui| {
        app.draw_frame(ui);

        app.apply_cursor(ui.ctx());
    });
    let icon = out.platform_output.cursor_icon;
    let image = out.platform_output.cursor_image.clone();
    let shapes = std::mem::take(&mut out.shapes);
    absorb_textures(&mut out.textures_delta);
    (
        rasterize_into(
            &ctx.tessellate(shapes, 1.0),
            WINDOW_W as usize,
            WINDOW_H as usize,
        ),
        icon,
        image,
    )
}

#[test]
fn full_window_borders_sit_on_the_grid() {
    let mut app = blank_app();
    app.replace_bars(data::random_walk::generate(BAR_SEED, BAR_COUNT, BAR_START));
    let grid = render_window(&mut app);
    let layout = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(WINDOW_W, WINDOW_H));
    let areas = split_areas(layout);

    for y in [
        areas[1].min.y,
        areas[2].min.y,
        areas[3].min.y,
        areas[4].min.y,
    ] {
        assert_eq!(y % 9.0, 0.0, "区域分隔线 {y} 没落在横网格线上");
    }
    assert_eq!(
        areas[2].max.y,
        areas[3].min.y + LINE_W,
        "功能区与任务栏没共用那条线"
    );

    let [left, _sp, _panel] = StockApp::work_columns(areas[2], app.split_x);
    let [chart1, _h1, chart2, _h2, chart3] = StockApp::left_rows(left, app.kline_h, app.vol_h);

    for y in [
        chart1.min.y,
        chart2.min.y,
        chart3.min.y,
        chart3.max.y - LINE_W,
    ] {
        assert_eq!(
            ((y - areas[2].min.y) as i32) % 9,
            0,
            "图表边界 {y} 没落在横网格线上"
        );
    }

    for (i, area) in [chart1, chart2, chart3].iter().enumerate() {
        let l = app.panes[i].indicator.layout();
        let (title, status, main, time) = chart_rows(*area, l);
        assert_eq!(title.min.y, area.min.y);
        assert_eq!(
            (main.min.y - LINE_W - area.min.y) % 9.0,
            0.0,
            "第 {} 块绘图区上沿不在网格线上",
            i + 1
        );
        if let Some(s) = status {
            assert_eq!((s.min.y - area.min.y) % 9.0, 0.0, "状态带上沿不在网格线上");
        }
        if let Some(t) = time {
            assert_eq!((t.min.y - area.min.y) % 9.0, 0.0, "时间带上沿不在网格线上");

            assert_eq!(
                t.max.y,
                chart_bar(*area).min.y + LINE_W,
                "时间带和状态栏没共用那条线"
            );
            assert_eq!(
                chart_bar(*area).max.y,
                area.max.y,
                "图表窗状态栏没铺到图表底边"
            );
        } else {
            assert_eq!(
                chart_bar(*area).min.y,
                main.max.y,
                "绘图区和状态栏没共用那条线"
            );
            assert_eq!(
                chart_bar(*area).max.y,
                area.max.y,
                "图表窗状态栏没铺到图表底边"
            );
        }
    }

    let col = 40usize;
    for y in [areas[1].min.y, areas[2].min.y] {
        let row = y as usize;
        assert_eq!(grid[row][col], Some(C_LINE), "区域分隔线 {y} 没画出来");
        assert!(is_title_face(grid[row - 1][col]), "区域分隔线 {y} 不止 1px");
    }
}

#[test]
fn child_title_blank_area_uses_one_pixel_horizontal_fill() {
    let mut app = blank_app();
    app.replace_bars(data::random_walk::generate(BAR_SEED, BAR_COUNT, BAR_START));
    let grid = render_window(&mut app);
    let chart = app
        .win_mgr
        .wins
        .iter()
        .position(|window| window.kind == WinKind::Chart)
        .expect("图表窗不存在");
    let title = app.win_mgr.wins[chart].w.title_band();
    let text = title_text_slot(&mut app, title, WIN_CHART_TITLE);
    let keys = sys_button_rects(title);
    let x = ((text.max.x + keys[0].min.x) / 2.0) as usize;
    let mut fill_rows = 0;
    let mut base_rows = 0;
    for y in title.min.y as usize + 1..title.max.y as usize - 1 {
        match grid[y][x] {
            Some(color) if color == theme().title_fill => fill_rows += 1,
            Some(color) if color == theme().title_bg => base_rows += 1,
            other => panic!("标题空白区出现意外颜色 {other:?}"),
        }
    }
    assert!(fill_rows > 0, "标题空白区没有底纹");
    assert!(base_rows > 0, "底纹没有保留 1px 间距");
}

#[test]
fn icon_pngs_on_disk_are_the_only_source() {
    let set = IconSet::load_or_create();
    let dir = icons_dir();
    for kind in IconKind::ALL {
        let name = kind.file_stem();
        let path = dir.join(format!("{name}.png"));

        if !path.exists() {
            assert!(
                set.get(kind).is_blank(),
                "{name}.png 不存在，图标却不是空的"
            );
            continue;
        }
        let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("读不到图标 {name}.png：{e}"));
        let mut reader = png::Decoder::new(std::io::Cursor::new(bytes))
            .read_info()
            .expect("PNG 头读不动");
        let (w, h) = {
            let info = reader.info();
            (info.width, info.height)
        };
        assert_eq!((w, h), (16, 16), "{name}.png 不是 16x16");
        let mut buf = vec![0u8; reader.output_buffer_size().unwrap_or(0)];
        let info = reader.next_frame(&mut buf).expect("PNG 数据读不动");
        assert_eq!(
            info.color_type,
            png::ColorType::Rgba,
            "{name}.png 不是真彩 + alpha"
        );
        for px in buf[..info.buffer_size()].chunks_exact(4) {
            let (r, g, b) = (px[0], px[1], px[2]);
            if (r, g, b) == ICON_TRANSPARENT_RGB {
                continue;
            }
            for c in [r, g, b] {
                assert_eq!(
                    c % 17,
                    0,
                    "{name}.png 里有没吸到 #aabbcc 形的颜色：{c:#04x}"
                );
            }
        }
        assert!(!set.get(kind).is_blank(), "{name}.png 读回来是空的");
    }
}

#[test]
fn title_symbols_are_png_masks() {
    let set = SymbolSet::load();
    for kind in SymbolKind::ALL {
        assert!(!set.get(kind).is_blank(), "{} 符号是空的", kind.file_stem());
        let bytes =
            std::fs::read(crate::assets::symbols_dir().join(format!("{}.png", kind.file_stem())))
                .expect("读不到符号 PNG");
        let image = decode_png_rgba(&bytes).expect("符号 PNG 解不开");
        assert_eq!((image.width, image.height), (16, 16));
        assert!(
            image.rgba.chunks_exact(4).all(|px| {
                px[3] == 0 || (px[0], px[1], px[2]) == SYMBOL_TRANSPARENT_RGB || px[3] == 0xFF
            }),
            "{} 符号不是硬边 PNG",
            kind.file_stem()
        );
    }
}

#[test]
fn text_buttons_include_symbols_and_icon_buttons_keep_icons() {
    assert_eq!(
        ButtonContent::Text("确定".to_owned()).class(),
        ButtonClass::Text
    );
    assert_eq!(
        ButtonContent::Symbol(SymbolKind::Pin).class(),
        ButtonClass::Text
    );
    assert_eq!(
        ButtonContent::Icon(IconKind::Save).class(),
        ButtonClass::Icon
    );
}

#[test]
fn the_text_input_caret_blinks_at_one_second_intervals() {
    assert!(caret_blink_visible_at(0.0));
    assert!(caret_blink_visible_at(0.49));
    assert!(!caret_blink_visible_at(0.50));
    assert!(!caret_blink_visible_at(0.99));
    assert!(caret_blink_visible_at(1.00));
    assert!(caret_blink_visible_at(2.25));
    assert!(!caret_blink_visible_at(2.75));
}

#[test]
fn theme_json_files_parse_and_accept_user_edits() {
    let light = Theme::from_json(
        include_str!("../assets/themes/light.json"),
        ThemeId::Classic,
    )
    .expect("浅色主题 JSON 解析失败");
    let dark = Theme::from_json(include_str!("../assets/themes/dark.json"), ThemeId::Dark)
        .expect("深色主题 JSON 解析失败");
    assert_eq!(light.id, ThemeId::Classic);
    assert_eq!(dark.id, ThemeId::Dark);

    let mut edit: serde_json::Value =
        serde_json::from_str(include_str!("../assets/themes/light.json")).expect("主题 JSON 结构");
    edit["colors"]["window"]["title_bg"] = serde_json::json!("#AABBCC");
    edit["colors"]["window"]["ctrl_border"] = serde_json::json!("#123456");
    edit["colors"]["window"]["tooltip_bg"] = serde_json::json!("#654321");
    let edited =
        Theme::from_json(&edit.to_string(), ThemeId::Classic).expect("修改后的主题应能解析");
    assert_eq!(edited.title_bg, egui::Color32::from_rgb(0xAA, 0xBB, 0xCC));
    assert_eq!(
        edited.ctrl_border,
        egui::Color32::from_rgb(0x12, 0x34, 0x56)
    );
    assert_eq!(edited.tooltip_bg, egui::Color32::from_rgb(0x65, 0x43, 0x21));

    edit["colors"]["chart"]
        .as_object_mut()
        .expect("chart 必须是对象")
        .remove("chart_bg");
    assert!(Theme::from_json(&edit.to_string(), ThemeId::Classic).is_err());
}

#[test]
fn ecs_spawns_one_entity_per_pane() {
    let app = blank_app();
    let ids = app.chart_world.pane_entity_ids();
    let unique: std::collections::HashSet<_> = ids.iter().copied().collect();
    assert_eq!(ids.len(), app.panes.len());
    assert_eq!(unique.len(), ids.len());
    assert_eq!(app.panes.entity(0), ids.first().copied());
}

#[test]
fn runtime_default_workspace_has_no_chart() {
    let mgr = WinMgr::with_default_windows();
    assert_eq!(mgr.chart_count(), 0, "启动时不应带图表窗");
    assert!(
        mgr.wins
            .iter()
            .any(|w| w.kind == WinKind::Panel && w.w.open),
        "控制面板仍应存在"
    );
}

#[test]
fn chart_view_state_is_independent_per_pane() {
    let mut app = blank_app();
    app.panes[0].view.scale = SCALE_MIN;
    app.panes[0].view.offset = 11;
    app.panes[0].view.visible = 31;
    app.panes[1].view.scale = SCALE_MAX;
    app.panes[1].view.offset = 7;
    app.panes[1].view.visible = 13;
    app.panes[2].view.offset = 3;

    assert_eq!(
        (
            app.panes[0].view.scale,
            app.panes[0].view.offset,
            app.panes[0].view.visible
        ),
        (SCALE_MIN, 11, 31)
    );
    assert_eq!(
        (
            app.panes[1].view.scale,
            app.panes[1].view.offset,
            app.panes[1].view.visible
        ),
        (SCALE_MAX, 7, 13)
    );
    assert_eq!(app.panes[2].view.offset, 3);
}

#[test]
fn pinned_windows_follow_pin_order_above_regular_windows() {
    let mut mgr = WinMgr::new();
    let a = mgr.add(
        WinKind::Chart,
        "A",
        0,
        egui::vec2(cells(20), cells(20)),
        egui::vec2(cells(20), cells(20)),
    );
    let b = mgr.add(
        WinKind::Chart,
        "B",
        1,
        egui::vec2(cells(20), cells(20)),
        egui::vec2(cells(20), cells(20)),
    );
    let c = mgr.add(
        WinKind::Chart,
        "C",
        2,
        egui::vec2(cells(20), cells(20)),
        egui::vec2(cells(20), cells(20)),
    );
    for i in 0..3 {
        mgr.wins[i].w.open = true;
    }
    mgr.toggle_pin(a);
    mgr.toggle_pin(c);
    let order = mgr.order_ids();
    assert_eq!(
        order,
        vec![mgr.wins[b].w.id, mgr.wins[a].w.id, mgr.wins[c].w.id]
    );
    mgr.toggle_pin(a);
    assert_eq!(mgr.order_ids().last().copied(), Some(mgr.wins[c].w.id));
}

#[test]
fn exclusive_regular_and_pinned_windows_form_three_layer_levels() {
    let mut mgr = WinMgr::new();
    let regular = mgr.add(
        WinKind::Chart,
        "regular",
        0,
        egui::vec2(cells(20), cells(20)),
        egui::vec2(cells(20), cells(20)),
    );
    let pinned = mgr.add(
        WinKind::Chart,
        "pinned",
        1,
        egui::vec2(cells(20), cells(20)),
        egui::vec2(cells(20), cells(20)),
    );
    let exclusive = mgr.add(
        WinKind::Chart,
        "exclusive",
        2,
        egui::vec2(cells(20), cells(20)),
        egui::vec2(cells(20), cells(20)),
    );
    mgr.wins[exclusive].exclusive = true;
    mgr.set_pin(pinned, true);
    assert_eq!(
        mgr.order_ids(),
        vec![
            mgr.wins[exclusive].w.id,
            mgr.wins[regular].w.id,
            mgr.wins[pinned].w.id
        ]
    );
}

#[test]
fn exclusive_window_rejects_other_windows_entering_its_rect() {
    let ctx = egui::Context::default();
    configure_tessellation(&ctx);
    ctx.set_pixels_per_point(1.0);
    let mut app = blank_app();
    app.text_engine = HardText::new(&ctx);
    app.replace_bars(data::random_walk::generate(BAR_SEED, 64, BAR_START));
    let work = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(cells(140), cells(80)));
    win_frame(&mut app, &ctx, work, Vec::new());
    app.add_chart_window();
    win_frame(&mut app, &ctx, work, Vec::new());

    let panel = app
        .win_mgr
        .wins
        .iter()
        .position(|window| window.kind == WinKind::Panel)
        .unwrap();
    app.win_mgr.wins[panel].w.pos = egui::pos2(work.max.x - cells(24), work.min.y);
    let charts: Vec<usize> = app
        .win_mgr
        .wins
        .iter()
        .enumerate()
        .filter(|(_, window)| window.kind == WinKind::Chart)
        .map(|(index, _)| index)
        .collect();
    let exclusive_id = app.win_mgr.wins[charts[0]].w.id;
    let mover_id = app.win_mgr.wins[charts[1]].w.id;
    app.win_mgr.wins[charts[0]].w.pos = egui::pos2(100.0, 100.0);
    app.win_mgr.wins[charts[0]].w.size = egui::vec2(200.0, 180.0);
    app.win_mgr.wins[charts[0]].exclusive = true;
    app.win_mgr.wins[charts[1]].w.pos = egui::pos2(520.0, 100.0);
    app.win_mgr.wins[charts[1]].w.size = egui::vec2(150.0, 100.0);
    win_frame(&mut app, &ctx, work, Vec::new());

    let mover = app
        .win_mgr
        .wins
        .iter()
        .position(|window| window.w.id == mover_id)
        .unwrap();
    let start = app.win_mgr.wins[mover].w.title_band().center();
    let target = start - egui::vec2(60.0, 0.0);
    let blocked_target = target - egui::vec2(300.0, 0.0);
    let press = |pos| {
        vec![
            egui::Event::PointerMoved(pos),
            egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: egui::Modifiers::NONE,
            },
        ]
    };
    let release = |pos| {
        vec![egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: egui::Modifiers::NONE,
        }]
    };
    win_frame(&mut app, &ctx, work, press(start));
    win_frame(
        &mut app,
        &ctx,
        work,
        vec![egui::Event::PointerMoved(target)],
    );
    let stopped = app
        .win_mgr
        .wins
        .iter()
        .find(|window| window.w.id == mover_id)
        .unwrap()
        .w
        .rect();
    win_frame(
        &mut app,
        &ctx,
        work,
        vec![egui::Event::PointerMoved(blocked_target)],
    );
    let slide_target = blocked_target + egui::vec2(0.0, 60.0);
    win_frame(
        &mut app,
        &ctx,
        work,
        vec![egui::Event::PointerMoved(slide_target)],
    );
    let mover_rect = app
        .win_mgr
        .wins
        .iter()
        .find(|window| window.w.id == mover_id)
        .unwrap()
        .w
        .rect();
    win_frame(&mut app, &ctx, work, release(slide_target));
    let exclusive_rect = app
        .win_mgr
        .wins
        .iter()
        .find(|window| window.w.id == exclusive_id)
        .unwrap()
        .w
        .rect();
    assert!(crate::ui::floatwin::shares_border(
        mover_rect,
        exclusive_rect
    ));
    assert!(!crate::ui::floatwin::interior_overlap(
        mover_rect,
        exclusive_rect
    ));
    assert!(
        mover_rect.min.x < 520.0,
        "窗口被独占区域挡住之前应该先向左移动"
    );
    assert_eq!(
        mover_rect.min.x,
        exclusive_rect.max.x - LINE_W,
        "碰撞后窗口应停在独占窗口的 1px 边框线上"
    );
    assert!(
        mover_rect.min.x < stopped.min.x,
        "窗口应先靠近独占区域，再吸附到共享边框"
    );
    assert!(
        mover_rect.min.y > stopped.min.y,
        "碰撞到独占窗口边缘后仍应能沿边缘滑动"
    );
}

#[test]
fn exclusive_window_stops_at_regular_windows_in_both_directions() {
    let ctx = egui::Context::default();
    configure_tessellation(&ctx);
    ctx.set_pixels_per_point(1.0);
    let mut app = blank_app();
    app.text_engine = HardText::new(&ctx);
    app.replace_bars(data::random_walk::generate(BAR_SEED, 64, BAR_START));
    let work = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(cells(140), cells(80)));
    win_frame(&mut app, &ctx, work, Vec::new());
    app.add_chart_window();
    win_frame(&mut app, &ctx, work, Vec::new());
    let charts: Vec<usize> = app
        .win_mgr
        .wins
        .iter()
        .enumerate()
        .filter(|(_, window)| window.kind == WinKind::Chart)
        .map(|(index, _)| index)
        .collect();
    let mover = charts[0];
    let blocker = charts[1];
    app.win_mgr.wins[mover].w.pos = egui::pos2(80.0, 120.0);
    app.win_mgr.wins[mover].w.size = egui::vec2(160.0, 120.0);
    app.win_mgr.wins[mover].exclusive = true;
    app.win_mgr.wins[blocker].w.pos = egui::pos2(420.0, 120.0);
    app.win_mgr.wins[blocker].w.size = egui::vec2(160.0, 120.0);
    let mover_id = app.win_mgr.wins[mover].w.id;
    let blocker_id = app.win_mgr.wins[blocker].w.id;
    win_frame(&mut app, &ctx, work, Vec::new());

    let start = app.win_mgr.wins[mover].w.title_band().center();
    let target = start + egui::vec2(500.0, 0.0);
    let press = vec![
        egui::Event::PointerMoved(start),
        egui::Event::PointerButton {
            pos: start,
            button: egui::PointerButton::Primary,
            pressed: true,
            modifiers: egui::Modifiers::NONE,
        },
    ];
    win_frame(&mut app, &ctx, work, vec![egui::Event::PointerMoved(start)]);
    win_frame(&mut app, &ctx, work, press);
    win_frame(
        &mut app,
        &ctx,
        work,
        vec![egui::Event::PointerMoved(target)],
    );
    let exclusive_rect = app
        .win_mgr
        .wins
        .iter()
        .find(|window| window.w.id == mover_id)
        .unwrap()
        .w
        .rect();
    let blocker_rect = app
        .win_mgr
        .wins
        .iter()
        .find(|window| window.w.id == blocker_id)
        .unwrap()
        .w
        .rect();
    assert_eq!(exclusive_rect.max.x, blocker_rect.min.x + LINE_W);
    assert!(shares_border(exclusive_rect, blocker_rect));
    win_frame(
        &mut app,
        &ctx,
        work,
        vec![egui::Event::PointerButton {
            pos: target,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: egui::Modifiers::NONE,
        }],
    );
}

#[test]
fn pinned_windows_move_through_exclusive_layers() {
    let ctx = egui::Context::default();
    configure_tessellation(&ctx);
    ctx.set_pixels_per_point(1.0);
    let mut app = blank_app();
    app.text_engine = HardText::new(&ctx);
    app.replace_bars(data::random_walk::generate(BAR_SEED, 64, BAR_START));
    let work = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(cells(140), cells(80)));
    win_frame(&mut app, &ctx, work, Vec::new());
    app.add_chart_window();
    win_frame(&mut app, &ctx, work, Vec::new());
    let charts: Vec<usize> = app
        .win_mgr
        .wins
        .iter()
        .enumerate()
        .filter(|(_, window)| window.kind == WinKind::Chart)
        .map(|(index, _)| index)
        .collect();
    let mover = charts[0];
    let blocker = charts[1];
    let min = WinMgr::chart_min_size();
    app.win_mgr.wins[mover].w.pos = egui::pos2(80.0, 120.0);
    app.win_mgr.wins[mover].w.size = min;
    app.win_mgr.set_pin(mover, true);
    app.win_mgr.wins[blocker].w.pos = egui::pos2(420.0, 120.0);
    app.win_mgr.wins[blocker].w.size = min;
    app.win_mgr.wins[blocker].exclusive = true;
    let mover_id = app.win_mgr.wins[mover].w.id;
    let blocker_id = app.win_mgr.wins[blocker].w.id;
    win_frame(&mut app, &ctx, work, Vec::new());

    let start = app.win_mgr.wins[mover].w.title_band().center();
    let target = start + egui::vec2(260.0, 0.0);
    win_frame(&mut app, &ctx, work, vec![egui::Event::PointerMoved(start)]);
    win_frame(
        &mut app,
        &ctx,
        work,
        vec![
            egui::Event::PointerMoved(start),
            egui::Event::PointerButton {
                pos: start,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: egui::Modifiers::NONE,
            },
        ],
    );
    win_frame(
        &mut app,
        &ctx,
        work,
        vec![egui::Event::PointerMoved(target)],
    );
    let mover_rect = app
        .win_mgr
        .wins
        .iter()
        .find(|window| window.w.id == mover_id)
        .unwrap()
        .w
        .rect();
    let blocker_rect = app
        .win_mgr
        .wins
        .iter()
        .find(|window| window.w.id == blocker_id)
        .unwrap()
        .w
        .rect();
    assert!(interior_overlap(mover_rect, blocker_rect));
    win_frame(
        &mut app,
        &ctx,
        work,
        vec![egui::Event::PointerButton {
            pos: target,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: egui::Modifiers::NONE,
        }],
    );
}

#[test]
fn fast_resize_lands_on_an_exclusive_window_border() {
    let ctx = egui::Context::default();
    configure_tessellation(&ctx);
    ctx.set_pixels_per_point(1.0);
    let mut app = blank_app();
    app.text_engine = HardText::new(&ctx);
    app.replace_bars(data::random_walk::generate(BAR_SEED, 64, BAR_START));
    let work = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(cells(140), cells(80)));
    win_frame(&mut app, &ctx, work, Vec::new());
    app.add_chart_window();
    win_frame(&mut app, &ctx, work, Vec::new());
    let charts: Vec<usize> = app
        .win_mgr
        .wins
        .iter()
        .enumerate()
        .filter(|(_, window)| window.kind == WinKind::Chart)
        .map(|(index, _)| index)
        .collect();
    let resizer = charts[0];
    let blocker = charts[1];
    let min = WinMgr::chart_min_size();
    app.win_mgr.wins[resizer].w.pos = egui::pos2(80.0, 120.0);
    app.win_mgr.wins[resizer].w.size = min;
    app.win_mgr.wins[blocker].w.pos = egui::pos2(500.0, 120.0);
    app.win_mgr.wins[blocker].w.size = min;
    app.win_mgr.wins[blocker].exclusive = true;
    let resizer_id = app.win_mgr.wins[resizer].w.id;
    let blocker_id = app.win_mgr.wins[blocker].w.id;
    win_frame(&mut app, &ctx, work, Vec::new());
    let start_rect = app
        .win_mgr
        .wins
        .iter()
        .find(|window| window.w.id == resizer_id)
        .unwrap()
        .w
        .rect();
    let grab = egui::pos2(start_rect.max.x - LINE_W, start_rect.center().y);
    let target = egui::pos2(start_rect.max.x + 300.0, grab.y);
    win_frame(
        &mut app,
        &ctx,
        work,
        vec![
            egui::Event::PointerMoved(grab),
            egui::Event::PointerButton {
                pos: grab,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: egui::Modifiers::NONE,
            },
        ],
    );
    win_frame(
        &mut app,
        &ctx,
        work,
        vec![egui::Event::PointerMoved(target)],
    );
    let resized = app
        .win_mgr
        .wins
        .iter()
        .find(|window| window.w.id == resizer_id)
        .unwrap()
        .w
        .rect();
    let blocker_rect = app
        .win_mgr
        .wins
        .iter()
        .find(|window| window.w.id == blocker_id)
        .unwrap()
        .w
        .rect();
    assert_eq!(resized.max.x, blocker_rect.min.x + LINE_W);
    assert!(shares_border(resized, blocker_rect));
    assert!(!interior_overlap(resized, blocker_rect));
    win_frame(
        &mut app,
        &ctx,
        work,
        vec![egui::Event::PointerButton {
            pos: target,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: egui::Modifiers::NONE,
        }],
    );
}

#[test]
fn blank_workspace_drag_does_not_move_or_select_windows() {
    let ctx = egui::Context::default();
    configure_tessellation(&ctx);
    ctx.set_pixels_per_point(1.0);
    let mut app = blank_app();
    app.text_engine = HardText::new(&ctx);
    app.replace_bars(data::random_walk::generate(BAR_SEED, 64, BAR_START));
    let work = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(cells(140), cells(80)));
    win_frame(&mut app, &ctx, work, Vec::new());
    app.add_chart_window();
    win_frame(&mut app, &ctx, work, Vec::new());

    let charts: Vec<usize> = app
        .win_mgr
        .wins
        .iter()
        .enumerate()
        .filter(|(_, window)| window.kind == WinKind::Chart)
        .map(|(index, _)| index)
        .collect();
    for (index, x) in charts.iter().zip([180.0, 620.0]) {
        app.win_mgr.wins[*index].w.pos = egui::pos2(x, 120.0);
    }
    win_frame(&mut app, &ctx, work, Vec::new());
    let before: Vec<egui::Pos2> = app.win_mgr.wins.iter().map(|window| window.w.pos).collect();

    let start = work.min + egui::vec2(20.0, 500.0);
    let end = start + egui::vec2(320.0, 120.0);
    let press = vec![
        egui::Event::PointerMoved(start),
        egui::Event::PointerButton {
            pos: start,
            button: egui::PointerButton::Primary,
            pressed: true,
            modifiers: egui::Modifiers::NONE,
        },
    ];
    win_frame(&mut app, &ctx, work, press);
    win_frame(&mut app, &ctx, work, vec![egui::Event::PointerMoved(end)]);
    win_frame(
        &mut app,
        &ctx,
        work,
        vec![egui::Event::PointerButton {
            pos: end,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: egui::Modifiers::NONE,
        }],
    );
    let after: Vec<egui::Pos2> = app.win_mgr.wins.iter().map(|window| window.w.pos).collect();
    assert_eq!(before, after);
    assert!(!app.win_mgr.drag_active());
}

#[test]
fn child_pin_button_toggles_from_the_title_bar() {
    let ctx = egui::Context::default();
    configure_tessellation(&ctx);
    ctx.set_pixels_per_point(1.0);
    let mut app = blank_app();
    let work = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(cells(120), cells(60)));
    let click = |pos: egui::Pos2| {
        vec![
            egui::Event::PointerMoved(pos),
            egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: egui::Modifiers::NONE,
            },
        ]
    };
    let release = |pos: egui::Pos2| {
        vec![egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: egui::Modifiers::NONE,
        }]
    };
    win_frame(&mut app, &ctx, work, vec![]);
    let chart = app
        .win_mgr
        .wins
        .iter()
        .position(|w| w.kind == WinKind::Chart)
        .expect("图表窗没建出来");
    let title = free_band(app.win_mgr.wins[chart].rect(), 0, WIN_TITLE_ROWS);
    let pin = sys_button_rects(title)[0].center();
    win_frame(&mut app, &ctx, work, click(pin));
    win_frame(&mut app, &ctx, work, release(pin));
    assert!(app.win_mgr.wins[chart].pinned, "置顶键没切到置顶状态");
    win_frame(&mut app, &ctx, work, click(pin));
    win_frame(&mut app, &ctx, work, release(pin));
    assert!(!app.win_mgr.wins[chart].pinned, "再次点击没解除置顶");
}

#[test]
fn status_bar_contains_a_current_time_field() {
    let app = blank_app();
    let segs = app.status_segments();
    let time = segs.last().expect("状态栏最后一格应是时间");
    assert_eq!(time.len(), 8, "时间格式应为 HH:MM:SS");
    assert_eq!(time.as_bytes()[2], b':');
    assert_eq!(time.as_bytes()[5], b':');
}

#[test]
fn control_panel_starts_pinned_without_exclusive() {
    let mgr = WinMgr::with_default_windows();
    let panel = mgr
        .wins
        .iter()
        .find(|w| w.kind == WinKind::Panel)
        .expect("控制面板不存在");
    assert!(panel.pinned, "控制面板默认应处于置顶状态");
    assert!(!panel.exclusive, "控制面板默认不应处于独占状态");
}

#[test]
fn system_button_labels_follow_the_live_window_state() {
    assert_eq!(
        SysButton::Maximize.label(false, false, false, false),
        "最大化"
    );
    assert_eq!(SysButton::Maximize.label(true, false, false, false), "还原");
    assert_eq!(SysButton::Minimize.label(false, true, false, false), "还原");
    assert_eq!(
        SysButton::Exclusive.label(false, false, false, false),
        "独占"
    );
    assert_eq!(
        SysButton::Exclusive.label(false, false, false, true),
        "解除独占"
    );
}

#[test]
fn user_indicator_copy_paths_stay_outside_assets() {
    let mut app = blank_app();
    app.open_source_window(PANE_CANDLE);
    let user_path = app.source.user_path.clone().expect("用户副本路径");
    assert!(user_path.starts_with(crate::assets::user_root()));
    assert!(!user_path.starts_with(crate::assets::scripts_dir()));
    assert_ne!(user_path, indicator::script_path("蜡烛图"));
}

#[test]
fn color_panel_has_pin_minimize_and_close_without_maximize() {
    let mut app = blank_app();
    app.open_color_panel(ColorTarget::Up);

    let ctx = egui::Context::default();
    configure_tessellation(&ctx);
    let frame = |app: &mut StockApp, events: Vec<egui::Event>| {
        let _ = render_pinned_events_in(&ctx, app, events);
    };
    let click = |pos: egui::Pos2| {
        vec![
            egui::Event::PointerMoved(pos),
            egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: egui::Modifiers::NONE,
            },
        ]
    };
    let release = |pos: egui::Pos2| {
        vec![egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: egui::Modifiers::NONE,
        }]
    };
    frame(&mut app, vec![]);
    let color = color_index(&app);
    assert!(app.win_mgr.wins[color].pinned, "颜色面板默认应置顶");
    let title = app.win_mgr.wins[color].w.title_band();
    let keys = sys_button_rects_n(title, 3);
    assert!(keys[0].center().x < keys[1].center().x);
    assert!(keys[1].center().x < keys[2].center().x);
    let pin = keys[0].center();
    frame(&mut app, click(pin));
    frame(&mut app, release(pin));
    assert!(!app.win_mgr.wins[color].pinned, "置顶键没切到解除置顶");

    let minimize = keys[1].center();
    frame(&mut app, click(minimize));
    frame(&mut app, release(minimize));
    assert!(app.win_mgr.wins[color].minimized);
    assert_eq!(app.win_mgr.rect_of(color).height(), WIN_TITLE_H);

    frame(&mut app, vec![]);
    let restore = egui::pos2(
        app.win_mgr.rect_of(color).min.x + 12.0,
        app.win_mgr.rect_of(color).center().y,
    );
    frame(&mut app, vec![egui::Event::PointerMoved(restore)]);
    frame(&mut app, click(restore));
    frame(&mut app, release(restore));
    assert!(!app.win_mgr.wins[color].minimized);
    assert_eq!(app.win_mgr.rect_of(color).size(), egui::vec2(CP_W, CP_H));
}

#[test]
fn control_panel_color_buttons_open_the_managed_panel() {
    let ctx = egui::Context::default();
    configure_tessellation(&ctx);
    ctx.set_pixels_per_point(1.0);
    let mut app = blank_app();
    app.text_engine = HardText::new(&ctx);
    if app.panel_buttons.is_empty() {
        app.build_panel_buttons();
    }
    let work = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(cells(140), cells(80)));
    let frame = |app: &mut StockApp, events: Vec<egui::Event>| win_frame(app, &ctx, work, events);
    let click = |pos: egui::Pos2| {
        vec![
            egui::Event::PointerMoved(pos),
            egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: egui::Modifiers::NONE,
            },
        ]
    };
    let release = |pos: egui::Pos2| {
        vec![egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: egui::Modifiers::NONE,
        }]
    };
    frame(&mut app, vec![]);
    let panel = app
        .win_mgr
        .wins
        .iter()
        .position(|window| window.kind == WinKind::Panel)
        .expect("控制面板不存在");
    let body = ManagedWin::body(app.win_mgr.wins[panel].rect());
    let button_pos = |slot: usize| {
        egui::pos2(
            body.min.x + grid_line(1),
            body.min.y + grid_line(1) + BTN_STEP * slot as f32 + BTN_H / 2.0,
        )
    };
    let press_button = |app: &mut StockApp, slot: usize| {
        let pos = button_pos(slot);
        frame(app, vec![egui::Event::PointerMoved(pos)]);
        frame(app, click(pos));
        frame(app, release(pos));
        frame(app, vec![]);
    };

    press_button(&mut app, 1);
    let color = color_index(&app);
    assert_eq!(app.color_panel.target, ColorTarget::Up);
    assert!(app.win_mgr.wins[color].pinned);

    app.win_mgr.hide_color();
    frame(&mut app, vec![]);
    press_button(&mut app, 2);
    let color = color_index(&app);
    assert_eq!(app.color_panel.target, ColorTarget::Down);
    assert!(app.win_mgr.wins[color].pinned);
}

#[test]
fn color_panel_is_a_managed_sibling_window() {
    let ctx = egui::Context::default();
    configure_tessellation(&ctx);
    let mut app = blank_app();
    app.open_color_panel(ColorTarget::Up);
    let frame = |app: &mut StockApp, events: Vec<egui::Event>| {
        let _ = render_pinned_events_in(&ctx, app, events);
    };
    frame(&mut app, vec![]);
    let color = color_index(&app);
    assert_eq!(app.win_mgr.wins[color].kind, WinKind::Color);
    assert!(app.win_mgr.wins[color].pinned);
    assert!(!app.win_mgr.wins[color].w.resizable);

    app.win_mgr.set_pin(color, false);
    let chart = app
        .win_mgr
        .wins
        .iter()
        .position(|window| window.kind == WinKind::Chart)
        .expect("图表窗不存在");
    let chart_id = app.win_mgr.wins[chart].w.id;
    let color_id = app.win_mgr.wins[color].w.id;
    let title = app.win_mgr.wins[chart].w.title_band();
    let click = egui::pos2(title.min.x + 100.0, title.center().y);
    frame(
        &mut app,
        vec![
            egui::Event::PointerMoved(click),
            egui::Event::PointerButton {
                pos: click,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: egui::Modifiers::NONE,
            },
        ],
    );
    frame(
        &mut app,
        vec![egui::Event::PointerButton {
            pos: click,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: egui::Modifiers::NONE,
        }],
    );
    let order = app.win_mgr.order_ids();
    let chart_at = order.iter().position(|id| *id == chart_id).unwrap();
    let color_at = order.iter().position(|id| *id == color_id).unwrap();
    assert!(chart_at > color_at, "点击图表窗后颜色面板仍压在它上面");
}

#[test]
fn fixedsys_digits_keep_zero_one_pixel_left() {
    assert_eq!(ascii_glyph_offset('0'), -1);
    assert_eq!(ascii_glyph_offset('1'), 0);
    assert_eq!(ascii_glyph_offset('A'), 0);
}

#[test]
fn crosshair_time_label_never_stacks_on_the_timeline_border() {
    let mut app = blank_app();
    app.replace_bars(data::random_walk::generate(BAR_SEED, 64, BAR_START));
    let _ = render_window_at(&mut app, None);

    let chart = app
        .win_mgr
        .wins
        .iter()
        .find(|w| w.kind == WinKind::Chart)
        .expect("图表窗不存在");
    let kline = ManagedWin::body(chart.rect());
    let (_, _, main, time) = chart_rows(kline, app.panes[PANE_CANDLE].indicator.layout());
    let time = time.expect("蜡烛图有时间带");
    let plot = frame_rects(main).0;

    let shared = main.max.y as usize;
    assert_eq!(time.min.y as usize, shared, "时间带上沿不是那条共用的线");

    let grid = render_window_at(&mut app, Some(plot.center()));
    app.panes[0]
        .view
        .crosshair
        .expect("鼠标在绘图区里应该有十字光标");

    let label_top = (plot.max.y - PITCH * CROSS_LABEL_ROWS as f32) as usize;
    assert_eq!(label_top % 9, 0, "索引牌上边框没落在网格线上");
    assert_eq!(label_top + 18, shared, "索引牌下边框没压在共用的那条线上");

    let (x0, x1) = (plot.min.x as usize, plot.max.x as usize);

    let longest = |row: usize| -> usize {
        let (mut best, mut cur) = (0usize, 0usize);
        for x in x0..x1 {
            cur = if grid[row][x] == Some(C_LINE) {
                cur + 1
            } else {
                0
            };
            best = best.max(cur);
        }
        best
    };

    assert!(
        longest(label_top) >= 19,
        "绘图区下沿没看到索引牌的上边框（它该是一条落在网格线上的横线）"
    );

    assert!(
        longest(shared - 1) <= 1,
        "索引牌和时间轴之间叠出了 2px 横线（连排 {}px）",
        longest(shared - 1)
    );
}

fn key_event(key: egui::Key, ctrl: bool) -> egui::Event {
    egui::Event::Key {
        key,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: if ctrl {
            egui::Modifiers::CTRL
        } else {
            egui::Modifiers::NONE
        },
    }
}

fn feed(buf: &mut TextBuffer, preedit: &mut String, events: &[egui::Event]) -> EditOutcome {
    apply_edit_events(buf, preedit, events, true, &char_allowed)
}

#[test]
fn editable_buffer_counts_columns_in_chars() {
    let mut buf = TextBuffer::from_text("中文abc");
    assert_eq!(buf.line_count(), 1);
    buf.move_end();
    buf.backspace();
    assert_eq!(buf.line(0), "中文ab", "退格没删掉最后一个 ASCII 字符");
    buf.move_home();
    buf.insert_char('测');
    assert_eq!(buf.line(0), "测中文ab", "行首插中文不对");
    buf.delete();
    assert_eq!(buf.line(0), "测文ab", "删一个中文字符不对（列不是字节）");
    for _ in 0..20 {
        buf.move_right();
    }
    assert_eq!(buf.cursor, (0, 4), "光标停不到行尾（4 个字符）");
}

#[test]
fn editable_buffer_joins_and_splits_lines() {
    let mut buf = TextBuffer::from_text("第一行\n第二行");
    buf.set_cursor((1, 0), false);
    buf.backspace();
    assert_eq!(buf.line_count(), 1, "行首退格应该并成一行");
    assert_eq!(buf.line(0), "第一行第二行");
    assert_eq!(buf.cursor, (0, 3), "并完光标该停在接缝上");
    buf.insert_newline();
    assert_eq!(buf.line_count(), 2, "回车应该断成两行");
    assert_eq!(buf.line(0), "第一行");
    assert_eq!(buf.line(1), "第二行");
    buf.set_cursor((0, 3), false);
    buf.delete();
    assert_eq!(buf.line_count(), 1, "行尾 Delete 应该并成一行");
    assert_eq!(buf.line(0), "第一行第二行");
}

#[test]
fn editable_buffer_vertical_moves_keep_the_column() {
    let mut buf = TextBuffer::from_text("abcd\nab\nabcdef");
    buf.set_cursor((0, 3), false);
    buf.move_vertical(true);
    assert_eq!(buf.cursor, (1, 2), "短行应该落到行尾");
    buf.move_vertical(true);
    assert_eq!(buf.cursor, (2, 2), "列保持在上一次落点，不该跳回 3");
    buf.move_vertical(false);
    assert_eq!(buf.cursor, (1, 2), "往上也一样");
}

#[test]
fn editable_buffer_tab_inserts_spaces() {
    let mut buf = TextBuffer::from_text("x");
    let mut preedit = String::new();
    buf.move_end();
    let out = feed(&mut buf, &mut preedit, &[key_event(egui::Key::Tab, false)]);
    assert!(out.changed);
    assert_eq!(buf.line(0), "x    ");
    assert!(!buf.to_text().contains('\t'), "制表符不许进缓冲区");
}

#[test]
fn editable_buffer_selection_copy_cut_paste_undo() {
    let mut buf = TextBuffer::from_text("一行\n二行");
    let mut preedit = String::new();
    let out = feed(&mut buf, &mut preedit, &[key_event(egui::Key::A, true)]);
    assert!(!out.changed, "全选不改内容");
    assert_eq!(buf.selected_text(), "一行\n二行");
    let out = feed(&mut buf, &mut preedit, &[key_event(egui::Key::C, true)]);
    assert_eq!(
        out.copy.as_deref(),
        Some("一行\n二行"),
        "复制要给出选中文本"
    );
    assert_eq!(buf.to_text(), "一行\n二行", "复制不许动内容");
    let out = feed(&mut buf, &mut preedit, &[key_event(egui::Key::X, true)]);
    assert!(out.changed);
    assert_eq!(out.copy.as_deref(), Some("一行\n二行"));
    assert_eq!(buf.to_text(), "", "剪切应该把内容拿走");
    let out = feed(
        &mut buf,
        &mut preedit,
        &[egui::Event::Paste("aa\nbb\ncc".to_owned())],
    );
    assert!(out.changed);
    assert_eq!(buf.line_count(), 3, "粘贴带换行的文本要拆成 3 行");
    assert_eq!(buf.to_text(), "aa\nbb\ncc");
    let out = feed(&mut buf, &mut preedit, &[key_event(egui::Key::Z, true)]);
    assert!(out.changed);
    assert_eq!(buf.to_text(), "", "撤销应该退回粘贴之前");
}

#[test]
fn editable_buffer_ime_preedit_and_commit() {
    let mut buf = TextBuffer::from_text("");
    let mut preedit = String::new();
    let pre = egui::Event::Ime(egui::ImeEvent::Preedit {
        text: "zhong".to_owned(),
        active_range_chars: None,
    });
    let out = feed(&mut buf, &mut preedit, &[pre]);
    assert!(!out.changed, "候选串不能写进缓冲区");
    assert_eq!(preedit, "zhong");
    assert_eq!(buf.to_text(), "", "候选串没落盘才对");
    let out = feed(
        &mut buf,
        &mut preedit,
        &[egui::Event::Ime(egui::ImeEvent::Commit("中".to_owned()))],
    );
    assert!(out.changed);
    assert_eq!(preedit, "", "上屏之后候选串要清掉");
    assert_eq!(buf.to_text(), "中");
    let out = feed(
        &mut buf,
        &mut preedit,
        &[egui::Event::Ime(egui::ImeEvent::DeleteSurrounding {
            before_chars: 1,
            after_chars: 0,
        })],
    );
    assert!(out.changed);
    assert_eq!(buf.to_text(), "", "DeleteSurrounding 该把那个中文字删掉");
}

#[test]
fn editor_code_runs_only_color_comments() {
    use crate::ui::editor::{SynKind, code_runs, syn_color};
    let line = "let x = \"a\"; // 注释 \"b\"";
    let runs = code_runs(line);
    let kinds: Vec<(&str, SynKind)> = runs.iter().map(|(s, k)| (s.as_str(), *k)).collect();

    assert_eq!(
        kinds,
        vec![
            ("let x = \"a\"; ", SynKind::Plain),
            ("// 注释 \"b\"", SynKind::Comment),
        ]
    );
    let joined: String = runs.iter().map(|(s, _)| s.as_str()).collect();
    assert_eq!(joined, line, "着色片段拼起来必须和原行一字不差");

    assert_eq!(syn_color(SynKind::Plain), theme().text);
    assert_eq!(syn_color(SynKind::Comment), theme().syn_comment);
    assert_eq!(
        Theme::CLASSIC.syn_comment,
        egui::Color32::from_rgb(0x00, 0x99, 0x00)
    );
    assert_eq!(
        Theme::DARK.syn_comment,
        egui::Color32::from_rgb(0xFF, 0x66, 0xFF),
        "深色注释颜色没有按浅色反相"
    );
}

#[test]
fn editor_ime_frames_never_interrupt_the_composition() {
    let ctx = egui::Context::default();
    configure_tessellation(&ctx);
    ctx.set_pixels_per_point(1.0);
    let mut app = blank_app();
    app.text_engine = HardText::new(&ctx);
    if app.text_engine.is_none() {
        return;
    }
    app.replace_bars(data::random_walk::generate(BAR_SEED, 64, BAR_START));
    app.open_source_window(PANE_CANDLE);
    let layout = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(WINDOW_W, WINDOW_H));

    let frame = |app: &mut StockApp, events: Vec<egui::Event>| {
        let mut raw = egui::RawInput::default();
        raw.screen_rect = Some(layout);
        raw.events = events;
        let mut out = ctx.run_ui(raw, |ui| {
            app.draw_frame(ui);
        });
        absorb_textures(&mut out.textures_delta);
        let _ = ctx.tessellate(std::mem::take(&mut out.shapes), 1.0);
        out.platform_output
    };
    let press = |pos: egui::Pos2| {
        vec![
            egui::Event::PointerMoved(pos),
            egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: egui::Modifiers::NONE,
            },
        ]
    };
    let release = |pos: egui::Pos2| {
        vec![egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: egui::Modifiers::NONE,
        }]
    };
    frame(&mut app, vec![]);
    let source = app
        .win_mgr
        .wins
        .iter()
        .find(|w| w.kind == WinKind::Source)
        .expect("源码编辑窗没建出来")
        .rect();
    let inside = egui::pos2(source.min.x + 40.0, source.min.y + SRC_BODY_TOP + 9.0);
    frame(&mut app, vec![egui::Event::PointerMoved(inside)]);
    frame(&mut app, press(inside));
    frame(&mut app, release(inside));

    let out = frame(
        &mut app,
        vec![egui::Event::Ime(egui::ImeEvent::Preedit {
            text: "zhong".to_owned(),
            active_range_chars: None,
        })],
    );
    let ime = out.ime.expect("有焦点 + 光标可见时必须报输入法候选窗位置");
    assert!(
        !ime.should_interrupt_composition,
        "候选串那一帧把输入法组合重置了"
    );

    let out = frame(
        &mut app,
        vec![egui::Event::Ime(egui::ImeEvent::Commit("中".to_owned()))],
    );
    let ime = out.ime.expect("上屏那一帧也得报输入法候选窗位置");
    assert!(
        !ime.should_interrupt_composition,
        "上屏那一帧把输入法组合重置了"
    );
    assert!(
        app.source.editor.buf.to_text().contains('中'),
        "上屏的中文没进编辑器缓冲区"
    );
}

#[test]
fn crosshair_time_label_never_leaks_into_the_status_bar() {
    let mut app = blank_app();
    app.replace_bars(data::random_walk::generate(BAR_SEED, 64, BAR_START));
    let _ = render_window_at(&mut app, None);
    let chart = app
        .win_mgr
        .wins
        .iter()
        .find(|w| w.kind == WinKind::Chart)
        .expect("图表窗不存在");
    let macd = ManagedWin::body(chart.rect());
    let l = app.panes[PANE_MACD].indicator.layout();
    assert_eq!(l.time_rows, 0, "这块图表的用例前提是「没有时间带」");
    let (_, _, main, time) = chart_rows(macd, l);
    assert!(time.is_none(), "MACD 不该有时间带");
    let plot = frame_rects(main).0;

    let boundary = macd.max.y as usize - 1;

    let grid = render_window_at(&mut app, Some(plot.center()));
    app.panes[0]
        .view
        .crosshair
        .expect("鼠标在图里应该有十字光标");
    let (x0, x1) = (plot.min.x as usize, plot.max.x as usize);
    let longest = |row: usize| -> usize {
        let (mut best, mut cur) = (0usize, 0usize);
        for x in x0..x1 {
            cur = if grid[row][x] == Some(C_LINE) {
                cur + 1
            } else {
                0
            };
            best = best.max(cur);
        }
        best
    };
    let label_top = boundary - (PITCH * CROSS_LABEL_ROWS as f32) as usize;
    assert!(longest(label_top) >= 19, "图表下沿没看到索引牌的上边框");
    assert!(
        longest(boundary + 1) <= 1,
        "索引牌的下边框漏进状态栏了（连排 {}px）",
        longest(boundary + 1)
    );
}

#[test]
fn editable_buffer_round_trips_the_source_file() {
    let raw = std::fs::read_to_string("src/indicator/candle.rs").expect("读不到 candle.rs");
    let want = raw.replace("\r\n", "\n");
    assert!(
        want.contains("日期") && want.contains("涨跌幅"),
        "这份文件本来就带中文字符"
    );
    let buf = TextBuffer::from_text(&want);
    assert_eq!(buf.to_text(), want, "装载之后和原文件对不上");
    assert!(buf.to_text().contains("日期"), "中文字符被过滤掉了");
    assert!(buf.to_text().contains("涨跌幅"), "中文字符被过滤掉了");

    let tail = TextBuffer::from_text("a\n");
    assert_eq!(tail.to_text(), "a\n", "末尾换行没保住");
}

#[test]
fn source_save_keeps_the_line_endings() {
    let mut path = std::env::temp_dir();
    path.push("rt_write_source_check.rs");
    std::fs::write(&path, "a\r\nb\r\n").unwrap();
    write_source(&path, "a\nb\nc").unwrap();
    assert_eq!(std::fs::read_to_string(&path).unwrap(), "a\r\nb\r\nc");
    let fresh = path.with_extension("fresh.rs");
    write_source(&fresh, "x\ny").unwrap();
    assert_eq!(std::fs::read_to_string(&fresh).unwrap(), "x\r\ny");
    let bytes = std::fs::read(&fresh).unwrap();
    assert_ne!(bytes[0], 0xEF, "写出去的文件不许带 BOM");
    let _ = std::fs::remove_file(&path);
    let _ = std::fs::remove_file(&fresh);
}

#[test]
fn single_line_input_limits_joins_and_maps_clicks() {
    let mut input = TextInput::new(HEX_LEN_MAX);
    input.set_text("2288FF");
    assert_eq!(input.text(), "2288FF");
    input.set_text("1234567890");
    assert_eq!(input.text(), "123456", "超过 6 个字符要砍尾巴");
    input.set_text("ab\ncd");
    assert_eq!(input.text(), "abcd", "单行框里不许出现第二行");

    assert_eq!(col_at("abc", 0.0), 0);
    assert_eq!(col_at("abc", 12.0), 1);
    assert_eq!(col_at("中文", 14.0), 1);
    assert_eq!(col_at("ab", 999.0), 2, "点在行尾右边就是行尾");
}

#[test]
fn editor_chrome_sits_on_the_grid() {
    assert_eq!(SRC_MENU_ROWS, TEXT_ROWS, "菜单栏不是 2 格");
    assert_eq!(SRC_BAR_ROWS, TEXT_ROWS, "按钮栏不是 2 格");
    assert_eq!(SRC_HEAD_ROWS, SRC_TITLE_ROWS + TEXT_ROWS + TEXT_ROWS);
    assert_eq!(
        SRC_BODY_TOP as i32,
        grid_line(SRC_HEAD_ROWS) as i32,
        "正文区上沿不在网格线上"
    );
    assert_eq!(SRC_BODY_TOP as i32 % PITCH as i32, 0);
    assert_eq!(
        SRC_ROWS,
        SRC_HEAD_ROWS + SRC_BODY_ROWS + 1,
        "标题栏 / 菜单栏 / 按钮栏 / 正文区 / 底边那条线加起来应该正好是窗口高"
    );

    let w = cells(2);
    assert_eq!(((w - LINE_W) as i32) % PITCH as i32, 0, "按钮宽不是整数格");
    assert!(w - 2.0 * LINE_W >= ICON_W, "内容区塞不下 16x16 图标");
    assert_eq!(
        ((ICON_W + ICON_GAP) as i32) % PITCH as i32,
        0,
        "图标 + 间隔不是整数格"
    );

    let mut app = blank_app();
    assert_eq!(EDITOR_TABS.len(), 3, "编辑器菜单：文件 / 编辑 / 查看");
    assert_eq!(EDITOR_BUTTONS.len(), 7, "按钮栏应该是 7 个图标按钮");
    for tab in EDITOR_TABS {
        let title = tab.title;
        let w = tab_width(&mut app, title);
        assert_eq!(
            ((w - LINE_W) as i32) % PITCH as i32,
            0,
            "菜单项宽不是整数格"
        );
        assert!(
            w - app.text_width(title) >= 2.0 * GRID,
            "菜单「{title}」左右空档不到 1 格"
        );
    }
}

#[test]
fn hover_tooltip_shares_the_button_bottom_border() {
    let ctx = egui::Context::default();
    configure_tessellation(&ctx);
    ctx.set_pixels_per_point(1.0);
    let mut app = blank_app();
    app.text_engine = HardText::new(&ctx);
    if app.text_engine.is_none() {
        return;
    }

    let rect = egui::Rect::from_min_size(egui::pos2(90.0, 90.0), egui::vec2(cells(6), BTN_H));
    let content = ButtonContent::Text("测试".to_owned());
    let tip = "按钮含义";
    let layout = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(WINDOW_W, WINDOW_H));

    let frame = |app: &mut StockApp, events: Vec<egui::Event>| {
        let mut raw = egui::RawInput::default();
        raw.screen_rect = Some(layout);
        raw.events = events;
        let mut out = ctx.run_ui(raw, |ui| {
            let painter = ui.painter().clone();
            app.draw_button(ui, &painter, rect, &content, false, tip);
            app.paint_tooltip(&painter, layout);
        });
        let shapes = std::mem::take(&mut out.shapes);
        absorb_textures(&mut out.textures_delta);
        rasterize_into(
            &ctx.tessellate(shapes, 1.0),
            WINDOW_W as usize,
            WINDOW_H as usize,
        )
    };
    frame(&mut app, vec![]);
    let grid = frame(&mut app, vec![egui::Event::PointerMoved(rect.center())]);
    let w = snap_cells_up(app.text_width(tip) + 2.0 * TEXT_PAD_X);
    let (x0, x1) = (rect.min.x as usize, (rect.min.x + w) as usize);
    let longest = |row: usize| -> usize {
        let (mut best, mut cur) = (0usize, 0usize);
        for x in x0..x1 {
            cur = if grid[row][x] == Some(C_LINE) {
                cur + 1
            } else {
                0
            };
            best = best.max(cur);
        }
        best
    };

    let top = (rect.max.y - LINE_W) as usize;
    assert_eq!(longest(top), x1 - x0, "便签上沿没压在按钮下沿那条线上");
    assert!(longest(top - 1) <= 1, "按钮下边框上面一行也画了横线");

    let bottom = top + grid_line(TEXT_ROWS) as usize;
    assert_eq!(longest(bottom), x1 - x0, "便签下边框没画出来");
    assert!(longest(bottom + 1) <= 1, "便签下边框下面一行也画了横线");
}

#[test]
fn editor_sections_are_split_by_single_grid_lines() {
    let mut app = blank_app();
    app.replace_bars(data::random_walk::generate(BAR_SEED, 64, BAR_START));
    app.open_source_window(PANE_MACD);
    let grid = render_overlays(&mut app);
    let panel = app
        .win_mgr
        .wins
        .iter()
        .find(|w| w.kind == WinKind::Source)
        .expect("源码编辑窗没建出来")
        .rect();
    let (x0, x1) = (panel.min.x as usize, panel.max.x as usize);

    let longest = |row: usize| -> usize {
        let (mut best, mut cur) = (0usize, 0usize);
        for x in x0..x1 {
            cur = if grid[row][x] == Some(C_LINE) {
                cur + 1
            } else {
                0
            };
            best = best.max(cur);
        }
        best
    };

    for rows in [
        SRC_TITLE_ROWS,
        SRC_TITLE_ROWS + SRC_MENU_ROWS,
        SRC_HEAD_ROWS,
    ] {
        let y = (panel.min.y + grid_line(rows)) as usize;
        assert_eq!(
            (y as f32 - panel.min.y) % PITCH,
            0.0,
            "分隔线 {y} 没落在窗口原点的横网格线上"
        );
        assert!(longest(y) > 100, "第 {rows} 条网格线上没画出分隔线");

        assert!(longest(y - 1) <= 1, "分隔线 {y} 上面一行也画了横线");
        assert!(longest(y + 1) <= 1, "分隔线 {y} 下面一行也画了横线");
    }
}

#[test]
fn hex_input_takes_typing_and_validates() {
    let ctx = egui::Context::default();
    configure_tessellation(&ctx);
    ctx.set_pixels_per_point(1.0);
    let mut app = color_panel_app();
    app.text_engine = HardText::new(&ctx);
    if app.text_engine.is_none() {
        return;
    }
    let layout = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(WINDOW_W, WINDOW_H));
    let id = egui::Id::new("rt_color_hex");

    let frame = |app: &mut StockApp, events: Vec<egui::Event>| {
        let mut raw = egui::RawInput::default();
        raw.screen_rect = Some(layout);
        raw.events = events;
        let mut out = ctx.run_ui(raw, |ui| {
            app.draw_frame(ui);
        });
        absorb_textures(&mut out.textures_delta);
        let _ = ctx.tessellate(std::mem::take(&mut out.shapes), 1.0);
    };
    frame(&mut app, vec![]);
    let panel = app.win_mgr.color_rect().expect("颜色面板不存在");
    let box_x = hex_box(panel).min.x;
    let inside = egui::pos2(box_x + 1.0 + 8.0, panel.min.y + CP_Y_HEX + 9.0);
    let press = vec![
        egui::Event::PointerMoved(inside),
        egui::Event::PointerButton {
            pos: inside,
            button: egui::PointerButton::Primary,
            pressed: true,
            modifiers: egui::Modifiers::NONE,
        },
    ];
    let release = vec![egui::Event::PointerButton {
        pos: inside,
        button: egui::PointerButton::Primary,
        pressed: false,
        modifiers: egui::Modifiers::NONE,
    }];

    frame(&mut app, vec![egui::Event::PointerMoved(inside)]);
    frame(&mut app, press);
    frame(&mut app, release);
    assert!(ctx.memory(|m| m.has_focus(id)), "点 Hex 框没拿到焦点");

    frame(&mut app, vec![key_event(egui::Key::A, true)]);
    frame(&mut app, vec![egui::Event::Text("00FF00".to_owned())]);
    assert_eq!(app.color_panel.hex.text(), "00FF00");
    assert_eq!(app.color_panel.rgb, (0, 255, 0), "合法 hex 没刷 RGB");
    assert!(app.color_panel.hex_valid);

    frame(&mut app, vec![key_event(egui::Key::A, true)]);
    frame(&mut app, vec![egui::Event::Text("00FF0".to_owned())]);
    assert!(!app.color_panel.hex_valid, "少一位该判非法");
    assert_eq!(app.color_panel.rgb, (0, 255, 0), "非法输入不该改颜色");

    frame(&mut app, vec![key_event(egui::Key::A, true)]);
    frame(
        &mut app,
        vec![
            egui::Event::Text("#".to_owned()),
            egui::Event::Text("FF0000".to_owned()),
        ],
    );
    assert_eq!(
        app.color_panel.hex.text(),
        "FF0000",
        "敲进来的 # 跑进输入框了（它是框外的固定前缀）"
    );
    assert_eq!(app.color_panel.rgb, (0xFF, 0x00, 0x00), "#FF0000 没刷 RGB");

    for _ in 0..12 {
        frame(&mut app, vec![key_event(egui::Key::Backspace, true)]);
    }
    let text = app.color_panel.hex.text();
    assert!(text.is_empty(), "输入框里该被退格清空：{text}");
    assert!(!app.color_panel.hex_valid, "空文本该判非法");
}

#[test]
fn color_slider_moves_only_by_dragging_the_handle() {
    let ctx = egui::Context::default();
    configure_tessellation(&ctx);
    ctx.set_pixels_per_point(1.0);
    let mut app = color_panel_app();
    app.text_engine = HardText::new(&ctx);
    if app.text_engine.is_none() {
        return;
    }
    let layout = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(WINDOW_W, WINDOW_H));
    let frame = |app: &mut StockApp, events: Vec<egui::Event>| {
        let mut raw = egui::RawInput::default();
        raw.screen_rect = Some(layout);
        raw.events = events;
        let mut out = ctx.run_ui(raw, |ui| {
            app.draw_frame(ui);
        });
        absorb_textures(&mut out.textures_delta);
        let _ = ctx.tessellate(std::mem::take(&mut out.shapes), 1.0);
    };

    frame(&mut app, vec![]);
    let panel = app.win_mgr.color_rect().expect("颜色面板不存在");
    let row = egui::Rect::from_min_size(
        egui::pos2(panel.min.x, panel.min.y + CP_Y_RGB),
        egui::vec2(CP_W, cells(CP_SLIDER_ROWS)),
    );
    let track = egui::Rect::from_min_size(
        egui::pos2(row.min.x + grid_line(CP_TRACK_LINE), row.min.y),
        egui::vec2(CP_TRACK_W, row.height()),
    );
    let press = |pos: egui::Pos2| {
        vec![
            egui::Event::PointerMoved(pos),
            egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: egui::Modifiers::NONE,
            },
        ]
    };
    let release = |pos: egui::Pos2| {
        vec![egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: egui::Modifiers::NONE,
        }]
    };
    let y = row.center().y;

    let handle_at = |app: &StockApp| {
        egui::Rect::from_min_size(
            egui::pos2(
                handle_left(track.min.x, app.color_panel.rgb.0 as f32 / 255.0),
                y,
            ),
            egui::vec2(CP_HANDLE_W, row.height()),
        )
    };
    frame(&mut app, vec![]);

    let knob = handle_at(&app);
    let far = egui::pos2(track.min.x + CP_HANDLE_TRAVEL - 4.0, y);
    assert!(!knob.contains(far), "这条用例要求落点不在把手上");
    let before = app.color_panel.rgb.0;
    frame(&mut app, press(far));
    assert_ne!(app.color_panel.rgb.0, before, "点击轨道没有移动滑块");
    frame(&mut app, release(far));

    let knob = handle_at(&app);
    let clicked_value = app.color_panel.rgb.0;
    let grab = egui::pos2(knob.min.x + CP_HANDLE_W - 2.0, y);
    frame(&mut app, press(grab));
    assert_eq!(
        app.color_panel.drag,
        ColorDrag::Slider(0),
        "按住把手没开拖动会话"
    );

    for _ in 0..3 {
        frame(&mut app, vec![]);
        assert_eq!(
            app.color_panel.rgb.0, clicked_value,
            "按住把手不放，值自己漂了"
        );
    }

    let past = egui::pos2(track.max.x + 300.0, y);
    frame(&mut app, vec![egui::Event::PointerMoved(past)]);
    assert_eq!(app.color_panel.rgb.0, 255, "拖到右端没到顶");

    assert!(
        app.pointer.pinned.is_none(),
        "拖颜色滑块时把指针钉住了（用户点名不许钉）"
    );

    for _ in 0..3 {
        frame(&mut app, vec![]);
        assert_eq!(
            app.color_panel.rgb.0, 255,
            "拖到头之后按住不放，值自己往回漂了"
        );
    }

    frame(
        &mut app,
        vec![egui::Event::PointerMoved(egui::pos2(past.x - 20.0, y))],
    );
    assert!(
        app.color_panel.rgb.0 < 255,
        "从轨道外往回拉，把手没有立刻跟手"
    );

    let inside = egui::pos2(past.x - 40.0, y);
    frame(&mut app, vec![egui::Event::PointerMoved(inside)]);
    let back = app.color_panel.rgb.0;
    assert!(
        (0..255).contains(&back),
        "手回到轨道里之后把手没立刻跟（{back}）"
    );
    frame(&mut app, release(inside));
}

#[test]
fn color_sliders_follow_the_mouse_wheel_while_hovered() {
    let ctx = egui::Context::default();
    configure_tessellation(&ctx);
    ctx.set_pixels_per_point(1.0);
    let mut app = color_panel_app();
    app.text_engine = HardText::new(&ctx);
    let layout = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(WINDOW_W, WINDOW_H));
    let frame = |app: &mut StockApp, events: Vec<egui::Event>| {
        let mut raw = egui::RawInput::default();
        raw.screen_rect = Some(layout);
        raw.events = events;
        let mut out = ctx.run_ui(raw, |ui| {
            app.draw_frame(ui);
        });
        absorb_textures(&mut out.textures_delta);
        let _ = ctx.tessellate(std::mem::take(&mut out.shapes), 1.0);
    };
    let wheel = |delta: f32| {
        vec![egui::Event::MouseWheel {
            unit: egui::MouseWheelUnit::Line,
            delta: egui::vec2(0.0, delta),
            phase: egui::TouchPhase::Move,
            modifiers: egui::Modifiers::NONE,
        }]
    };

    frame(&mut app, vec![]);
    let panel = app.win_mgr.color_rect().expect("颜色面板不存在");
    let row = egui::Rect::from_min_size(
        egui::pos2(panel.min.x, panel.min.y + CP_Y_RGB),
        egui::vec2(CP_W, cells(CP_SLIDER_ROWS)),
    );
    let track = egui::Rect::from_min_size(
        egui::pos2(row.min.x + grid_line(CP_TRACK_LINE), row.min.y),
        egui::vec2(CP_TRACK_W, row.height()),
    );
    frame(&mut app, vec![]);
    frame(&mut app, vec![egui::Event::PointerMoved(track.center())]);
    let red = app.color_panel.rgb.0;
    frame(&mut app, wheel(-3.0));
    assert_eq!(app.color_panel.rgb.0, red.saturating_sub(3));

    let hue_band = cp_band(panel, CP_ROW_HUE, CP_HUE_ROWS);
    frame(&mut app, vec![egui::Event::PointerMoved(hue_band.center())]);
    let hue = app.color_panel.h;
    frame(&mut app, wheel(1.0));
    assert_ne!(app.color_panel.h, hue, "滚轮没有移动色相滑块");

    app.color_panel.set_hsv(0, 255, 255);
    frame(&mut app, vec![]);
    let hue_inner = hue_band.shrink(LINE_W);
    let grab = egui::pos2(hue_inner.min.x + 5.0, hue_inner.center().y);
    frame(&mut app, vec![egui::Event::PointerMoved(grab)]);
    frame(
        &mut app,
        vec![
            egui::Event::PointerMoved(grab),
            egui::Event::PointerButton {
                pos: grab,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: egui::Modifiers::NONE,
            },
        ],
    );
    frame(
        &mut app,
        vec![egui::Event::PointerMoved(grab + egui::vec2(1.0, 0.0))],
    );
    assert!(
        (1..=3).contains(&app.color_panel.h),
        "色相滑块没有按 1px 连续移动：{}",
        app.color_panel.h
    );
}

#[test]
fn editor_body_takes_focus_and_types() {
    let ctx = egui::Context::default();
    configure_tessellation(&ctx);
    ctx.set_pixels_per_point(1.0);
    let mut app = blank_app();
    app.text_engine = HardText::new(&ctx);
    if app.text_engine.is_none() {
        return;
    }
    app.replace_bars(data::random_walk::generate(BAR_SEED, 64, BAR_START));
    app.open_source_window(PANE_CANDLE);
    let layout = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(WINDOW_W, WINDOW_H));
    let id = egui::Id::new(EDITOR_ID);

    let frame = |app: &mut StockApp, events: Vec<egui::Event>| {
        let mut raw = egui::RawInput::default();
        raw.screen_rect = Some(layout);
        raw.events = events;
        let mut out = ctx.run_ui(raw, |ui| {
            app.draw_frame(ui);
        });
        absorb_textures(&mut out.textures_delta);
        let _ = ctx.tessellate(std::mem::take(&mut out.shapes), 1.0);
    };
    let press = |pos: egui::Pos2| {
        vec![
            egui::Event::PointerMoved(pos),
            egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: egui::Modifiers::NONE,
            },
        ]
    };
    let release = |pos: egui::Pos2| {
        vec![egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: egui::Modifiers::NONE,
        }]
    };

    frame(&mut app, vec![]);
    let source = app
        .win_mgr
        .wins
        .iter()
        .find(|w| w.kind == WinKind::Source)
        .expect("源码编辑窗没建出来");
    let source = source.rect();
    let inside = egui::pos2(source.min.x + 40.0, source.min.y + SRC_BODY_TOP + 9.0);
    let outside = egui::pos2(layout.max.x - 10.0, split_areas(layout)[2].min.y + 10.0);
    frame(&mut app, vec![egui::Event::PointerMoved(inside)]);
    frame(&mut app, press(inside));
    frame(&mut app, release(inside));
    assert!(ctx.memory(|m| m.has_focus(id)), "点了正文区没拿到焦点");

    let col = app.source.editor.buf.cursor;
    for _ in 0..3 {
        frame(&mut app, vec![]);
    }
    assert!(ctx.memory(|m| m.has_focus(id)), "点完之后焦点没留住");
    frame(&mut app, vec![egui::Event::Text("xy".to_owned())]);
    assert_eq!(
        app.source.editor.buf.cursor,
        (col.0, col.1 + 2),
        "拿到焦点之后敲的字没进缓冲区"
    );

    frame(&mut app, press(outside));
    frame(&mut app, release(outside));
    assert!(!ctx.memory(|m| m.has_focus(id)), "点窗口外面焦点没还回去");
    let text = app.source.editor.buf.to_text();
    frame(&mut app, vec![egui::Event::Text("z".to_owned())]);
    assert_eq!(
        app.source.editor.buf.to_text(),
        text,
        "没焦点还在收键盘事件"
    );
}

#[test]
fn editor_drag_selection_keeps_the_press_origin_char() {
    let ctx = egui::Context::default();
    configure_tessellation(&ctx);
    ctx.set_pixels_per_point(1.0);
    let mut app = blank_app();
    app.text_engine = HardText::new(&ctx);
    if app.text_engine.is_none() {
        return;
    }
    app.replace_bars(data::random_walk::generate(BAR_SEED, 64, BAR_START));
    app.open_source_window(PANE_CANDLE);
    app.source.editor.buf = TextBuffer::from_text("abcdef\nsecond");
    let _ = render_events_in(&ctx, &mut app, vec![]);
    let win = app
        .win_mgr
        .wins
        .iter()
        .find(|w| w.kind == WinKind::Source)
        .expect("源码编辑窗没建出来")
        .w
        .rect();
    let code_x = text_left_x(win.min.x + grid_line(SRC_GUTTER_COLS), win.min.x);
    let y = win.min.y + SRC_BODY_TOP + LINE_W + SRC_LINE_H / 2.0;
    let start = egui::pos2(code_x + 1.0, y);
    let end = egui::pos2(code_x + 4.0 * 9.0 + 6.0, y);
    let _ = render_events_in(
        &ctx,
        &mut app,
        vec![
            egui::Event::PointerMoved(start),
            egui::Event::PointerButton {
                pos: start,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: egui::Modifiers::NONE,
            },
        ],
    );

    let _ = render_events_in(&ctx, &mut app, vec![egui::Event::PointerMoved(end)]);
    assert_eq!(
        app.source.editor.buf.selection(),
        Some(((0, 0), (0, 5))),
        "框选漏掉了按下点所在的起始文字"
    );
    let _ = render_events_in(
        &ctx,
        &mut app,
        vec![egui::Event::PointerButton {
            pos: end,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: egui::Modifiers::NONE,
        }],
    );
}

fn secondary_click(pos: egui::Pos2) -> (Vec<egui::Event>, Vec<egui::Event>) {
    (
        vec![
            egui::Event::PointerMoved(pos),
            egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Secondary,
                pressed: true,
                modifiers: egui::Modifiers::NONE,
            },
        ],
        vec![egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Secondary,
            pressed: false,
            modifiers: egui::Modifiers::NONE,
        }],
    )
}

fn primary_click(pos: egui::Pos2) -> (Vec<egui::Event>, Vec<egui::Event>) {
    (
        vec![
            egui::Event::PointerMoved(pos),
            egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: egui::Modifiers::NONE,
            },
        ],
        vec![egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: egui::Modifiers::NONE,
        }],
    )
}

#[test]
fn main_blank_context_menu_opens_and_runs_an_action() {
    let ctx = egui::Context::default();
    configure_tessellation(&ctx);
    ctx.set_pixels_per_point(1.0);
    let mut app = blank_app();
    app.replace_bars(data::random_walk::generate(BAR_SEED, 64, BAR_START));
    let _ = render_events_in(&ctx, &mut app, vec![]);
    let layout = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(WINDOW_W, WINDOW_H));
    let work = split_areas(layout)[2];
    let blank = (1..work.height() as i32)
        .step_by(9)
        .flat_map(|dy| (1..work.width() as i32).step_by(9).map(move |dx| (dx, dy)))
        .map(|(dx, dy)| work.min + egui::vec2(dx as f32, dy as f32))
        .find(|p| {
            !app.win_mgr
                .wins
                .iter()
                .enumerate()
                .any(|(i, w)| w.w.open && app.win_mgr.rect_of(i).contains(*p))
        })
        .expect("工作区里找不到一块空白");
    let _ = render_events_in(&ctx, &mut app, vec![egui::Event::PointerMoved(blank)]);
    let (press, release) = secondary_click(blank);
    let _ = render_events_in(&ctx, &mut app, press);

    let opened = app.context_menu.open().expect("右键按下的那一帧没打开菜单");
    assert_eq!(opened.kind, ContextMenuKind::MainBlank);
    let _ = render_events_in(&ctx, &mut app, release);
    let opened = app.context_menu.open().expect("松开右键后菜单不该消失");
    assert_eq!(opened.kind, ContextMenuKind::MainBlank);
    let panel = menu_rect(&mut app, opened.kind, opened.anchor, layout);
    let before = app.win_mgr.chart_count();
    let (press, release) = primary_click(item_rect(panel, 0).center());
    let _ = render_events_in(&ctx, &mut app, press);
    let _ = render_events_in(&ctx, &mut app, release);
    assert_eq!(
        app.win_mgr.chart_count(),
        before + 1,
        "右键菜单「新增图表」没执行"
    );
    assert!(app.context_menu.open().is_none(), "点完菜单项没有收起");
}

#[test]
fn main_context_menu_opens_the_debug_console() {
    let ctx = egui::Context::default();
    configure_tessellation(&ctx);
    ctx.set_pixels_per_point(1.0);
    let mut app = blank_app();
    let layout = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(WINDOW_W, WINDOW_H));
    let anchor = split_areas(layout)[2].center();
    app.context_menu.open_main(anchor);
    let _ = render_events_in(&ctx, &mut app, vec![]);
    let panel = menu_rect(&mut app, ContextMenuKind::MainBlank, anchor, layout);
    let (press, release) = primary_click(item_rect(panel, 1).center());
    let _ = render_events_in(&ctx, &mut app, press);
    let _ = render_events_in(&ctx, &mut app, release);
    assert!(app.win_mgr.console_open(), "右键菜单没有打开调试控制台");
}

#[test]
fn source_and_console_open_pinned() {
    let mut app = blank_app();
    app.open_source_window(PANE_CANDLE);
    let source = app
        .win_mgr
        .wins
        .iter()
        .find(|w| w.kind == WinKind::Source)
        .expect("源码编辑窗没建出来");
    assert!(source.pinned, "指标代码窗默认应置顶");

    app.open_console_window();
    let console = app
        .win_mgr
        .wins
        .iter()
        .find(|w| w.kind == WinKind::Console)
        .expect("控制台窗没建出来");
    assert!(console.pinned, "调试控制台默认应置顶");
}

#[test]
fn blank_right_click_uses_visible_window_rects_after_minimize() {
    let ctx = egui::Context::default();
    configure_tessellation(&ctx);
    ctx.set_pixels_per_point(1.0);
    let mut app = blank_app();
    app.replace_bars(data::random_walk::generate(BAR_SEED, 64, BAR_START));
    let _ = render_events_in(&ctx, &mut app, vec![]);
    let i = app
        .win_mgr
        .wins
        .iter()
        .position(|w| w.kind == WinKind::Chart)
        .expect("图表窗没建出来");
    let old = app.win_mgr.wins[i].rect();
    let p = old.center();
    let title = app.win_mgr.wins[i].title.clone();
    let w = WinMgr::mini_width(&mut app, &title);
    app.win_mgr.minimize(i, w);
    let _ = render_events_in(&ctx, &mut app, vec![]);
    assert!(
        !app.win_mgr.rect_of(i).contains(p),
        "最小化那条标题栏不该还占着原来的大矩形"
    );
    assert!(
        !app.win_mgr
            .wins
            .iter()
            .enumerate()
            .any(|(k, win)| win.w.open && app.win_mgr.rect_of(k).contains(p)),
        "取样点应该已经是空桌面"
    );
    let (press, _release) = secondary_click(p);
    let _ = render_events_in(&ctx, &mut app, press);
    let opened = app
        .context_menu
        .open()
        .expect("窗口最小化后露出的大片桌面上右键没反应");
    assert_eq!(opened.kind, ContextMenuKind::MainBlank);
}

#[test]
fn editor_context_menu_can_select_the_whole_buffer() {
    let ctx = egui::Context::default();
    configure_tessellation(&ctx);
    ctx.set_pixels_per_point(1.0);
    let mut app = blank_app();
    app.text_engine = HardText::new(&ctx);
    if app.text_engine.is_none() {
        return;
    }
    app.replace_bars(data::random_walk::generate(BAR_SEED, 64, BAR_START));
    app.open_source_window(PANE_CANDLE);
    let _ = render_events_in(&ctx, &mut app, vec![]);
    let win = app
        .win_mgr
        .wins
        .iter()
        .find(|w| w.kind == WinKind::Source)
        .expect("源码编辑窗没建出来")
        .w
        .rect();
    let at = egui::pos2(
        text_left_x(win.min.x + grid_line(SRC_GUTTER_COLS), win.min.x) + 16.0,
        win.min.y + SRC_BODY_TOP + LINE_W + SRC_LINE_H / 2.0,
    );
    let _ = render_events_in(&ctx, &mut app, vec![egui::Event::PointerMoved(at)]);
    let (press, release) = secondary_click(at);
    let _ = render_events_in(&ctx, &mut app, press);
    let _ = render_events_in(&ctx, &mut app, release);
    let opened = app.context_menu.open().expect("编辑器右键没打开菜单");
    assert_eq!(opened.kind, ContextMenuKind::Editor);
    let layout = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(WINDOW_W, WINDOW_H));
    let panel = menu_rect(&mut app, opened.kind, opened.anchor, layout);
    let (press, release) = primary_click(item_rect(panel, 3).center());
    let _ = render_events_in(&ctx, &mut app, press);
    let _ = render_events_in(&ctx, &mut app, release);
    let text = app.source.editor.buf.to_text();
    assert_eq!(
        app.source.editor.buf.selected_text(),
        text,
        "编辑器右键「全选」没生效"
    );
}

#[test]
fn console_log_has_a_clickable_vertical_scrollbar() {
    let ctx = egui::Context::default();
    configure_tessellation(&ctx);
    ctx.set_pixels_per_point(1.0);
    let mut app = blank_app();
    app.text_engine = HardText::new(&ctx);
    app.open_console_window();
    for i in 0..80 {
        app.console.log(format!("line {i}"));
    }
    let _ = render_events_in(&ctx, &mut app, vec![]);
    let console = app
        .win_mgr
        .wins
        .iter()
        .find(|w| w.kind == WinKind::Console)
        .expect("控制台窗没建出来");
    let rect = ManagedWin::body(console.rect());
    let bar_line = rect.max.x - 2.0 * LINE_W - SRC_BAR_W;
    assert_eq!(
        (bar_line - rect.min.x) % PITCH,
        0.0,
        "控制台滚动条没有落在窗口网格线上"
    );
    assert_eq!(
        (cells(TEXT_ROWS) - LINE_W) % PITCH,
        0.0,
        "控制台滚动条上端没有落在网格线上"
    );
    let before = app.console.scroll();
    assert!(before > 0, "日志应自动跟到末尾");
    let p = egui::pos2(rect.max.x - 5.0, rect.min.y + cells(TEXT_ROWS) + 8.0);
    let _ = render_events_in(
        &ctx,
        &mut app,
        vec![
            egui::Event::PointerMoved(p),
            egui::Event::PointerButton {
                pos: p,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: egui::Modifiers::NONE,
            },
        ],
    );
    let _ = render_events_in(
        &ctx,
        &mut app,
        vec![egui::Event::PointerButton {
            pos: p,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: egui::Modifiers::NONE,
        }],
    );
    assert!(
        app.console.scroll() < before,
        "点击控制台滚动条顶部没有向上移动"
    );
}

#[test]
fn console_scrollbar_thumb_ends_on_the_footer_border() {
    let ctx = egui::Context::default();
    configure_tessellation(&ctx);
    ctx.set_pixels_per_point(1.0);
    let mut app = blank_app();
    app.text_engine = HardText::new(&ctx);
    app.open_console_window();
    for i in 0..120 {
        app.console.log(format!("line {i}"));
    }

    let pixels = render_events_in(&ctx, &mut app, vec![]);
    let console = app
        .win_mgr
        .wins
        .iter()
        .find(|w| w.kind == WinKind::Console)
        .expect("控制台窗没建出来");
    let body = ManagedWin::body(console.rect());
    let footer_top = (body.max.y - crate::ui::console::CONSOLE_FOOTER_H) as usize;
    let thumb_x = (body.max.x - 5.0) as usize;

    assert_eq!(
        pixels[footer_top - 1][thumb_x],
        Some(C_SCROLL_THUMB),
        "滑块没有贴到最底下"
    );
    assert_eq!(
        pixels[footer_top][thumb_x],
        Some(C_CTRL_BORDER),
        "滑块底边和页脚边框没有合成 1px"
    );
    assert_ne!(
        pixels[footer_top - 2][thumb_x],
        Some(C_CTRL_BORDER),
        "滑块下面出现了第二条边框线"
    );
}

#[test]
fn context_menu_has_no_internal_lines_and_stays_open_on_pointer_leave() {
    let ctx = egui::Context::default();
    configure_tessellation(&ctx);
    ctx.set_pixels_per_point(1.0);
    let mut app = blank_app();
    app.text_engine = HardText::new(&ctx);
    app.replace_bars(data::random_walk::generate(BAR_SEED, 64, BAR_START));
    let layout = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(WINDOW_W, WINDOW_H));
    let work = split_areas(layout)[2];
    let anchor = work.center();
    app.context_menu.open_main(anchor);
    let grid = render_overlays_at(&mut app, Some(anchor));
    let opened = app.context_menu.open().expect("右键菜单没有打开");
    let panel = menu_rect(&mut app, opened.kind, opened.anchor, layout);
    for j in 0..2 {
        let seam = (panel.min.y + grid_line((j + 1) * TEXT_ROWS) - LINE_W) as usize;
        for x in [panel.min.x + 3.0, panel.max.x - 4.0] {
            assert_ne!(
                grid[seam][x as usize],
                Some(C_LINE),
                "右键菜单第 {j} 项下面还有内部横线"
            );
        }
    }

    let away = egui::pos2(work.min.x + 20.0, work.max.y - 20.0);
    assert!(!panel.contains(away));
    let _ = render_overlays_at(&mut app, Some(away));
    assert!(
        app.context_menu.open().is_some(),
        "指针移出右键菜单后菜单不应立刻消失"
    );

    let (press, release) = primary_click(away);
    let _ = render_events_in(&ctx, &mut app, press);
    let _ = render_events_in(&ctx, &mut app, release);
    assert!(
        app.context_menu.open().is_none(),
        "点菜单外之后右键菜单应收起"
    );
}

#[test]
fn minimized_restore_button_uses_one_pixel_shell_line() {
    let mut app = blank_app();
    app.replace_bars(data::random_walk::generate(BAR_SEED, 64, BAR_START));
    let _ = render_window(&mut app);
    let i = app
        .win_mgr
        .wins
        .iter()
        .position(|w| w.kind == WinKind::Chart)
        .expect("图表窗没建出来");
    let title = app.win_mgr.wins[i].title.clone();
    let w = WinMgr::mini_width(&mut app, &title);
    app.win_mgr.minimize(i, w);
    let grid = render_window(&mut app);
    let mini = app.win_mgr.rect_of(i);
    let keys = sys_button_rects_n(free_band(mini, 0, WIN_TITLE_ROWS), 3);
    let restore = keys[1];
    let y = mini.min.y as usize + 4;
    let line = restore.min.x as usize;
    assert_eq!(grid[y][line], Some(C_CTRL_BORDER), "还原键左边框没画出来");
    assert_ne!(
        grid[y][line - 1],
        Some(C_CTRL_BORDER),
        "还原键左边出现了第二条线（2px 叠线）"
    );
}

#[test]
fn switching_indicator_reloads_the_open_editor() {
    let mut app = blank_app();
    app.open_source_window(PANE_MACD);
    assert!(app.source.editor.buf.to_text().contains("DIF:"));
    app.set_pane_indicator(PANE_MACD, 4);
    assert!(
        app.source.editor.buf.to_text().contains("RSV:"),
        "换了指标，编辑窗里的缓冲区还是旧指标的"
    );
    assert_eq!(app.source.pane, PANE_MACD);
    assert_eq!(
        app.source.editor.buf.to_text(),
        indicator::script_source(app.panes[PANE_MACD].indicator.name()).replace("\r\n", "\n"),
        "缓冲区和当前指标的脚本对不上"
    );
}

#[test]
fn reopening_the_same_pane_keeps_the_edits() {
    let mut app = blank_app();
    app.open_source_window(PANE_CANDLE);
    app.source
        .editor
        .buf
        .insert_text("// 我改过", &char_allowed);
    app.open_source_window(PANE_CANDLE);
    assert!(
        app.source.editor.buf.to_text().contains("我改过"),
        "同一个 Pane 再点一次「代码」把改动冲掉了"
    );
    app.open_source_window(PANE_MACD);
    assert!(
        app.source.editor.buf.to_text().contains("DIF:"),
        "换了 Pane 该重新灌一份指标代码"
    );
}

#[test]
fn long_lines_are_truncated_to_fit() {
    let long = "a".repeat(200);
    let cut = fit_line(&long, 90.0);
    assert!(text_advance(&cut) <= 90.0, "截断之后还是超宽");
    assert_eq!(cut.chars().count(), 10, "90px 只放得下 10 个 ASCII 格");
    assert_eq!(fit_line("short", 90.0), "short", "放得下就原样返回");
    let mixed = fit_line("中文中文", 90.0);
    assert_eq!(mixed, "中文中文", "90px 放得下 4 个中文（72px）");
    let narrow = fit_line("中文中文", 60.0);
    assert_eq!(narrow, "中文中", "60px 只放得下 3 个中文");
}

fn key_symbol_ink(
    ctx: &egui::Context,
    kind: SysButton,
    maximized: bool,
    keys: WinKeys,
) -> (Vec<Vec<Option<egui::Color32>>>, egui::Rect) {
    let canvas = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(140.0, 60.0));
    let btn = egui::Rect::from_min_size(egui::pos2(20.0, 20.0), egui::vec2(SYS_BTN_W, SYS_BTN_H));

    let title = egui::Rect::from_min_size(btn.min, egui::vec2(MINI_W, WIN_TITLE_H));
    let scan = match keys {
        WinKeys::MinAndClose => sys_button_rects_n(title, 3)[1],
        _ => btn,
    };
    let mut app = blank_app();
    let mut raw = egui::RawInput::default();
    raw.screen_rect = Some(canvas);
    let mut out = ctx.run_ui(raw, |ui| {
        let painter = ui.painter().clone();
        match keys {
            WinKeys::MinAndClose => {
                paint_title_bar(
                    &mut app,
                    ui,
                    &painter,
                    title,
                    7,
                    "图表",
                    WinKeys::MinAndClose,
                    false,
                    false,
                    true,
                    false,
                );
            }
            _ => {
                paint_sys_button(
                    &mut app, &painter, btn, false, kind, maximized, false, false, false,
                );
            }
        }
    });
    absorb_textures(&mut out.textures_delta);
    let grid = rasterize_into(
        &ctx.tessellate(std::mem::take(&mut out.shapes), 1.0),
        140,
        60,
    );

    let mut ink = egui::Rect::NOTHING;
    for y in scan.min.y as usize..scan.max.y as usize {
        for x in scan.min.x as usize..scan.max.x as usize {
            if grid[y][x] == Some(C_TEXT) {
                let p = egui::pos2(x as f32, y as f32);
                if ink == egui::Rect::NOTHING {
                    ink = egui::Rect::from_min_max(p, p + egui::vec2(1.0, 1.0));
                } else {
                    ink.min = ink.min.min(p);
                    ink.max = ink.max.max(p + egui::vec2(1.0, 1.0));
                }
            }
        }
    }
    (grid, ink)
}

#[test]
fn sys_key_symbols_are_squares_centered_in_the_key() {
    let ctx = egui::Context::default();
    configure_tessellation(&ctx);
    ctx.set_pixels_per_point(1.0);
    set_theme(Theme::CLASSIC);
    let btn = egui::Rect::from_min_size(egui::pos2(20.0, 20.0), egui::vec2(SYS_BTN_W, SYS_BTN_H));
    for kind in [
        SysButton::Pin,
        SysButton::Minimize,
        SysButton::Maximize,
        SysButton::Close,
    ] {
        let (_, ink) = key_symbol_ink(&ctx, kind, false, WinKeys::MinMaxClose);
        assert!(
            ink.width() > 0.0 && ink.height() > 0.0,
            "{kind:?} 没画出符号"
        );
        assert!(
            (ink.center().x - btn.center().x).abs() <= 1.0
                && (ink.center().y - btn.center().y).abs() <= 1.0,
            "{kind:?} 符号没居中"
        );
        assert!(btn.contains_rect(ink), "{kind:?} 符号超出按钮");
    }

    let app = blank_app();
    assert_ne!(
        app.symbols.get(SymbolKind::Pin),
        app.symbols.get(SymbolKind::Unpin),
        "置顶和解除置顶符号不能相同"
    );
    assert_ne!(
        app.symbols.get(SymbolKind::Maximize),
        app.symbols.get(SymbolKind::Restore),
        "最大化和还原符号不能相同"
    );
}

#[test]
fn scrollbar_thumb_and_track_are_never_the_same_gray() {
    for t in [Theme::CLASSIC, Theme::DARK] {
        let (thumb, track) = (t.scroll_thumb, t.scroll_track);
        assert_ne!(thumb, track, "{} 皮肤的滚动条和槽同色", t.name());
        let lum = |c: egui::Color32| c.r() as i32 + c.g() as i32 + c.b() as i32;
        assert_eq!(
            (lum(thumb) - lum(track)).abs(),
            3 * 0x22,
            "{} 皮肤的滚动条和槽差得不是一档灰（0x22）",
            t.name()
        );
    }
}

#[test]
fn the_scrollbar_track_and_thumb_are_inverted_between_the_two_skins() {
    assert_eq!(
        Theme::CLASSIC.scroll_track,
        egui::Color32::from_rgb(0xAA, 0xAA, 0xAA)
    );
    assert_eq!(
        Theme::CLASSIC.scroll_thumb,
        egui::Color32::from_rgb(0xCC, 0xCC, 0xCC)
    );
    assert_eq!(
        Theme::DARK.scroll_track,
        egui::Color32::from_rgb(0x55, 0x55, 0x55)
    );
    assert_eq!(
        Theme::DARK.scroll_thumb,
        egui::Color32::from_rgb(0x33, 0x33, 0x33)
    );
    let lum = |c: egui::Color32| c.r() as i32 + c.g() as i32 + c.b() as i32;
    for t in [Theme::CLASSIC, Theme::DARK] {
        assert_ne!(
            lum(t.scroll_thumb),
            lum(t.scroll_track),
            "{} 皮肤的滚动条把手和槽同灰",
            t.name()
        );
    }
}

#[test]
fn the_dark_skin_inverts_the_window_and_keeps_chart_colors_light() {
    let (l, d) = (Theme::CLASSIC, Theme::DARK);
    assert_eq!(d.title_bg, invert_color(l.title_bg));
    assert_eq!(l.title_fill, egui::Color32::from_rgb(0x99, 0x99, 0x99));
    assert_eq!(d.title_fill, egui::Color32::from_rgb(0x66, 0x66, 0x66));
    assert_eq!(d.menu_bg, invert_color(l.menu_bg));
    assert_eq!(d.work_bg, invert_color(l.work_bg));
    assert_eq!(d.desktop_bg, invert_color(l.desktop_bg));
    assert_eq!(d.status_bg, invert_color(l.status_bg));
    assert_eq!(d.panel_bg, invert_color(l.panel_bg));
    assert_eq!(d.splitter, invert_color(l.splitter));
    assert_eq!(d.scroll_track, invert_color(l.scroll_track));
    assert_eq!(d.scroll_thumb, invert_color(l.scroll_thumb));
    assert_eq!(d.btn_face, invert_color(l.btn_face));
    assert_eq!(d.btn_hover, invert_color(l.btn_hover));
    assert_eq!(d.btn_press, invert_color(l.btn_press));
    assert_eq!(d.sel_bg, invert_color(l.sel_bg));
    assert_eq!(d.text, invert_color(l.text));
    assert_eq!(d.syn_comment, invert_color(l.syn_comment));
    assert_eq!(d.pointer_trail, invert_color(l.pointer_trail));
    assert_eq!(d.pointer_trail_hot, invert_color(l.pointer_trail_hot));
    assert_eq!(d.chart_bg, egui::Color32::from_rgb(0x00, 0x00, 0x00));
    assert_eq!(d.chart_line, egui::Color32::from_rgb(0xFF, 0x00, 0x00));
    assert_eq!(d.axis_bg, egui::Color32::from_rgb(0x22, 0x22, 0x22));
    assert_eq!(d.tick, egui::Color32::from_rgb(0x22, 0x22, 0x22));
    assert_eq!(d.cross, l.cross);
    assert_eq!(d.cross_label, egui::Color32::from_rgb(0x11, 0x11, 0x11));
    assert_eq!(d.ind_zero, egui::Color32::from_rgb(0xFF, 0x00, 0x00));
    assert_eq!(d.candle_up, l.candle_up);
    assert_eq!(d.candle_down, egui::Color32::from_rgb(0x00, 0xFF, 0xFF));
    assert_eq!(d.text_up, l.text_up);
    assert_eq!(d.text_down, l.text_down);
    assert_eq!(d.dif, l.dif);
    assert_eq!(d.dea, l.dea);
    assert_eq!(d.kdj_k, l.kdj_k);
    assert_eq!(d.kdj_d, l.kdj_d);
    assert_eq!(d.kdj_j, l.kdj_j);
    assert_eq!(d.rsi_1, l.rsi_1);
    assert_eq!(d.rsi_2, l.rsi_2);
    assert_eq!(d.rsi_3, l.rsi_3);
    assert_eq!(d.boll_mid, l.boll_mid);
    assert_eq!(d.boll_up, l.boll_up);
    assert_eq!(d.boll_down, l.boll_down);
    assert_eq!(d.obv, l.obv);
    assert_eq!(d.cci, l.cci);
    assert_eq!(d.bias, l.bias);
    assert_eq!(d.ma, l.ma);
    assert_eq!(d.ema, l.ema);
    assert_eq!(d.ctrl_border, egui::Color32::from_rgb(0xFF, 0x00, 0x00));
    assert_eq!(d.line, egui::Color32::from_rgb(0xFF, 0x00, 0x00));
    assert_eq!(d.symbol, egui::Color32::from_rgb(0xFF, 0xFF, 0xFF));
    assert_eq!(d.text_white, l.text_white);
}

#[test]
fn the_pointed_color_values_are_exactly_what_the_user_asked_for() {
    let (l, d) = (Theme::CLASSIC, Theme::DARK);

    assert_eq!(l.work_bg, egui::Color32::from_rgb(0xFF, 0xFF, 0xFF));
    assert_eq!(l.chart_bg, egui::Color32::from_rgb(0xFF, 0xFF, 0xFF));
    assert_eq!(l.desktop_bg, egui::Color32::from_rgb(0xEE, 0xEE, 0xEE));
    assert_eq!(
        Theme::CLASSIC.btn_hover,
        egui::Color32::from_rgb(0xDD, 0xDD, 0xDD)
    );

    assert_eq!(d.work_bg, egui::Color32::from_rgb(0x00, 0x00, 0x00));
    assert_eq!(d.chart_bg, egui::Color32::from_rgb(0x00, 0x00, 0x00));
    assert_eq!(d.desktop_bg, egui::Color32::from_rgb(0x11, 0x11, 0x11));
    assert_eq!(
        Theme::DARK.btn_hover,
        egui::Color32::from_rgb(0x22, 0x22, 0x22)
    );
    assert_eq!(d.ctrl_border, egui::Color32::from_rgb(0xFF, 0x00, 0x00));
    assert_eq!(d.line, egui::Color32::from_rgb(0xFF, 0x00, 0x00));

    for t in [l, d] {
        assert_eq!(
            t.chart_bg,
            t.work_bg,
            "{} 皮肤：绘图区底和内容底不同色",
            t.name()
        );
        assert_ne!(
            t.work_bg,
            t.desktop_bg,
            "{} 皮肤：内容底和外面同色，里外分不开",
            t.name()
        );
    }

    for t in [Theme::CLASSIC, Theme::DARK] {
        assert_ne!(
            t.btn_hover,
            t.btn_face,
            "{} 皮肤：按钮高亮和按钮面同色",
            t.name()
        );
        assert_ne!(
            t.btn_press,
            t.menu_bg,
            "{} 皮肤：菜单悬停色和菜单底色同色",
            t.name()
        );
    }
}

#[test]
fn title_bar_blank_area_uses_the_pixel_fill() {
    let mut app = blank_app();
    app.replace_bars(data::random_walk::generate(BAR_SEED, BAR_COUNT, BAR_START));
    let grid = render_window(&mut app);
    let layout = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(WINDOW_W, WINDOW_H));
    let title = split_areas(layout)[0];
    let (x0, y0) = (
        title.min.x as usize + grid_line(10) as usize,
        title.min.y as usize + 4,
    );
    let mut fill_rows = 0;
    let mut base_rows = 0;
    for dy in 0..6usize {
        for dx in 0..12usize {
            match grid[y0 + dy][x0 + dx] {
                Some(color) if color == theme().title_fill => fill_rows += 1,
                Some(color) if color == C_TITLE_BG => base_rows += 1,
                other => panic!("标题栏 ({}, {}) 出现意外颜色 {other:?}", x0 + dx, y0 + dy),
            }
        }
    }
    assert!(fill_rows > 0, "标题栏空白区没有底纹");
    assert!(base_rows > 0, "标题栏底纹没有 1px 间距");
}

#[test]
fn the_title_text_slot_is_shadow_free_and_closed_by_a_line() {
    let mut app = blank_app();
    app.replace_bars(data::random_walk::generate(BAR_SEED, BAR_COUNT, BAR_START));
    let grid = render_window(&mut app);
    let layout = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(WINDOW_W, WINDOW_H));
    let title = split_areas(layout)[0];

    let slot = title_text_slot(&mut app, title, APP_NAME);
    assert_eq!(slot.min, title.min, "文字栏没从标题栏左边起");
    assert_eq!(
        (slot.max.x - title.min.x) % PITCH,
        1.0,
        "文字栏右边那条线没落在网格线上"
    );
    let sy = slot.min.y as usize;

    let lx = slot.max.x as usize - LINE_W as usize;
    assert_eq!(
        (lx - title.min.x as usize) % PITCH as usize,
        0,
        "文字栏右边那条线没落在网格线上（第 0 条网格线 = 标题栏左边）"
    );

    let menu_widths = [tab_width(&mut app, TOP_TABS[0].title)];
    let menu_first = menu_item_rects(split_areas(layout)[1], &menu_widths, MenuAlign::Left)[0];
    assert_eq!(
        lx,
        menu_first.max.x as usize - LINE_W as usize,
        "标题栏那条线和菜单栏第一项那条线不在同一列"
    );
    for y in (sy + LINE_W as usize)..(sy + title.height() as usize - LINE_W as usize) {
        assert_eq!(
            grid[y][lx],
            Some(C_CTRL_BORDER),
            "文字栏右边那条竖线缺了第 {y} 行"
        );
        assert_ne!(
            grid[y][lx - 1],
            Some(C_CTRL_BORDER),
            "文字栏右边那条竖线叠成 2px 了"
        );
    }
    for row in (sy + 3)..(sy + 6) {
        for x in (lx + 1)..(lx + 5) {
            assert!(
                matches!(grid[row][x], Some(color) if color == C_TITLE_BG || color == theme().title_fill),
                "文字栏线右边 ({x}, {row}) 出现了阴影或杂色"
            );
        }
    }
}

#[test]
fn menu_panels_widen_for_long_items_and_never_clip_text() {
    let mut app = blank_app();
    let ctx = egui::Context::default();
    configure_tessellation(&ctx);
    ctx.set_pixels_per_point(1.0);
    app.text_engine = HardText::new(&ctx);
    if app.text_engine.is_none() {
        return;
    }

    let mut levels: Vec<&[MenuNode]> = TOP_TABS.iter().map(|t| t.items).collect();
    for tab in TOP_TABS.iter() {
        for node in tab.items {
            if !node.children.is_empty() {
                levels.push(node.children);
            }
            for child in node.children {
                if !child.children.is_empty() {
                    levels.push(child.children);
                }
            }
        }
    }
    let mut widened = 0;
    for level in levels {
        let w = menu_panel_width(&mut app, level);
        assert_eq!(w % PITCH, LINE_W, "面板宽不是 cells(n)");
        assert!(w >= MENU_PANEL_W, "面板比默认宽还窄");
        if w > MENU_PANEL_W {
            widened += 1;
        }
        for node in level {
            let arrow = if node.has_children() { PITCH } else { 0.0 };
            let need = app.text_width(node.label) + 2.0 * TEXT_PAD_X + arrow;
            assert!(
                w >= need,
                "「{}」摆不下：面板 {w} / 需要 {need}",
                node.label
            );
        }
    }
    assert!(widened > 0, "一条加宽的面板都没有 - 这条测试没测到东西");
}

fn crop(
    grid: &[Vec<Option<egui::Color32>>],
    x: usize,
    y: usize,
    w: usize,
    h: usize,
) -> Vec<Vec<Option<egui::Color32>>> {
    (y..y + h).map(|r| grid[r][x..x + w].to_vec()).collect()
}

#[test]
#[ignore]
fn dump_zoom_for_visual_check() {
    let ctx = egui::Context::default();
    configure_tessellation(&ctx);
    ctx.set_pixels_per_point(1.0);
    set_theme(Theme::CLASSIC);
    let mut app = blank_app();
    app.text_engine = HardText::new(&ctx);
    app.replace_bars(data::random_walk::generate(BAR_SEED, BAR_COUNT, BAR_START));
    let canvas = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(90.0, 60.0));
    let mut raw = egui::RawInput::default();
    raw.screen_rect = Some(canvas);
    let mut out = ctx.run_ui(raw, |ui| {
        let painter = ui.painter().clone();
        for (i, kind) in [SysButton::Pin, SysButton::Maximize, SysButton::Close]
            .into_iter()
            .enumerate()
        {
            let btn = egui::Rect::from_min_size(
                egui::pos2(20.0 + 25.0 * i as f32, 20.0),
                egui::vec2(SYS_BTN_W, SYS_BTN_H),
            );
            paint_sys_button(
                &mut app,
                &painter,
                btn,
                false,
                kind,
                i == 1,
                false,
                i == 0,
                false,
            );
        }
    });
    absorb_textures(&mut out.textures_delta);
    let small = rasterize_into(
        &ctx.tessellate(std::mem::take(&mut out.shapes), 1.0),
        90,
        60,
    );
    let zoom = 8;
    let big: Vec<Vec<Option<egui::Color32>>> = (0..60 * zoom)
        .map(|y| (0..90 * zoom).map(|x| small[y / zoom][x / zoom]).collect())
        .collect();
    write_bmp(
        "target/zoom_keys.bmp",
        &crop(&big, 0, 150, 90 * zoom, 25 * zoom),
    );
}

#[test]
fn menu_items_have_separator_lines() {
    let ctx = egui::Context::default();
    configure_tessellation(&ctx);
    ctx.set_pixels_per_point(1.0);
    set_theme(Theme::CLASSIC);
    let mut app = blank_app();
    app.text_engine = HardText::new(&ctx);
    app.replace_bars(data::random_walk::generate(BAR_SEED, 64, BAR_START));
    app.menu.open = Some(0);
    let grid = render_overlays(&mut app);

    let areas = split_areas(egui::Rect::from_min_size(
        egui::Pos2::ZERO,
        egui::vec2(WINDOW_W, WINDOW_H),
    ));
    let anchor = menu_item_rects(
        areas[1],
        &[tab_width(&mut app, TOP_TABS[0].title)],
        MenuAlign::Left,
    )[0];
    let layout = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(WINDOW_W, WINDOW_H));
    let panel = MenuPanel::bounded(anchor, MENU_PANEL_W, TOP_TABS[0].items.len(), layout);
    for j in 0..panel.rows.saturating_sub(1) {
        let y = (panel.row(j).max.y - LINE_W) as usize;
        for x in [panel.rect.min.x + 2.0, panel.rect.max.x - 3.0] {
            assert_eq!(grid[y][x as usize], Some(C_LINE), "第 {j} 项下面缺少分隔线");
        }
    }
}

#[test]
fn menu_bar_and_popup_use_the_normal_cursor() {
    let ctx = egui::Context::default();
    configure_tessellation(&ctx);
    ctx.set_pixels_per_point(1.0);
    let mut app = blank_app();
    app.text_engine = HardText::new(&ctx);
    let layout = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(WINDOW_W, WINDOW_H));
    let areas = split_areas(layout);
    let widths = [tab_width(&mut app, TOP_TABS[0].title)];
    let tab = menu_item_rects(areas[1], &widths, MenuAlign::Left)[0];
    app.menu.open = Some(0);
    let _ = render_overlays_at(&mut app, Some(tab.center()));
    assert_eq!(app.pointer.forced_hint, Some(CursorKind::Normal));

    let panel = MenuPanel::bounded(tab, MENU_PANEL_W, TOP_TABS[0].items.len(), layout);
    let _ = render_overlays_at(&mut app, Some(panel.row(0).center()));
    assert_eq!(app.pointer.forced_hint, Some(CursorKind::Normal));
}

#[test]
fn menu_border_pixels_do_not_activate_tabs_or_items() {
    let ctx = egui::Context::default();
    configure_tessellation(&ctx);
    ctx.set_pixels_per_point(1.0);
    let mut app = blank_app();
    app.text_engine = HardText::new(&ctx);
    let layout = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(WINDOW_W, WINDOW_H));
    let areas = split_areas(layout);
    let tab = menu_item_rects(
        areas[1],
        &[tab_width(&mut app, TOP_TABS[0].title)],
        MenuAlign::Left,
    )[0];
    let border = egui::pos2(tab.min.x, tab.center().y);
    let click = |pos: egui::Pos2| {
        vec![
            egui::Event::PointerMoved(pos),
            egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: egui::Modifiers::NONE,
            },
        ]
    };
    let release = |pos: egui::Pos2| {
        vec![egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: egui::Modifiers::NONE,
        }]
    };
    let _ = render_events_in(&ctx, &mut app, click(border));
    let _ = render_events_in(&ctx, &mut app, release(border));
    assert_eq!(app.menu.open, None, "点菜单边框不该展开菜单");

    let inside = egui::pos2(tab.min.x + 2.0, tab.center().y);
    let _ = render_events_in(&ctx, &mut app, click(inside));
    let _ = render_events_in(&ctx, &mut app, release(inside));
    assert_eq!(app.menu.open, Some(0), "点菜单内容区应正常展开");

    let panel_width = menu_panel_width(&mut app, TOP_TABS[0].items);
    let panel = MenuPanel::bounded(tab, panel_width, TOP_TABS[0].items.len(), layout);
    let row = panel.row(0);
    let item_border = egui::pos2(row.min.x, row.center().y);
    let grid = render_events_in(&ctx, &mut app, vec![egui::Event::PointerMoved(item_border)]);
    let inside_x = (row.min.x + 2.0) as usize;
    let inside_y = (row.min.y + 3.0) as usize;
    assert_ne!(
        grid[inside_y][inside_x],
        Some(theme().btn_press),
        "菜单项边框不该触发悬停高亮"
    );
    let _ = render_events_in(&ctx, &mut app, click(item_border));
    let _ = render_events_in(&ctx, &mut app, release(item_border));
    assert_eq!(app.menu.open, Some(0), "点菜单项边框不该执行菜单项");
}

#[test]
fn menu_tab_hover_uses_a_visible_button_highlight() {
    let ctx = egui::Context::default();
    configure_tessellation(&ctx);
    ctx.set_pixels_per_point(1.0);
    set_theme(Theme::CLASSIC);
    let mut app = blank_app();
    app.text_engine = HardText::new(&ctx);
    app.replace_bars(data::random_walk::generate(BAR_SEED, 64, BAR_START));
    let layout = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(WINDOW_W, WINDOW_H));
    let areas = split_areas(layout);
    let tab = menu_item_rects(
        areas[1],
        &[tab_width(&mut app, TOP_TABS[0].title)],
        MenuAlign::Left,
    )[0];
    render_events_in(
        &ctx,
        &mut app,
        vec![egui::Event::PointerMoved(tab.center())],
    );
    let grid = render_events_in(&ctx, &mut app, vec![]);
    let x = (tab.min.x + 2.0) as usize;
    let y = (tab.min.y + 3.0) as usize;
    assert_eq!(
        grid[y][x],
        Some(theme().btn_press),
        "菜单项悬停时没有显示按钮式高亮"
    );
}

#[test]
fn main_menu_popup_stays_open_until_click_or_key() {
    let ctx = egui::Context::default();
    configure_tessellation(&ctx);
    ctx.set_pixels_per_point(1.0);
    let mut app = blank_app();
    app.text_engine = HardText::new(&ctx);
    let layout = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(WINDOW_W, WINDOW_H));
    let work = split_areas(layout)[2];

    app.menu.open = Some(0);
    let _ = render_events_in(
        &ctx,
        &mut app,
        vec![egui::Event::PointerMoved(work.center())],
    );
    assert_eq!(app.menu.open, Some(0), "指针移出菜单后主菜单不该自动收回");

    let (press, release) = primary_click(work.center());
    let _ = render_events_in(&ctx, &mut app, press);
    let _ = render_events_in(&ctx, &mut app, release);
    assert_eq!(app.menu.open, None, "点击菜单外区域后主菜单应收回");

    app.menu.open = Some(0);
    let _ = render_events_in(
        &ctx,
        &mut app,
        vec![egui::Event::Key {
            key: egui::Key::Escape,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::NONE,
        }],
    );
    assert_eq!(app.menu.open, None, "键盘操作后主菜单应收回");
}

#[test]
fn menu_popup_row_hover_uses_a_visible_button_highlight() {
    let ctx = egui::Context::default();
    configure_tessellation(&ctx);
    ctx.set_pixels_per_point(1.0);
    set_theme(Theme::CLASSIC);
    let mut app = blank_app();
    app.text_engine = HardText::new(&ctx);
    app.replace_bars(data::random_walk::generate(BAR_SEED, 64, BAR_START));
    let layout = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(WINDOW_W, WINDOW_H));
    let areas = split_areas(layout);
    let anchor = menu_item_rects(
        areas[1],
        &[tab_width(&mut app, TOP_TABS[0].title)],
        MenuAlign::Left,
    )[0];
    let width = menu_panel_width(&mut app, TOP_TABS[0].items);
    let panel = MenuPanel::bounded(anchor, width, TOP_TABS[0].items.len(), layout);
    let row = panel.row(1);
    app.menu.open = Some(0);
    render_events_in(
        &ctx,
        &mut app,
        vec![egui::Event::PointerMoved(row.center())],
    );
    let grid = render_events_in(&ctx, &mut app, vec![]);
    let x = (row.min.x + 2.0) as usize;
    let y = (row.min.y + 3.0) as usize;
    assert_eq!(
        grid[y][x],
        Some(theme().btn_press),
        "弹出菜单项悬停时没有显示按钮式高亮"
    );
}

#[test]
fn pane_menus_stay_open_until_a_click_or_key() {
    let ctx = egui::Context::default();
    configure_tessellation(&ctx);
    ctx.set_pixels_per_point(1.0);
    let mut app = blank_app();
    app.text_engine = HardText::new(&ctx);
    app.replace_bars(data::random_walk::generate(BAR_SEED, 64, BAR_START));

    render_events_in(&ctx, &mut app, vec![]);

    let i = app.win_mgr.live_index().expect("窗口区里一块窗口都没有");
    let pane = app.win_mgr.wins[i].pane;
    let body = ManagedWin::body(app.win_mgr.rect_of(i));
    let (title, _, plot, _) = chart_rows(body, app.panes[pane].indicator.layout());
    let widths: Vec<f32> = StockApp::PANE_TABS
        .iter()
        .map(|t| tab_width(&mut app, t))
        .collect();
    let switch = StockApp::pane_button_rects(title, &widths)[0];

    app.pane_menu = Some(pane);
    render_events_in(
        &ctx,
        &mut app,
        vec![egui::Event::PointerMoved(switch.center())],
    );
    assert_eq!(app.pane_menu, Some(pane), "指针还在图表栏上就把面板收了");

    render_events_in(
        &ctx,
        &mut app,
        vec![egui::Event::PointerMoved(plot.center())],
    );
    assert_eq!(
        app.pane_menu,
        Some(pane),
        "鼠标移出图表栏后菜单不该自动收回"
    );

    let outside = plot.center();
    render_events_in(
        &ctx,
        &mut app,
        vec![
            egui::Event::PointerMoved(outside),
            egui::Event::PointerButton {
                pos: outside,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: egui::Modifiers::NONE,
            },
        ],
    );
    render_events_in(
        &ctx,
        &mut app,
        vec![egui::Event::PointerButton {
            pos: outside,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: egui::Modifiers::NONE,
        }],
    );
    assert_eq!(app.pane_menu, None, "点击菜单外区域后菜单应收回");

    app.pane_menu = Some(pane);
    let _ = render_events_in(
        &ctx,
        &mut app,
        vec![egui::Event::Key {
            key: egui::Key::Escape,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::NONE,
        }],
    );
    assert_eq!(app.pane_menu, None, "键盘操作后菜单应收回");
}

#[test]
fn editor_scrollbar_can_be_dragged() {
    let ctx = egui::Context::default();
    configure_tessellation(&ctx);
    ctx.set_pixels_per_point(1.0);
    let mut app = blank_app();
    app.replace_bars(data::random_walk::generate(BAR_SEED, 64, BAR_START));
    app.open_source_window(PANE_MACD);
    load_long_indicator_script(&mut app);

    render_events_in(&ctx, &mut app, vec![]);
    let i = app
        .win_mgr
        .wins
        .iter()
        .position(|w| w.kind == WinKind::Source)
        .expect("源码编辑窗没建出来");
    let win = app.win_mgr.wins[i].w.rect();
    assert_eq!(app.source.editor.scroll, 0, "刚打开该从第 1 行开始");

    let body_top = win.min.y + SRC_BODY_TOP;
    let track_h = (win.max.y - LINE_W) - body_top;
    let rows = app.source.editor.rows as f32;
    let total = app.source.editor.buf.line_count() as f32;
    let handle_h = (track_h * rows / total)
        .round()
        .clamp(SRC_BAR_W, track_h.max(SRC_BAR_W));
    let travel = (win.max.y - body_top - handle_h).max(1.0);
    let bar_x = win.max.x - 5.0;
    let grab = egui::pos2(bar_x, body_top + 2.0);
    let press = |pos: egui::Pos2| {
        vec![
            egui::Event::PointerMoved(pos),
            egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: egui::Modifiers::NONE,
            },
        ]
    };
    let drag = |pos: egui::Pos2| vec![egui::Event::PointerMoved(pos)];
    let release = |pos: egui::Pos2| {
        vec![egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: egui::Modifiers::NONE,
        }]
    };

    render_events_in(&ctx, &mut app, press(grab));

    let target = egui::pos2(bar_x, body_top + 2.0 + travel * 0.6);
    render_events_in(&ctx, &mut app, drag(target));
    let max_scroll = app.source.editor.max_scroll();
    let want = (0.6 * max_scroll as f32).round() as usize;
    assert!(
        app.source.editor.scroll.abs_diff(want) <= 1,
        "拖到 60% 没滚到 60%：{} / 期望 {want}",
        app.source.editor.scroll
    );

    assert_eq!(app.source.editor.buf.cursor, (0, 0), "拖滚动条把光标挪走了");
    render_events_in(&ctx, &mut app, release(target));

    let t = app.source.editor.scroll as f32 / max_scroll as f32;
    let h = egui::pos2(bar_x, (body_top + travel * t).round() + handle_h / 2.0);
    let top = egui::pos2(bar_x, body_top + handle_h / 2.0);
    render_events_in(&ctx, &mut app, press(h));
    render_events_in(&ctx, &mut app, drag(top));
    assert_eq!(app.source.editor.scroll, 0, "拖回最上面没回到第 1 行");
    render_events_in(&ctx, &mut app, release(top));
}

#[test]
fn editor_scrollbar_follows_the_pointer_back_immediately_after_hitting_the_bottom() {
    let ctx = egui::Context::default();
    configure_tessellation(&ctx);
    ctx.set_pixels_per_point(1.0);
    let mut app = blank_app();
    app.replace_bars(data::random_walk::generate(BAR_SEED, 64, BAR_START));
    app.open_source_window(PANE_MACD);
    load_long_indicator_script(&mut app);
    render_events_in(&ctx, &mut app, vec![]);
    let i = app
        .win_mgr
        .wins
        .iter()
        .position(|w| w.kind == WinKind::Source)
        .expect("源码编辑窗没建出来");
    let win = app.win_mgr.wins[i].w.rect();
    let body_top = win.min.y + SRC_BODY_TOP;
    let bar_x = win.max.x - 5.0;
    let press = |pos: egui::Pos2| {
        vec![
            egui::Event::PointerMoved(pos),
            egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: egui::Modifiers::NONE,
            },
        ]
    };
    let drag = |pos: egui::Pos2| vec![egui::Event::PointerMoved(pos)];
    let release = |pos: egui::Pos2| {
        vec![egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: egui::Modifiers::NONE,
        }]
    };
    let max_scroll = app.source.editor.max_scroll();
    assert!(max_scroll > 0, "这份源码一屏就装下了，测不了滚动");

    let grab = egui::pos2(bar_x, body_top + 2.0);
    render_events_in(&ctx, &mut app, press(grab));
    render_events_in(&ctx, &mut app, drag(egui::pos2(bar_x, win.max.y + 400.0)));
    assert_eq!(app.source.editor.scroll, max_scroll, "没滚到最底下");

    render_events_in(&ctx, &mut app, drag(egui::pos2(bar_x, win.max.y + 1400.0)));
    assert_eq!(app.source.editor.scroll, max_scroll, "到底之后还被拖动了");

    let pin = app.pointer.pinned.expect("拖滚动条时该把指针钉在把手上");
    assert_eq!(pin.x, bar_x, "钉的是把手，横向不该碰");
    assert!(
        (body_top..win.max.y).contains(&pin.y),
        "指针没被拉回把手上：还在 {pin:?}"
    );

    render_events_in(&ctx, &mut app, drag(egui::pos2(pin.x, pin.y - 40.0)));
    assert!(
        app.source.editor.scroll < max_scroll,
        "往回抬了 40px 却还钉在最底下（多拖出去的那段被攒着了）"
    );
    render_events_in(&ctx, &mut app, release(egui::pos2(bar_x, win.max.y)));
}

#[test]
fn system_pointer_scrollbar_reverses_immediately_after_hitting_the_bottom() {
    let ctx = egui::Context::default();
    configure_tessellation(&ctx);
    ctx.set_pixels_per_point(1.0);
    let mut app = blank_app();
    app.pointer.system = true;
    app.replace_bars(data::random_walk::generate(BAR_SEED, 64, BAR_START));
    app.open_source_window(PANE_MACD);
    load_long_indicator_script(&mut app);
    render_events_in(&ctx, &mut app, vec![]);
    let i = app
        .win_mgr
        .wins
        .iter()
        .position(|w| w.kind == WinKind::Source)
        .unwrap();
    let win = app.win_mgr.wins[i].w.rect();
    let body_top = win.min.y + SRC_BODY_TOP;
    let bar_x = win.max.x - 5.0;
    let max_scroll = app.source.editor.max_scroll();
    assert!(max_scroll > 0);
    let grab = egui::pos2(bar_x, body_top + 2.0);
    render_events_in(
        &ctx,
        &mut app,
        vec![
            egui::Event::PointerMoved(grab),
            egui::Event::PointerButton {
                pos: grab,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: egui::Modifiers::NONE,
            },
        ],
    );
    let far = egui::pos2(bar_x, win.max.y + 1400.0);
    render_events_in(&ctx, &mut app, vec![egui::Event::PointerMoved(far)]);
    assert_eq!(app.source.editor.scroll, max_scroll);
    render_events_in(
        &ctx,
        &mut app,
        vec![egui::Event::PointerMoved(far - egui::vec2(0.0, 40.0))],
    );
    assert!(
        app.source.editor.scroll < max_scroll,
        "系统指针在滚动条到底后回移，滚动位置仍在吃死区"
    );
    assert!(app.pointer.pinned.is_none());
}

#[test]
fn pressing_the_scrollbar_never_drags_the_window_edge_instead() {
    let ctx = egui::Context::default();
    configure_tessellation(&ctx);
    ctx.set_pixels_per_point(1.0);
    let mut app = blank_app();
    app.replace_bars(data::random_walk::generate(BAR_SEED, 64, BAR_START));
    app.open_source_window(PANE_MACD);
    load_long_indicator_script(&mut app);
    render_events_in(&ctx, &mut app, vec![]);
    let i = app
        .win_mgr
        .wins
        .iter()
        .position(|w| w.kind == WinKind::Source)
        .expect("源码编辑窗没建出来");
    let before = app.win_mgr.wins[i].w.rect();
    let body_top = before.min.y + SRC_BODY_TOP;

    let bar_x = before.max.x - 5.0;
    assert!(
        !edge_at(before, egui::pos2(bar_x, body_top + 2.0), EDGE_BAND).any(),
        "滚动条区域不该落进 1px 缩放边框线"
    );
    let press = |pos: egui::Pos2| {
        vec![
            egui::Event::PointerMoved(pos),
            egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: egui::Modifiers::NONE,
            },
        ]
    };
    let drag = |pos: egui::Pos2| vec![egui::Event::PointerMoved(pos)];
    let release = |pos: egui::Pos2| {
        vec![egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: egui::Modifiers::NONE,
        }]
    };
    let max_scroll = app.source.editor.max_scroll();
    assert!(max_scroll > 0, "这份源码一屏就装下了，测不了滚动");

    let grab = egui::pos2(bar_x, body_top + 2.0);
    render_events_in(&ctx, &mut app, press(grab));
    render_events_in(
        &ctx,
        &mut app,
        drag(egui::pos2(before.max.x + 400.0, before.max.y + 400.0)),
    );
    let after = app.win_mgr.wins[i].w.rect();
    assert_eq!(after, before, "拖滚动条把窗口的边拖走了（尺寸 / 位置变了）");
    assert!(
        !app.win_mgr.wins[i].w.resizing(),
        "拖滚动条却开了一次缩放会话"
    );
    assert!(app.source.editor.scroll > 0, "拖着滚动条往下走却没滚动");
    render_events_in(&ctx, &mut app, release(grab));
}

#[test]
fn dragging_the_scrollbar_pins_the_pointer_horizontally_too() {
    let ctx = egui::Context::default();
    configure_tessellation(&ctx);
    ctx.set_pixels_per_point(1.0);
    let mut app = blank_app();
    app.replace_bars(data::random_walk::generate(BAR_SEED, 64, BAR_START));
    app.open_source_window(PANE_MACD);
    load_long_indicator_script(&mut app);
    render_events_in(&ctx, &mut app, vec![]);
    let i = app
        .win_mgr
        .wins
        .iter()
        .position(|w| w.kind == WinKind::Source)
        .expect("源码编辑窗没建出来");
    let win = app.win_mgr.wins[i].w.rect();
    let body_top = win.min.y + SRC_BODY_TOP;
    let bar_x = win.max.x - 5.0;
    let press = |pos: egui::Pos2| {
        vec![
            egui::Event::PointerMoved(pos),
            egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: egui::Modifiers::NONE,
            },
        ]
    };
    let drag = |pos: egui::Pos2| vec![egui::Event::PointerMoved(pos)];
    let release = |pos: egui::Pos2| {
        vec![egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: egui::Modifiers::NONE,
        }]
    };
    assert!(
        app.source.editor.max_scroll() > 40,
        "这份源码一屏就装下了，测不了滚动"
    );
    let grab = egui::pos2(bar_x, body_top + 2.0);
    render_events_in(&ctx, &mut app, press(grab));
    render_events_in(&ctx, &mut app, drag(grab + egui::vec2(0.0, 30.0)));
    let pin = app.pointer.pinned.expect("拖滚动条时该把指针钉在把手上");
    assert_eq!(pin.x, bar_x, "指针横向跟着鼠标漂了");

    let scroll_before = app.source.editor.scroll;
    render_events_in(
        &ctx,
        &mut app,
        drag(egui::pos2(win.min.x + 20.0, grab.y + 90.0)),
    );
    let pin2 = app.pointer.pinned.expect("拖滚动条时该把指针钉在把手上");
    assert_eq!(pin2.x, pin.x, "鼠标横向一甩，指针横向跟着跑了");
    assert!(pin2.y > pin.y, "纵向没跟着把手走");
    assert!(app.source.editor.scroll > scroll_before, "往下拖了却没滚动");
    render_events_in(&ctx, &mut app, release(grab));
}

#[test]
fn no_frame_leaves_the_screen_without_a_pointer() {
    let ctx = egui::Context::default();
    configure_tessellation(&ctx);
    ctx.set_pixels_per_point(1.0);
    set_theme(Theme::CLASSIC);
    let mut app = blank_app();
    app.replace_bars(data::random_walk::generate(BAR_SEED, 64, BAR_START));
    let _ = render_pinned_events_in(&ctx, &mut app, vec![]);
    let i = app
        .win_mgr
        .wins
        .iter()
        .position(|w| w.kind == WinKind::Chart)
        .expect("图表窗没建出来");
    let work = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(WINDOW_W, WINDOW_H));
    let work_area = split_areas(work)[2];
    app.win_mgr.wins[i].w.size = egui::vec2(cells(40), cells(24));
    app.win_mgr.wins[i].w.pos = work_area.min + egui::vec2(200.0, 200.0);
    let _ = render_pinned_events_in(&ctx, &mut app, vec![]);
    let before = app.win_mgr.wins[i].w.rect();
    let handle = free_band(before, 0, WIN_TITLE_ROWS).center();
    let _ = render_pinned_events_in(&ctx, &mut app, vec![egui::Event::PointerMoved(handle)]);

    let (dots, icon, image) = render_pinned_events_in(
        &ctx,
        &mut app,
        vec![
            egui::Event::PointerMoved(handle),
            egui::Event::PointerButton {
                pos: handle,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: egui::Modifiers::NONE,
            },
        ],
    );
    assert_eq!(icon, egui::CursorIcon::None, "按下那一帧系统箭头没有隐藏");
    assert!(image.is_none(), "按下那一帧还挂着系统自定义指针位图");
    let kind = app.cursor_kind(&ctx);
    assert!(
        cursor_painted_at(&mut app, &dots, handle, kind),
        "按下那一帧自己画的那一枚还没画上去（下一帧再撤系统那只就晚了）"
    );

    let (dots, icon, image) =
        render_pinned_events_in(&ctx, &mut app, vec![egui::Event::PointerMoved(handle)]);
    assert_eq!(icon, egui::CursorIcon::None, "第二帧系统箭头又冒出来了");
    assert!(image.is_none(), "第二帧又挂上了系统自定义光标位图");
    let kind = app.cursor_kind(&ctx);
    assert!(
        cursor_painted_at(&mut app, &dots, handle, kind),
        "撤系统指针的那一帧自己画的那一枚必须在屏幕上"
    );
}

#[test]
fn software_pointer_draws_every_frame_and_hides_the_system_one() {
    let ctx = egui::Context::default();
    configure_tessellation(&ctx);
    ctx.set_pixels_per_point(1.0);
    set_theme(Theme::CLASSIC);
    let mut app = blank_app();
    app.replace_bars(data::random_walk::generate(BAR_SEED, 64, BAR_START));
    let _ = render_pinned_events_in(&ctx, &mut app, vec![]);
    let i = app
        .win_mgr
        .wins
        .iter()
        .position(|w| w.kind == WinKind::Chart)
        .expect("图表窗没建出来");
    let work = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(WINDOW_W, WINDOW_H));
    let work_area = split_areas(work)[2];
    app.win_mgr.wins[i].w.size = egui::vec2(cells(40), cells(24));
    app.win_mgr.wins[i].w.pos = work_area.min + egui::vec2(200.0, 200.0);
    let _ = render_pinned_events_in(&ctx, &mut app, vec![]);
    let before = app.win_mgr.wins[i].w.rect();
    let handle = free_band(before, 0, WIN_TITLE_ROWS).center();
    let neutral = work_area.min + egui::vec2(400.0, 400.0);
    for (label, events, want) in [
        ("悬停", vec![egui::Event::PointerMoved(neutral)], neutral),
        (
            "按下",
            vec![
                egui::Event::PointerMoved(handle),
                egui::Event::PointerButton {
                    pos: handle,
                    button: egui::PointerButton::Primary,
                    pressed: true,
                    modifiers: egui::Modifiers::NONE,
                },
            ],
            handle,
        ),
        ("拖动中", vec![egui::Event::PointerMoved(handle)], handle),
    ] {
        let (dots, icon, image) = render_pinned_events_in(&ctx, &mut app, events);
        assert_eq!(icon, egui::CursorIcon::None, "{label}：没把系统指针藏起来");
        assert!(image.is_none(), "{label}：还挂着系统那份自定义光标位图");
        let kind = app.cursor_kind(&ctx);
        assert!(
            cursor_painted_at(&mut app, &dots, want, kind),
            "{label}：自己画的那一枚没压在指针那一像素上"
        );
    }
}

#[test]
fn dragging_glues_our_own_pointer_to_the_grab_point() {
    let ctx = egui::Context::default();
    configure_tessellation(&ctx);
    ctx.set_pixels_per_point(1.0);
    set_theme(Theme::CLASSIC);
    let mut app = blank_app();
    app.replace_bars(data::random_walk::generate(BAR_SEED, 64, BAR_START));
    let press = |pos: egui::Pos2| {
        vec![
            egui::Event::PointerMoved(pos),
            egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: egui::Modifiers::NONE,
            },
        ]
    };
    let drag = |pos: egui::Pos2| vec![egui::Event::PointerMoved(pos)];
    let release = |pos: egui::Pos2| {
        vec![egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: egui::Modifiers::NONE,
        }]
    };

    let _ = render_pinned_events_in(&ctx, &mut app, vec![]);
    let i = app
        .win_mgr
        .wins
        .iter()
        .position(|w| w.kind == WinKind::Chart)
        .expect("图表窗没建出来");
    let work = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(WINDOW_W, WINDOW_H));
    let work = split_areas(work)[2];
    app.win_mgr.wins[i].w.size = egui::vec2(cells(40), cells(24));
    app.win_mgr.wins[i].w.pos = work.min + egui::vec2(200.0, 200.0);
    let _ = render_pinned_events_in(&ctx, &mut app, vec![]);
    let before = app.win_mgr.wins[i].w.rect();
    let handle = free_band(before, 0, WIN_TITLE_ROWS).center();
    let walk = egui::vec2(37.0, 23.0);

    let moved_by = (walk * WINDOW_DRAG_SPEED).round();

    let _ = render_pinned_events_in(&ctx, &mut app, vec![egui::Event::PointerMoved(handle)]);

    let (_, icon0, image0) = render_pinned_events_in(&ctx, &mut app, press(handle));
    assert_eq!(app.pointer.pinned, Some(handle), "按下那一帧没把指针钉住");
    assert!(image0.is_none(), "按下那一帧还挂着系统自定义指针位图");
    assert_eq!(icon0, egui::CursorIcon::None, "按下那一帧系统箭头没有隐藏");
    let (dots, icon, image) = render_pinned_events_in(&ctx, &mut app, drag(handle + walk));
    let moved = app.win_mgr.wins[i].w.rect();
    assert_eq!(moved.min, before.min + moved_by, "拖标题栏没跟手");

    assert_eq!(icon, egui::CursorIcon::None, "拖动中没把系统箭头藏起来");
    assert!(image.is_none(), "拖动中还挂着系统那份自定义光标位图");
    let pin = app
        .pinned_pointer(&ctx)
        .expect("拖动中该有一枚钉在抓点上的指针");
    assert_eq!(
        pin,
        moved.min + (handle - before.min),
        "指针没钉在按下那一刻抓住的那一点上"
    );

    assert_eq!(
        app.cursor_kind(&ctx),
        CursorKind::Move,
        "拖标题栏不是「移动窗口」状态"
    );
    assert!(
        cursor_painted_at(&mut app, &dots, pin, CursorKind::Move),
        "指针没有以左上角为热点画在抓点上"
    );

    let _ = render_pinned_events_in(&ctx, &mut app, release(handle + walk));
    let neutral = work.min + egui::vec2(400.0, 400.0);
    let (after, icon_after, image_after) = render_pinned_events_in(&ctx, &mut app, drag(neutral));
    assert!(app.pinned_pointer(&ctx).is_none(), "松手之后还钉着指针");
    assert!(
        !cursor_painted_at(&mut app, &after, pin, CursorKind::Move),
        "松手之后指针还画在屏幕上"
    );
    assert_eq!(
        app.cursor_kind(&ctx),
        CursorKind::Normal,
        "空地上不是「正常」状态"
    );
    assert!(image_after.is_none(), "松手之后又挂上了系统自定义指针位图");
    assert_eq!(
        icon_after,
        egui::CursorIcon::None,
        "松手之后系统箭头又冒出来了"
    );
    assert!(
        cursor_painted_at(&mut app, &after, neutral, CursorKind::Normal),
        "松手之后软件指针没有画在新的位置上"
    );
}

#[test]
fn cursor_colors_ignore_the_theme() {
    let mut app = blank_app();
    let shoot = |app: &mut StockApp| -> Vec<(CursorKind, Vec<u8>)> {
        CursorKind::ALL
            .iter()
            .map(|&kind| (kind, app.cursors.rgba(kind).to_vec()))
            .collect()
    };
    set_theme(Theme::CLASSIC);
    let light = shoot(&mut app);
    set_theme(Theme::DARK);
    let dark = shoot(&mut app);
    set_theme(Theme::CLASSIC);
    for ((kind, a), (_, b)) in light.iter().zip(dark.iter()) {
        assert_eq!(a, b, "{} 那张图跟着皮肤改了颜色", kind.label());
    }
}

#[test]
fn every_cursor_state_has_a_non_blank_image_and_top_left_hotspot() {
    let mut app = blank_app();
    assert!(!app.cursors.is_blank(), "assets/cursors 里一张图都没读到");
    for kind in CursorKind::ALL {
        let rgba = app.cursors.rgba(kind);

        assert!(
            rgba.chunks_exact(4).any(|p| p[3] != 0),
            "{} 那张图是空的",
            kind.label()
        );

        let hot = app.cursors.hotspot(kind);
        assert_eq!(hot, [0, 0], "{} 的热点不在图片左上角", kind.label());
    }
}

#[test]
fn a_missing_cursor_asset_leaves_the_system_pointer_alone() {
    let ctx = egui::Context::default();
    configure_tessellation(&ctx);
    ctx.set_pixels_per_point(1.0);
    let mut app = blank_app();
    app.cursors = Cursors::blank();
    let (_, icon, image) = render_pinned_events_in(&ctx, &mut app, vec![]);
    assert!(image.is_none(), "读不到图还去换系统指针");
    assert_ne!(icon, egui::CursorIcon::None, "读不到图却把系统指针藏起来了");
}

#[test]
fn the_pointer_state_follows_what_the_mouse_is_doing() {
    let ctx = egui::Context::default();
    configure_tessellation(&ctx);
    ctx.set_pixels_per_point(1.0);
    set_theme(Theme::CLASSIC);
    let mut app = blank_app();
    app.replace_bars(data::random_walk::generate(BAR_SEED, 64, BAR_START));
    let layout = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(WINDOW_W, WINDOW_H));

    fn step(
        ctx: &egui::Context,
        app: &mut StockApp,
        layout: egui::Rect,
        t: &mut f64,
        events: Vec<egui::Event>,
    ) {
        let mut raw = egui::RawInput::default();
        raw.screen_rect = Some(layout);
        raw.time = Some(*t);
        raw.events = events;
        let mut out = ctx.run_ui(raw, |ui| {
            app.draw_frame(ui);
        });
        absorb_textures(&mut out.textures_delta);
        let _ = ctx.tessellate(std::mem::take(&mut out.shapes), 1.0);
        *t += 1.0 / 60.0;
    }
    let press = |pos: egui::Pos2, button: egui::PointerButton| {
        vec![
            egui::Event::PointerMoved(pos),
            egui::Event::PointerButton {
                pos,
                button,
                pressed: true,
                modifiers: egui::Modifiers::NONE,
            },
        ]
    };
    let up = |pos: egui::Pos2, button: egui::PointerButton| {
        vec![egui::Event::PointerButton {
            pos,
            button,
            pressed: false,
            modifiers: egui::Modifiers::NONE,
        }]
    };
    let mut t = 0.0f64;
    step(&ctx, &mut app, layout, &mut t, vec![]);
    let i = app
        .win_mgr
        .wins
        .iter()
        .position(|w| w.kind == WinKind::Chart)
        .expect("图表窗没建出来");

    app.win_mgr.wins[i].w.size = egui::vec2(cells(40), cells(24));
    app.win_mgr.wins[i].w.pos = split_areas(layout)[2].min + egui::vec2(200.0, 200.0);
    step(&ctx, &mut app, layout, &mut t, vec![]);
    let win = app.win_mgr.wins[i].w.rect();

    macro_rules! kind_at {
        ($p:expr) => {{
            step(
                &ctx,
                &mut app,
                layout,
                &mut t,
                vec![egui::Event::PointerMoved($p)],
            );
            app.cursor_kind(&ctx)
        }};
    }

    let neutral = egui::pos2(win.min.x + 20.0, win.center().y);
    assert_eq!(kind_at!(neutral), CursorKind::Normal, "空地上");

    let handle = free_band(win, 0, WIN_TITLE_ROWS).center();
    assert_eq!(kind_at!(handle), CursorKind::Move, "标题栏上");

    assert_eq!(
        kind_at!(egui::pos2(win.max.x - 1.0, win.center().y)),
        CursorKind::ResizeH,
        "右边框上"
    );
    assert_eq!(
        kind_at!(egui::pos2(win.center().x, win.max.y - 1.0)),
        CursorKind::ResizeV,
        "下边框上"
    );
    assert_eq!(
        kind_at!(egui::pos2(win.max.x - 1.0, win.max.y - 1.0)),
        CursorKind::ResizeNwSe,
        "右下角"
    );
    assert_eq!(
        kind_at!(egui::pos2(win.min.x, win.max.y - 1.0)),
        CursorKind::ResizeNeSw,
        "左下角"
    );

    let idle = egui::pos2(layout.center().x, layout.max.y - 10.0);
    step(
        &ctx,
        &mut app,
        layout,
        &mut t,
        press(idle, egui::PointerButton::Primary),
    );
    assert_eq!(app.cursor_kind(&ctx), CursorKind::Left, "左键按下");
    step(
        &ctx,
        &mut app,
        layout,
        &mut t,
        press(idle, egui::PointerButton::Secondary),
    );
    assert_eq!(app.cursor_kind(&ctx), CursorKind::Right, "右键按下");
    step(
        &ctx,
        &mut app,
        layout,
        &mut t,
        up(idle, egui::PointerButton::Secondary),
    );
    step(
        &ctx,
        &mut app,
        layout,
        &mut t,
        up(idle, egui::PointerButton::Primary),
    );
    assert_eq!(app.cursor_kind(&ctx), CursorKind::Normal, "松手之后");

    step(
        &ctx,
        &mut app,
        layout,
        &mut t,
        vec![egui::Event::MouseWheel {
            unit: egui::MouseWheelUnit::Line,
            delta: egui::vec2(0.0, 3.0),
            phase: egui::TouchPhase::Move,
            modifiers: egui::Modifiers::NONE,
        }],
    );
    assert_eq!(app.cursor_kind(&ctx), CursorKind::Wheel, "滚轮滚动");
    t += WHEEL_HOLD_S + 0.1;
    step(&ctx, &mut app, layout, &mut t, vec![]);
    assert_eq!(
        app.cursor_kind(&ctx),
        CursorKind::Normal,
        "滚完之后该自己灭掉"
    );

    app.open_source_window(PANE_MACD);
    step(&ctx, &mut app, layout, &mut t, vec![]);
    let src = app
        .win_mgr
        .wins
        .iter()
        .position(|w| w.kind == WinKind::Source)
        .expect("源码编辑窗没建出来");
    let swin = app.win_mgr.wins[src].w.rect();
    let bar = egui::pos2(swin.max.x - 5.0, swin.min.y + SRC_BODY_TOP + 8.0);
    assert_eq!(kind_at!(bar), CursorKind::Slider, "滚动条上");

    let body = egui::pos2(
        swin.min.x + 150.0,
        swin.min.y + SRC_BODY_TOP + SRC_ROW_PITCH * 2.0 + LINE_W,
    );
    assert_eq!(kind_at!(body), CursorKind::Text, "编辑器正文区上");

    step(
        &ctx,
        &mut app,
        layout,
        &mut t,
        press(body, egui::PointerButton::Primary),
    );
    assert_eq!(app.cursor_kind(&ctx), CursorKind::Text, "正文区左键按下");
    step(
        &ctx,
        &mut app,
        layout,
        &mut t,
        up(body, egui::PointerButton::Primary),
    );
    assert_eq!(app.cursor_kind(&ctx), CursorKind::Text, "正文区左键松开");
    step(
        &ctx,
        &mut app,
        layout,
        &mut t,
        press(body, egui::PointerButton::Secondary),
    );
    assert_eq!(
        app.cursor_kind(&ctx),
        CursorKind::Right,
        "正文区右键仍显示右键指针"
    );
    step(
        &ctx,
        &mut app,
        layout,
        &mut t,
        up(body, egui::PointerButton::Secondary),
    );
    step(
        &ctx,
        &mut app,
        layout,
        &mut t,
        vec![egui::Event::MouseWheel {
            unit: egui::MouseWheelUnit::Line,
            delta: egui::vec2(0.0, 1.0),
            phase: egui::TouchPhase::Move,
            modifiers: egui::Modifiers::NONE,
        }],
    );
    assert_eq!(
        app.cursor_kind(&ctx),
        CursorKind::Wheel,
        "正文区滚轮仍显示滚轮指针"
    );
}

#[test]
fn source_window_minimizes_like_any_other_child_window() {
    let ctx = egui::Context::default();
    configure_tessellation(&ctx);
    ctx.set_pixels_per_point(1.0);
    let mut app = blank_app();
    app.replace_bars(data::random_walk::generate(BAR_SEED, 64, BAR_START));
    app.open_source_window(PANE_MACD);
    render_events_in(&ctx, &mut app, vec![]);
    let i = app
        .win_mgr
        .wins
        .iter()
        .position(|w| w.kind == WinKind::Source)
        .expect("源码编辑窗没建出来");
    let work = app.work_rect;
    let full = app.win_mgr.wins[i].w.rect();
    assert!(work.contains(full.center()), "源码编辑窗没摆在窗口区里");
    assert!(app.win_mgr.source_open(), "源码编辑窗没开着");

    let title = app.source_title();
    let mini_w = WinMgr::mini_width(&mut app, &title);
    app.win_mgr.minimize(i, mini_w);
    let strip = app.win_mgr.rect_of(i);
    assert_eq!(
        strip.min,
        app.win_mgr.taskbar_items_rect().min,
        "最小化那条没进任务栏"
    );
    assert_eq!(strip.width(), mini_w, "最小化那条的宽不对");
    assert_eq!(
        strip.height(),
        WIN_TITLE_H,
        "最小化那条只有一条标题栏那么高"
    );

    app.win_mgr.restore(i);
    assert_eq!(app.win_mgr.wins[i].w.rect(), full, "还原没回到原位原尺寸");
    assert_eq!(app.source_title(), title, "源码编辑窗的标题变了");
}

#[test]
fn taskbar_holds_minimized_windows_and_title_clicks_restore_them() {
    let ctx = egui::Context::default();
    configure_tessellation(&ctx);
    ctx.set_pixels_per_point(1.0);
    let mut app = blank_app();
    app.text_engine = HardText::new(&ctx);
    app.replace_bars(data::random_walk::generate(BAR_SEED, 64, BAR_START));
    render_events_in(&ctx, &mut app, vec![]);
    let layout = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(WINDOW_W, WINDOW_H));
    let areas = split_areas(layout);
    assert_eq!(areas[3].width(), areas[4].width());
    assert_eq!(areas[3].height(), WIN_TITLE_H);
    assert!(areas[2].max.y <= areas[3].min.y + LINE_W);

    let chart = app
        .win_mgr
        .wins
        .iter()
        .position(|window| window.kind == WinKind::Chart)
        .expect("chart window");
    let full = app.win_mgr.wins[chart].rect();
    let title = app.win_mgr.wins[chart].title.clone();
    let mini_w = WinMgr::mini_width(&mut app, &title);
    app.win_mgr.minimize(chart, mini_w);
    let mini = app.win_mgr.rect_of(chart);
    assert_eq!(mini.min.y, areas[3].min.y);
    assert!(mini.min.x > areas[3].min.x);
    assert!(areas[3].contains(mini.center()));
    assert!(!areas[2].contains(mini.center()));

    let restore = egui::pos2(mini.min.x + 12.0, mini.center().y);
    render_events_in(&ctx, &mut app, vec![egui::Event::PointerMoved(restore)]);
    render_events_in(
        &ctx,
        &mut app,
        vec![
            egui::Event::PointerMoved(restore),
            egui::Event::PointerButton {
                pos: restore,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: egui::Modifiers::NONE,
            },
        ],
    );
    render_events_in(
        &ctx,
        &mut app,
        vec![egui::Event::PointerButton {
            pos: restore,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: egui::Modifiers::NONE,
        }],
    );
    assert!(!app.win_mgr.wins[chart].minimized);
    assert_eq!(app.win_mgr.wins[chart].rect(), full);
}

#[test]
fn taskbar_restores_only_the_clicked_minimized_window() {
    let ctx = egui::Context::default();
    configure_tessellation(&ctx);
    ctx.set_pixels_per_point(1.0);
    let mut app = blank_app();
    app.text_engine = HardText::new(&ctx);
    app.replace_bars(data::random_walk::generate(BAR_SEED, 64, BAR_START));
    app.open_source_window(PANE_MACD);
    render_events_in(&ctx, &mut app, vec![]);
    let chart = app
        .win_mgr
        .wins
        .iter()
        .position(|window| window.kind == WinKind::Chart)
        .expect("chart window");
    let source = app
        .win_mgr
        .wins
        .iter()
        .position(|window| window.kind == WinKind::Source)
        .expect("source window");
    let chart_id = app.win_mgr.wins[chart].w.id;
    let source_id = app.win_mgr.wins[source].w.id;
    for i in [chart, source] {
        let title = app.win_mgr.wins[i].title.clone();
        let width = WinMgr::mini_width(&mut app, &title);
        app.win_mgr.minimize(i, width);
    }
    render_events_in(&ctx, &mut app, vec![]);
    let mini = app.win_mgr.rect_of(chart);
    let click = egui::pos2(mini.min.x + 12.0, mini.center().y);
    render_events_in(&ctx, &mut app, vec![egui::Event::PointerMoved(click)]);
    render_events_in(
        &ctx,
        &mut app,
        vec![
            egui::Event::PointerMoved(click),
            egui::Event::PointerButton {
                pos: click,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: egui::Modifiers::NONE,
            },
        ],
    );
    render_events_in(
        &ctx,
        &mut app,
        vec![egui::Event::PointerButton {
            pos: click,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: egui::Modifiers::NONE,
        }],
    );
    let chart = app
        .win_mgr
        .wins
        .iter()
        .position(|window| window.w.id == chart_id)
        .unwrap();
    let source = app
        .win_mgr
        .wins
        .iter()
        .position(|window| window.w.id == source_id)
        .unwrap();
    assert!(!app.win_mgr.wins[chart].minimized);
    assert!(app.win_mgr.wins[source].minimized);
}

#[test]
fn taskbar_pages_overflowing_minimized_windows() {
    let ctx = egui::Context::default();
    configure_tessellation(&ctx);
    ctx.set_pixels_per_point(1.0);
    let mut app = blank_app();
    app.text_engine = HardText::new(&ctx);
    app.replace_bars(data::random_walk::generate(BAR_SEED, 64, BAR_START));
    for n in 0..18 {
        let title = format!("任务{n}");
        let width = WinMgr::mini_width(&mut app, &title);
        let i = app.win_mgr.add(
            WinKind::Chart,
            &title,
            0,
            egui::vec2(cells(40), cells(30)),
            egui::vec2(cells(40), cells(30)),
        );
        app.win_mgr.minimize(i, width);
    }
    render_events_in(&ctx, &mut app, vec![]);
    let visible = |app: &StockApp| {
        (0..app.win_mgr.wins.len())
            .filter(|&i| app.win_mgr.wins[i].minimized)
            .filter(|&i| app.win_mgr.rect_of(i).is_positive())
            .collect::<Vec<_>>()
    };
    let first_page = visible(&app);
    assert!(first_page.len() < 18);
    let layout = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(WINDOW_W, WINDOW_H));
    let taskbar = split_areas(layout)[3];
    let next = egui::pos2(taskbar.max.x - SYS_BTN_W / 2.0, taskbar.center().y);
    render_events_in(&ctx, &mut app, vec![egui::Event::PointerMoved(next)]);
    render_events_in(
        &ctx,
        &mut app,
        vec![
            egui::Event::PointerMoved(next),
            egui::Event::PointerButton {
                pos: next,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: egui::Modifiers::NONE,
            },
        ],
    );
    render_events_in(
        &ctx,
        &mut app,
        vec![egui::Event::PointerButton {
            pos: next,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: egui::Modifiers::NONE,
        }],
    );
    let second_page = visible(&app);
    assert!(!second_page.is_empty());
    assert_ne!(first_page, second_page, "翻页后任务栏项目没有变化");
}

#[test]
fn a_press_inside_the_front_window_never_reaches_the_one_behind() {
    let ctx = egui::Context::default();
    configure_tessellation(&ctx);
    ctx.set_pixels_per_point(1.0);
    let mut app = blank_app();
    app.text_engine = HardText::new(&ctx);
    app.replace_bars(data::random_walk::generate(BAR_SEED, BAR_COUNT, BAR_START));
    if app.panel_buttons.is_empty() {
        app.build_panel_buttons();
    }

    let frame = |app: &mut StockApp, events: Vec<egui::Event>| {
        let _ = render_events_in(&ctx, app, events);
    };
    frame(&mut app, vec![]);
    let work = app.work_rect;

    let back = app
        .win_mgr
        .wins
        .iter()
        .position(|w| w.kind == WinKind::Chart)
        .expect("图表窗没建出来");
    app.win_mgr.wins[back].w.pos = work.min;
    app.win_mgr.wins[back].w.size = egui::vec2(cells(40), cells(30));
    app.add_chart_window();
    frame(&mut app, vec![]);
    let front = app.win_mgr.wins.len() - 1;
    let back_rect = app.win_mgr.wins[back].w.rect();
    app.win_mgr.wins[front].w.pos = back_rect.min + egui::vec2(100.0, 0.0);
    app.win_mgr.wins[front].w.size = egui::vec2(cells(40), cells(30));

    frame(&mut app, vec![]);

    let p = egui::pos2(back_rect.max.x - LINE_W, back_rect.center().y);
    assert!(
        app.win_mgr.wins[front].w.rect().contains(p),
        "取样点没落在前窗里"
    );
    assert_eq!(
        edge_at(back_rect, p, EDGE_BAND),
        WinEdge::Right,
        "取样点没落在后窗右边那条命中带上"
    );
    let before_back = back_rect;
    let before_front = app.win_mgr.wins[front].w.rect();
    frame(
        &mut app,
        vec![
            egui::Event::PointerMoved(p),
            egui::Event::PointerButton {
                pos: p,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: egui::Modifiers::NONE,
            },
        ],
    );
    frame(
        &mut app,
        vec![egui::Event::PointerMoved(p + egui::vec2(30.0, 0.0))],
    );
    assert_eq!(
        app.win_mgr.live_index(),
        Some(front),
        "点在前面的窗口里，却把后面那块抬到了最上层（穿透了）"
    );
    assert_eq!(
        app.win_mgr.wins[back].w.rect(),
        before_back,
        "后面那块窗口被拽着改尺寸了（穿透了）"
    );
    assert!(
        !app.win_mgr.wins[back].w.resizing(),
        "后面那块窗口进了缩放会话"
    );
    assert_eq!(
        app.win_mgr.wins[front].w.rect(),
        before_front,
        "前窗不该跟着动（这一按不是它的缩放）"
    );
    frame(
        &mut app,
        vec![egui::Event::PointerButton {
            pos: p + egui::vec2(30.0, 0.0),
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: egui::Modifiers::NONE,
        }],
    );
}

#[test]
fn the_resize_cursor_only_shows_where_the_grab_works() {
    let ctx = egui::Context::default();
    configure_tessellation(&ctx);
    ctx.set_pixels_per_point(1.0);
    let mut app = blank_app();
    app.text_engine = HardText::new(&ctx);
    app.replace_bars(data::random_walk::generate(BAR_SEED, BAR_COUNT, BAR_START));
    if app.panel_buttons.is_empty() {
        app.build_panel_buttons();
    }
    let work = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(cells(120), cells(70)));
    app.work_rect = work;
    win_frame(&mut app, &ctx, work, vec![]);
    let i = app.win_mgr.live_index().expect("没有窗口");
    app.win_mgr.wins[i].w.size = egui::vec2(cells(40), cells(24));
    app.win_mgr.wins[i].w.pos = work.min + egui::vec2(200.0, 200.0);
    win_frame(&mut app, &ctx, work, vec![]);
    let rect = app.win_mgr.wins[i].w.rect();

    let arrow = |app: &mut StockApp, dx: f32| {
        win_frame_cursor(
            app,
            &ctx,
            work,
            vec![egui::Event::PointerMoved(egui::pos2(
                rect.max.x + dx,
                rect.center().y,
            ))],
        )
    };
    assert_eq!(
        arrow(&mut app, -LINE_W),
        egui::CursorIcon::ResizeHorizontal,
        "边框线上没给缩放箭头"
    );
    for dx in [0.0, 1.0, 2.0, -2.0, -3.0] {
        assert_ne!(
            arrow(&mut app, dx),
            egui::CursorIcon::ResizeHorizontal,
            "边框线外（{dx}px）还给了缩放箭头"
        );
    }

    let grab = egui::pos2(rect.max.x - LINE_W, rect.center().y);
    win_frame(
        &mut app,
        &ctx,
        work,
        vec![
            egui::Event::PointerMoved(grab),
            egui::Event::PointerButton {
                pos: grab,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: egui::Modifiers::NONE,
            },
            egui::Event::PointerMoved(grab + egui::vec2(20.0, 0.0)),
        ],
    );
    assert_eq!(
        app.win_mgr.wins[i].w.rect().width(),
        rect.width() + 20.0,
        "箭头在带子最外沿出现了，按下去却拽不动"
    );
}

#[test]
fn pane_menu_survives_a_second_chart_window() {
    let ctx = egui::Context::default();
    configure_tessellation(&ctx);
    ctx.set_pixels_per_point(1.0);
    let mut app = blank_app();
    app.text_engine = HardText::new(&ctx);
    app.replace_bars(data::random_walk::generate(BAR_SEED, BAR_COUNT, BAR_START));
    if app.panel_buttons.is_empty() {
        app.build_panel_buttons();
    }
    let work = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(cells(120), cells(70)));
    app.work_rect = work;
    let frame = |app: &mut StockApp, events: Vec<egui::Event>| win_frame(app, &ctx, work, events);
    frame(&mut app, vec![]);

    app.add_chart_window();
    frame(&mut app, vec![]);
    let front = app.win_mgr.wins.len() - 1;
    let pane = app.win_mgr.wins[front].pane;
    let body = ManagedWin::body(app.win_mgr.rect_of(front));
    let (title, _, _, _) = chart_rows(body, app.panes[pane].indicator.layout());
    let widths: Vec<f32> = StockApp::PANE_TABS
        .iter()
        .map(|t| tab_width(&mut app, t))
        .collect();
    let p = StockApp::pane_button_rects(title, &widths)[0].center();
    frame(
        &mut app,
        vec![
            egui::Event::PointerMoved(p),
            egui::Event::PointerButton {
                pos: p,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: egui::Modifiers::NONE,
            },
        ],
    );
    frame(
        &mut app,
        vec![egui::Event::PointerButton {
            pos: p,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: egui::Modifiers::NONE,
        }],
    );
    assert_eq!(app.pane_menu, Some(pane), "点「切换」没开出指标面板");

    for k in 0..3 {
        frame(&mut app, vec![]);
        assert_eq!(
            app.pane_menu,
            Some(pane),
            "第 {k} 帧之后指标面板自己收回了（被另一块图表窗的那条栏抹掉了）"
        );
    }

    let other = (0..app.win_mgr.wins.len())
        .find(|&i| i != front && app.win_mgr.wins[i].kind == WinKind::Chart)
        .expect("没有第二块图表窗");
    let other_pane = app.win_mgr.wins[other].pane;
    let other_size = app.win_mgr.wins[other].w.size;
    app.win_mgr.wins[other].w.pos =
        egui::pos2(app.work_rect.max.x - other_size.x, app.work_rect.min.y);
    frame(&mut app, vec![]);
    let body = ManagedWin::body(app.win_mgr.rect_of(other));
    let (title, _, _, _) = chart_rows(body, app.panes[other_pane].indicator.layout());
    let p2 = StockApp::pane_button_rects(title, &widths)[0].center();
    frame(&mut app, vec![egui::Event::PointerMoved(p2)]);
    frame(
        &mut app,
        vec![
            egui::Event::PointerMoved(p2),
            egui::Event::PointerButton {
                pos: p2,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: egui::Modifiers::NONE,
            },
        ],
    );
    frame(
        &mut app,
        vec![egui::Event::PointerButton {
            pos: p2,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: egui::Modifiers::NONE,
        }],
    );
    assert_eq!(
        app.pane_menu,
        Some(other_pane),
        "另一块图表窗上的「切换」没开出自己的面板"
    );
}

#[test]
fn the_wheel_only_reaches_the_active_window() {
    let ctx = egui::Context::default();
    configure_tessellation(&ctx);
    ctx.set_pixels_per_point(1.0);
    let mut app = blank_app();
    app.text_engine = HardText::new(&ctx);
    app.replace_bars(data::random_walk::generate(BAR_SEED, BAR_COUNT, BAR_START));
    if app.panel_buttons.is_empty() {
        app.build_panel_buttons();
    }

    let frame = |app: &mut StockApp, events: Vec<egui::Event>| {
        let _ = render_events_in(&ctx, app, events);
    };
    frame(&mut app, vec![]);
    let work = app.work_rect;

    app.open_source_window(PANE_CANDLE);
    load_long_indicator_script(&mut app);
    frame(&mut app, vec![]);
    let src = app
        .win_mgr
        .wins
        .iter()
        .position(|w| w.kind == WinKind::Source)
        .expect("源码窗没建出来");
    let src_id = app.win_mgr.wins[src].w.id;
    app.win_mgr.wins[src].w.pos = work.min;
    app.win_mgr.wins[src].w.size = work.size();
    app.win_mgr.toggle_pin(src);
    app.add_chart_window();
    frame(&mut app, vec![]);
    let chart = app.win_mgr.wins.len() - 1;
    let chart_pane = app.win_mgr.wins[chart].pane;
    app.win_mgr.wins[chart].w.size = egui::vec2(cells(50), cells(30));
    app.win_mgr.wins[chart].w.pos = work.min + egui::vec2(cells(30), cells(20));
    frame(&mut app, vec![]);
    let press = |pos: egui::Pos2| {
        vec![
            egui::Event::PointerMoved(pos),
            egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: egui::Modifiers::NONE,
            },
        ]
    };
    let release = |pos: egui::Pos2| {
        vec![egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: egui::Modifiers::NONE,
        }]
    };
    let wheel = |pos: egui::Pos2| {
        vec![
            egui::Event::PointerMoved(pos),
            egui::Event::MouseWheel {
                unit: egui::MouseWheelUnit::Line,
                delta: egui::vec2(0.0, -3.0),
                phase: egui::TouchPhase::Move,
                modifiers: egui::Modifiers::default(),
            },
        ]
    };

    let chart_body = app.win_mgr.wins[chart].w.rect().center();
    assert!(
        app.win_mgr.wins[src].w.rect().contains(chart_body),
        "取样点得同时落在代码窗那块面板里才算测到"
    );
    frame(&mut app, press(chart_body));
    let (scale_before, scroll_before) =
        (app.panes[chart_pane].view.scale, app.source.editor.scroll);
    frame(&mut app, release(chart_body));
    assert_eq!(
        app.win_mgr.live_index(),
        Some(chart),
        "点了图表窗，激活的该是它"
    );
    frame(&mut app, wheel(chart_body));
    assert_eq!(
        app.source.editor.scroll, scroll_before,
        "滚轮滚在激活的图表窗上，底下的代码窗却跟着滚了"
    );
    assert!(
        app.panes[chart_pane].view.scale != scale_before,
        "滚轮没归激活的图表窗（K 线该缩放）"
    );

    let body = egui::pos2(
        work.min.x + SRC_GUTTER_W + GRID * 3.0,
        work.min.y + SRC_BODY_TOP + 60.0,
    );
    frame(&mut app, press(body));
    frame(&mut app, release(body));
    assert_eq!(
        app.win_mgr.live_index().map(|i| app.win_mgr.wins[i].w.id),
        Some(src_id),
        "鼠标移到代码窗可见区域后焦点应转到代码窗"
    );
    let scroll_before = app.source.editor.scroll;
    frame(&mut app, wheel(body));
    assert!(
        app.source.editor.scroll > scroll_before,
        "激活的是代码窗，滚轮该把它滚下去"
    );
}

#[test]
fn the_color_panel_can_be_dragged_left_of_its_close_key() {
    let ctx = egui::Context::default();
    configure_tessellation(&ctx);
    ctx.set_pixels_per_point(1.0);
    let mut app = blank_app();
    app.text_engine = HardText::new(&ctx);
    app.replace_bars(data::random_walk::generate(BAR_SEED, BAR_COUNT, BAR_START));

    let frame = |app: &mut StockApp, events: Vec<egui::Event>| {
        let _ = render_events_in(&ctx, app, events);
    };
    frame(&mut app, vec![]);
    app.open_color_panel(ColorTarget::Up);
    frame(&mut app, vec![]);
    let color = color_index(&app);
    let size = app.win_mgr.wins[color].w.size;
    app.win_mgr.wins[color].w.pos = egui::pos2(
        (app.work_rect.center().x - size.x / 2.0).floor(),
        (app.work_rect.center().y - size.y / 2.0).floor(),
    );
    app.win_mgr.wins[color].restore = (app.win_mgr.wins[color].w.pos, size);
    frame(&mut app, vec![]);
    let title = app.win_mgr.wins[color].w.title_band();
    let keys = sys_button_rects_n(title, 3);

    let p = egui::pos2(keys[0].min.x - 6.0, title.center().y);
    assert!(title.contains(p), "取样点没落在标题栏上");
    let before = app.win_mgr.wins[color].w.rect();
    let walk = egui::vec2(-40.0, -20.0);
    frame(
        &mut app,
        vec![
            egui::Event::PointerMoved(p),
            egui::Event::PointerButton {
                pos: p,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: egui::Modifiers::NONE,
            },
        ],
    );
    assert!(
        app.win_mgr.wins[color].w.dragging(),
        "关闭键左边那一块没开拖动会话"
    );
    frame(&mut app, vec![egui::Event::PointerMoved(p + walk)]);

    let moved_by = (walk * WINDOW_DRAG_SPEED).round();
    assert_eq!(
        app.win_mgr.wins[color].w.rect().min,
        before.min + moved_by,
        "关闭键左边那一块拖不动面板"
    );
    frame(
        &mut app,
        vec![egui::Event::PointerButton {
            pos: p + walk,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: egui::Modifiers::NONE,
        }],
    );
}

#[test]
fn the_editor_caret_is_a_thick_steady_vertical_bar() {
    let ctx = egui::Context::default();
    configure_tessellation(&ctx);
    ctx.set_pixels_per_point(1.0);
    set_theme(Theme::CLASSIC);
    let mut app = blank_app();
    app.text_engine = HardText::new(&ctx);
    app.replace_bars(data::random_walk::generate(BAR_SEED, BAR_COUNT, BAR_START));
    app.open_source_window(PANE_MACD);
    let work = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(cells(120), cells(70)));
    app.work_rect = work;
    let frame = |app: &mut StockApp, events: Vec<egui::Event>| win_frame(app, &ctx, work, events);
    frame(&mut app, vec![]);
    let i = app
        .win_mgr
        .wins
        .iter()
        .position(|w| w.kind == WinKind::Source)
        .expect("源码窗没建出来");
    let win = app.win_mgr.wins[i].w.rect();

    let body = egui::pos2(
        win.min.x + SRC_GUTTER_W + GRID * 3.0,
        win.min.y + SRC_BODY_TOP + 20.0,
    );
    frame(
        &mut app,
        vec![
            egui::Event::PointerMoved(body),
            egui::Event::PointerButton {
                pos: body,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: egui::Modifiers::NONE,
            },
        ],
    );
    frame(
        &mut app,
        vec![egui::Event::PointerButton {
            pos: body,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: egui::Modifiers::NONE,
        }],
    );

    let body_top = win.min.y + SRC_BODY_TOP;
    let gutter_line = win.min.x + grid_line(SRC_GUTTER_COLS);
    let code_x = text_left_x(gutter_line, win.min.x);
    let (cl, cc) = app.source.editor.buf.cursor;
    let head: String = app.source.editor.buf.line(cl).chars().take(cc).collect();
    let caret = egui::Rect::from_min_size(
        egui::pos2(
            code_x + text_advance(&head),
            body_top + cl as f32 * SRC_ROW_PITCH + LINE_W + (SRC_LINE_H - CARET_H) / 2.0,
        ),
        egui::vec2(CARET_W, CARET_H),
    );
    assert!(CARET_W >= 2.0, "光标没有加粗到 2px");
    assert!(CARET_H < SRC_LINE_H, "光标应该比整行矮一点，上下留白");

    for frame_no in 0..8 {
        let grid = frame(&mut app, vec![]);
        let lit = (caret.min.y as usize..caret.max.y as usize)
            .map(|y| {
                (caret.min.x as usize..caret.max.x as usize)
                    .filter(|&x| grid[y][x] == Some(C_TEXT))
                    .count()
            })
            .sum::<usize>();
        assert!(
            lit as f32 >= CARET_W * CARET_H / 2.0,
            "第 {frame_no} 帧光标没亮（用户点名不要闪烁）"
        );
    }
}

#[test]
fn editor_and_console_input_use_the_same_caret_geometry() {
    let editor = text_caret_rect(10.0, 100.0, SRC_LINE_H);
    let input = text_caret_rect(10.0, 100.0, cells(TEXT_ROWS));
    assert_eq!(editor.size(), input.size());
    assert_eq!(editor.width(), CARET_W);
    assert_eq!(editor.height(), CARET_H);
    assert!(editor.height() < SRC_LINE_H);
    assert_eq!(editor.min.x, 10.0);
    assert!(
        (editor.center().y - (100.0 + SRC_LINE_H / 2.0)).abs() <= 0.5,
        "光标没有在行内垂直居中"
    );
}

#[test]
fn new_windows_start_at_the_work_center() {
    let ctx = egui::Context::default();
    configure_tessellation(&ctx);
    ctx.set_pixels_per_point(1.0);
    let mut app = blank_app();
    app.text_engine = HardText::new(&ctx);
    let work = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(cells(120), cells(70)));
    let frame = |app: &mut StockApp, events: Vec<egui::Event>| win_frame(app, &ctx, work, events);
    frame(&mut app, vec![]);

    let chart = app
        .win_mgr
        .wins
        .iter()
        .position(|w| w.kind == WinKind::Chart)
        .unwrap();
    let chart_rect = app.win_mgr.wins[chart].rect();
    assert!(
        (chart_rect.center().x - work.center().x).abs() <= 1.0
            && (chart_rect.center().y - work.center().y).abs() <= 1.0,
        "图表窗初始位置不在窗口区正中间"
    );

    app.add_chart_window();
    let second = app.win_mgr.wins.last().unwrap().rect();
    assert!(
        (second.center().x - work.center().x).abs() <= 1.0
            && (second.center().y - work.center().y).abs() <= 1.0,
        "第二扇窗没有居中出生"
    );

    for w in app.win_mgr.wins.iter_mut() {
        w.w.open = false;
    }
    app.add_chart_window();
    let reopened = app.win_mgr.wins.last().unwrap().rect();
    assert!(
        (reopened.center().x - work.center().x).abs() <= 1.0
            && (reopened.center().y - work.center().y).abs() <= 1.0,
        "关闭窗口后再次出生仍应居中"
    );
}

#[test]
fn control_panel_starts_at_the_bottom_right_without_exclusive() {
    let ctx = egui::Context::default();
    configure_tessellation(&ctx);
    ctx.set_pixels_per_point(1.0);
    let mut app = blank_app();
    app.text_engine = HardText::new(&ctx);
    let work = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(cells(120), cells(70)));
    win_frame(&mut app, &ctx, work, vec![]);
    let panel = app
        .win_mgr
        .wins
        .iter()
        .find(|w| w.kind == WinKind::Panel)
        .unwrap();
    assert_eq!(panel.rect().max, work.max, "控制面板不在右下角");
    assert!(!panel.exclusive, "控制面板初始不应独占");
}

#[test]
fn taskbar_menu_and_minimized_items_have_a_two_cell_bordered_gap() {
    let ctx = egui::Context::default();
    configure_tessellation(&ctx);
    ctx.set_pixels_per_point(1.0);
    let mut app = blank_app();
    app.text_engine = HardText::new(&ctx);
    let work = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(cells(120), cells(70)));
    win_frame(&mut app, &ctx, work, vec![]);
    let gap = app.win_mgr.taskbar_gap_rect_for_test();
    assert_eq!(gap.width(), cells(2), "任务栏留白不是两个单元格");
    assert_eq!(
        app.win_mgr.taskbar_items_rect().min.x,
        gap.max.x - LINE_W,
        "最小化项目没有接在留白右边框上"
    );
}

#[test]
fn maximize_then_minimize_restores_the_pre_maximize_rect() {
    let ctx = egui::Context::default();
    configure_tessellation(&ctx);
    ctx.set_pixels_per_point(1.0);
    let mut app = blank_app();
    app.text_engine = HardText::new(&ctx);
    let work = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(cells(120), cells(70)));
    win_frame(&mut app, &ctx, work, vec![]);
    let chart = app
        .win_mgr
        .wins
        .iter()
        .position(|w| w.kind == WinKind::Chart)
        .unwrap();
    let before = app.win_mgr.wins[chart].w.rect();
    app.win_mgr.wins[chart].w.toggle_maximize(work);
    let title = app.win_mgr.wins[chart].title.clone();
    let mini_w = WinMgr::mini_width(&mut app, &title);
    app.win_mgr.minimize(chart, mini_w);
    app.win_mgr.restore(chart);
    assert_eq!(app.win_mgr.wins[chart].w.rect(), before);
}

#[test]
fn double_clicking_a_normal_title_bar_maximizes_it() {
    let ctx = egui::Context::default();
    configure_tessellation(&ctx);
    ctx.set_pixels_per_point(1.0);
    let mut app = blank_app();
    app.text_engine = HardText::new(&ctx);
    let work = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(cells(120), cells(70)));
    let press = |pos: egui::Pos2| {
        vec![
            egui::Event::PointerMoved(pos),
            egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: egui::Modifiers::NONE,
            },
        ]
    };
    let release = |pos: egui::Pos2| {
        vec![egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: egui::Modifiers::NONE,
        }]
    };
    win_frame(&mut app, &ctx, work, vec![]);
    let chart = app
        .win_mgr
        .wins
        .iter()
        .position(|w| w.kind == WinKind::Chart)
        .unwrap();
    let rect = app.win_mgr.wins[chart].w.rect();
    let title = free_band(rect, 0, WIN_TITLE_ROWS);
    let keys = sys_button_rects(title);
    let p = egui::pos2((title.min.x + keys[0].min.x) / 2.0, title.center().y);
    win_frame(&mut app, &ctx, work, press(p));
    win_frame(&mut app, &ctx, work, release(p));
    win_frame(&mut app, &ctx, work, press(p));
    win_frame(&mut app, &ctx, work, release(p));
    assert!(app.win_mgr.wins[chart].w.maximized);
    assert_eq!(app.win_mgr.wins[chart].w.rect(), work);
}

#[test]
fn blank_right_click_closes_other_menu_bars() {
    let ctx = egui::Context::default();
    configure_tessellation(&ctx);
    ctx.set_pixels_per_point(1.0);
    let mut app = blank_app();
    app.text_engine = HardText::new(&ctx);
    app.menu = MenuState {
        open: Some(0),
        path: vec![0],
    };
    app.source.editor.menu = MenuState {
        open: Some(0),
        path: vec![0],
    };
    app.pane_menu = Some(PANE_CANDLE);
    let layout = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(WINDOW_W, WINDOW_H));
    let p = split_areas(layout)[2].min + egui::vec2(5.0, 5.0);
    let _ = render_events_in(
        &ctx,
        &mut app,
        vec![
            egui::Event::PointerMoved(p),
            egui::Event::PointerButton {
                pos: p,
                button: egui::PointerButton::Secondary,
                pressed: true,
                modifiers: egui::Modifiers::NONE,
            },
        ],
    );
    assert!(app.menu.open.is_none());
    assert!(app.source.editor.menu.open.is_none());
    assert!(app.pane_menu.is_none());
    assert!(app.context_menu.open().is_some());
}

#[test]
fn editor_toolbar_is_text_and_the_first_button_clears_the_buffer() {
    assert_eq!(
        EDITOR_BUTTONS.map(|(label, _, _)| label),
        ["新建", "保存", "剪切", "复制", "粘贴", "撤销", "查找"]
    );
    let mut app = blank_app();
    app.open_source_window(PANE_CANDLE);
    assert!(!app.source.editor.buf.to_text().is_empty());
    app.new_source();
    assert!(app.source.editor.buf.to_text().is_empty());
    assert!(app.source.editor.modified);
}

#[test]
fn overlapping_borders_resize_the_last_operated_window() {
    let ctx = egui::Context::default();
    configure_tessellation(&ctx);
    ctx.set_pixels_per_point(1.0);
    let mut app = blank_app();
    app.text_engine = HardText::new(&ctx);
    let work = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(cells(120), cells(70)));
    let frame = |app: &mut StockApp, events: Vec<egui::Event>| win_frame(app, &ctx, work, events);
    frame(&mut app, vec![]);
    let back = app
        .win_mgr
        .wins
        .iter()
        .position(|w| w.kind == WinKind::Chart)
        .unwrap();
    app.win_mgr.wins[back].w.pos = work.min + egui::vec2(100.0, 100.0);
    app.win_mgr.wins[back].w.size = egui::vec2(cells(40), cells(30));
    app.add_chart_window();
    let front = app.win_mgr.wins.len() - 1;
    app.win_mgr.wins[front].w.pos = egui::pos2(
        app.win_mgr.wins[back].rect().max.x - LINE_W,
        app.win_mgr.wins[back].rect().min.y,
    );
    app.win_mgr.wins[front].w.size = egui::vec2(cells(30), cells(30));
    frame(&mut app, vec![]);
    let back_id = app.win_mgr.wins[back].w.id;
    let front_id = app.win_mgr.wins[front].w.id;
    let focus = app.win_mgr.wins[back].rect().min + egui::vec2(50.0, 120.0);
    frame(
        &mut app,
        vec![
            egui::Event::PointerMoved(focus),
            egui::Event::PointerButton {
                pos: focus,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: egui::Modifiers::NONE,
            },
        ],
    );
    frame(
        &mut app,
        vec![egui::Event::PointerButton {
            pos: focus,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: egui::Modifiers::NONE,
        }],
    );
    let back = app
        .win_mgr
        .wins
        .iter()
        .position(|w| w.w.id == back_id)
        .unwrap();
    let border = egui::pos2(
        app.win_mgr.wins[back].rect().max.x - LINE_W,
        app.win_mgr.wins[back].rect().center().y,
    );
    frame(&mut app, vec![egui::Event::PointerMoved(border)]);
    frame(
        &mut app,
        vec![
            egui::Event::PointerMoved(border),
            egui::Event::PointerButton {
                pos: border,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: egui::Modifiers::NONE,
            },
        ],
    );
    let back = app
        .win_mgr
        .wins
        .iter()
        .position(|w| w.w.id == back_id)
        .unwrap();
    let front = app
        .win_mgr
        .wins
        .iter()
        .position(|w| w.w.id == front_id)
        .unwrap();
    assert!(app.win_mgr.wins[back].w.resizing());
    assert!(!app.win_mgr.wins[front].w.resizing());
}

#[test]
fn taskbar_title_drag_reorders_minimized_windows() {
    let ctx = egui::Context::default();
    configure_tessellation(&ctx);
    configure_zero_spacing(&ctx);
    ctx.set_pixels_per_point(1.0);
    let mut app = blank_app();
    app.text_engine = HardText::new(&ctx);
    let work = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(cells(120), cells(70)));
    let frame = |app: &mut StockApp, events: Vec<egui::Event>| win_frame(app, &ctx, work, events);
    frame(&mut app, vec![]);
    if let Some(panel) = app
        .win_mgr
        .wins
        .iter()
        .position(|w| w.kind == WinKind::Panel)
    {
        app.win_mgr.wins[panel].w.open = false;
    }
    app.add_chart_window();
    app.add_chart_window();
    let ids: Vec<u64> = app
        .win_mgr
        .wins
        .iter()
        .filter(|w| w.kind == WinKind::Chart)
        .map(|w| w.w.id)
        .collect();
    for i in 0..app.win_mgr.wins.len() {
        if app.win_mgr.wins[i].kind == WinKind::Chart {
            let title = app.win_mgr.wins[i].title.clone();
            let width = WinMgr::mini_width(&mut app, &title);
            app.win_mgr.minimize(i, width);
        }
    }
    frame(&mut app, vec![]);
    assert_eq!(app.win_mgr.taskbar_ids(), ids);
    let first = app
        .win_mgr
        .wins
        .iter()
        .position(|w| w.w.id == ids[0])
        .unwrap();
    let last = app
        .win_mgr
        .wins
        .iter()
        .position(|w| w.w.id == ids[2])
        .unwrap();
    let first_rect = app.win_mgr.rect_of(first);
    let last_rect = app.win_mgr.rect_of(last);
    let keys = sys_button_rects_n(free_band(first_rect, 0, WIN_TITLE_ROWS), 3);
    let handle = egui::pos2(
        (first_rect.min.x + keys[0].min.x) / 2.0,
        first_rect.center().y,
    );
    let target = egui::pos2(last_rect.max.x - 2.0, last_rect.center().y);
    frame(
        &mut app,
        vec![
            egui::Event::PointerMoved(handle),
            egui::Event::PointerButton {
                pos: handle,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: egui::Modifiers::NONE,
            },
        ],
    );
    frame(&mut app, vec![egui::Event::PointerMoved(target)]);
    frame(
        &mut app,
        vec![egui::Event::PointerButton {
            pos: target,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: egui::Modifiers::NONE,
        }],
    );
    assert_eq!(
        app.win_mgr.taskbar_ids(),
        vec![ids[1], ids[2], ids[0]],
        "拖动任务栏标题后没有按落点重新排列"
    );
}

#[test]
fn non_exclusive_windows_can_overlap_until_exclusive_is_enabled() {
    let ctx = egui::Context::default();
    configure_tessellation(&ctx);
    configure_zero_spacing(&ctx);
    ctx.set_pixels_per_point(1.0);
    let mut app = blank_app();
    app.text_engine = HardText::new(&ctx);
    let work = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(cells(120), cells(70)));
    let frame = |app: &mut StockApp, events: Vec<egui::Event>| win_frame(app, &ctx, work, events);
    frame(&mut app, vec![]);
    if let Some(panel) = app
        .win_mgr
        .wins
        .iter()
        .position(|w| w.kind == WinKind::Panel)
    {
        app.win_mgr.wins[panel].w.open = false;
    }
    app.add_chart_window();
    let first = app
        .win_mgr
        .wins
        .iter()
        .position(|w| w.kind == WinKind::Chart)
        .unwrap();
    let second = app.win_mgr.wins.len() - 1;
    let first_id = app.win_mgr.wins[first].w.id;
    app.win_mgr.wins[first].w.pos = work.min + egui::vec2(100.0, 100.0);
    app.win_mgr.wins[first].w.size = egui::vec2(cells(30), cells(20));
    app.win_mgr.wins[second].w.pos = work.min + egui::vec2(150.0, 150.0);
    app.win_mgr.wins[second].w.size = egui::vec2(cells(30), cells(20));
    app.win_mgr.wins[first].exclusive = false;
    app.win_mgr.wins[second].exclusive = false;
    frame(&mut app, vec![]);
    let before = app.win_mgr.wins[second].rect();
    let title = app.win_mgr.wins[second].w.title_band();
    let handle = title.center();
    let walk = egui::vec2(40.0, 0.0);
    frame(
        &mut app,
        vec![
            egui::Event::PointerMoved(handle),
            egui::Event::PointerButton {
                pos: handle,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: egui::Modifiers::NONE,
            },
        ],
    );
    frame(&mut app, vec![egui::Event::PointerMoved(handle + walk)]);
    assert_eq!(
        app.win_mgr.wins[second].rect().min,
        before.min + (walk * WINDOW_DRAG_SPEED).round(),
        "非独占窗口之间不应互相阻挡"
    );
    frame(
        &mut app,
        vec![egui::Event::PointerButton {
            pos: handle + walk,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: egui::Modifiers::NONE,
        }],
    );

    app.win_mgr.toggle_exclusive(first);
    frame(&mut app, vec![]);
    let first = app
        .win_mgr
        .wins
        .iter()
        .position(|w| w.w.id == first_id)
        .unwrap();
    let guard = app.win_mgr.wins[first].rect();
    app.win_mgr.wins[second].w.pos = egui::pos2(guard.max.x + 100.0, guard.min.y + 40.0);
    app.win_mgr.wins[second].w.size = egui::vec2(cells(30), cells(20));
    frame(&mut app, vec![]);
    let title = app.win_mgr.wins[second].w.title_band();
    let handle = title.center();
    let walk = egui::vec2(-200.0, 0.0);
    frame(
        &mut app,
        vec![
            egui::Event::PointerMoved(handle),
            egui::Event::PointerButton {
                pos: handle,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: egui::Modifiers::NONE,
            },
        ],
    );
    frame(&mut app, vec![egui::Event::PointerMoved(handle + walk)]);
    let stopped = app.win_mgr.wins[second].rect();
    assert_eq!(
        stopped.min.x,
        guard.max.x - LINE_W,
        "独占窗口启用后应停在共享边框上"
    );
}

#[test]
#[ignore]
fn bench_frame_cost() {
    let mut app = blank_app();
    app.replace_bars(data::random_walk::generate(BAR_SEED, BAR_COUNT, BAR_START));
    let ctx = egui::Context::default();
    configure_tessellation(&ctx);
    ctx.set_pixels_per_point(1.0);
    let layout = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(WINDOW_W, WINDOW_H));

    app.text_engine = HardText::new(&ctx);
    if app.panel_buttons.is_empty() {
        app.build_panel_buttons();
    }
    let frame = |app: &mut StockApp| {
        let mut raw = egui::RawInput::default();
        raw.screen_rect = Some(layout);
        let t0 = std::time::Instant::now();
        let mut out = ctx.run_ui(raw, |ui| {
            app.draw_frame(ui);
        });
        let build = t0.elapsed().as_secs_f64();
        let shapes = std::mem::take(&mut out.shapes);
        absorb_textures(&mut out.textures_delta);
        let t1 = std::time::Instant::now();
        let prim = ctx.tessellate(shapes, 1.0);
        (build, t1.elapsed().as_secs_f64(), prim.len())
    };
    for _ in 0..10 {
        let _ = frame(&mut app);
    }
    let (mut build, mut tess) = (0.0f64, 0.0f64);
    let mut prims = 0;
    let n = 120;
    for _ in 0..n {
        let (b, t, p) = frame(&mut app);
        build += b;
        tess += t;
        prims = p;
    }
    eprintln!(
        "BENCH 每帧：构建界面 {:.2} ms + 三角化 {:.2} ms = {:.2} ms（{prims} 个图元）",
        build * 1000.0 / n as f64,
        tess * 1000.0 / n as f64,
        (build + tess) * 1000.0 / n as f64
    );
}
