use crate::grid::*;
use eframe::egui;

pub(crate) const ICON_W: f32 = 2.0 * GRID;

pub(crate) const ICON_H: f32 = ICON_W;

pub(crate) const ICON_PX: usize = 16;

pub(crate) const ICON_GAP: f32 = 3.0 * PITCH - ICON_W;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum IconKind {
    Cross,

    New,

    Save,

    Compile,

    Cut,

    Copy,

    Paste,

    Undo,

    Find,
}

impl IconKind {
    pub(crate) const ALL: [IconKind; Self::COUNT] = [
        IconKind::Cross,
        IconKind::New,
        IconKind::Save,
        IconKind::Compile,
        IconKind::Cut,
        IconKind::Copy,
        IconKind::Paste,
        IconKind::Undo,
        IconKind::Find,
    ];

    pub(crate) const COUNT: usize = 9;

    pub(crate) fn index(self) -> usize {
        match self {
            IconKind::Cross => 0,
            IconKind::New => 1,
            IconKind::Save => 2,
            IconKind::Compile => 3,
            IconKind::Cut => 4,
            IconKind::Copy => 5,
            IconKind::Paste => 6,
            IconKind::Undo => 7,
            IconKind::Find => 8,
        }
    }

    pub(crate) fn file_stem(self) -> &'static str {
        match self {
            IconKind::Cross => "cross",
            IconKind::New => "new",
            IconKind::Save => "save",
            IconKind::Compile => "compile",
            IconKind::Cut => "cut",
            IconKind::Copy => "copy",
            IconKind::Paste => "paste",
            IconKind::Undo => "undo",
            IconKind::Find => "find",
        }
    }

    #[cfg(test)]
    pub(crate) fn label(self) -> &'static str {
        match self {
            IconKind::Cross => "删除",
            IconKind::New => "新建",
            IconKind::Save => "保存",
            IconKind::Compile => "编译",
            IconKind::Cut => "剪切",
            IconKind::Copy => "复制",
            IconKind::Paste => "粘贴",
            IconKind::Undo => "撤销",
            IconKind::Find => "查找",
        }
    }
}

pub(crate) const ICON_TRANSPARENT_RGB: (u8, u8, u8) = (0xFF, 0x00, 0xFF);

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) struct IconImage {
    px: [[Option<egui::Color32>; ICON_PX]; ICON_PX],
}

pub(crate) fn snap_aabbcc_channel(v: u8) -> u8 {
    let step = ((v as u32 + 8) / 17).min(15);
    (step * 17) as u8
}

fn snap_aabbcc(c: egui::Color32) -> egui::Color32 {
    egui::Color32::from_rgb(
        snap_aabbcc_channel(c.r()),
        snap_aabbcc_channel(c.g()),
        snap_aabbcc_channel(c.b()),
    )
}

impl IconImage {
    pub(crate) fn blank() -> Self {
        Self {
            px: [[None; ICON_PX]; ICON_PX],
        }
    }

    fn from_pixels(w: usize, h: usize, data: &[u8], channels: usize) -> Self {
        let mut px = [[None; ICON_PX]; ICON_PX];
        if w == 0 || h == 0 || channels == 0 {
            return Self { px };
        }
        let at = |x: usize, y: usize| -> (u8, u8, u8, u8) {
            let i = (y * w + x) * channels;
            let get = |k: usize| data.get(i + k).copied().unwrap_or(255);
            match channels {
                1 => (get(0), get(0), get(0), 255),
                2 => (get(0), get(0), get(0), get(1)),
                3 => (get(0), get(1), get(2), 255),
                _ => (get(0), get(1), get(2), get(3)),
            }
        };
        for (y, row) in px.iter_mut().enumerate() {
            for (x, cell) in row.iter_mut().enumerate() {
                let sx = (x * w / ICON_PX).min(w - 1);
                let sy = (y * h / ICON_PX).min(h - 1);
                let (r, g, b, a) = at(sx, sy);
                if a < 128 || (r, g, b) == ICON_TRANSPARENT_RGB {
                    continue;
                }
                *cell = Some(snap_aabbcc(egui::Color32::from_rgb(r, g, b)));
            }
        }
        Self { px }
    }

    pub(crate) fn is_blank(&self) -> bool {
        self.px.iter().all(|row| row.iter().all(|p| p.is_none()))
    }

    pub(crate) fn paint(&self, painter: &egui::Painter, rect: egui::Rect) {
        let x0 = rect.min.x.round();
        let y0 = rect.min.y.round();
        let mut mesh = egui::Mesh::default();
        for row in 0..ICON_PX {
            let mut col = 0usize;
            while col < ICON_PX {
                let Some(px) = self.px[row][col] else {
                    col += 1;
                    continue;
                };
                let start = col;
                while col < ICON_PX && self.px[row][col] == Some(px) {
                    col += 1;
                }
                mesh.add_colored_rect(
                    egui::Rect::from_min_size(
                        egui::pos2(x0 + start as f32, y0 + row as f32),
                        egui::vec2((col - start) as f32, LINE_W),
                    ),
                    px,
                );
            }
        }
        if !mesh.is_empty() {
            painter.add(egui::Shape::mesh(mesh));
        }
    }
}

#[derive(Clone)]
pub(crate) struct IconSet {
    icons: [IconImage; IconKind::COUNT],
}

impl Default for IconSet {
    fn default() -> Self {
        Self::blank()
    }
}

impl IconSet {
    pub(crate) fn blank() -> Self {
        Self {
            icons: [IconImage::blank(); IconKind::COUNT],
        }
    }

    pub(crate) fn load_or_create() -> Self {
        let dir = crate::assets::icons_dir();
        let mut icons = [IconImage::blank(); IconKind::COUNT];
        for kind in IconKind::ALL {
            let path = dir.join(format!("{}.png", kind.file_stem()));
            let from_disk = std::fs::read(&path)
                .ok()
                .and_then(|bytes| decode_png(&bytes));
            let img = match from_disk {
                Some(img) if !img.is_blank() => img,
                _ => IconImage::blank(),
            };
            icons[kind.index()] = img;
        }
        Self { icons }
    }

    pub(crate) fn get(&self, kind: IconKind) -> &IconImage {
        &self.icons[kind.index()]
    }
}

#[cfg(test)]
pub(crate) use crate::assets::icons_dir;

pub(crate) fn decode_png_rgba(bytes: &[u8]) -> Option<egui::IconData> {
    let mut decoder = png::Decoder::new(std::io::Cursor::new(bytes));
    decoder.set_transformations(png::Transformations::normalize_to_color8());
    let mut reader = decoder.read_info().ok()?;
    let mut buf = vec![0u8; reader.output_buffer_size()?];
    let info = reader.next_frame(&mut buf).ok()?;
    if info.bit_depth != png::BitDepth::Eight {
        return None;
    }
    let mut rgba = Vec::with_capacity((info.width * info.height * 4) as usize);
    match info.color_type {
        png::ColorType::Rgba => rgba.extend_from_slice(&buf[..info.buffer_size()]),
        png::ColorType::Rgb => {
            for px in buf[..info.buffer_size()].chunks_exact(3) {
                rgba.extend_from_slice(&[px[0], px[1], px[2], 0xFF]);
            }
        }
        png::ColorType::GrayscaleAlpha => {
            for px in buf[..info.buffer_size()].chunks_exact(2) {
                rgba.extend_from_slice(&[px[0], px[0], px[0], px[1]]);
            }
        }
        png::ColorType::Grayscale => {
            for px in &buf[..info.buffer_size()] {
                rgba.extend_from_slice(&[*px, *px, *px, 0xFF]);
            }
        }
        png::ColorType::Indexed => return None,
    }
    Some(egui::IconData {
        rgba,
        width: info.width,
        height: info.height,
    })
}

pub(crate) const CURSOR_HOTSPOT: Option<[u16; 2]> = Some([0, 0]);

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum CursorKind {
    Normal,

    Left,

    Right,

    Move,

    ResizeH,

    ResizeV,

    ResizeNwSe,

    ResizeNeSw,

    Slider,

    Wheel,

    Text,
}

impl CursorKind {
    pub(crate) const ALL: [CursorKind; Self::COUNT] = [
        CursorKind::Normal,
        CursorKind::Left,
        CursorKind::Right,
        CursorKind::Move,
        CursorKind::ResizeH,
        CursorKind::ResizeV,
        CursorKind::ResizeNwSe,
        CursorKind::ResizeNeSw,
        CursorKind::Slider,
        CursorKind::Wheel,
        CursorKind::Text,
    ];

    pub(crate) const COUNT: usize = 11;

    pub(crate) fn index(self) -> usize {
        match self {
            CursorKind::Normal => 0,
            CursorKind::Left => 1,
            CursorKind::Right => 2,
            CursorKind::Move => 3,
            CursorKind::ResizeH => 4,
            CursorKind::ResizeV => 5,
            CursorKind::ResizeNwSe => 6,
            CursorKind::ResizeNeSw => 7,
            CursorKind::Slider => 8,
            CursorKind::Wheel => 9,
            CursorKind::Text => 10,
        }
    }

    pub(crate) fn file_stem(self) -> &'static str {
        match self {
            CursorKind::Normal => "normal",
            CursorKind::Left => "left",
            CursorKind::Right => "right",
            CursorKind::Move => "move",
            CursorKind::ResizeH => "resize_h",
            CursorKind::ResizeV => "resize_v",
            CursorKind::ResizeNwSe => "resize_nwse",
            CursorKind::ResizeNeSw => "resize_nesw",
            CursorKind::Slider => "slider",
            CursorKind::Wheel => "wheel",
            CursorKind::Text => "text",
        }
    }

    #[cfg(test)]
    pub(crate) fn label(self) -> &'static str {
        match self {
            CursorKind::Normal => "正常",
            CursorKind::Left => "左键按下",
            CursorKind::Right => "右键按下",
            CursorKind::Move => "移动窗口",
            CursorKind::ResizeH => "改尺寸（左右）",
            CursorKind::ResizeV => "改尺寸（上下）",
            CursorKind::ResizeNwSe => "改尺寸（右下）",
            CursorKind::ResizeNeSw => "改尺寸（左下）",
            CursorKind::Slider => "滑动滑块",
            CursorKind::Wheel => "滚轮滚动",
            CursorKind::Text => "文本输入",
        }
    }
}

pub(crate) struct CursorImage {
    img: IconImage,

    hot: [u16; 2],

    #[cfg_attr(not(test), allow(dead_code))]
    cache: Option<std::sync::Arc<[u8]>>,
}

pub(crate) struct Cursors {
    imgs: [CursorImage; CursorKind::COUNT],
}

impl CursorImage {
    fn blank() -> Self {
        Self::new(IconImage::blank())
    }

    fn new(img: IconImage) -> Self {
        let hot = CURSOR_HOTSPOT.unwrap_or([0, 0]);
        Self {
            img,
            hot,
            cache: None,
        }
    }

    #[cfg_attr(not(test), allow(dead_code))]
    fn rgba_for(&mut self) -> std::sync::Arc<[u8]> {
        if let Some(rgba) = &self.cache {
            return rgba.clone();
        }
        let mut buf = vec![0u8; ICON_PX * ICON_PX * 4];
        for row in 0..ICON_PX {
            for col in 0..ICON_PX {
                let Some(px) = self.img.px[row][col] else {
                    continue;
                };
                let i = (row * ICON_PX + col) * 4;
                buf[i] = px.r();
                buf[i + 1] = px.g();
                buf[i + 2] = px.b();
                buf[i + 3] = 0xFF;
            }
        }
        let rgba: std::sync::Arc<[u8]> = buf.into();
        self.cache = Some(rgba.clone());
        rgba
    }
}

impl Cursors {
    pub(crate) fn hotspot(&self, kind: CursorKind) -> [u16; 2] {
        self.imgs[kind.index()].hot
    }

    pub(crate) fn load() -> Self {
        let dir = crate::assets::cursors_dir();
        let mut imgs = std::array::from_fn(|_| CursorImage::blank());
        for kind in CursorKind::ALL {
            let path = dir.join(format!("{}.png", kind.file_stem()));
            let img = std::fs::read(&path)
                .ok()
                .and_then(|bytes| decode_png(&bytes))
                .filter(|img| !img.is_blank())
                .unwrap_or_else(IconImage::blank);
            imgs[kind.index()] = CursorImage::new(img);
        }
        Self { imgs }
    }

    pub(crate) fn is_blank(&self) -> bool {
        self.imgs.iter().all(|c| c.img.is_blank())
    }

    #[cfg(test)]
    pub(crate) fn blank() -> Self {
        Self {
            imgs: std::array::from_fn(|_| CursorImage::blank()),
        }
    }

    fn img(&self, kind: CursorKind) -> &IconImage {
        let img = &self.imgs[kind.index()].img;
        if img.is_blank() {
            &self.imgs[CursorKind::Normal.index()].img
        } else {
            img
        }
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn rgba(&mut self, kind: CursorKind) -> std::sync::Arc<[u8]> {
        self.imgs[kind.index()].rgba_for()
    }

    pub(crate) fn paint(&self, painter: &egui::Painter, tip: egui::Pos2, kind: CursorKind) {
        let [hx, hy] = self.hotspot(kind);
        let rect = egui::Rect::from_min_size(
            egui::pos2(tip.x.round() - hx as f32, tip.y.round() - hy as f32),
            egui::vec2(ICON_W, ICON_H),
        );
        self.img(kind).paint(painter, rect);
    }
}

pub(crate) fn decode_png(bytes: &[u8]) -> Option<IconImage> {
    let mut decoder = png::Decoder::new(std::io::Cursor::new(bytes));
    decoder.set_transformations(png::Transformations::normalize_to_color8());
    let mut reader = decoder.read_info().ok()?;
    let mut buf = vec![0u8; reader.output_buffer_size()?];
    let info = reader.next_frame(&mut buf).ok()?;
    if info.bit_depth != png::BitDepth::Eight {
        return None;
    }
    let channels = match info.color_type {
        png::ColorType::Grayscale => 1,
        png::ColorType::GrayscaleAlpha => 2,
        png::ColorType::Rgb => 3,
        png::ColorType::Rgba => 4,

        png::ColorType::Indexed => return None,
    };
    Some(IconImage::from_pixels(
        info.width as usize,
        info.height as usize,
        &buf,
        channels,
    ))
}
