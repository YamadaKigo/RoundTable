use crate::grid::*;
use eframe::egui;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

include!(concat!(env!("OUT_DIR"), "/embedded_fonts.rs"));

pub(crate) const FONT_SIZE: f32 = 16.0;

pub(crate) const TEXT_BOX_H: f32 = 2.0 * GRID + LINE_W;

pub(crate) const ADV_WIDE: f32 = 2.0 * PITCH;

pub(crate) const ADV_ASCII: f32 = PITCH;

pub(crate) const fn char_advance(c: char) -> f32 {
    if is_wide_char(c) { ADV_WIDE } else { ADV_ASCII }
}

pub(crate) fn text_advance(s: &str) -> f32 {
    s.chars().map(char_advance).sum()
}

pub(crate) fn text_width(s: &str) -> i32 {
    text_advance(s) as i32
}

pub(crate) const fn is_wide_char(c: char) -> bool {
    matches!(
        c as u32,
        0x2E80..=0x303F
            | 0x3400..=0x4DBF
            | 0x4E00..=0x9FFF
            | 0xF900..=0xFAFF
            | 0xFE30..=0xFE4F
            | 0xFF00..=0xFF60
            | 0xFFE0..=0xFFE6
    )
}

pub(crate) const GGO_BITMAP: u32 = 1;

pub(crate) const GDI_ERROR: u32 = 0xFFFF_FFFF;

pub(crate) const GDI_FONT_HEIGHT: i32 = -(FONT_SIZE as i32);

pub(crate) const GLYPH_BUF_MAX: usize = 4096;

pub(crate) const GLYPH_ATLAS: usize = 512;

pub(crate) const GLYPH_GAP: usize = 1;

pub(crate) const DEFAULT_CHARSET: u32 = 1;
pub(crate) const OUT_TT_PRECIS: u32 = 4;
pub(crate) const CLIP_DEFAULT_PRECIS: u32 = 0;
pub(crate) const NONANTIALIASED_QUALITY: u32 = 3;
pub(crate) const DEFAULT_PITCH: u32 = 0;

pub(crate) const ANSI_CHARSET: u32 = 0;

pub(crate) const FIXED_PITCH_MODERN: u32 = 0x31;

pub(crate) const GDI_FACES: [&str; 4] = ["宋体", "新宋体", "SimSun", "NSimSun"];

pub(crate) const ASCII_FON_FILES: [&str; 2] = ["vgafix.fon", "8514fix.fon"];

pub(crate) const RAS_DIB_W: i32 = 32;
pub(crate) const RAS_DIB_H: i32 = 32;

pub(crate) const RAS_PEN_Y: i32 = 16;

pub(crate) const RAS_CELL_W: usize = 8;
pub(crate) const RAS_CELL_H: usize = 16;

pub(crate) const BLACKNESS: u32 = 0x0000_0042;

pub(crate) const fn dib_row_bytes(w: i32) -> usize {
    (w as usize).div_ceil(32) * 4
}

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

#[repr(C)]
#[allow(dead_code)]
pub(crate) struct Mat2 {
    pub(crate) e: [i32; 4],
}

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

#[repr(C)]
#[derive(Default)]
pub(crate) struct BitMapInfoHeader {
    pub(crate) size: u32,
    pub(crate) width: i32,
    pub(crate) height: i32,
    pub(crate) planes: u16,
    pub(crate) bit_count: u16,
    pub(crate) compression: u32,
    pub(crate) size_image: u32,
    pub(crate) x_pels_per_meter: i32,
    pub(crate) y_pels_per_meter: i32,
    pub(crate) clr_used: u32,
    pub(crate) clr_important: u32,
}

#[repr(C)]
#[derive(Default)]
pub(crate) struct BitMapInfo {
    pub(crate) header: BitMapInfoHeader,
    pub(crate) colors: [u32; 2],
}

#[repr(C)]
pub(crate) struct RectI {
    pub(crate) left: i32,
    pub(crate) top: i32,
    pub(crate) right: i32,
    pub(crate) bottom: i32,
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
    pub(crate) fn CreateDIBSection(
        hdc: isize,
        bmi: *const BitMapInfo,
        usage: u32,
        bits: *mut *mut u8,
        section: isize,
        offset: u32,
    ) -> isize;
    pub(crate) fn ExtTextOutW(
        hdc: isize,
        x: i32,
        y: i32,
        options: u32,
        rect: *const RectI,
        text: *const u16,
        count: u32,
        dx: *const i32,
    ) -> i32;
    pub(crate) fn SetTextColor(hdc: isize, color: u32) -> u32;
    pub(crate) fn SetBkColor(hdc: isize, color: u32) -> u32;
    pub(crate) fn SetBkMode(hdc: isize, mode: i32) -> i32;
    pub(crate) fn PatBlt(hdc: isize, x: i32, y: i32, w: i32, h: i32, rop: u32) -> i32;
    pub(crate) fn GdiFlush() -> i32;
    pub(crate) fn AddFontResourceExW(name: *const u16, fl: u32, res: isize) -> i32;
}

#[link(name = "kernel32")]
unsafe extern "system" {
    pub(crate) fn GetWindowsDirectoryW(buf: *mut u16, size: u32) -> u32;
}

pub(crate) fn load_private_font(file: &str) -> bool {
    if let Some(path) = embedded_font_path(file)
        && register_private_font(&path)
    {
        return true;
    }
    if register_private_font(&crate::assets::fonts_dir().join(file)) {
        return true;
    }

    let mut dir = [0u16; 260];
    let mut paths: Vec<String> = Vec::new();
    unsafe {
        let n = GetWindowsDirectoryW(dir.as_mut_ptr(), dir.len() as u32);
        if n > 0 && (n as usize) < dir.len() {
            let root = String::from_utf16_lossy(&dir[..n as usize]);
            paths.push(format!("{root}\\Fonts\\{file}"));
        }
    }
    paths.push(file.to_owned());
    for p in paths {
        if register_private_font(Path::new(&p)) {
            return true;
        }
    }
    false
}

fn embedded_font_path(file: &str) -> Option<PathBuf> {
    let bytes = embedded_font(file)?;
    let dir = crate::assets::user_root().join("fonts");
    let path = dir.join(file);
    let current = std::fs::metadata(&path).ok().map(|meta| meta.len());
    if current != Some(bytes.len() as u64) {
        std::fs::create_dir_all(&dir).ok()?;
        std::fs::write(&path, bytes).ok()?;
    }
    Some(path)
}

fn register_private_font(path: &Path) -> bool {
    if !path.is_file() {
        return false;
    }
    let wide: Vec<u16> = path
        .to_string_lossy()
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();
    unsafe { AddFontResourceExW(wide.as_ptr(), 0x10, 0) > 0 }
}

#[derive(Clone, Copy)]
pub(crate) struct Glyph {
    pub(crate) w: usize,
    pub(crate) h: usize,

    pub(crate) ox: i32,

    pub(crate) oy: i32,
    pub(crate) atlas: [usize; 2],
}

pub(crate) struct HardText {
    pub(crate) dc_tt: isize,
    pub(crate) font_tt: isize,

    pub(crate) cjk_ascent: i32,

    pub(crate) dc_ras: isize,
    pub(crate) font_ras: isize,
    pub(crate) ras_dib: isize,
    pub(crate) ras_bits: *mut u8,

    pub(crate) ascii_ascent: i32,
    pub(crate) texture: egui::TextureHandle,
    pub(crate) next_x: usize,
    pub(crate) next_y: usize,
    pub(crate) shelf_h: usize,
    pub(crate) glyphs: HashMap<char, Option<Glyph>>,
}

impl HardText {
    pub(crate) fn new(ctx: &egui::Context) -> Option<Self> {
        unsafe {
            let dc_tt = CreateCompatibleDC(0);
            if dc_tt == 0 {
                return None;
            }

            for file in ["simsun.ttc", "simsunb.ttf"] {
                load_private_font(file);
            }

            let mut font_tt = 0isize;
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
                    font_tt = f;
                    break;
                }
            }
            if font_tt == 0 {
                DeleteDC(dc_tt);
                return None;
            }
            SelectObject(dc_tt, font_tt);

            let mut tm = TextMetricW::default();
            if GetTextMetricsW(dc_tt, &mut tm) == 0 || tm.height <= 0 {
                DeleteObject(font_tt);
                DeleteDC(dc_tt);
                return None;
            }
            let cjk_ascent = tm.ascent;

            let mut ascii_ascent = 0;
            let mut dc_ras = 0isize;
            let mut font_ras = 0isize;
            let mut ras_dib = 0isize;
            let mut ras_bits: *mut u8 = std::ptr::null_mut();
            if ASCII_FON_FILES.iter().any(|f| load_private_font(f)) {
                dc_ras = CreateCompatibleDC(0);
                let face: Vec<u16> = "Fixedsys"
                    .encode_utf16()
                    .chain(std::iter::once(0))
                    .collect();
                font_ras = CreateFontW(
                    GDI_FONT_HEIGHT,
                    0,
                    0,
                    0,
                    400,
                    0,
                    0,
                    0,
                    ANSI_CHARSET,
                    OUT_TT_PRECIS,
                    CLIP_DEFAULT_PRECIS,
                    NONANTIALIASED_QUALITY,
                    FIXED_PITCH_MODERN,
                    face.as_ptr(),
                );
                let mut bmi = BitMapInfo::default();
                bmi.header.size = std::mem::size_of::<BitMapInfoHeader>() as u32;
                bmi.header.width = RAS_DIB_W;
                bmi.header.height = -RAS_DIB_H;
                bmi.header.planes = 1;
                bmi.header.bit_count = 1;
                ras_dib = CreateDIBSection(dc_ras, &bmi, 0, &mut ras_bits, 0, 0);
                let mut tm_ras = TextMetricW::default();
                let ok = GetTextMetricsW(dc_ras, &mut tm_ras);
                if dc_ras == 0
                    || font_ras == 0
                    || ras_dib == 0
                    || ras_bits.is_null()
                    || ok == 0
                    || tm_ras.ascent <= 0
                    || tm_ras.ascent as usize >= RAS_CELL_H
                {
                    if font_ras != 0 {
                        DeleteObject(font_ras);
                    }
                    if ras_dib != 0 {
                        DeleteObject(ras_dib);
                    }
                    if dc_ras != 0 {
                        DeleteDC(dc_ras);
                    }
                    dc_ras = 0;
                    font_ras = 0;
                    ras_dib = 0;
                    ras_bits = std::ptr::null_mut();
                } else {
                    SelectObject(dc_ras, ras_dib);
                    SelectObject(dc_ras, font_ras);
                    ascii_ascent = tm_ras.ascent;
                }
            }

            let blank =
                egui::ColorImage::filled([GLYPH_ATLAS, GLYPH_ATLAS], egui::Color32::TRANSPARENT);
            let texture = ctx.load_texture("rt_hard_text", blank, egui::TextureOptions::NEAREST);

            eprintln!(
                "[文字] 宋体 16px（基线 {}px），ASCII Fixedsys 12pt（字格基线 {}px）",
                cjk_ascent, ascii_ascent
            );
            Some(Self {
                dc_tt,
                font_tt,
                cjk_ascent,
                dc_ras,
                font_ras,
                ras_dib,
                ras_bits,
                ascii_ascent,
                texture,
                next_x: 0,
                next_y: 0,
                shelf_h: 0,
                glyphs: HashMap::new(),
            })
        }
    }

    pub(crate) fn glyph(&mut self, c: char) -> Option<Glyph> {
        if let Some(g) = self.glyphs.get(&c) {
            return *g;
        }
        let g = self.rasterize(c);
        self.glyphs.insert(c, g);
        g
    }

    pub(crate) fn width(&self, s: &str) -> i32 {
        text_width(s)
    }

    pub(crate) fn rasterize(&mut self, c: char) -> Option<Glyph> {
        if is_wide_char(c) {
            self.rasterize_outline(c, true)
        } else if self.font_ras != 0 {
            self.rasterize_raster(c)
        } else {
            self.rasterize_outline(c, false)
        }
    }

    pub(crate) fn rasterize_outline(&mut self, c: char, wide: bool) -> Option<Glyph> {
        let mat = Mat2 {
            e: [0x0001_0000, 0, 0, 0x0001_0000],
        };
        let mut gm = GlyphMetrics::default();
        let dc = self.dc_tt;
        unsafe {
            let need = GetGlyphOutlineW(
                dc,
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
            if need == 0 || w == 0 || h == 0 {
                return Some(Glyph {
                    w: 0,
                    h: 0,
                    ox: 0,
                    oy: 0,
                    atlas: [0, 0],
                });
            }

            let mut buf = vec![0u8; need as usize];
            let got = GetGlyphOutlineW(
                dc,
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

            let row = (got as usize) / h;
            if row * h != got as usize || row * 8 < w {
                return None;
            }

            let ink_w = if wide { w } else { w.min(RAS_CELL_W) };
            let (ax, ay) = self.alloc(ink_w, h)?;
            let mut patch = egui::ColorImage::filled([ink_w, h], egui::Color32::TRANSPARENT);
            for y in 0..h {
                for x in 0..ink_w {
                    let (s0, s1) = if ink_w == w {
                        (x, x + 1)
                    } else {
                        let a = ((x as f32 * w as f32) / ink_w as f32).round() as usize;
                        let b = (((x + 1) as f32 * w as f32) / ink_w as f32).round() as usize;
                        (a, b.max(a + 1))
                    };
                    let on =
                        (s0..s1).any(|s| s < w && buf[y * row + s / 8] & (0x80 >> (s % 8)) != 0);
                    if on {
                        patch[(x, y)] = egui::Color32::WHITE;
                    }
                }
            }
            self.texture
                .set_partial([ax, ay], patch, egui::TextureOptions::NEAREST);

            let ox = if wide {
                gm.origin_x
            } else {
                gm.origin_x + ((RAS_CELL_W as i32 - ink_w as i32) / 2)
            };
            Some(Glyph {
                w: ink_w,
                h,
                ox,
                oy: gm.origin_y,
                atlas: [ax, ay],
            })
        }
    }

    pub(crate) fn rasterize_raster(&mut self, c: char) -> Option<Glyph> {
        let (ax, ay) = self.alloc(RAS_CELL_W, RAS_CELL_H)?;
        unsafe {
            PatBlt(self.dc_ras, 0, 0, RAS_DIB_W, RAS_DIB_H, BLACKNESS);
            SetBkMode(self.dc_ras, 2);
            SetBkColor(self.dc_ras, 0);
            SetTextColor(self.dc_ras, 0x00FF_FFFF);
            let text = [c as u16];
            ExtTextOutW(
                self.dc_ras,
                0,
                RAS_PEN_Y,
                0,
                std::ptr::null(),
                text.as_ptr(),
                1,
                std::ptr::null(),
            );
            GdiFlush();

            let row = dib_row_bytes(RAS_DIB_W);
            let bits = std::slice::from_raw_parts(self.ras_bits, row * RAS_DIB_H as usize);
            let mut patch =
                egui::ColorImage::filled([RAS_CELL_W, RAS_CELL_H], egui::Color32::TRANSPARENT);
            let mut ink = false;
            for y in 0..RAS_CELL_H {
                let src_y = RAS_PEN_Y as usize + y;
                for x in 0..RAS_CELL_W {
                    let on = bits[src_y * row + x / 8] & (0x80 >> (x % 8)) != 0;
                    if on {
                        patch[(x, y)] = egui::Color32::WHITE;
                        ink = true;
                    }
                }
            }
            let _ = ink;
            self.texture
                .set_partial([ax, ay], patch, egui::TextureOptions::NEAREST);

            let ox = ascii_glyph_offset(c);
            Some(Glyph {
                w: RAS_CELL_W,
                h: RAS_CELL_H,
                ox,
                oy: self.ascii_ascent,
                atlas: [ax, ay],
            })
        }
    }

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
            return None;
        }
        let pos = (self.next_x, self.next_y);
        self.next_x += w + GLYPH_GAP;
        self.shelf_h = self.shelf_h.max(h);
        Some(pos)
    }

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
        let top = (pos.y - TEXT_BOX_H / 2.0).round();
        let baseline = top + self.cjk_ascent as f32;
        let mut pen = x0.round();

        let mut mesh = egui::Mesh::with_texture(self.texture.id());
        for c in s.chars() {
            let step = char_advance(c);
            let x = pen.round();
            let Some(g) = self.glyph(c) else {
                painter.rect_stroke(
                    egui::Rect::from_min_size(egui::pos2(x, top), egui::vec2(step, TEXT_BOX_H)),
                    egui::CornerRadius::ZERO,
                    egui::Stroke::new(LINE_W, color),
                    egui::StrokeKind::Inside,
                );
                pen += step;
                continue;
            };
            if g.w > 0 && g.h > 0 {
                let rect = egui::Rect::from_min_size(
                    egui::pos2(x + g.ox as f32, baseline - g.oy as f32),
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
            pen += step;
        }
        if !mesh.is_empty() {
            painter.add(egui::Shape::mesh(mesh));
        }
    }
}

impl Drop for HardText {
    fn drop(&mut self) {
        unsafe {
            DeleteObject(self.font_tt);
            DeleteDC(self.dc_tt);
            if self.font_ras != 0 {
                DeleteObject(self.font_ras);
            }
            if self.ras_dib != 0 {
                DeleteObject(self.ras_dib);
            }
            if self.dc_ras != 0 {
                DeleteDC(self.dc_ras);
            }
        }
    }
}

pub(crate) const fn ascii_glyph_offset(c: char) -> i32 {
    if c == '0' { -1 } else { 0 }
}

pub(crate) fn split_areas(layout: egui::Rect) -> [egui::Rect; 4] {
    let x0 = layout.min.x;
    let x1 = layout.max.x;
    let y0 = layout.min.y;
    let band =
        |y: f32, h: f32| egui::Rect::from_min_size(egui::pos2(x0, y), egui::vec2(x1 - x0, h));

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
