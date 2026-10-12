// ===========================================================================
//  模块 font：硬边文字（Windows GDI 1bpp 字形）
// ===========================================================================
//  字号 + GDI 字形引擎。文字一律走 StockApp::text 这一个入口（在 app.rs 里）。
//  本文件 = 拆分前那份单文件里的「一、字体与字号」+「五、硬边文字」两节。
use crate::grid::*;
use eframe::egui;
use std::collections::HashMap;

// ===========================================================================
//  一、字体与字号
// ===========================================================================

// 统一字号：宋体 12px。文字由 Windows GDI 按这个字号光栅化成单色点阵字形（见第五节）。
pub(crate) const FONT_SIZE: f32 = 12.0;

// ===========================================================================
//  五、硬边文字：Windows GDI 单色字形
// ===========================================================================
//  为什么不用 egui 自己的文字渲染：egui 0.36 的字形是 vello 做的解析式抗锯齿，
//  图集里存的是覆盖率灰度，而且没有任何开关能关掉（TessellationOptions::feathering
//  的文档明确写着「不影响文字」；FontTweak 只有 hinting 和位置微调；TextOptions 里的
//  color_transfer_function 只是一条连续曲线，做不出硬边）。而 12px 宋体的汉字笔画
//  本来就只有一个像素宽，把灰度抹平只会让笔画断开，比带抗锯齿更难看。
//  所以文字走 Windows 自己那条路：GetGlyphOutlineW + GGO_BITMAP 拿到的是 1bpp 单色
//  字形位图（就是 Win9x 上那种点阵字形，笔画完整、边缘零抗锯齿），自己拼进一张贴图，
//  按整数像素落点、NEAREST 采样画出去，中间没有任何插值。
//  约定：全工程的文字只能走 StockApp::text 这一个入口，别处不许再调 painter.text。

/// GGO_BITMAP：要单色（1bpp）字形位图，而不是矢量轮廓。
pub(crate) const GGO_BITMAP: u32 = 1;

/// GetGlyphOutlineW 出错时的返回值。
pub(crate) const GDI_ERROR: u32 = 0xFFFF_FFFF;

/// 字符高度取负数 = 用「字符高度」而不是「单元格高度」，正好等于字号。
pub(crate) const GDI_FONT_HEIGHT: i32 = -(FONT_SIZE as i32);

/// 单色字形位图的字节数上限：12px 的字形远小于它，超过就当这个字没有。
pub(crate) const GLYPH_BUF_MAX: usize = 4096;

/// 字形图集的边长（像素）。512x512 装得下几千个 12px 字形。
pub(crate) const GLYPH_ATLAS: usize = 512;

/// 字形之间留的空隙，免得相邻字形的边缘互相渗色。
pub(crate) const GLYPH_GAP: usize = 1;

/// CreateFontW 的参数：默认字符集、TrueType 优先、默认裁剪精度、非抗锯齿品质、默认间距。
pub(crate) const DEFAULT_CHARSET: u32 = 1;
pub(crate) const OUT_TT_PRECIS: u32 = 4;
pub(crate) const CLIP_DEFAULT_PRECIS: u32 = 0;
pub(crate) const NONANTIALIASED_QUALITY: u32 = 3;
pub(crate) const DEFAULT_PITCH: u32 = 0;

/// 候选字体名：宋体优先，取不到再退到别的。
pub(crate) const GDI_FACES: [&str; 4] = ["宋体", "新宋体", "SimSun", "NSimSun"];

/// GLYPHMETRICS：GDI 回给我们的字形度量。
#[repr(C)]
#[derive(Clone, Copy, Default)]
#[allow(dead_code)]
pub(crate) struct GlyphMetrics {
    pub(crate) black_box_x: u32,
    pub(crate) black_box_y: u32,
    pub(crate) origin_x: i32,
    pub(crate) origin_y: i32,
    pub(crate) cell_inc_x: i16,
    pub(crate) cell_inc_y: i16,
}

/// MAT2：字形变换矩阵（16.16 定点），必须传单位阵。
#[repr(C)]
#[allow(dead_code)]
pub(crate) struct Mat2 {
    pub(crate) e: [i32; 4],
}

/// TEXTMETRICW：必须和 Windows 里的定义一样大（60 字节），少一个字段就会被写坏内存。
#[repr(C)]
#[derive(Default)]
#[allow(dead_code)]
pub(crate) struct TextMetricW {
    pub(crate) height: i32,
    pub(crate) ascent: i32,
    pub(crate) descent: i32,
    pub(crate) internal_leading: i32,
    pub(crate) external_leading: i32,
    pub(crate) ave_char_width: i32,
    pub(crate) max_char_width: i32,
    pub(crate) weight: i32,
    pub(crate) overhang: i32,
    pub(crate) digitized_aspect_x: i32,
    pub(crate) digitized_aspect_y: i32,
    pub(crate) first_char: u16,
    pub(crate) last_char: u16,
    pub(crate) default_char: u16,
    pub(crate) break_char: u16,
    pub(crate) italic: u8,
    pub(crate) underlined: u8,
    pub(crate) struck_out: u8,
    pub(crate) pitch_and_family: u8,
    pub(crate) char_set: u8,
}

#[link(name = "gdi32")]
unsafe extern "system" {
    pub(crate) fn CreateCompatibleDC(hdc: isize) -> isize;
    pub(crate) fn DeleteDC(hdc: isize) -> i32;
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn CreateFontW(
        c_height: i32,
        c_width: i32,
        escapement: i32,
        orientation: i32,
        weight: i32,
        italic: u32,
        underline: u32,
        strike_out: u32,
        charset: u32,
        out_precision: u32,
        clip_precision: u32,
        quality: u32,
        pitch_and_family: u32,
        face: *const u16,
    ) -> isize;
    pub(crate) fn SelectObject(hdc: isize, obj: isize) -> isize;
    pub(crate) fn DeleteObject(obj: isize) -> i32;
    pub(crate) fn GetTextMetricsW(hdc: isize, tm: *mut TextMetricW) -> i32;
    pub(crate) fn GetGlyphOutlineW(
        hdc: isize,
        ch: u32,
        format: u32,
        gm: *mut GlyphMetrics,
        buf_size: u32,
        buf: *mut u8,
        mat: *const Mat2,
    ) -> u32;
}

/// 一个光栅化好的字形：位图尺寸、相对笔位/基线的偏移、步进，以及它在图集里的落点。
#[derive(Clone, Copy)]
pub(crate) struct Glyph {
    pub(crate) w: usize,
    pub(crate) h: usize,
    pub(crate) ox: i32,
    pub(crate) oy: i32,
    pub(crate) advance: i32,
    pub(crate) atlas: [usize; 2],
}

/// 硬边文字引擎：一个 GDI 内存 DC + 一张字形图集贴图 + 字形缓存。
pub(crate) struct HardText {
    pub(crate) dc: isize,
    pub(crate) font: isize,
    pub(crate) ascent: i32,
    pub(crate) line_h: i32,
    pub(crate) texture: egui::TextureHandle,
    pub(crate) next_x: usize,
    pub(crate) next_y: usize,
    pub(crate) shelf_h: usize,
    pub(crate) glyphs: HashMap<char, Option<Glyph>>,
}

impl HardText {
    /// 建内存 DC、选字体、问一次行高，再开一张空图集。任何一步失败都返回 None。
    pub(crate) fn new(ctx: &egui::Context) -> Option<Self> {
        unsafe {
            let dc = CreateCompatibleDC(0);
            if dc == 0 {
                return None;
            }

            // 逐个试候选字体名，第一个建成功的就用它。
            let mut font = 0isize;
            for face in GDI_FACES {
                let wide: Vec<u16> = face.encode_utf16().chain(std::iter::once(0)).collect();
                let f = CreateFontW(
                    GDI_FONT_HEIGHT,
                    0,
                    0,
                    0,
                    400,
                    0,
                    0,
                    0,
                    DEFAULT_CHARSET,
                    OUT_TT_PRECIS,
                    CLIP_DEFAULT_PRECIS,
                    NONANTIALIASED_QUALITY,
                    DEFAULT_PITCH,
                    wide.as_ptr(),
                );
                if f != 0 {
                    font = f;
                    break;
                }
            }
            if font == 0 {
                DeleteDC(dc);
                return None;
            }
            SelectObject(dc, font);

            let mut tm = TextMetricW::default();
            if GetTextMetricsW(dc, &mut tm) == 0 || tm.height <= 0 {
                DeleteObject(font);
                DeleteDC(dc);
                return None;
            }

            // 图集一开始整张透明，以后每做一个新字形就用 set_partial 补一小块。
            let blank =
                egui::ColorImage::filled([GLYPH_ATLAS, GLYPH_ATLAS], egui::Color32::TRANSPARENT);
            let texture = ctx.load_texture("rt_hard_text", blank, egui::TextureOptions::NEAREST);

            eprintln!("[文字] GDI 单色字形：行高 {}px，基线 {}px", tm.height, tm.ascent);
            Some(Self {
                dc,
                font,
                ascent: tm.ascent,
                line_h: tm.height,
                texture,
                next_x: 0,
                next_y: 0,
                shelf_h: 0,
                glyphs: HashMap::new(),
            })
        }
    }

    /// 取字形（带缓存）。None 表示这个字在这套字体里没有。
    pub(crate) fn glyph(&mut self, c: char) -> Option<Glyph> {
        if let Some(g) = self.glyphs.get(&c) {
            return *g;
        }
        let g = self.rasterize(c);
        self.glyphs.insert(c, g);
        g
    }

    /// 一串文字的像素宽度：所有字形步进之和，是整数。
    pub(crate) fn width(&mut self, s: &str) -> i32 {
        s.chars().map(|c| self.glyph(c).map_or(0, |g| g.advance)).sum()
    }

    /// 让 GDI 现场光栅化一个字形：先问要多大缓冲，再取位图，最后写进图集。
    pub(crate) fn rasterize(&mut self, c: char) -> Option<Glyph> {
        let mat = Mat2 {
            e: [0x0001_0000, 0, 0, 0x0001_0000],
        };
        let mut gm = GlyphMetrics::default();
        unsafe {
            let need = GetGlyphOutlineW(
                self.dc,
                c as u32,
                GGO_BITMAP,
                &mut gm,
                0,
                std::ptr::null_mut(),
                &mat,
            );
            if need == GDI_ERROR || need as usize > GLYPH_BUF_MAX {
                return None;
            }

            let (w, h) = (gm.black_box_x as usize, gm.black_box_y as usize);
            let advance = gm.cell_inc_x as i32;
            if need == 0 || w == 0 || h == 0 {
                // 空格这类没有墨迹的字形：只有步进，没有位图。
                return Some(Glyph {
                    w: 0,
                    h: 0,
                    ox: 0,
                    oy: 0,
                    advance,
                    atlas: [0, 0],
                });
            }

            let mut buf = vec![0u8; need as usize];
            let got = GetGlyphOutlineW(
                self.dc,
                c as u32,
                GGO_BITMAP,
                &mut gm,
                need,
                buf.as_mut_ptr(),
                &mat,
            );
            if got == GDI_ERROR || got == 0 {
                return None;
            }
            // 每行字节数：GDI 的 1bpp 位图按 4 字节对齐，用「总字节数 / 高度」反推最稳。
            let row = (got as usize) / h;
            if row * h != got as usize || row * 8 < w {
                return None;
            }

            let (ax, ay) = self.alloc(w, h)?;
            let mut patch = egui::ColorImage::filled([w, h], egui::Color32::TRANSPARENT);
            for y in 0..h {
                for x in 0..w {
                    // 1bpp：一个字节 8 个像素，最高位在最左边。
                    if buf[y * row + x / 8] & (0x80 >> (x % 8)) != 0 {
                        patch[(x, y)] = egui::Color32::WHITE;
                    }
                }
            }
            self.texture
                .set_partial([ax, ay], patch, egui::TextureOptions::NEAREST);

            Some(Glyph {
                w,
                h,
                ox: gm.origin_x,
                oy: gm.origin_y,
                advance,
                atlas: [ax, ay],
            })
        }
    }

    /// 在图集里挤出一块 w x h 的地方：简单的货架式打包，一行填满就换下一行。
    pub(crate) fn alloc(&mut self, w: usize, h: usize) -> Option<(usize, usize)> {
        if w + GLYPH_GAP > GLYPH_ATLAS || h + GLYPH_GAP > GLYPH_ATLAS {
            return None;
        }
        if self.next_x + w + GLYPH_GAP > GLYPH_ATLAS {
            self.next_x = 0;
            self.next_y += self.shelf_h + GLYPH_GAP;
            self.shelf_h = 0;
        }
        if self.next_y + h + GLYPH_GAP > GLYPH_ATLAS {
            // 图集满了就不再扩：缺的字直接不画（512x512 实际用不完）。
            return None;
        }
        let pos = (self.next_x, self.next_y);
        self.next_x += w + GLYPH_GAP;
        self.shelf_h = self.shelf_h.max(h);
        Some(pos)
    }

    /// 把一串文字画到 painter 上：整数像素落点 + 顶点色，贴图 NEAREST 采样，绝不插值。
    pub(crate) fn draw(
        &mut self,
        painter: &egui::Painter,
        pos: egui::Pos2,
        align: egui::Align2,
        s: &str,
        color: egui::Color32,
    ) {
        let w = self.width(s);
        let x0 = match align.x() {
            egui::Align::Min => pos.x,
            egui::Align::Center => pos.x - w as f32 / 2.0,
            egui::Align::Max => pos.x - w as f32,
        };
        let top = (pos.y - self.line_h as f32 / 2.0).round();
        let baseline = top + self.ascent as f32;
        let mut pen = x0.round();

        let mut mesh = egui::Mesh::with_texture(self.texture.id());
        for c in s.chars() {
            let Some(g) = self.glyph(c) else {
                // 字体里没有这个字：画个空方框占位（宁可缺字，也不出灰边）。
                painter.rect_stroke(
                    egui::Rect::from_min_size(
                        egui::pos2(pen, top),
                        egui::vec2(FONT_SIZE, self.line_h as f32),
                    ),
                    egui::CornerRadius::ZERO,
                    egui::Stroke::new(LINE_W, color),
                    egui::StrokeKind::Inside,
                );
                pen += FONT_SIZE;
                continue;
            };
            if g.w > 0 && g.h > 0 {
                let rect = egui::Rect::from_min_size(
                    egui::pos2(pen + g.ox as f32, baseline - g.oy as f32),
                    egui::vec2(g.w as f32, g.h as f32),
                );
                let side = GLYPH_ATLAS as f32;
                let uv = egui::Rect::from_min_max(
                    egui::pos2(g.atlas[0] as f32 / side, g.atlas[1] as f32 / side),
                    egui::pos2(
                        (g.atlas[0] + g.w) as f32 / side,
                        (g.atlas[1] + g.h) as f32 / side,
                    ),
                );
                mesh.add_rect_with_uv(rect, uv, color);
            }
            pen += g.advance as f32;
        }
        if !mesh.is_empty() {
            painter.add(egui::Shape::mesh(mesh));
        }
    }
}

impl Drop for HardText {
    fn drop(&mut self) {
        unsafe {
            DeleteObject(self.font);
            DeleteDC(self.dc);
        }
    }
}

/// 把逻辑布局矩形切成四块骨架：标题栏 / 菜单栏 / 功能区 / 状态栏。
/// 标题栏、菜单栏贴顶（上边落在第 0 / 3 / 6 条网格线上，高度 cells(3)），状态栏贴底（高度 cells(3)），
/// 功能区吃掉中间全部空间 —— 窗口拉高它就变高（左栏的 K 线区跟着长），拉宽两栏就各自变宽。
/// 默认尺寸下算出来和原来一模一样（第 0 / 3 / 6 / 64 条网格线）；窗口小于下限时按下限排版。
/// 相邻两块在共用的那条网格线上重叠 1px（见开头铁律第 7 条）：底色接缝永远只有 1px，
/// 不会出现两条线堆成 2px，也不会有没画到的缝。
pub(crate) fn split_areas(layout: egui::Rect) -> [egui::Rect; 4] {
    let x0 = layout.min.x;
    let x1 = layout.max.x;
    let y0 = layout.min.y;
    let band = |y: f32, h: f32| {
        egui::Rect::from_min_size(egui::pos2(x0, y), egui::vec2(x1 - x0, h))
    };
    // 状态栏贴底：它的上沿就是功能区能用到的最下面那条线（两块在那条线上重叠 1px）。
    let status = band(layout.max.y - STATUS_H, STATUS_H);
    [
        band(y0 + grid_line(TITLE_LINE), TITLEBAR_H),
        band(y0 + grid_line(MENU_LINE), MENUBAR_H),
        egui::Rect::from_min_max(
            egui::pos2(x0, y0 + grid_line(WORK_LINE)),
            egui::pos2(x1, status.min.y + LINE_W),
        ),
        status,
    ]
}

