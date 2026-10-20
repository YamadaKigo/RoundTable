use crate::grid::LINE_W;
use crate::ui::icon::decode_png_rgba;
use eframe::egui;

pub(crate) const SYMBOL_PX: usize = 16;
pub(crate) const SYMBOL_W: f32 = SYMBOL_PX as f32;
pub(crate) const SYMBOL_H: f32 = SYMBOL_W;
pub(crate) const SYMBOL_TRANSPARENT_RGB: (u8, u8, u8) = (0xFF, 0x00, 0xFF);

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum SymbolKind {
    Minimize,
    Maximize,
    Restore,
    Close,
    Pin,
    Unpin,
    Exclusive,
    ExclusiveOn,
    PageUp,
    PageDown,
}

impl SymbolKind {
    pub(crate) const ALL: [SymbolKind; Self::COUNT] = [
        SymbolKind::Minimize,
        SymbolKind::Maximize,
        SymbolKind::Restore,
        SymbolKind::Close,
        SymbolKind::Pin,
        SymbolKind::Unpin,
        SymbolKind::Exclusive,
        SymbolKind::ExclusiveOn,
        SymbolKind::PageUp,
        SymbolKind::PageDown,
    ];

    pub(crate) const COUNT: usize = 10;

    pub(crate) fn file_stem(self) -> &'static str {
        match self {
            SymbolKind::Minimize => "minimize",
            SymbolKind::Maximize => "maximize",
            SymbolKind::Restore => "restore",
            SymbolKind::Close => "close",
            SymbolKind::Pin => "pin",
            SymbolKind::Unpin => "unpin",
            SymbolKind::Exclusive => "exclusive",
            SymbolKind::ExclusiveOn => "exclusive_on",
            SymbolKind::PageUp => "page_up",
            SymbolKind::PageDown => "page_down",
        }
    }

    fn index(self) -> usize {
        match self {
            SymbolKind::Minimize => 0,
            SymbolKind::Maximize => 1,
            SymbolKind::Restore => 2,
            SymbolKind::Close => 3,
            SymbolKind::Pin => 4,
            SymbolKind::Unpin => 5,
            SymbolKind::Exclusive => 6,
            SymbolKind::ExclusiveOn => 7,
            SymbolKind::PageUp => 8,
            SymbolKind::PageDown => 9,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct SymbolImage {
    mask: [bool; SYMBOL_PX * SYMBOL_PX],
}

impl SymbolImage {
    pub(crate) const fn blank() -> Self {
        Self {
            mask: [false; SYMBOL_PX * SYMBOL_PX],
        }
    }

    fn from_rgba(data: &egui::IconData) -> Option<Self> {
        if data.width as usize != SYMBOL_PX || data.height as usize != SYMBOL_PX {
            return None;
        }
        let mut mask = [false; SYMBOL_PX * SYMBOL_PX];
        for (i, px) in data.rgba.chunks_exact(4).enumerate() {
            let rgb = (px[0], px[1], px[2]);
            mask[i] = px[3] >= 128 && rgb != SYMBOL_TRANSPARENT_RGB;
        }
        Some(Self { mask })
    }

    #[cfg(test)]
    pub(crate) fn is_blank(&self) -> bool {
        !self.mask.iter().any(|on| *on)
    }

    pub(crate) fn paint(&self, painter: &egui::Painter, rect: egui::Rect, color: egui::Color32) {
        let x0 = rect.min.x.round();
        let y0 = rect.min.y.round();
        let mut mesh = egui::Mesh::default();
        for row in 0..SYMBOL_PX {
            let mut col = 0usize;
            while col < SYMBOL_PX {
                if !self.mask[row * SYMBOL_PX + col] {
                    col += 1;
                    continue;
                }
                let start = col;
                while col < SYMBOL_PX && self.mask[row * SYMBOL_PX + col] {
                    col += 1;
                }
                mesh.add_colored_rect(
                    egui::Rect::from_min_size(
                        egui::pos2(x0 + start as f32, y0 + row as f32),
                        egui::vec2((col - start) as f32, LINE_W),
                    ),
                    color,
                );
            }
        }
        if !mesh.is_empty() {
            painter.add(egui::Shape::mesh(mesh));
        }
    }
}

#[derive(Clone)]
pub(crate) struct SymbolSet {
    symbols: [SymbolImage; SymbolKind::COUNT],
}

impl SymbolSet {
    pub(crate) fn load() -> Self {
        let dir = crate::assets::symbols_dir();
        let symbols = SymbolKind::ALL.map(|kind| {
            std::fs::read(dir.join(format!("{}.png", kind.file_stem())))
                .ok()
                .and_then(|bytes| decode_png_rgba(&bytes))
                .and_then(|data| SymbolImage::from_rgba(&data))
                .unwrap_or_else(SymbolImage::blank)
        });
        Self { symbols }
    }

    pub(crate) fn get(&self, kind: SymbolKind) -> SymbolImage {
        self.symbols[kind.index()]
    }
}
