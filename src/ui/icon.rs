use crate::domain::layout::*;
use crate::engine::kit;

pub(crate) const ICON_W: f32 = 2.0 * GRID;

#[cfg(test)]
pub(crate) const ICON_H: f32 = ICON_W;

#[cfg(test)]
pub(crate) const ICON_PX: usize = 16;

pub(crate) const CURSOR_PX: usize = 31;

pub(crate) const CURSOR_W: f32 = CURSOR_PX as f32;

pub(crate) const CURSOR_H: f32 = CURSOR_W;

pub(crate) const CURSOR_CENTER: usize = CURSOR_PX / 2;

pub(crate) const ICON_GAP: f32 = 3.0 * PITCH - ICON_W;

#[cfg(test)]
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

#[cfg(test)]
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

    pub(crate) fn label(self) -> &'static str {
        match self {
            IconKind::Cross => "Delete",
            IconKind::New => "New",
            IconKind::Save => "Save",
            IconKind::Compile => "Compile",
            IconKind::Cut => "Cut",
            IconKind::Copy => "Copy",
            IconKind::Paste => "Paste",
            IconKind::Undo => "Undo",
            IconKind::Find => "Find",
        }
    }
}

pub(crate) const ICON_TRANSPARENT_RGB: (u8, u8, u8) = (0xFF, 0x00, 0xFF);

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) struct PixelImage<const N: usize> {
    px: [[Option<kit::Color32>; N]; N],
}

#[cfg(test)]
pub(crate) type IconImage = PixelImage<ICON_PX>;

type CursorPixels = PixelImage<CURSOR_PX>;

pub(crate) fn snap_aabbcc_channel(v: u8) -> u8 {
    let step = ((v as u32 + 8) / 17).min(15);
    (step * 17) as u8
}

fn snap_aabbcc(c: kit::Color32) -> kit::Color32 {
    kit::Color32::from_rgb(
        snap_aabbcc_channel(c.r()),
        snap_aabbcc_channel(c.g()),
        snap_aabbcc_channel(c.b()),
    )
}

impl<const N: usize> PixelImage<N> {
    pub(crate) fn blank() -> Self {
        Self { px: [[None; N]; N] }
    }

    fn from_pixels(w: usize, h: usize, data: &[u8], channels: usize) -> Self {
        let mut px = [[None; N]; N];
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
                let sx = (x * w / N).min(w - 1);
                let sy = (y * h / N).min(h - 1);
                let (r, g, b, a) = at(sx, sy);
                if a < 128 || (r, g, b) == ICON_TRANSPARENT_RGB {
                    continue;
                }
                *cell = Some(snap_aabbcc(kit::Color32::from_rgb(r, g, b)));
            }
        }
        Self { px }
    }

    pub(crate) fn is_blank(&self) -> bool {
        self.px.iter().all(|row| row.iter().all(|p| p.is_none()))
    }

    pub(crate) fn paint(&self, painter: &kit::Painter, rect: kit::Rect) {
        let x0 = rect.min.x.round();
        let y0 = rect.min.y.round();
        let mut mesh = kit::Mesh::default();
        for row in 0..N {
            let mut col = 0usize;
            while col < N {
                let Some(px) = self.px[row][col] else {
                    col += 1;
                    continue;
                };
                let start = col;
                while col < N && self.px[row][col] == Some(px) {
                    col += 1;
                }
                mesh.add_colored_rect(
                    kit::Rect::from_min_size(
                        kit::pos2(x0 + start as f32, y0 + row as f32),
                        kit::vec2((col - start) as f32, LINE_W),
                    ),
                    px,
                );
            }
        }
        if !mesh.is_empty() {
            painter.add(kit::Shape::mesh(mesh));
        }
    }
}

#[cfg(test)]
#[derive(Clone)]
pub(crate) struct IconSet {
    icons: [IconImage; IconKind::COUNT],
}

#[cfg(test)]
impl Default for IconSet {
    fn default() -> Self {
        Self::blank()
    }
}

#[cfg(test)]
impl IconSet {
    pub(crate) fn blank() -> Self {
        Self {
            icons: [IconImage::blank(); IconKind::COUNT],
        }
    }

    pub(crate) fn load_or_create() -> Self {
        let dir = crate::platform::assets::icons_dir();
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
pub(crate) use crate::platform::assets::icons_dir;

pub(crate) fn decode_png_rgba(bytes: &[u8]) -> Option<kit::IconData> {
    let image = crate::engine::core::png::decode_rgba(bytes)?;
    Some(kit::IconData {
        rgba: image.rgba,
        width: image.width,
        height: image.height,
    })
}

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

    pub(crate) fn system_icon(self) -> kit::CursorIcon {
        match self {
            CursorKind::Move => kit::CursorIcon::Move,
            CursorKind::ResizeH => kit::CursorIcon::ResizeHorizontal,
            CursorKind::ResizeV => kit::CursorIcon::ResizeVertical,
            CursorKind::ResizeNwSe => kit::CursorIcon::ResizeNwSe,
            CursorKind::ResizeNeSw => kit::CursorIcon::ResizeNeSw,
            CursorKind::Text => kit::CursorIcon::Text,
            CursorKind::Normal
            | CursorKind::Left
            | CursorKind::Right
            | CursorKind::Slider
            | CursorKind::Wheel => kit::CursorIcon::Default,
        }
    }

    #[cfg(test)]
    pub(crate) fn label(self) -> &'static str {
        match self {
            CursorKind::Normal => "Normal",
            CursorKind::Left => "Left Button Down",
            CursorKind::Right => "Right Button Down",
            CursorKind::Move => "Move Window",
            CursorKind::ResizeH => "Resize Horizontally",
            CursorKind::ResizeV => "Resize Vertically",
            CursorKind::ResizeNwSe => "Resize Bottom-Right",
            CursorKind::ResizeNeSw => "Resize Bottom-Left",
            CursorKind::Slider => "Drag Slider",
            CursorKind::Wheel => "Scroll Wheel",
            CursorKind::Text => "Text Input",
        }
    }
}

pub(crate) struct CursorImage {
    img: CursorPixels,

    #[cfg(test)]
    cache: Option<std::sync::Arc<[u8]>>,
}

pub(crate) struct Cursors {
    imgs: [CursorImage; CursorKind::COUNT],
}

impl CursorImage {
    fn blank() -> Self {
        Self::new(CursorPixels::blank())
    }

    fn new(img: CursorPixels) -> Self {
        Self {
            img,
            #[cfg(test)]
            cache: None,
        }
    }

    #[cfg(test)]
    fn rgba_for(&mut self) -> std::sync::Arc<[u8]> {
        if let Some(rgba) = &self.cache {
            return rgba.clone();
        }
        let mut buf = vec![0u8; CURSOR_PX * CURSOR_PX * 4];
        for row in 0..CURSOR_PX {
            for col in 0..CURSOR_PX {
                let Some(px) = self.img.px[row][col] else {
                    continue;
                };
                let i = (row * CURSOR_PX + col) * 4;
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
    pub(crate) fn hotspot(&self, _kind: CursorKind) -> [u16; 2] {
        [CURSOR_CENTER as u16; 2]
    }

    pub(crate) fn load() -> Self {
        let dir = crate::platform::assets::cursors_dir();
        let mut imgs = std::array::from_fn(|_| CursorImage::blank());
        for kind in CursorKind::ALL {
            let path = dir.join(format!("{}.png", kind.file_stem()));
            let img = std::fs::read(&path)
                .ok()
                .and_then(|bytes| decode_cursor_png(&bytes))
                .filter(|img| !img.is_blank())
                .unwrap_or_else(CursorPixels::blank);
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

    fn img(&self, kind: CursorKind) -> &CursorPixels {
        let img = &self.imgs[kind.index()].img;
        if img.is_blank() {
            &self.imgs[CursorKind::Normal.index()].img
        } else {
            img
        }
    }

    #[cfg(test)]
    pub(crate) fn rgba(&mut self, kind: CursorKind) -> std::sync::Arc<[u8]> {
        self.imgs[kind.index()].rgba_for()
    }

    pub(crate) fn paint(&self, painter: &kit::Painter, tip: kit::Pos2, kind: CursorKind) {
        let [hx, hy] = self.hotspot(kind);
        let rect = kit::Rect::from_min_size(
            kit::pos2(tip.x.round() - hx as f32, tip.y.round() - hy as f32),
            kit::vec2(CURSOR_W, CURSOR_H),
        );
        self.img(kind).paint(painter, rect);
    }
}

#[cfg(test)]
pub(crate) fn decode_png(bytes: &[u8]) -> Option<IconImage> {
    decode_pixels(bytes)
}

fn decode_cursor_png(bytes: &[u8]) -> Option<CursorPixels> {
    decode_pixels(bytes)
}

fn decode_pixels<const N: usize>(bytes: &[u8]) -> Option<PixelImage<N>> {
    let image = crate::engine::core::png::decode_rgba(bytes)?;
    Some(PixelImage::from_pixels(
        image.width as usize,
        image.height as usize,
        &image.rgba,
        4,
    ))
}
