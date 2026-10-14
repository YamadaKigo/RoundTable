// ===========================================================================
//  模块 ui::icon：像素图标（16x16，从 PNG 读进来再自绘）
// ===========================================================================
//  图标不再写死在代码里，而是一张一张的 16x16 PNG，放在 assets/icons/<名字>.png：
//    - 换图标不用改代码：拿画图工具改盘上那张 PNG，重启就生效；
//    - #ff00ff（洋红）是透明色，别的像素一律按它自己的颜色画；
//    - 颜色统一吸到最近的 0x11 倍数（也就是 #aabbcc 形，和 theme 的色值规矩同一条）；
//    - 白色墨迹（#ffffff）画的时候换成 theme().text，所以同一份图标在经典 / 暗色两套
//      皮肤下都看得清；要固定颜色就在 PNG 里直接用别的色。
//  「读图片」只读到「哪个像素有墨、什么颜色」为止，画出去还是一格一格 1px 的方块
//  （整数像素、零抗锯齿）- 和全工程其它自绘元素一个口径。
//  assets/icons 里那张不在 / 读不动 / 画出来是空的，就退回代码里那份内置点阵
//  （IconKind::art，纯 ASCII 的 '#' 与 '.'），并把内置那份写回盘上，目录永远是齐的。
//
//  图标一律画在 16x16（2 格内容）的方框里、只用整数像素：没有抗锯齿，也不会有半像素落点。
//  宽度常量和间隔常量也放在这儿：图标区 + 间隔凑成整数格，后面跟文字时才落在内容格起点上。
use crate::grid::*;
use crate::theme::*;
use eframe::egui;


/// 图标方框的边长：2 个内容格 = 16px。
pub(crate) const ICON_W: f32 = 2.0 * GRID;

/// 图标区高度：和宽度一样是 2 个内容格 = 16px（方框）。
pub(crate) const ICON_H: f32 = ICON_W;

/// 图标点阵的行列数。
pub(crate) const ICON_PX: usize = 16;

/// 图标和后面那串文字之间的间隔：图标 + 间隔凑成 3 格（27px），文字才会落在内容格起点上。
pub(crate) const ICON_GAP: f32 = 3.0 * PITCH - ICON_W;

/// 手绘图标表：都画在 16x16 的方框里，纯像素、无抗锯齿。
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum IconKind {
    /// 叉（关闭 / 删除 / 重置）。
    Cross,
    /// 新建（一张纸 + 一个加号）。
    New,
    /// 保存（软盘）。
    Save,
    /// 编译（齿轮）。
    Compile,
    /// 剪切（剪刀）。
    Cut,
    /// 复制（两张纸）。
    Copy,
    /// 粘贴（剪贴板）。
    Paste,
    /// 撤销（向左的回转箭头）。
    Undo,
    /// 查找（放大镜）。
    Find,
}

impl IconKind {
    /// 全部图标：下标就是 index()，装图标 / 遍历都走它。
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

    /// 图标个数：数组长度和地方都取它，省得写死两处。
    pub(crate) const COUNT: usize = 9;

    /// 在 ALL 里的下标（装进 IconSet 用的就是它）。
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

    /// 对应的图片文件名（assets/icons/<名字>.png）。
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
    /// 图标长什么样：16 行、每行 16 个字符（'#' = 有墨、'.' = 空）。
    pub(crate) fn art(self) -> [&'static str; ICON_PX] {
        match self {
            IconKind::Cross => [
                "................",
                "................",
                "..##........##..",
                "..###......###..",
                "...###....###...",
                "....###..###....",
                ".....######.....",
                "......####......",
                "......####......",
                ".....######.....",
                "....###..###....",
                "...###....###...",
                "..###......###..",
                "..##........##..",
                "................",
                "................",
            ],
            IconKind::New => [
                "................",
                "...##########...",
                "...#........#...",
                "...#...##...#...",
                "...#...##...#...",
                "...#..####..#...",
                "...#..####..#...",
                "...#...##...#...",
                "...#...##...#...",
                "...#........#...",
                "...#........#...",
                "...##########...",
                "................",
                "................",
                "................",
                "................",
            ],
            IconKind::Save => [
                "................",
                ".##############.",
                ".#............#.",
                ".#.####..####.#.",
                ".#.####..####.#.",
                ".#.####..####.#.",
                ".#............#.",
                ".#............#.",
                ".#.##########.#.",
                ".#.#........#.#.",
                ".#.#........#.#.",
                ".#.#........#.#.",
                ".#.##########.#.",
                ".#............#.",
                ".##############.",
                "................",
            ],
            IconKind::Compile => [
                "................",
                "....#......#....",
                "....#.####.#....",
                "....##....##....",
                "...###....###...",
                ".####..##..####.",
                ".##..######..##.",
                ".##..######..##.",
                ".####..##..####.",
                "...###....###...",
                "....##....##....",
                "....#.####.#....",
                "....#......#....",
                "................",
                "................",
                "................",
            ],
            IconKind::Cut => [
                "................",
                "..##........##..",
                ".#..#......#..#.",
                ".#..#......#..#.",
                ".#..#......#..#.",
                "..##.......###..",
                "...##.....##....",
                "....##...##.....",
                ".....##.##......",
                "......###.......",
                "......###.......",
                ".....##.##......",
                "....##...##.....",
                "...##.....##....",
                "..##.......###..",
                "................",
            ],
            IconKind::Copy => [
                "................",
                "................",
                "..######..######",
                "..#....#..#....#",
                "..#....#..#....#",
                "..#....#..#....#",
                "..#....#..#....#",
                "..#....#..#....#",
                "..#....#..#....#",
                "..#....#..#....#",
                "..######..######",
                "................",
                "................",
                "................",
                "................",
                "................",
            ],
            IconKind::Paste => [
                "................",
                ".....######.....",
                ".....######.....",
                "...##########...",
                "...#........#...",
                "...#........#...",
                "...#........#...",
                "...#........#...",
                "...#........#...",
                "...#........#...",
                "...#........#...",
                "...##########...",
                "................",
                "................",
                "................",
                "................",
            ],
            IconKind::Undo => [
                "................",
                "................",
                "...###..........",
                "..#####.........",
                ".#######........",
                "########........",
                ".#######........",
                "..#####.........",
                "...###..######..",
                "........#.....#.",
                "........#.....#.",
                "........#.....#.",
                ".........#...#..",
                "..........###...",
                "................",
                "................",
            ],
            IconKind::Find => [
                "................",
                "................",
                ".....####.......",
                "....#....#......",
                "...##....##.....",
                "...#......#.....",
                "...##....##.....",
                "....#....#......",
                ".....####.##....",
                "..........###...",
                "...........###..",
                "............###.",
                ".............###",
                "..............##",
                "................",
                "................",
            ],
        }
    }

    /// 图标的名字（状态栏回显 / 以后做悬浮提示用）。
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
// ===========================================================================
//  二、图标图片：16x16 点阵，从 PNG 读进来再自绘
// ===========================================================================

/// PNG 里的透明色：#ff00ff（洋红）。整张图里凡是这个颜色的像素都算没墨。
pub(crate) const ICON_TRANSPARENT_RGB: (u8, u8, u8) = (0xFF, 0x00, 0xFF);

/// 白色墨迹：画的时候换成当前皮肤的文字色（见 IconImage::paint）。
const ICON_INK_FOLLOWS_THEME: egui::Color32 = egui::Color32::WHITE;

/// 一张 16x16 的图标：每个像素要么透明（None）、要么一个颜色。
/// 颜色一律是 #aabbcc 形（每个通道都是 0x11 的倍数）。
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) struct IconImage {
    px: [[Option<egui::Color32>; ICON_PX]; ICON_PX],
}

/// 把一个颜色通道吸到最近的 0x11 倍数（#aabbcc 规矩）：0x00 / 0x11 / ... / 0xFF 共十七档。
pub(crate) fn snap_aabbcc_channel(v: u8) -> u8 {
    let step = ((v as u32 + 8) / 17).min(15);
    (step * 17) as u8
}

/// 一个颜色整体吸到 #aabbcc 形。
fn snap_aabbcc(c: egui::Color32) -> egui::Color32 {
    egui::Color32::from_rgb(
        snap_aabbcc_channel(c.r()),
        snap_aabbcc_channel(c.g()),
        snap_aabbcc_channel(c.b()),
    )
}

impl IconImage {
    /// 一张全透明的图标（数组初始化用）。
    pub(crate) fn blank() -> Self {
        Self {
            px: [[None; ICON_PX]; ICON_PX],
        }
    }

    /// 由内置点阵建图：'#' 是白色墨迹（画的时候跟随主题文字色），别的字符都是没墨。
    pub(crate) fn from_art(art: &[&str; ICON_PX]) -> Self {
        let mut px = [[None; ICON_PX]; ICON_PX];
        for (row, line) in art.iter().enumerate() {
            for (col, ch) in line.chars().take(ICON_PX).enumerate() {
                if ch == '#' {
                    px[row][col] = Some(ICON_INK_FOLLOWS_THEME);
                }
            }
        }
        Self { px }
    }

    /// 由一串 8 位像素建图：先按「#ff00ff / 半透明 = 透明」筛，剩下的颜色吸到 #aabbcc 形，
    /// 再最近邻缩放到 16x16（源图不是 16x16 也收 - 缩着放进来，不挑食）。
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
        for y in 0..ICON_PX {
            for x in 0..ICON_PX {
                let sx = (x * w / ICON_PX).min(w - 1);
                let sy = (y * h / ICON_PX).min(h - 1);
                let (r, g, b, a) = at(sx, sy);
                if a < 128 || (r, g, b) == ICON_TRANSPARENT_RGB {
                    continue;
                }
                px[y][x] = Some(snap_aabbcc(egui::Color32::from_rgb(r, g, b)));
            }
        }
        Self { px }
    }

    /// 整张图一个墨点都没有（读到的图不能用时的判据）。
    pub(crate) fn is_blank(&self) -> bool {
        self.px.iter().all(|row| row.iter().all(|p| p.is_none()))
    }

    /// 画在给定矩形里：落点吸到整数像素，同一行相邻的同色像素并成一条 1px 长条
    /// （比逐像素填方块省图形，画出来一模一样）。白色墨迹换成当前皮肤的文字色。
    pub(crate) fn paint(&self, painter: &egui::Painter, rect: egui::Rect) {
        let x0 = rect.min.x.round();
        let y0 = rect.min.y.round();
        let themed = theme().text;
        let resolved = |c: egui::Color32| if c == ICON_INK_FOLLOWS_THEME { themed } else { c };
        for (row, line) in self.px.iter().enumerate() {
            let mut col = 0usize;
            while col < ICON_PX {
                let Some(c) = line[col] else {
                    col += 1;
                    continue;
                };
                let color = resolved(c);
                let start = col;
                while col < ICON_PX && line[col].map(resolved) == Some(color) {
                    col += 1;
                }
                painter.rect_filled(
                    egui::Rect::from_min_size(
                        egui::pos2(x0 + start as f32, y0 + row as f32),
                        egui::vec2((col - start) as f32, LINE_W),
                    ),
                    egui::CornerRadius::ZERO,
                    color,
                );
            }
        }
    }

    /// 导出成 PNG 用的 RGBA 像素：透明处写 #ff00ff（约定的透明色），白色墨迹原样写白。
    fn to_rgba(&self) -> Vec<u8> {
        let (tr, tg, tb) = ICON_TRANSPARENT_RGB;
        let mut out = Vec::with_capacity(ICON_PX * ICON_PX * 4);
        for row in &self.px {
            for cell in row {
                match cell {
                    Some(c) => out.extend_from_slice(&[c.r(), c.g(), c.b(), 255]),
                    None => out.extend_from_slice(&[tr, tg, tb, 255]),
                }
            }
        }
        out
    }
}

/// 全部图标：开机装一次。先读 assets/icons/<名字>.png；读不到就把内置点阵写一份出去再用。
#[derive(Clone)]
pub(crate) struct IconSet {
    icons: [IconImage; IconKind::COUNT],
}

impl Default for IconSet {
    fn default() -> Self {
        Self::builtin()
    }
}

impl IconSet {
    /// 只用代码里那份内置点阵，不碰磁盘（测试用；读盘失败也退到这儿）。
    pub(crate) fn builtin() -> Self {
        let mut icons = [IconImage::blank(); IconKind::COUNT];
        for kind in IconKind::ALL {
            icons[kind.index()] = IconImage::from_art(&kind.art());
        }
        Self { icons }
    }

    /// 开机装图标：assets/icons 里那张读得到就用它；读不到 / 是空的就按内置点阵写一份出来再用。
    pub(crate) fn load_or_create() -> Self {
        let dir = icons_dir();
        let _ = std::fs::create_dir_all(&dir);
        let mut icons = [IconImage::blank(); IconKind::COUNT];
        for kind in IconKind::ALL {
            let path = dir.join(format!("{}.png", kind.file_stem()));
            let from_disk = std::fs::read(&path).ok().and_then(|bytes| decode_png(&bytes));
            let img = match from_disk {
                Some(img) if !img.is_blank() => img,
                _ => {
                    let fallback = IconImage::from_art(&kind.art());
                    let _ = write_icon_png(&path, &fallback);
                    fallback
                }
            };
            icons[kind.index()] = img;
        }
        Self { icons }
    }

    /// 取一张图标（下标就是 IconKind::index()）。
    pub(crate) fn get(&self, kind: IconKind) -> &IconImage {
        &self.icons[kind.index()]
    }
}

/// 图标目录：仓库根下的 assets/icons（和源码一个口径，看 CARGO_MANIFEST_DIR）。
pub(crate) fn icons_dir() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("assets")
        .join("icons")
}

/// 解一张 PNG：只认 8 位灰度 / 真彩（含 alpha）；调色板 / 低位深 / 16 位交给 png 自己展开。
/// 读不动就 None（调用方退回内置点阵）。
fn decode_png(bytes: &[u8]) -> Option<IconImage> {
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
        // 展开之后不该再见到调色板：留着这一支只是为了 match 写全。
        png::ColorType::Indexed => return None,
    };
    Some(IconImage::from_pixels(
        info.width as usize,
        info.height as usize,
        &buf,
        channels,
    ))
}

/// 把一张 IconImage 写成一个 16x16 的 PNG（默认图标就这么落盘的）。
pub(crate) fn write_icon_png(path: &std::path::Path, img: &IconImage) -> std::io::Result<()> {
    let file = std::fs::File::create(path)?;
    let mut enc = png::Encoder::new(std::io::BufWriter::new(file), ICON_PX as u32, ICON_PX as u32);
    enc.set_color(png::ColorType::Rgba);
    enc.set_depth(png::BitDepth::Eight);
    let mut writer = enc
        .write_header()
        .map_err(|e| std::io::Error::other(e.to_string()))?;
    writer
        .write_image_data(&img.to_rgba())
        .map_err(|e| std::io::Error::other(e.to_string()))?;
    writer
        .finish()
        .map_err(|e| std::io::Error::other(e.to_string()))?;
    Ok(())
}
