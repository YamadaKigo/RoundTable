use crate::domain::layout::*;
use crate::engine::kit;
use std::collections::HashMap;

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

pub(crate) const GLYPH_ATLAS_W: usize = 1024;
pub(crate) const GLYPH_ATLAS_H: usize = 1024;
pub(crate) const GLYPH_ATLAS_MAX_PAGES: usize = 8;
pub(crate) const GLYPH_RETRY_FRAMES: u64 = 120;

pub(crate) const GLYPH_GAP: usize = 1;

pub(crate) const DEFAULT_CHARSET: u32 = 1;
pub(crate) const OUT_TT_PRECIS: u32 = 4;
pub(crate) const CLIP_DEFAULT_PRECIS: u32 = 0;
pub(crate) const NONANTIALIASED_QUALITY: u32 = 3;
pub(crate) const DEFAULT_PITCH: u32 = 0;

pub(crate) const GDI_FACES: [&str; 3] = ["寒蝉点阵体 16px", "寒蝉点阵体", "ChillBitmap_16px"];

pub(crate) const RAS_CELL_W: usize = 8;
pub(crate) const RAS_CELL_H: usize = 16;

pub(crate) const CJK_FONT_BYTES: &[u8] = include_bytes!("../../assets/fonts/ChillBitmap_16px.ttf");
pub(crate) const SPLEEN_BITMAP: &[u8; 4096] = include_bytes!("../../assets/fonts/spleen-8x16.raw");
pub(crate) const SWEET16_BITMAP: &[u8; 6144] =
    include_bytes!("../../assets/fonts/Sweet16mono-8x16.raw");

#[repr(C)]
#[derive(Default)]
#[cfg(test)]
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
#[cfg(test)]
pub(crate) struct BitMapInfo {
    pub(crate) header: BitMapInfoHeader,
    pub(crate) colors: [u32; 2],
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub(crate) struct GlyphMetrics {
    pub(crate) black_box_x: u32,
    pub(crate) black_box_y: u32,
    pub(crate) origin_x: i32,
    pub(crate) origin_y: i32,
    pub(crate) cell_inc_x: i16,
    pub(crate) cell_inc_y: i16,
}

#[repr(C)]
pub(crate) struct Mat2 {
    pub(crate) e: [i32; 4],
}

#[repr(C)]
#[derive(Default)]
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
    #[cfg(test)]
    pub(crate) fn CreateDIBSection(
        hdc: isize,
        bmi: *const BitMapInfo,
        usage: u32,
        bits: *mut *mut u8,
        section: isize,
        offset: u32,
    ) -> isize;
    pub(crate) fn AddFontMemResourceEx(
        font_data: *const core::ffi::c_void,
        size: u32,
        reserved: *mut core::ffi::c_void,
        font_count: *mut u32,
    ) -> isize;
    pub(crate) fn RemoveFontMemResourceEx(resource: isize) -> i32;
}

#[derive(Clone, Copy)]
pub(crate) struct Glyph {
    pub(crate) w: usize,
    pub(crate) h: usize,

    pub(crate) ox: i32,

    pub(crate) oy: i32,
    pub(crate) atlas: [usize; 2],
    pub(crate) page: usize,
    pub(crate) last_used: u64,
}

#[derive(Clone, Copy, Default)]
enum GlyphSlot {
    #[default]
    Unknown,
    Missing(u64),
    Ready(Glyph),
}

pub(crate) struct AtlasPage {
    pub(crate) texture: kit::TextureHandle,
    pub(crate) next_x: usize,
    pub(crate) next_y: usize,
    pub(crate) shelf_h: usize,
}

fn new_atlas_page(ctx: &kit::Context, index: usize) -> AtlasPage {
    let blank = kit::ColorImage::filled([GLYPH_ATLAS_W, GLYPH_ATLAS_H], kit::Color32::TRANSPARENT);
    AtlasPage {
        texture: ctx.load_texture(
            format!("rt_hard_text_{index}"),
            blank,
            kit::TextureOptions::NEAREST,
        ),
        next_x: 0,
        next_y: 0,
        shelf_h: 0,
    }
}

pub(crate) struct HardText {
    pub(crate) ctx: kit::Context,
    cjk_font_resource: isize,
    pub(crate) dc_tt: isize,
    pub(crate) font_tt: isize,

    pub(crate) cjk_ascent: i32,

    pub(crate) ascii_bold: bool,
    pub(crate) pages: Vec<AtlasPage>,
    glyphs: HashMap<char, GlyphSlot>,
    ascii_glyphs: [GlyphSlot; 128],
    pub(crate) generation: u64,
    pub(crate) alloc_failed: bool,
}

impl HardText {
    #[cfg(test)]
    pub(crate) fn new(ctx: &kit::Context) -> Option<Self> {
        Self::new_after_font_preload(ctx, preload_font_files())
    }

    pub(crate) fn new_after_font_preload(
        ctx: &kit::Context,
        cjk_font_resource: isize,
    ) -> Option<Self> {
        if cjk_font_resource == 0 {
            return None;
        }
        unsafe {
            let dc_tt = CreateCompatibleDC(0);
            if dc_tt == 0 {
                RemoveFontMemResourceEx(cjk_font_resource);
                return None;
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
                RemoveFontMemResourceEx(cjk_font_resource);
                return None;
            }
            SelectObject(dc_tt, font_tt);

            let mut tm = TextMetricW::default();
            if GetTextMetricsW(dc_tt, &mut tm) == 0 || tm.height <= 0 {
                DeleteObject(font_tt);
                DeleteDC(dc_tt);
                RemoveFontMemResourceEx(cjk_font_resource);
                return None;
            }
            let cjk_ascent = tm.ascent;

            let pages = vec![new_atlas_page(ctx, 0)];

            eprintln!(
                "[Font] ChillBitmap 16px (baseline {}px), ASCII Sweet16 regular / Spleen 8x16 bold",
                cjk_ascent
            );
            Some(Self {
                ctx: ctx.clone(),
                cjk_font_resource,
                dc_tt,
                font_tt,
                cjk_ascent,
                ascii_bold: false,
                pages,
                glyphs: HashMap::new(),
                ascii_glyphs: [GlyphSlot::Unknown; 128],
                generation: 0,
                alloc_failed: false,
            })
        }
    }

    pub(crate) fn glyph(&mut self, c: char) -> Option<Glyph> {
        let generation = self.generation;
        if c.is_ascii() {
            let slot = c as usize;
            match self.ascii_glyphs[slot] {
                GlyphSlot::Ready(mut glyph) => {
                    glyph.last_used = generation;
                    self.ascii_glyphs[slot] = GlyphSlot::Ready(glyph);
                    return Some(glyph);
                }
                GlyphSlot::Missing(retry) if generation < retry => return None,
                GlyphSlot::Unknown => {}
                GlyphSlot::Missing(_) => {}
            }
            self.alloc_failed = false;
            let mut glyph = self.rasterize(c);
            if glyph.is_none() && self.alloc_failed && self.add_page() {
                self.alloc_failed = false;
                glyph = self.rasterize(c);
            }
            if let Some(value) = &mut glyph {
                value.last_used = generation;
            }
            self.ascii_glyphs[slot] = match glyph {
                Some(glyph) => GlyphSlot::Ready(glyph),
                None => GlyphSlot::Missing(generation + GLYPH_RETRY_FRAMES),
            };
            return glyph;
        }
        if let Some(GlyphSlot::Ready(mut glyph)) = self.glyphs.get(&c).copied() {
            glyph.last_used = generation;
            self.glyphs.insert(c, GlyphSlot::Ready(glyph));
            return Some(glyph);
        }
        if let Some(GlyphSlot::Missing(retry)) = self.glyphs.get(&c).copied()
            && generation < retry
        {
            return None;
        }
        self.alloc_failed = false;
        let mut glyph = self.rasterize(c);
        if glyph.is_none() && self.alloc_failed && self.add_page() {
            self.alloc_failed = false;
            glyph = self.rasterize(c);
        }
        let slot = match glyph {
            Some(mut value) => {
                value.last_used = generation;
                GlyphSlot::Ready(value)
            }
            None => GlyphSlot::Missing(generation + GLYPH_RETRY_FRAMES),
        };
        self.glyphs.insert(c, slot);
        glyph
    }

    pub(crate) fn width(&self, s: &str) -> i32 {
        text_width(s)
    }

    pub(crate) fn set_ascii_bold(&mut self, bold: bool) -> bool {
        if self.ascii_bold == bold {
            return false;
        }
        self.ascii_bold = bold;
        self.ascii_glyphs = [GlyphSlot::Unknown; 128];
        self.generation = self.generation.wrapping_add(1);
        true
    }

    pub(crate) fn rasterize(&mut self, c: char) -> Option<Glyph> {
        if is_wide_char(c) {
            self.rasterize_outline(c, true)
        } else if c.is_ascii() {
            self.rasterize_ascii(c)
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
                    page: 0,
                    last_used: self.generation,
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
            let (page, ax, ay) = self.alloc(ink_w, h)?;
            let mut patch = kit::ColorImage::filled([ink_w, h], kit::Color32::TRANSPARENT);
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
                        patch[(x, y)] = kit::Color32::WHITE;
                    }
                }
            }
            self.pages[page]
                .texture
                .set_partial([ax, ay], patch, kit::TextureOptions::NEAREST);

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
                page,
                last_used: self.generation,
            })
        }
    }

    pub(crate) fn rasterize_ascii(&mut self, c: char) -> Option<Glyph> {
        let (page, ax, ay) = self.alloc(RAS_CELL_W, RAS_CELL_H)?;
        let bitmap = if self.ascii_bold {
            SPLEEN_BITMAP.as_slice()
        } else {
            SWEET16_BITMAP.as_slice()
        };
        let oy = self.cjk_ascent - 1;
        let glyph = c as usize * RAS_CELL_H;
        let mut patch =
            kit::ColorImage::filled([RAS_CELL_W, RAS_CELL_H], kit::Color32::TRANSPARENT);
        for y in 0..RAS_CELL_H {
            let row = bitmap[glyph + y];
            for x in 0..RAS_CELL_W {
                let bit = 0x80 >> x;
                if row & bit != 0 {
                    patch[(x, y)] = kit::Color32::WHITE;
                }
            }
        }
        self.pages[page]
            .texture
            .set_partial([ax, ay], patch, kit::TextureOptions::NEAREST);

        Some(Glyph {
            w: RAS_CELL_W,
            h: RAS_CELL_H,
            ox: ascii_glyph_offset(c),
            oy,
            atlas: [ax, ay],
            page,
            last_used: self.generation,
        })
    }

    pub(crate) fn alloc(&mut self, w: usize, h: usize) -> Option<(usize, usize, usize)> {
        if w + GLYPH_GAP > GLYPH_ATLAS_W || h + GLYPH_GAP > GLYPH_ATLAS_H {
            self.alloc_failed = true;
            return None;
        }
        for page in 0..self.pages.len() {
            if let Some((x, y)) = self.try_alloc_page(page, w, h) {
                return Some((page, x, y));
            }
        }
        if !self.add_page() {
            let page = self.recycle_least_recent_page()?;
            return self.try_alloc_page(page, w, h).map(|(x, y)| (page, x, y));
        }
        let page = self.pages.len() - 1;
        self.try_alloc_page(page, w, h).map(|(x, y)| (page, x, y))
    }

    fn try_alloc_page(&mut self, page: usize, w: usize, h: usize) -> Option<(usize, usize)> {
        let atlas = &mut self.pages[page];
        if atlas.next_x + w + GLYPH_GAP > GLYPH_ATLAS_W {
            atlas.next_x = 0;
            atlas.next_y += atlas.shelf_h + GLYPH_GAP;
            atlas.shelf_h = 0;
        }
        if atlas.next_y + h + GLYPH_GAP > GLYPH_ATLAS_H {
            return None;
        }
        let pos = (atlas.next_x, atlas.next_y);
        atlas.next_x += w + GLYPH_GAP;
        atlas.shelf_h = atlas.shelf_h.max(h);
        Some(pos)
    }

    pub(crate) fn add_page(&mut self) -> bool {
        if self.pages.len() >= GLYPH_ATLAS_MAX_PAGES {
            return false;
        }
        let index = self.pages.len();
        let page = new_atlas_page(&self.ctx, index);
        self.pages.push(page);
        true
    }

    fn recycle_least_recent_page(&mut self) -> Option<usize> {
        let mut best = None;
        for page in (0..self.pages.len()).rev() {
            let mut newest = 0u64;
            for slot in self.ascii_glyphs.iter().chain(self.glyphs.values()) {
                if let GlyphSlot::Ready(glyph) = slot
                    && glyph.page == page
                {
                    newest = newest.max(glyph.last_used);
                }
            }
            if best.is_none_or(|(_, score)| newest < score) {
                best = Some((page, newest));
            }
        }
        let (page, _) = best?;
        for slot in &mut self.ascii_glyphs {
            if matches!(*slot, GlyphSlot::Ready(glyph) if glyph.page == page) {
                *slot = GlyphSlot::Unknown;
            }
        }
        self.glyphs
            .retain(|_, slot| !matches!(*slot, GlyphSlot::Ready(glyph) if glyph.page == page));
        let atlas = &mut self.pages[page];
        atlas.texture.clear_transparent();
        atlas.next_x = 0;
        atlas.next_y = 0;
        atlas.shelf_h = 0;
        Some(page)
    }

    pub(crate) fn draw(
        &mut self,
        painter: &kit::Painter,
        pos: kit::Pos2,
        align: kit::Align2,
        s: &str,
        color: kit::Color32,
    ) {
        self.generation = self.generation.wrapping_add(1);
        let w = self.width(s);
        let x0 = match align.x() {
            kit::Align::Min => pos.x,
            kit::Align::Center => pos.x - w as f32 / 2.0,
            kit::Align::Max => pos.x - w as f32,
        };
        let top = (pos.y - TEXT_BOX_H / 2.0).round();
        let baseline = top + self.cjk_ascent as f32;
        let mut pen = x0.round();

        let mut meshes: Vec<kit::Mesh> = self
            .pages
            .iter()
            .map(|page| kit::Mesh::with_texture(page.texture.id()))
            .collect();
        for mesh in &mut meshes {
            mesh.vertices.reserve(s.len().saturating_mul(4));
            mesh.indices.reserve(s.len().saturating_mul(6));
        }
        for c in s.chars() {
            let step = char_advance(c);
            let x = pen.round();
            let Some(g) = self.glyph(c) else {
                painter.rect_stroke(
                    kit::Rect::from_min_size(kit::pos2(x, top), kit::vec2(step, TEXT_BOX_H)),
                    kit::CornerRadius::ZERO,
                    kit::Stroke::new(LINE_W, color),
                    kit::StrokeKind::Inside,
                );
                pen += step;
                continue;
            };
            if g.w > 0 && g.h > 0 {
                let rect = kit::Rect::from_min_size(
                    kit::pos2(x + g.ox as f32, baseline - g.oy as f32),
                    kit::vec2(g.w as f32, g.h as f32),
                );
                let uv = kit::Rect::from_min_max(
                    kit::pos2(
                        g.atlas[0] as f32 / GLYPH_ATLAS_W as f32,
                        g.atlas[1] as f32 / GLYPH_ATLAS_H as f32,
                    ),
                    kit::pos2(
                        (g.atlas[0] + g.w) as f32 / GLYPH_ATLAS_W as f32,
                        (g.atlas[1] + g.h) as f32 / GLYPH_ATLAS_H as f32,
                    ),
                );
                meshes[g.page].add_rect_with_uv(rect, uv, color);
            }
            pen += step;
        }
        for mesh in meshes {
            if !mesh.is_empty() {
                painter.add(kit::Shape::mesh(mesh));
            }
        }
    }
}

pub(crate) fn preload_font_files() -> isize {
    let Ok(size) = u32::try_from(CJK_FONT_BYTES.len()) else {
        return 0;
    };
    let mut count = 0u32;
    let resource = unsafe {
        AddFontMemResourceEx(
            CJK_FONT_BYTES.as_ptr().cast(),
            size,
            std::ptr::null_mut(),
            &mut count,
        )
    };
    if count == 0 {
        if resource != 0 {
            unsafe { RemoveFontMemResourceEx(resource) };
        }
        return 0;
    }
    resource
}

impl Drop for HardText {
    fn drop(&mut self) {
        unsafe {
            DeleteObject(self.font_tt);
            DeleteDC(self.dc_tt);
            RemoveFontMemResourceEx(self.cjk_font_resource);
        }
    }
}

pub(crate) const fn ascii_glyph_offset(c: char) -> i32 {
    let _ = c;
    0
}

pub(crate) fn split_areas(layout: kit::Rect) -> [kit::Rect; 5] {
    let layout = layout.shrink(grid_line(1));
    let x0 = layout.min.x;
    let x1 = layout.max.x;
    let y0 = layout.min.y;
    let band = |y: f32, h: f32| kit::Rect::from_min_size(kit::pos2(x0, y), kit::vec2(x1 - x0, h));

    let status = band(layout.max.y - STATUS_H, STATUS_H);
    let taskbar = band(status.min.y - TASKBAR_H + LINE_W, TASKBAR_H);
    [
        band(y0 + grid_line(TITLE_LINE), TITLEBAR_H),
        band(y0 + grid_line(MENU_LINE), MENUBAR_H),
        kit::Rect::from_min_max(
            kit::pos2(x0, y0 + grid_line(WORK_LINE)),
            kit::pos2(x1, taskbar.min.y + LINE_W),
        ),
        taskbar,
        status,
    ]
}
