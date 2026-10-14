// ===========================================================================
//  模块 font：硬边点阵文字（Windows GDI 1bpp 字形）
// ===========================================================================
//  字号 + 字形引擎。文字一律走 StockApp::text 这一个入口（在 app.rs 里）。
//  本文件 = 拆分前那份单文件里的「一、字体与字号」+「五、硬边文字」两节。
use crate::grid::*;
use eframe::egui;
use std::collections::HashMap;

// ===========================================================================
//  一、字体与字号：16px 点阵字（汉字宋体 16px + ASCII 走 Fixedsys 8x16）
// ===========================================================================

// 统一字号：16px。
//   - 汉字：宋体（SimSun）16px - 宋体自带 16px 的点阵字形，笔画恰好一个像素粗；
//   - ASCII：走 Fixedsys 8x16 点阵字（Windows 自带的 vgafix.fon，1bpp 硬边，最经典的点阵字）；
//     想换回宋体 16px 的 ASCII 就把下面那个 ASCII_FROM_CJK 改成 true（字形换了，字格和排版一个字都不动）。
// 两种字都由 Windows GDI 光栅化成 1bpp 单色点阵（见第五节），没有一个抗锯齿像素。
pub(crate) const FONT_SIZE: f32 = 16.0;

// 网格文字模型（本文件的核心约定，改文字排版之前必读）：
//   一个字符的字格就是它占的那几个内容格，一模一样大：
//     - 汉字：2x2 个内容格 = 宽 8+1+8 = 17px、高 8+1+8 = 17px；字形 16x16 摆在左上角
//       （16 塞进 17 只会多出 1px，统一留在右边和下边，保证整数像素落点）；
//     - ASCII：1x2 个内容格 = 宽 8px、高 8+1+8 = 17px；字形 8x16 摆满。
//   于是任何一串文字的宽度都是 9px（1 个内容格 + 1 条线格）的整数倍，起点又落在内容格起点上，
//   整串必然和 9px 网格对齐：汉字每 2 格一个、ASCII 每 1 格一个。
//   纵向：一个文本行盒 = 2 个内容格（内容区高 TEXT_BOX_H = 17px，上下各 1px 线格），
//   汉字和 ASCII 共用同一条基线 - 基线在内容区上沿往下 cjk_ascent（宋体 16px 实测 14px）处；
//   ASCII 的 8x16 点阵字格里基线在自上往下 ascii_ascent（Fixedsys 16px 实测 12px）处，
//   所以两种字混排时底边对齐，不会一个高一个低。汉字 16px 的字形正好填满内容区上 16px，
//   ASCII 的 8x16 字格顶在内容区上沿往下 2px 处、底边落在内容区最后一行上，两者都不出格。

/// 一个文本行盒的内容高度：2 个内容格 = 8 + 1 + 8 = 17px（行盒外框是 cells(2) = 19px）。
pub(crate) const TEXT_BOX_H: f32 = 2.0 * GRID + LINE_W;

/// 一个汉字的横向步进：2 个内容格 + 中间 1 条线格 = 18px。
pub(crate) const ADV_WIDE: f32 = 2.0 * PITCH;

/// 一个 ASCII 字符（含空格）的横向步进：1 个内容格 = 9px。
pub(crate) const ADV_ASCII: f32 = PITCH;

/// ASCII 用哪套字形（两套的字格一模一样：8px 宽的内容格 + 17px 高的文本行盒，
/// 所以换它只改字形的样子，一个字都不动排版）：
///   true  = 宋体 16px 的 ASCII（和汉字同一套笔画、同一条基线）；
///   false = Fixedsys 8x16 点阵字（Windows 自带 vgafix.fon，8x16 字格）。
/// 两条路都走 GDI 的 1bpp 单色字形，都是硬边、零抗锯齿。
pub(crate) const ASCII_FROM_CJK: bool = false;

/// 一个字符的横向步进（px）：汉字 18（2 个内容格）、ASCII 9（1 个内容格）。
/// 界面文字只允许这两种（见铁律），所以步进只可能是这两个值之一。
pub(crate) const fn char_advance(c: char) -> f32 {
    if is_wide_char(c) {
        ADV_WIDE
    } else {
        ADV_ASCII
    }
}

/// 一串文字的宽度（px）：逐字累加，一定是 9px 的整数倍。
pub(crate) fn text_advance(s: &str) -> f32 {
    s.chars().map(char_advance).sum()
}

/// 排版用的文字宽度：就是墨迹宽度（已经是 9 的整数倍，不用再吸格）。
pub(crate) fn text_width(s: &str) -> i32 {
    text_advance(s) as i32
}

/// 宽字符（全角）判定：只认「中文」- 汉字（含扩展 A / 兼容区）+ 中文标点 + 全角形式。
/// 界面文字只允许中文和 ASCII：别的符号（非 ASCII 的标点 / 箭头 / 数学符号）由源码扫描测试把关。
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

// ===========================================================================
//  五、硬边文字：Windows GDI 单色字形
// ===========================================================================
//  为什么不用 egui 自己的文字渲染：egui 0.36 的字形是 vello 做的解析式抗锯齿，
//  图集里存的是覆盖率灰度，而且没有任何开关能关掉（TessellationOptions::feathering
//  的文档明确写着「不影响文字」）。而 16px 宋体 / Fixedsys 的点阵字笔画本来就只有一个像素宽，
//  把灰度抹平只会让笔画断开，比带抗锯齿更难看。
//  所以文字走 Windows 自己那条路，分两条：
//    - 汉字走 GetGlyphOutlineW + GGO_BITMAP：拿到的是 1bpp 单色字形位图（宋体自带的 16px 点阵）；
//    - ASCII 走 Fixedsys 点阵字（.fon 光栅字体，GetGlyphOutlineW 不支持它）：自己建一个 1bpp 的
//      DIB，用 ExtTextOutW 把字画进去，再按位读回来 - 同样是 1bpp、零抗锯齿；
//      把 ASCII_FROM_CJK 打开就改回宋体 16px 的 ASCII（和汉字同一套笔画、同一条基线）。
//  两条路的结果都拼进同一张贴图，按整数像素落点、NEAREST 采样画出去，中间没有任何插值。
//  约定：全工程的文字只能走 StockApp::text 这一个入口，别处不许再调 painter.text。

/// GGO_BITMAP：要单色（1bpp）字形位图，而不是矢量轮廓。
pub(crate) const GGO_BITMAP: u32 = 1;

/// GetGlyphOutlineW 出错时的返回值。
pub(crate) const GDI_ERROR: u32 = 0xFFFF_FFFF;

/// 字符高度取负数 = 用「字符高度」而不是「单元格高度」，正好等于字号。
pub(crate) const GDI_FONT_HEIGHT: i32 = -(FONT_SIZE as i32);

/// 单色字形位图的字节数上限：16px 的字形远小于它，超过就当这个字没有。
pub(crate) const GLYPH_BUF_MAX: usize = 4096;

/// 字形图集的边长（像素）。512x512 装得下几千个 16px 字形。
pub(crate) const GLYPH_ATLAS: usize = 512;

/// 字形之间留的空隙，免得相邻字形的边缘互相渗色。
pub(crate) const GLYPH_GAP: usize = 1;

/// CreateFontW 的参数：默认字符集、TrueType 优先、默认裁剪精度、非抗锯齿品质、默认间距。
pub(crate) const DEFAULT_CHARSET: u32 = 1;
pub(crate) const OUT_TT_PRECIS: u32 = 4;
pub(crate) const CLIP_DEFAULT_PRECIS: u32 = 0;
pub(crate) const NONANTIALIASED_QUALITY: u32 = 3;
pub(crate) const DEFAULT_PITCH: u32 = 0;

/// ASCII 点阵字的字符集取 ANSI（0）：只有这个字符集下的 Fixedsys 才是真正的 8x16 点阵字，
/// 别的字符集 GDI 会挑一个拉伸过的替代字形（实测笔画会歪）。
pub(crate) const ANSI_CHARSET: u32 = 0;

/// ASCII 点阵字的字距族：FF_MODERN（等宽）+ FIXED_PITCH。
pub(crate) const FIXED_PITCH_MODERN: u32 = 0x31;

/// 候选汉字字体名：宋体优先，取不到再退到别的。
pub(crate) const GDI_FACES: [&str; 4] = ["宋体", "新宋体", "SimSun", "NSimSun"];

/// ASCII 点阵字的候选文件名：Fixedsys 就是 Windows 自带的 vgafix.fon（8x16）。
pub(crate) const ASCII_FON_FILES: [&str; 2] = ["vgafix.fon", "8514fix.fon"];

/// 点阵字（ASCII）光栅化用的小画布：32x32、1bpp、自上而下。
pub(crate) const RAS_DIB_W: i32 = 32;
pub(crate) const RAS_DIB_H: i32 = 32;

/// 点阵字的笔位 y：字形画在这儿，字格取 [0, 8) x [RAS_PEN_Y, RAS_PEN_Y + 16) 这一块。
pub(crate) const RAS_PEN_Y: i32 = 16;

/// 点阵字的字格尺寸。
pub(crate) const RAS_CELL_W: usize = 8;
pub(crate) const RAS_CELL_H: usize = 16;

/// PatBlt 的 BLACKNESS 光栅操作：把目标矩形整块填 0（1bpp 里 0 = 没墨）。
pub(crate) const BLACKNESS: u32 = 0x0000_0042;

/// 1bpp 位图一行按 4 字节对齐。
pub(crate) const fn dib_row_bytes(w: i32) -> usize {
    ((w as usize + 31) / 32) * 4
}

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

/// BITMAPINFOHEADER：CreateDIBSection 要的那份头。
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

/// BITMAPINFO：1bpp 的调色板两项（黑 / 白），CreateDIBSection 只要头，调色板照样要跟着。
#[repr(C)]
#[derive(Default)]
pub(crate) struct BitMapInfo {
    pub(crate) header: BitMapInfoHeader,
    pub(crate) colors: [u32; 2],
}

/// RECT：ExtTextOutW 的裁剪矩形（这里不用，传空指针）。
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

/// 进程内加载一份点阵字体文件（FR_PRIVATE = 0x10，只影响本进程，不需要管理员权限）。
/// 优先用 Windows 目录下的绝对路径，取不到再试裸文件名（GDI 会去系统字体目录找）。
pub(crate) fn load_private_font(file: &str) -> bool {
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
        let wide: Vec<u16> = p.encode_utf16().chain(std::iter::once(0)).collect();
        let ok = unsafe { AddFontResourceExW(wide.as_ptr(), 0x10, 0) };
        if ok > 0 {
            return true;
        }
    }
    false
}

/// 一个光栅化好的字形：位图尺寸、相对笔位 / 基线的偏移，以及它在图集里的落点。
#[derive(Clone, Copy)]
pub(crate) struct Glyph {
    pub(crate) w: usize,
    pub(crate) h: usize,
    /// 墨迹左边相对笔位的偏移（汉字照 GDI 的 origin_x；ASCII 的 8x16 点阵字格就贴着笔位，是 0）。
    pub(crate) ox: i32,
    /// 字格上边相对基线的距离（正数 = 在基线上方）。
    pub(crate) oy: i32,
    pub(crate) atlas: [usize; 2],
}

/// 硬边文字引擎：两个 GDI 内存 DC（汉字走轮廓、ASCII 走点阵位图）+ 一张字形图集贴图 + 字形缓存。
pub(crate) struct HardText {
    /// 汉字的 DC：只用来 GetGlyphOutlineW。
    pub(crate) dc_tt: isize,
    pub(crate) font_tt: isize,
    /// 汉字基线的落点（相对内容区上沿的像素数，宋体 16px 实测 14）。
    pub(crate) cjk_ascent: i32,
    /// ASCII 点阵字的 DC + 1bpp DIB（ExtTextOutW 画进去再读回来）。
    pub(crate) dc_ras: isize,
    pub(crate) font_ras: isize,
    pub(crate) ras_dib: isize,
    pub(crate) ras_bits: *mut u8,
    /// ASCII 字格里基线的落点（相对 8x16 字格上沿，Fixedsys 实测 12）。
    pub(crate) ascii_ascent: i32,
    pub(crate) texture: egui::TextureHandle,
    pub(crate) next_x: usize,
    pub(crate) next_y: usize,
    pub(crate) shelf_h: usize,
    pub(crate) glyphs: HashMap<char, Option<Glyph>>,
}

impl HardText {
    /// 建汉字 DC（宋体）、ASCII 点阵 DC（Fixedsys + 1bpp DIB）、问一次基线，再开一张空图集。
    /// 汉字字体拿不到就返回 None（整个文字层关闭）。
    pub(crate) fn new(ctx: &egui::Context) -> Option<Self> {
        unsafe {
            let dc_tt = CreateCompatibleDC(0);
            if dc_tt == 0 {
                return None;
            }

            // 逐个试候选汉字字体名，第一个建成功的就用它。
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

            // ASCII 点阵字：先把 .fon 加载进本进程，再建字体；拿不到就走汉字字体兜底（见 rasterize）。
            let mut ascii_ascent = 0;
            let mut dc_ras = 0isize;
            let mut font_ras = 0isize;
            let mut ras_dib = 0isize;
            let mut ras_bits: *mut u8 = std::ptr::null_mut();
            if ASCII_FON_FILES.iter().any(|f| load_private_font(f)) {
                dc_ras = CreateCompatibleDC(0);
                let face: Vec<u16> = "Fixedsys".encode_utf16().chain(std::iter::once(0)).collect();
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
                bmi.header.height = -RAS_DIB_H; // 负高度 = 自上而下
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

            // 图集一开始整张透明，以后每做一个新字形就用 set_partial 补一小块。
            let blank =
                egui::ColorImage::filled([GLYPH_ATLAS, GLYPH_ATLAS], egui::Color32::TRANSPARENT);
            let texture = ctx.load_texture("rt_hard_text", blank, egui::TextureOptions::NEAREST);

            eprintln!(
                "[文字] 宋体 16px（基线 {}px）+ Fixedsys 8x16（字格基线 {}px）",
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

    /// 取字形（带缓存）。None 表示这个字在这套字体里没有。
    pub(crate) fn glyph(&mut self, c: char) -> Option<Glyph> {
        if let Some(g) = self.glyphs.get(&c) {
            return *g;
        }
        let g = self.rasterize(c);
        self.glyphs.insert(c, g);
        g
    }

    /// 一串文字的像素宽度。
    pub(crate) fn width(&self, s: &str) -> i32 {
        text_width(s)
    }

    /// 现场光栅化一个字形：汉字走 GDI 的 1bpp 轮廓；ASCII 默认走 Fixedsys 点阵字
    /// （1bpp DIB 位图），ASCII_FROM_CJK 打开时才走宋体轮廓；两条路都往同一个图集里贴。
    pub(crate) fn rasterize(&mut self, c: char) -> Option<Glyph> {
        if is_wide_char(c) {
            self.rasterize_outline(c, true)
        } else if self.font_ras != 0 && !ASCII_FROM_CJK {
            self.rasterize_raster(c)
        } else {
            // 宋体的 ASCII 字形：墨迹按整列压进 8px 的内容格，和汉字共用一条基线。
            self.rasterize_outline(c, false)
        }
    }

    /// 汉字（和走宋体那一路的 ASCII）走 GetGlyphOutlineW + GGO_BITMAP：先问要多大缓冲，
    /// 再取位图，写进图集。ASCII 这一路额外把墨迹按整列压进 8px 宽（宋体的 ASCII 是比例字，
    /// 墨迹 6~11px 宽），压完正好占满一个 ASCII 内容格、和汉字共用一条基线。
    pub(crate) fn rasterize_outline(&mut self, c: char, wide: bool) -> Option<Glyph> {
        let mat = Mat2 {
            e: [0x0001_0000, 0, 0, 0x0001_0000],
        };
        let mut gm = GlyphMetrics::default();
        unsafe {
            let need = GetGlyphOutlineW(
                self.dc_tt,
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
                // 空格这类没有墨迹的字形：只有步进，没有位图。
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
                self.dc_tt,
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

            // 兜底路径下 ASCII 的墨迹要压进 8px 的字格：按整列取或（点阵字的经典做法，
            // 笔画不断、字与字不重叠）；汉字不动，照它自己 16px 的墨迹排。
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
                    let on = (s0..s1)
                        .any(|s| s < w && buf[y * row + s / 8] & (0x80 >> (s % 8)) != 0);
                    if on {
                        patch[(x, y)] = egui::Color32::WHITE;
                    }
                }
            }
            self.texture
                .set_partial([ax, ay], patch, egui::TextureOptions::NEAREST);

            // 兜底路径下墨迹在自己那 8px 字格里居中（宽 8 的内容格 + 两边各 1px 线格）。
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

    /// ASCII 走点阵字体：清空画布 -> ExtTextOutW 画一个字符 -> 按位读回 8x16 的字格，写进图集。
    /// 读回来的是「字格」，不是紧贴墨迹的外框：所有 ASCII 字共用同一个字格，基线天然对齐。
    pub(crate) fn rasterize_raster(&mut self, c: char) -> Option<Glyph> {
        let (ax, ay) = self.alloc(RAS_CELL_W, RAS_CELL_H)?;
        unsafe {
            PatBlt(
                self.dc_ras,
                0,
                0,
                RAS_DIB_W,
                RAS_DIB_H,
                BLACKNESS,
            );
            SetBkMode(self.dc_ras, 2); // OPAQUE：背景交给 PatBlt 清，这里只写字
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
            Some(Glyph {
                w: RAS_CELL_W,
                h: RAS_CELL_H,
                ox: 0,
                oy: self.ascii_ascent,
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
    /// 行盒高 TEXT_BOX_H（17px），基线在行盒上沿往下 cjk_ascent 处 - 汉字和 ASCII 共用它。
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
            // 这个字的步进：汉字 18px（2 个内容格）、ASCII 9px（1 个内容格）。
            let step = char_advance(c);
            let x = pen.round();
            let Some(g) = self.glyph(c) else {
                // 字体里没有这个字：画个空方框占位（宁可缺字，也不出灰边）。
                painter.rect_stroke(
                    egui::Rect::from_min_size(
                        egui::pos2(x, top),
                        egui::vec2(step, TEXT_BOX_H),
                    ),
                    egui::CornerRadius::ZERO,
                    egui::Stroke::new(LINE_W, color),
                    egui::StrokeKind::Inside,
                );
                pen += step;
                continue;
            };
            if g.w > 0 && g.h > 0 {
                // 字格贴着自己的内容格起点（汉字照宋体给的 origin_x，ASCII 点阵字格就是 0）。
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

/// 把逻辑布局矩形切成四块骨架：标题栏 / 菜单栏 / 功能区 / 状态栏。
/// 四块都是 2 格高（一个文本行盒），功能区吃掉中间全部空间 - 窗口拉高它就变高，
/// 拉宽两栏就各自变宽。相邻两块在共用的那条网格线上重叠 1px（见开头铁律第 7 条）：
/// 底色接缝永远只有 1px，不会出现两条线堆成 2px，也不会有没画到的缝。
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