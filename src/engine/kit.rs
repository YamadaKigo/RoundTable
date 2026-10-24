pub(crate) use crate::engine::core::math::{Color32, Pos2, Rect, Vec2, pos2, vec2};
use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::ops::Mul;
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering as AtomicOrdering};
use std::sync::{Arc, Mutex};

const CLICK_DRAG_SLOP_SQ: f32 = 16.0;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum Align {
    #[default]
    Min,
    Center,
    Max,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Align2(pub Align, pub Align);

impl Align2 {
    pub(crate) const LEFT_CENTER: Self = Self(Align::Min, Align::Center);
    pub(crate) const CENTER_CENTER: Self = Self(Align::Center, Align::Center);
    pub(crate) const RIGHT_CENTER: Self = Self(Align::Max, Align::Center);

    pub(crate) const fn x(self) -> Align {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct CornerRadius;

impl CornerRadius {
    pub(crate) const ZERO: Self = Self;
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct Margin;

impl Margin {
    pub(crate) const ZERO: Self = Self;
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct Shadow;

impl Shadow {
    pub(crate) const NONE: Self = Self;
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Stroke {
    pub width: f32,
    pub color: Color32,
}

impl Stroke {
    pub(crate) const fn new(width: f32, color: Color32) -> Self {
        Self { width, color }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum StrokeKind {
    #[default]
    Inside,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Sense {
    click: bool,
    drag: bool,
    focus: bool,
}

impl Sense {
    pub(crate) const fn hover() -> Self {
        Self {
            click: false,
            drag: false,
            focus: false,
        }
    }

    pub(crate) const fn click() -> Self {
        Self {
            click: true,
            drag: false,
            focus: true,
        }
    }

    #[cfg(test)]
    pub(crate) const fn drag() -> Self {
        Self {
            click: false,
            drag: true,
            focus: true,
        }
    }

    pub(crate) const fn click_and_drag() -> Self {
        Self {
            click: true,
            drag: true,
            focus: true,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub(crate) struct Id(u64);

impl Id {
    pub(crate) fn new(value: impl Hash) -> Self {
        let mut hasher = FastHasher::default();
        value.hash(&mut hasher);
        Self(hasher.finish())
    }

    pub(crate) fn with(self, suffix: impl Hash) -> Self {
        let mut hasher = FastHasher::default();
        self.hash(&mut hasher);
        suffix.hash(&mut hasher);
        Self(hasher.finish())
    }
}

struct FastHasher(u64);

impl Default for FastHasher {
    fn default() -> Self {
        Self(0xcbf2_9ce4_8422_2325)
    }
}

impl Hasher for FastHasher {
    fn finish(&self) -> u64 {
        self.0
    }

    fn write(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.0 ^= u64::from(*byte);
            self.0 = self.0.wrapping_mul(0x0000_0100_0000_01b3);
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum Key {
    #[default]
    A,
    C,
    S,
    X,
    Y,
    Z,
    Backspace,
    Delete,
    Enter,
    Escape,
    Tab,
    ArrowLeft,
    ArrowRight,
    ArrowUp,
    ArrowDown,
    Home,
    End,
    Space,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Modifiers {
    pub alt: bool,
    pub ctrl: bool,
    pub shift: bool,
    pub mac_cmd: bool,
    pub command: bool,
}

impl Modifiers {
    pub(crate) const NONE: Self = Self {
        alt: false,
        ctrl: false,
        shift: false,
        mac_cmd: false,
        command: false,
    };
    #[cfg(test)]
    pub(crate) const CTRL: Self = Self {
        alt: false,
        ctrl: true,
        shift: false,
        mac_cmd: false,
        command: true,
    };
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum PointerButton {
    #[default]
    Primary,
    Secondary,
    Middle,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum MouseWheelUnit {
    #[default]
    Point,
    Line,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum ImeEvent {
    Preedit {
        text: String,
        active_range_chars: Option<std::ops::Range<usize>>,
    },
    Commit(String),
    #[cfg(test)]
    DeleteSurrounding {
        before_chars: usize,
        after_chars: usize,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Event {
    PointerMoved(Pos2),
    PointerButton {
        pos: Pos2,
        button: PointerButton,
        pressed: bool,
        modifiers: Modifiers,
    },
    MouseMoved(Vec2),
    MouseWheel {
        unit: MouseWheelUnit,
        delta: Vec2,
        modifiers: Modifiers,
    },
    Key {
        key: Key,
        physical_key: Option<Key>,
        pressed: bool,
        repeat: bool,
        modifiers: Modifiers,
    },
    Text(String),
    Paste(String),
    Ime(ImeEvent),
    WindowFocused(bool),
}

#[derive(Clone, Debug, Default)]
pub(crate) struct RawInput {
    pub screen_rect: Option<Rect>,
    pub time: Option<f64>,
    pub events: Vec<Event>,
    pub modifiers: Modifiers,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct PointerState {
    latest: Option<Pos2>,
    interact: Option<Pos2>,
    press_origin: Option<Pos2>,
    primary_down: bool,
    secondary_down: bool,
    primary_pressed: bool,
    primary_released: bool,
    primary_clicked: bool,
    secondary_pressed: bool,
    secondary_released: bool,
    secondary_clicked: bool,
    double_clicked: bool,
    last_click: Option<(f64, Pos2)>,
    press_seq: u64,
    primary_press_owner: Option<Id>,
    secondary_press_owner: Option<Id>,
    has_hit_regions: bool,
}

impl PointerState {
    pub(crate) fn latest_pos(&self) -> Option<Pos2> {
        self.latest
    }

    pub(crate) fn hover_pos(&self) -> Option<Pos2> {
        self.latest
    }

    pub(crate) fn interact_pos(&self) -> Option<Pos2> {
        self.interact
    }

    pub(crate) fn press_origin(&self) -> Option<Pos2> {
        self.press_origin
    }

    pub(crate) fn primary_down(&self) -> bool {
        self.primary_down
    }

    pub(crate) fn secondary_down(&self) -> bool {
        self.secondary_down
    }

    pub(crate) fn primary_pressed(&self) -> bool {
        self.primary_pressed
    }

    pub(crate) fn primary_clicked(&self) -> bool {
        self.primary_clicked
    }

    pub(crate) fn secondary_pressed(&self) -> bool {
        self.secondary_pressed
    }

    pub(crate) fn secondary_clicked(&self) -> bool {
        self.secondary_clicked
    }

    pub(crate) fn button_double_clicked(&self, button: PointerButton) -> bool {
        button == PointerButton::Primary && self.double_clicked
    }
}

#[derive(Clone, Debug, Default)]
pub(crate) struct InputState {
    pub time: f64,
    pub events: Vec<Event>,
    pub modifiers: Modifiers,
    pub pointer: PointerState,
    pub screen_rect: Rect,
    pub last_scroll_time: f64,
}

impl InputState {
    pub(crate) fn key_pressed(&self, key: Key) -> bool {
        self.events.iter().any(|event| {
            matches!(
                event,
                Event::Key {
                    key: actual,
                    pressed: true,
                    ..
                } if *actual == key
            )
        })
    }

    pub(crate) fn time_since_last_scroll(&self) -> f64 {
        if self.last_scroll_time == 0.0 {
            f64::INFINITY
        } else {
            self.time - self.last_scroll_time
        }
    }

    pub(crate) fn content_rect(&self) -> Rect {
        self.screen_rect
    }
}

#[derive(Default)]
pub(crate) struct Memory {
    focus: Option<Id>,
    drags: HashSet<(u64, Id)>,
    active: Option<Id>,
    secondary_active: Option<Id>,
    hit_regions: Vec<HitRegion>,
}

#[derive(Clone, Copy)]
struct HitRegion {
    id: Id,
    rect: Rect,
    sequence: u64,
    enabled: bool,
}

impl Memory {
    pub(crate) fn request_focus(&mut self, id: Id) {
        self.focus = Some(id);
    }

    pub(crate) fn surrender_focus(&mut self, id: Id) {
        if self.focus == Some(id) {
            self.focus = None;
        }
    }

    pub(crate) fn has_focus(&self, id: Id) -> bool {
        self.focus == Some(id)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum CursorIcon {
    #[default]
    Default,
    None,
    Text,
    Move,
    ResizeVertical,
    ResizeHorizontal,
    ResizeNwSe,
    ResizeNeSw,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct IMEOutput {
    pub rect: Rect,
    pub cursor_rect: Rect,
    pub should_interrupt_composition: bool,
}

pub(crate) mod output {
    pub(crate) use super::IMEOutput;
}

#[derive(Clone, Debug, Default)]
pub(crate) struct Output {
    pub cursor_icon: CursorIcon,
    pub commands: Vec<ViewportCommand>,
    pub ime: Option<IMEOutput>,
    pub copied_text: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ViewportCommand {
    Close,
    Minimized(bool),
}

#[derive(Clone, Debug, Default)]
pub(crate) struct PlatformOutput {
    pub cursor_icon: CursorIcon,
    pub commands: Vec<ViewportCommand>,
    pub ime: Option<IMEOutput>,
    pub copied_text: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(crate) struct TextureId(pub u64);

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct TextureOptions;

impl TextureOptions {
    pub(crate) const NEAREST: Self = Self;
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ColorImage {
    pub(crate) size: [usize; 2],
    pub(crate) pixels: Vec<Color32>,
}

impl ColorImage {
    pub(crate) fn filled(size: [usize; 2], color: Color32) -> Self {
        Self {
            size,
            pixels: vec![color; size[0] * size[1]],
        }
    }
}

impl std::ops::Index<(usize, usize)> for ColorImage {
    type Output = Color32;

    fn index(&self, index: (usize, usize)) -> &Self::Output {
        &self.pixels[index.1 * self.size[0] + index.0]
    }
}

impl std::ops::IndexMut<(usize, usize)> for ColorImage {
    fn index_mut(&mut self, index: (usize, usize)) -> &mut Self::Output {
        &mut self.pixels[index.1 * self.size[0] + index.0]
    }
}

#[derive(Clone, Debug)]
pub(crate) enum ImageData {
    Color(Arc<ColorImage>),
}

#[derive(Clone, Debug)]
pub(crate) struct ImageDelta {
    pub image: ImageData,
    pub pos: Option<[usize; 2]>,
}

#[derive(Clone, Default, Debug)]
pub(crate) struct TexturesDelta {
    pub set: HashMap<TextureId, Vec<ImageDelta>>,
    pub free: Vec<TextureId>,
}

impl TexturesDelta {
    #[cfg(test)]
    pub(crate) fn clear(&mut self) {
        self.set.clear();
        self.free.clear();
    }
}

#[derive(Clone)]
pub(crate) struct TextureHandle {
    id: TextureId,
    state: Arc<Mutex<TextureState>>,
}

struct TextureState {
    image: ColorImage,
    pending: Vec<ImageDelta>,
    full_dirty: bool,
}

impl TextureHandle {
    pub(crate) fn id(&self) -> TextureId {
        self.id
    }

    pub(crate) fn set_partial(&mut self, pos: [usize; 2], patch: ColorImage, _: TextureOptions) {
        let mut state = self.state.lock().unwrap();
        let image = &mut state.image;
        for y in 0..patch.size[1] {
            for x in 0..patch.size[0] {
                let dx = pos[0] + x;
                let dy = pos[1] + y;
                if dx < image.size[0] && dy < image.size[1] {
                    image[(dx, dy)] = patch[(x, y)];
                }
            }
        }
        if !state.full_dirty {
            state.pending.push(ImageDelta {
                image: ImageData::Color(Arc::new(patch)),
                pos: Some(pos),
            });
        }
    }

    pub(crate) fn clear_transparent(&mut self) {
        let mut state = self.state.lock().unwrap();
        state.image.pixels.fill(Color32::TRANSPARENT);
        state.pending.clear();
        state.full_dirty = true;
    }
}

#[derive(Clone, Debug, Default)]
pub(crate) struct IconData {
    pub rgba: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct Vertex {
    pub pos: Pos2,
    pub uv: Pos2,
    pub color: Color32,
}

#[derive(Clone, Debug)]
pub(crate) struct Mesh {
    pub texture_id: TextureId,
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    content_hash: u64,
}

impl Default for Mesh {
    fn default() -> Self {
        Self {
            texture_id: TextureId(u64::MAX),
            vertices: Vec::new(),
            indices: Vec::new(),
            content_hash: mesh_hash_seed(),
        }
    }
}

impl Mesh {
    pub(crate) fn with_texture(texture_id: TextureId) -> Self {
        Self {
            texture_id,
            vertices: Vec::new(),
            indices: Vec::new(),
            content_hash: mesh_hash_seed(),
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.indices.is_empty()
    }

    pub(crate) fn add_colored_rect(&mut self, rect: Rect, color: Color32) {
        self.add_rect_with_uv(
            rect,
            Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
            color,
        );
    }

    pub(crate) fn add_rect_with_uv(&mut self, rect: Rect, uv: Rect, color: Color32) {
        let base = self.vertices.len() as u32;
        self.vertices.extend([
            Vertex {
                pos: rect.min,
                uv: uv.min,
                color,
            },
            Vertex {
                pos: Pos2::new(rect.max.x, rect.min.y),
                uv: Pos2::new(uv.max.x, uv.min.y),
                color,
            },
            Vertex {
                pos: rect.max,
                uv: uv.max,
                color,
            },
            Vertex {
                pos: Pos2::new(rect.min.x, rect.max.y),
                uv: Pos2::new(uv.min.x, uv.max.y),
                color,
            },
        ]);
        self.indices
            .extend([base, base + 1, base + 2, base, base + 2, base + 3]);
        let mut hash = self.content_hash;
        for value in [
            u64::from(rect.min.x.to_bits()),
            u64::from(rect.min.y.to_bits()),
            u64::from(rect.max.x.to_bits()),
            u64::from(rect.max.y.to_bits()),
            u64::from(uv.min.x.to_bits()),
            u64::from(uv.min.y.to_bits()),
            u64::from(uv.max.x.to_bits()),
            u64::from(uv.max.y.to_bits()),
            u64::from(color.to_u32()),
            u64::from(base),
        ] {
            hash = mesh_hash_write(hash, value);
        }
        self.content_hash = hash;
    }

    pub(crate) fn content_hash(&self) -> u64 {
        self.content_hash
    }

    pub(crate) fn add_hard_line(&mut self, from: Pos2, to: Pos2, width: f32, color: Color32) {
        add_hard_line(self, from, to, width, color);
    }
}

const fn mesh_hash_seed() -> u64 {
    0xcbf2_9ce4_8422_2325
}

const fn mesh_hash_write(hash: u64, value: u64) -> u64 {
    (hash ^ value).wrapping_mul(0x0000_0100_0000_01b3)
}

#[derive(Clone, Debug)]
pub(crate) enum Shape {
    Mesh(Mesh),
    LineSegment { points: [Pos2; 2], stroke: Stroke },
    StrokeRect { rect: Rect, stroke: Stroke },
    FilledRect { rect: Rect, color: Color32 },
}

impl Shape {
    pub(crate) fn mesh(mesh: Mesh) -> Self {
        Self::Mesh(mesh)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Order {
    #[default]
    Middle,
    Tooltip,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum PaintLayer {
    #[default]
    Base,
    Chart,
    Dynamic,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct LayerId {
    pub order: Order,
    pub id: Id,
}

impl LayerId {
    pub(crate) fn new(order: Order, id: Id) -> Self {
        Self { order, id }
    }
}

#[derive(Clone, Debug, Default)]
pub(crate) struct ClippedPrimitive {
    pub clip_rect: Rect,
    pub primitive: Primitive,
    pub layer: PaintLayer,
}

#[derive(Clone, Debug)]
pub(crate) enum Primitive {
    Mesh(Mesh),
}

impl Default for Primitive {
    fn default() -> Self {
        Self::Mesh(Mesh::default())
    }
}

#[cfg(test)]
pub(crate) mod epaint {
    pub(crate) use super::Primitive;
}

#[derive(Clone, Debug, Default)]
pub(crate) struct FullOutput {
    pub shapes: Vec<ClippedPrimitive>,
    pub textures_delta: TexturesDelta,
    pub platform_output: PlatformOutput,
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Transform;

impl Mul<Rect> for Transform {
    type Output = Rect;

    fn mul(self, rect: Rect) -> Rect {
        rect
    }
}

#[derive(Default)]
pub(crate) struct Spacing {
    pub item_spacing: Vec2,
    pub window_margin: Margin,
    pub menu_margin: Margin,
    pub button_padding: Vec2,
    pub indent: f32,
    pub interact_size: Vec2,
    pub slider_width: f32,
    pub slider_rail_height: f32,
    pub combo_width: f32,
    pub combo_height: f32,
    pub text_edit_width: f32,
    pub menu_width: f32,
    pub menu_spacing: f32,
    pub tooltip_width: f32,
    pub extra_text_line_spacing: f32,
    pub icon_width: f32,
    pub icon_width_inner: f32,
    pub icon_spacing: f32,
    pub default_area_size: Vec2,
}

#[derive(Default)]
pub(crate) struct Interaction {
    pub interact_radius: f32,
}

#[derive(Default)]
pub(crate) struct Visuals {
    pub window_corner_radius: CornerRadius,
    pub menu_corner_radius: CornerRadius,
    pub window_shadow: Shadow,
    pub disabled_alpha: f32,
}

#[derive(Default)]
pub(crate) struct Style {
    pub spacing: Spacing,
    pub interaction: Interaction,
    pub visuals: Visuals,
}

#[derive(Default)]
pub(crate) struct TessellationOptions {
    pub feathering: bool,
    pub round_text_to_pixels: bool,
    pub round_line_segments_to_pixels: bool,
    pub round_rects_to_pixels: bool,
}

struct Frame {
    shapes: Vec<(Order, u64, Shape, Rect, PaintLayer)>,
    output: Output,
    input: InputState,
    memory: Memory,
    sequence: u64,
    layer: PaintLayer,
    hit_regions: Vec<HitRegion>,
    hit_sequence: u64,
}

type TextureMap = HashMap<TextureId, Arc<Mutex<TextureState>>>;

pub(crate) struct Context {
    frame: Rc<RefCell<Frame>>,
    textures: Arc<Mutex<TextureMap>>,
    next_texture: Arc<AtomicU64>,
    time: Rc<Cell<f64>>,
}

impl Default for Context {
    fn default() -> Self {
        Self {
            frame: Rc::new(RefCell::new(Frame {
                shapes: Vec::new(),
                output: Output::default(),
                input: InputState::default(),
                memory: Memory::default(),
                sequence: 0,
                layer: PaintLayer::Base,
                hit_regions: Vec::new(),
                hit_sequence: 0,
            })),
            textures: Arc::new(Mutex::new(HashMap::new())),
            next_texture: Arc::new(AtomicU64::new(1)),
            time: Rc::new(Cell::new(0.0)),
        }
    }
}

impl Clone for Context {
    fn clone(&self) -> Self {
        Self {
            frame: Rc::clone(&self.frame),
            textures: self.textures.clone(),
            next_texture: self.next_texture.clone(),
            time: Rc::clone(&self.time),
        }
    }
}

impl Context {
    pub(crate) fn all_styles_mut(&self, mut f: impl FnMut(&mut Style)) {
        let mut style = Style::default();
        f(&mut style);
    }

    pub(crate) fn tessellation_options_mut(&self, mut f: impl FnMut(&mut TessellationOptions)) {
        let mut options = TessellationOptions::default();
        f(&mut options);
    }

    pub(crate) fn input<R>(&self, f: impl FnOnce(&InputState) -> R) -> R {
        f(&self.frame.borrow().input)
    }

    pub(crate) fn take_input_events(&self) -> Vec<Event> {
        std::mem::take(&mut self.frame.borrow_mut().input.events)
    }

    pub(crate) fn memory<R>(&self, f: impl FnOnce(&Memory) -> R) -> R {
        f(&self.frame.borrow().memory)
    }

    pub(crate) fn memory_mut<R>(&self, f: impl FnOnce(&mut Memory) -> R) -> R {
        f(&mut self.frame.borrow_mut().memory)
    }

    pub(crate) fn output_mut<R>(&self, f: impl FnOnce(&mut Output) -> R) -> R {
        f(&mut self.frame.borrow_mut().output)
    }

    pub(crate) fn send_viewport_cmd(&self, command: ViewportCommand) {
        self.output_mut(|output| output.commands.push(command));
    }

    pub(crate) fn set_cursor_icon(&self, icon: CursorIcon) {
        self.output_mut(|output| output.cursor_icon = icon);
    }

    pub(crate) fn copy_text(&self, text: String) {
        self.output_mut(|output| output.copied_text = Some(text));
    }

    pub(crate) fn layer_transform_to_global(&self, _layer: LayerId) -> Option<Transform> {
        Some(Transform)
    }

    pub(crate) fn layer_painter(&self, layer: LayerId) -> Painter {
        Painter::new(self.clone(), Rect::NOTHING, layer.order)
    }

    pub(crate) fn with_layer<R>(&self, layer: PaintLayer, run: impl FnOnce() -> R) -> R {
        struct Guard {
            frame: Rc<RefCell<Frame>>,
            previous: PaintLayer,
        }

        impl Drop for Guard {
            fn drop(&mut self) {
                self.frame.borrow_mut().layer = self.previous;
            }
        }

        let previous = self.frame.borrow().layer;
        self.frame.borrow_mut().layer = layer;
        let _guard = Guard {
            frame: Rc::clone(&self.frame),
            previous,
        };
        run()
    }

    pub(crate) fn current_layer(&self) -> PaintLayer {
        self.frame.borrow().layer
    }

    pub(crate) fn load_texture(
        &self,
        _name: impl Into<String>,
        image: ColorImage,
        _options: TextureOptions,
    ) -> TextureHandle {
        let id = TextureId(self.next_texture.fetch_add(1, AtomicOrdering::Relaxed));
        let state = Arc::new(Mutex::new(TextureState {
            image,
            pending: Vec::new(),
            full_dirty: true,
        }));
        self.textures.lock().unwrap().insert(id, state.clone());
        TextureHandle { id, state }
    }

    pub(crate) fn run_ui(&self, raw: RawInput, run: impl FnOnce(&mut Ui)) -> FullOutput {
        self.prepare(raw);
        let mut ui = Ui::new(self.clone(), self.input(|input| input.screen_rect));
        run(&mut ui);
        self.commit_hit_regions();
        self.finish()
    }

    #[cfg(test)]
    pub(crate) fn tessellate(
        &self,
        shapes: Vec<ClippedPrimitive>,
        _scale: f32,
    ) -> Vec<ClippedPrimitive> {
        shapes
    }

    fn prepare(&self, raw: RawInput) {
        let mut frame = self.frame.borrow_mut();
        frame.shapes.clear();
        frame.output = Output::default();
        frame.hit_regions.clear();
        frame.hit_sequence = 0;
        if let Some(raw_time) = raw.time {
            self.time.set(raw_time);
        } else {
            self.time.set(self.time.get() + 0.016);
        }
        let mut input = std::mem::take(&mut frame.input);
        input.time = self.time.get();
        input.events = raw.events;
        input.modifiers = raw.modifiers;
        input.screen_rect = raw
            .screen_rect
            .unwrap_or(Rect::from_min_size(Pos2::ZERO, Vec2::new(800.0, 600.0)));
        input.pointer.primary_pressed = false;
        input.pointer.primary_released = false;
        input.pointer.primary_clicked = false;
        input.pointer.secondary_pressed = false;
        input.pointer.secondary_released = false;
        input.pointer.secondary_clicked = false;
        input.pointer.double_clicked = false;
        input.pointer.primary_press_owner = None;
        input.pointer.secondary_press_owner = None;

        for event in &input.events {
            match event {
                Event::PointerMoved(pos) => {
                    input.pointer.latest = Some(*pos);
                    input.pointer.interact = Some(*pos);
                    if input
                        .pointer
                        .last_click
                        .is_some_and(|(_, at)| at.distance(*pos) > 4.0)
                    {
                        input.pointer.last_click = None;
                    }
                }
                Event::PointerButton {
                    pos,
                    button,
                    pressed,
                    ..
                } => {
                    input.pointer.latest = Some(*pos);
                    input.pointer.interact = Some(*pos);
                    match (button, pressed) {
                        (PointerButton::Primary, true) => {
                            input.pointer.primary_down = true;
                            input.pointer.primary_pressed = true;
                            input.pointer.press_origin = Some(*pos);
                            input.pointer.press_seq += 1;
                            if let Some((last, at)) = input.pointer.last_click {
                                input.pointer.double_clicked =
                                    input.time - last <= 0.5 && at.distance(*pos) <= 4.0;
                            }
                            input.pointer.last_click = Some((input.time, *pos));
                        }
                        (PointerButton::Primary, false) => {
                            input.pointer.primary_down = false;
                            input.pointer.primary_released = true;
                            input.pointer.primary_clicked = true;
                        }
                        (PointerButton::Secondary, true) => {
                            input.pointer.secondary_down = true;
                            input.pointer.secondary_pressed = true;
                        }
                        (PointerButton::Secondary, false) => {
                            input.pointer.secondary_down = false;
                            input.pointer.secondary_released = true;
                            input.pointer.secondary_clicked = true;
                        }
                        _ => {}
                    }
                }
                Event::WindowFocused(false) => {
                    input.pointer.primary_down = false;
                    input.pointer.secondary_down = false;
                    input.pointer.press_origin = None;
                }
                Event::MouseWheel { .. } => {
                    input.last_scroll_time = input.time;
                }
                _ => {}
            }
        }
        input.pointer.has_hit_regions = !frame.memory.hit_regions.is_empty();
        if input.pointer.primary_pressed
            && let Some(at) = input.pointer.interact
        {
            let owner = frame
                .memory
                .hit_regions
                .iter()
                .filter(|hit| hit.enabled && hit.rect.contains(at))
                .max_by_key(|hit| hit.sequence)
                .map(|hit| hit.id);
            input.pointer.primary_press_owner = owner;
            frame.memory.active = owner;
        }
        if input.pointer.secondary_pressed
            && let Some(at) = input.pointer.interact
        {
            let owner = frame
                .memory
                .hit_regions
                .iter()
                .filter(|hit| hit.enabled && hit.rect.contains(at))
                .max_by_key(|hit| hit.sequence)
                .map(|hit| hit.id);
            input.pointer.secondary_press_owner = owner;
            frame.memory.secondary_active = owner;
        }
        frame.input = input;
    }

    fn commit_hit_regions(&self) {
        let mut frame = self.frame.borrow_mut();
        let mut hit_regions = std::mem::take(&mut frame.hit_regions);
        frame.memory.hit_regions.clear();
        frame.memory.hit_regions.append(&mut hit_regions);
    }

    fn finish(&self) -> FullOutput {
        let mut frame = self.frame.borrow_mut();
        frame.shapes.sort_by_key(|entry| (entry.0, entry.1));
        let shapes = frame
            .shapes
            .drain(..)
            .flat_map(|(_, _, shape, clip, layer)| shape_to_primitives(shape, clip, layer))
            .collect();

        let mut set = HashMap::new();
        for (id, texture) in self.textures.lock().unwrap().iter() {
            let mut texture = texture.lock().unwrap();
            if texture.full_dirty {
                texture.full_dirty = false;
                texture.pending.clear();
                set.insert(
                    *id,
                    vec![ImageDelta {
                        image: ImageData::Color(Arc::new(texture.image.clone())),
                        pos: None,
                    }],
                );
            } else if !texture.pending.is_empty() {
                let pending = std::mem::take(&mut texture.pending);
                set.insert(*id, pending);
            }
        }
        FullOutput {
            shapes,
            textures_delta: TexturesDelta {
                set,
                free: Vec::new(),
            },
            platform_output: PlatformOutput {
                cursor_icon: frame.output.cursor_icon,
                commands: frame.output.commands.clone(),
                ime: frame.output.ime.clone(),
                copied_text: frame.output.copied_text.clone(),
            },
        }
    }

    fn push_shape(
        &self,
        order: Order,
        shape: Shape,
        clip: Rect,
        painter_layer: Option<PaintLayer>,
    ) {
        let mut frame = self.frame.borrow_mut();
        frame.sequence += 1;
        let sequence = frame.sequence;
        let layer = painter_layer.unwrap_or(frame.layer);
        frame.shapes.push((order, sequence, shape, clip, layer));
    }

    fn register_hit(&self, id: Id, rect: Rect, enabled: bool) {
        let mut frame = self.frame.borrow_mut();
        frame.hit_sequence += 1;
        let sequence = frame.hit_sequence;
        frame.hit_regions.push(HitRegion {
            id,
            rect,
            sequence,
            enabled,
        });
    }
}

#[derive(Clone)]
pub(crate) struct Painter {
    context: Context,
    clip: Rect,
    order: Order,
    layer: Option<PaintLayer>,
}

impl Painter {
    fn new(context: Context, clip: Rect, order: Order) -> Self {
        Self {
            context,
            clip,
            order,
            layer: None,
        }
    }

    pub(crate) fn rect_filled(&self, rect: Rect, _radius: CornerRadius, color: Color32) {
        self.context.push_shape(
            self.order,
            Shape::FilledRect { rect, color },
            self.clip,
            self.layer,
        );
    }

    pub(crate) fn rect_stroke(
        &self,
        rect: Rect,
        _radius: CornerRadius,
        stroke: Stroke,
        _kind: StrokeKind,
    ) {
        self.context.push_shape(
            self.order,
            Shape::StrokeRect { rect, stroke },
            self.clip,
            self.layer,
        );
    }

    pub(crate) fn line_segment(&self, points: [Pos2; 2], stroke: Stroke) {
        self.context.push_shape(
            self.order,
            Shape::LineSegment { points, stroke },
            self.clip,
            self.layer,
        );
    }

    pub(crate) fn add(&self, shape: Shape) {
        self.context
            .push_shape(self.order, shape, self.clip, self.layer);
    }

    pub(crate) fn with_clip_rect(&self, rect: Rect) -> Self {
        let clip = if self.clip.is_positive() {
            self.clip.intersect(rect)
        } else {
            rect
        };
        Self {
            context: self.context.clone(),
            clip,
            order: self.order,
            layer: self.layer,
        }
    }

    pub(crate) fn with_layer<R>(&self, layer: PaintLayer, run: impl FnOnce() -> R) -> R {
        self.context.with_layer(layer, run)
    }

    pub(crate) fn paint_layer(&self, layer: PaintLayer) -> Self {
        Self {
            context: self.context.clone(),
            clip: self.clip,
            order: self.order,
            layer: Some(layer),
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Response {
    hovered: bool,
    clicked: bool,
    pressed: bool,
    secondary_clicked: bool,
    down: bool,
    drag_started: bool,
    dragged: bool,
    #[cfg(test)]
    drag_stopped: bool,
}

impl Response {
    pub(crate) fn hovered(&self) -> bool {
        self.hovered
    }

    pub(crate) fn clicked(&self) -> bool {
        self.clicked
    }

    pub(crate) fn pressed(&self) -> bool {
        self.pressed
    }

    pub(crate) fn secondary_clicked(&self) -> bool {
        self.secondary_clicked
    }

    pub(crate) fn is_pointer_button_down_on(&self) -> bool {
        self.down
    }

    pub(crate) fn drag_started(&self) -> bool {
        self.drag_started
    }

    pub(crate) fn dragged(&self) -> bool {
        self.dragged
    }

    #[cfg(test)]
    pub(crate) fn drag_stopped(&self) -> bool {
        self.drag_stopped
    }
}

pub(crate) struct Ui {
    context: Context,
    painter: Painter,
    max_rect: Rect,
    enabled: bool,
    layer: LayerId,
    interaction_salt: u64,
}

impl Ui {
    fn new(context: Context, max_rect: Rect) -> Self {
        let layer = LayerId::default();
        Self {
            painter: Painter::new(context.clone(), Rect::NOTHING, Order::Middle),
            context,
            max_rect,
            enabled: true,
            layer,
            interaction_salt: 0,
        }
    }

    pub(crate) fn ctx(&self) -> &Context {
        &self.context
    }

    pub(crate) fn painter(&self) -> &Painter {
        &self.painter
    }

    pub(crate) fn max_rect(&self) -> Rect {
        self.max_rect
    }

    pub(crate) fn layer_id(&self) -> LayerId {
        self.layer
    }

    pub(crate) fn input<R>(&self, f: impl FnOnce(&InputState) -> R) -> R {
        self.context.input(f)
    }

    pub(crate) fn memory<R>(&self, f: impl FnOnce(&Memory) -> R) -> R {
        self.context.memory(f)
    }

    pub(crate) fn memory_mut<R>(&self, f: impl FnOnce(&mut Memory) -> R) -> R {
        self.context.memory_mut(f)
    }

    pub(crate) fn output_mut<R>(&self, f: impl FnOnce(&mut Output) -> R) -> R {
        self.context.output_mut(f)
    }

    pub(crate) fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub(crate) fn interaction_salt(&self) -> u64 {
        self.interaction_salt
    }

    pub(crate) fn set_interaction_salt(&mut self, salt: u64) {
        self.interaction_salt = salt;
    }

    pub(crate) fn disable(&mut self) {
        self.enabled = false;
    }

    pub(crate) fn scope<R>(&mut self, run: impl FnOnce(&mut Ui) -> R) -> R {
        let mut child = Ui {
            context: self.context.clone(),
            painter: self.painter.clone(),
            max_rect: self.max_rect,
            enabled: self.enabled,
            layer: self.layer,
            interaction_salt: self.interaction_salt,
        };
        run(&mut child)
    }

    pub(crate) fn interact(&mut self, rect: Rect, id: Id, sense: Sense) -> Response {
        self.context.register_hit(id, rect, self.enabled);
        let (
            hovered,
            primary_pressed,
            primary_down,
            primary_released,
            primary_clicked,
            secondary_pressed,
            secondary_released,
            secondary_clicked,
            primary_owner,
            secondary_owner,
            has_hit_regions,
        ) = self.context.input(|input| {
            (
                input.pointer.hover_pos().is_some_and(|p| rect.contains(p)),
                input.pointer.primary_pressed(),
                input.pointer.primary_down(),
                input.pointer.primary_released,
                input.pointer.primary_clicked(),
                input.pointer.secondary_pressed(),
                input.pointer.secondary_released,
                input.pointer.secondary_clicked(),
                input.pointer.primary_press_owner,
                input.pointer.secondary_press_owner,
                input.pointer.has_hit_regions,
            )
        });
        let mut active = self.context.memory(|memory| memory.active);
        if self.enabled
            && hovered
            && primary_pressed
            && (sense.click || sense.drag)
            && (primary_owner == Some(id)
                || (!has_hit_regions && active.is_none())
                || (primary_owner.is_none() && active.is_none()))
        {
            active = Some(id);
            self.context.memory_mut(|memory| memory.active = Some(id));
        }
        if self.enabled
            && hovered
            && secondary_pressed
            && sense.click
            && (!has_hit_regions || secondary_owner.is_none())
        {
            self.context
                .memory_mut(|memory| memory.secondary_active = Some(id));
        }
        let owns = active == Some(id);
        let secondary_owns = self
            .context
            .memory(|memory| memory.secondary_active == Some(id));
        let pointer = self
            .context
            .input(|input| input.pointer.interact_pos().or(input.pointer.latest_pos()));
        let press_origin = self.context.input(|input| input.pointer.press_origin());
        let press_seq = self.context.input(|input| input.pointer.press_seq);
        let in_press = press_origin.is_some_and(|p| rect.contains(p));
        let moved = press_origin
            .is_some_and(|p| pointer.is_some_and(|at| (at - p).length_sq() > CLICK_DRAG_SLOP_SQ));
        let dragged_before_release = self
            .context
            .memory(|memory| memory.drags.contains(&(press_seq, id)));
        let drag_started = self.enabled
            && sense.drag
            && owns
            && primary_down
            && in_press
            && moved
            && self
                .context
                .memory_mut(|memory| memory.drags.insert((press_seq, id)));
        let drag_stopped = self.enabled
            && sense.drag
            && owns
            && primary_released
            && self
                .context
                .memory_mut(|memory| memory.drags.remove(&(press_seq, id)));
        #[cfg(not(test))]
        let _ = drag_stopped;
        let down = self.enabled && sense.click && owns && primary_down && in_press;
        let clicked = self.enabled
            && sense.click
            && owns
            && primary_clicked
            && hovered
            && !moved
            && !dragged_before_release
            && in_press;
        if owns && primary_released {
            self.context.memory_mut(|memory| memory.active = None);
        }
        if secondary_owns && secondary_released {
            self.context
                .memory_mut(|memory| memory.secondary_active = None);
        }
        Response {
            hovered: self.enabled && hovered,
            clicked,
            pressed: self.enabled && sense.click && owns && primary_pressed,
            secondary_clicked: self.enabled
                && sense.click
                && secondary_clicked
                && hovered
                && secondary_owns,
            down,
            drag_started,
            dragged: self.enabled && sense.drag && owns && primary_down && in_press && moved,
            #[cfg(test)]
            drag_stopped,
        }
    }

    pub(crate) fn allocate_rect(&mut self, rect: Rect, sense: Sense) -> Response {
        self.interact(rect, Id::new(("allocate", bits(rect))), sense)
    }
}

fn bits(rect: Rect) -> (u32, u32, u32, u32) {
    (
        rect.min.x.to_bits(),
        rect.min.y.to_bits(),
        rect.max.x.to_bits(),
        rect.max.y.to_bits(),
    )
}

fn shape_to_primitives(shape: Shape, clip: Rect, layer: PaintLayer) -> Vec<ClippedPrimitive> {
    match shape {
        Shape::Mesh(mesh) => vec![ClippedPrimitive {
            clip_rect: clip,
            primitive: Primitive::Mesh(mesh),
            layer,
        }],
        Shape::FilledRect { rect, color } => {
            let mut mesh = Mesh::default();
            mesh.add_colored_rect(rect, color);
            vec![ClippedPrimitive {
                clip_rect: clip,
                primitive: Primitive::Mesh(mesh),
                layer,
            }]
        }
        Shape::LineSegment { points, stroke } => {
            let mut mesh = Mesh::default();
            let thickness = stroke.width.max(1.0);
            add_hard_line(&mut mesh, points[0], points[1], thickness, stroke.color);
            vec![ClippedPrimitive {
                clip_rect: clip,
                primitive: Primitive::Mesh(mesh),
                layer,
            }]
        }
        Shape::StrokeRect { rect, stroke } => {
            let mut mesh = Mesh::default();
            let w = stroke.width.max(1.0);
            mesh.add_colored_rect(
                Rect::from_min_max(rect.min, pos2(rect.max.x, rect.min.y + w)),
                stroke.color,
            );
            mesh.add_colored_rect(
                Rect::from_min_max(pos2(rect.min.x, rect.max.y - w), rect.max),
                stroke.color,
            );
            mesh.add_colored_rect(
                Rect::from_min_max(rect.min, pos2(rect.min.x + w, rect.max.y)),
                stroke.color,
            );
            mesh.add_colored_rect(
                Rect::from_min_max(pos2(rect.max.x - w, rect.min.y), rect.max),
                stroke.color,
            );
            vec![ClippedPrimitive {
                clip_rect: clip,
                primitive: Primitive::Mesh(mesh),
                layer,
            }]
        }
    }
}

fn add_hard_line(mesh: &mut Mesh, from: Pos2, to: Pos2, thickness: f32, color: Color32) {
    let mut x = from.x.round() as i32;
    let mut y = from.y.round() as i32;
    let end_x = to.x.round() as i32;
    let end_y = to.y.round() as i32;
    let dx = (end_x - x).abs();
    let sx = if x < end_x { 1 } else { -1 };
    let dy = -(end_y - y).abs();
    let sy = if y < end_y { 1 } else { -1 };
    let mut error = dx + dy;
    let horizontal = dx >= -dy;
    let (initial_major, initial_minor) = if horizontal { (x, y) } else { (y, x) };
    let mut run_start = initial_major;
    let mut run_end = initial_major;
    let mut run_line = initial_minor;
    loop {
        let (major, minor) = if horizontal { (x, y) } else { (y, x) };
        if minor != run_line {
            add_hard_run(
                mesh, run_start, run_end, run_line, horizontal, thickness, color,
            );
            run_start = major;
            run_end = major;
            run_line = minor;
        } else {
            run_end = major;
        }
        if x == end_x && y == end_y {
            break;
        }
        let twice = error * 2;
        if twice >= dy {
            error += dy;
            x += sx;
        }
        if twice <= dx {
            error += dx;
            y += sy;
        }
    }
    add_hard_run(
        mesh, run_start, run_end, run_line, horizontal, thickness, color,
    );
}

fn add_hard_run(
    mesh: &mut Mesh,
    start: i32,
    end: i32,
    line: i32,
    horizontal: bool,
    thickness: f32,
    color: Color32,
) {
    let low = start.min(end) as f32;
    let high = start.max(end) as f32 + 1.0;
    let rect = if horizontal {
        Rect::from_min_max(pos2(low, line as f32), pos2(high, line as f32 + thickness))
    } else {
        Rect::from_min_max(pos2(line as f32, low), pos2(line as f32 + thickness, high))
    };
    mesh.add_colored_rect(rect, color);
}
