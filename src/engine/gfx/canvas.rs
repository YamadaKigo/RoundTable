use crate::engine::core::math::{Color32, Rect, pos2};
use crate::engine::kit::{ClippedPrimitive, ColorImage, Mesh, PaintLayer, Primitive, TextureId};
use std::collections::HashMap;

const DIRTY_TILE: usize = 32;
const MAX_DIRTY_RECTS: usize = 24;
const LAYER_COUNT: usize = 3;

#[derive(Default)]
struct LayerSurface {
    pixels: Vec<u32>,
    coverage: Vec<bool>,
    tile_hashes: Vec<u64>,
}

#[derive(Default)]
pub(crate) struct Canvas {
    width: usize,
    height: usize,
    pixels: Vec<u32>,
    dirty_tiles: Vec<bool>,
    dirty_width: usize,
    dirty_height: usize,
    all_dirty: bool,
    force_write: bool,
    mark_enabled: bool,
    alpha_write: bool,
    layers: [LayerSurface; LAYER_COUNT],
    layer_hashes: [Option<u64>; LAYER_COUNT],
    layer_renders: [u64; LAYER_COUNT],
    coverage_scratch: Vec<bool>,
    tile_hash_scratch: Vec<u64>,
    damage_scratch: Vec<bool>,
    primitive_hash_scratch: Vec<u64>,
    layer_primitives: [Vec<usize>; LAYER_COUNT],
}

impl Canvas {
    pub(crate) fn resize(&mut self, width: usize, height: usize) {
        if self.width == width && self.height == height {
            return;
        }
        self.width = width;
        self.height = height;
        self.pixels.resize(width.saturating_mul(height), 0);
        self.dirty_width = width.div_ceil(DIRTY_TILE);
        self.dirty_height = height.div_ceil(DIRTY_TILE);
        self.dirty_tiles.clear();
        self.dirty_tiles
            .resize(self.dirty_width.saturating_mul(self.dirty_height), true);
        let tile_count = self.dirty_width.saturating_mul(self.dirty_height);
        self.coverage_scratch.clear();
        self.coverage_scratch.resize(tile_count, false);
        self.tile_hash_scratch.clear();
        self.tile_hash_scratch.resize(tile_count, 0);
        self.damage_scratch.clear();
        self.damage_scratch.resize(tile_count, true);
        for layer in &mut self.layers {
            layer.pixels.clear();
            layer.pixels.resize(self.pixels.len(), 0);
            layer.coverage.clear();
            layer.coverage.resize(tile_count, false);
            layer.tile_hashes.clear();
            layer.tile_hashes.resize(tile_count, u64::MAX);
        }
        self.layer_hashes = [None; LAYER_COUNT];
        self.all_dirty = true;
        self.force_write = false;
        self.mark_enabled = true;
        self.alpha_write = false;
    }

    pub(crate) fn invalidate(&mut self) {
        self.dirty_tiles.fill(true);
        self.all_dirty = true;
        self.layer_hashes = [None; LAYER_COUNT];
        self.damage_scratch.fill(true);
        for layer in &mut self.layers {
            layer.tile_hashes.fill(u64::MAX);
        }
    }

    pub(crate) fn width(&self) -> usize {
        self.width
    }

    pub(crate) fn height(&self) -> usize {
        self.height
    }

    pub(crate) fn pixels(&self) -> &[u32] {
        &self.pixels
    }

    pub(crate) fn draw(
        &mut self,
        primitives: &[ClippedPrimitive],
        textures: &HashMap<TextureId, crate::engine::kit::ColorImage>,
    ) {
        let mut hashes = [0u64; LAYER_COUNT];
        let mut primitive_hashes = std::mem::take(&mut self.primitive_hash_scratch);
        primitive_hashes.clear();
        primitive_hashes.reserve(primitives.len());
        let mut layer_primitives = std::mem::take(&mut self.layer_primitives);
        for indices in &mut layer_primitives {
            indices.clear();
        }
        for primitive in primitives {
            let layer = layer_index(primitive.layer);
            let hash = primitive_hash(primitive);
            primitive_hashes.push(hash);
            hashes[layer] = combine_hash(hashes[layer], hash);
            layer_primitives[layer].push(primitive_hashes.len() - 1);
        }
        let mut changed = [false; LAYER_COUNT];
        for index in 0..LAYER_COUNT {
            changed[index] = self.layer_hashes[index] != Some(hashes[index]);
        }
        if !changed.iter().any(|value| *value) {
            self.primitive_hash_scratch = primitive_hashes;
            self.layer_primitives = layer_primitives;
            return;
        }
        for index in 0..LAYER_COUNT {
            if !changed[index] {
                continue;
            }
            self.layer_renders[index] += 1;
            let mut new_coverage = std::mem::take(&mut self.coverage_scratch);
            let mut new_hashes = std::mem::take(&mut self.tile_hash_scratch);
            layer_tile_state(
                primitives,
                (self.width, self.height, self.dirty_width),
                &mut new_coverage,
                &mut new_hashes,
                &primitive_hashes,
                &layer_primitives[index],
            );
            let mut damage_tiles = std::mem::take(&mut self.damage_scratch);
            for tile in 0..damage_tiles.len() {
                damage_tiles[tile] = self.layers[index].tile_hashes[tile] != new_hashes[tile];
            }
            let damage = tiles_to_rects(&damage_tiles, self.dirty_width, self.width, self.height);
            for area in &damage {
                clear_pixels(&mut self.layers[index].pixels, self.width, *area);
                self.render_layer(index, primitives, &layer_primitives[index], textures, *area);
                self.compose(*area);
            }
            self.coverage_scratch =
                std::mem::replace(&mut self.layers[index].coverage, new_coverage);
            self.tile_hash_scratch =
                std::mem::replace(&mut self.layers[index].tile_hashes, new_hashes);
            self.layer_hashes[index] = Some(hashes[index]);
            self.damage_scratch = damage_tiles;
        }
        self.primitive_hash_scratch = primitive_hashes;
        self.layer_primitives = layer_primitives;
    }

    fn render_layer(
        &mut self,
        index: usize,
        primitives: &[ClippedPrimitive],
        indices: &[usize],
        textures: &HashMap<TextureId, ColorImage>,
        damage: Rect,
    ) {
        std::mem::swap(&mut self.pixels, &mut self.layers[index].pixels);
        self.force_write = true;
        self.mark_enabled = false;
        self.alpha_write = true;
        self.draw_primitives(primitives, indices, textures, damage);
        self.mark_enabled = true;
        self.force_write = false;
        self.alpha_write = false;
        std::mem::swap(&mut self.pixels, &mut self.layers[index].pixels);
    }

    fn draw_primitives(
        &mut self,
        primitives: &[ClippedPrimitive],
        indices: &[usize],
        textures: &HashMap<TextureId, ColorImage>,
        damage: Rect,
    ) {
        for index in indices {
            let primitive = &primitives[*index];
            let Primitive::Mesh(mesh) = &primitive.primitive;
            let clip = if primitive.clip_rect.is_positive() {
                damage.intersect(primitive.clip_rect)
            } else {
                damage
            };
            if !clip.is_positive() {
                continue;
            }
            let texture = textures.get(&mesh.texture_id);
            if mesh.texture_id == TextureId(u64::MAX) {
                if self.draw_rects(mesh, clip, None) {
                    continue;
                }
            } else if let Some(image) = texture
                && self.draw_rects(mesh, clip, Some(image))
            {
                continue;
            }
            self.draw_triangles(mesh, clip, texture);
        }
    }

    fn compose(&mut self, area: Rect) {
        let Some((x0, x1, y0, y1)) = self.normalize_visible(area) else {
            return;
        };
        let mut changed = false;
        for y in y0..y1 {
            for x in x0..x1 {
                let index = y * self.width + x;
                let mut value = self.layers[0].pixels[index] & 0x00FF_FFFF;
                let chart = self.layers[1].pixels[index];
                if chart & 0xFF00_0000 != 0 {
                    value = chart & 0x00FF_FFFF;
                }
                let dynamic = self.layers[2].pixels[index];
                if dynamic & 0xFF00_0000 != 0 {
                    value = dynamic & 0x00FF_FFFF;
                }
                if self.all_dirty || self.pixels[index] != value {
                    self.pixels[index] = value;
                    changed = true;
                }
            }
        }
        if changed {
            self.mark((x0, x1, y0, y1));
        }
    }

    fn draw_rects(&mut self, mesh: &Mesh, clip: Rect, texture: Option<&ColorImage>) -> bool {
        if mesh.indices.is_empty() || mesh.indices.len() % 6 != 0 {
            return false;
        }
        for indices in mesh.indices.chunks_exact(6) {
            let Some(quad) = mesh_quad(mesh, indices) else {
                return false;
            };
            let rect = quad.rect;
            if let Some(image) = texture {
                self.fill_textured_rect(rect, quad.uv, quad.color, image, clip);
            } else {
                self.fill_rect(rect, quad.color, clip);
            }
        }
        true
    }

    fn draw_triangles(&mut self, mesh: &Mesh, clip: Rect, texture: Option<&ColorImage>) {
        for tri in mesh.indices.chunks_exact(3) {
            let Some(a) = mesh.vertices.get(tri[0] as usize) else {
                continue;
            };
            let Some(b) = mesh.vertices.get(tri[1] as usize) else {
                continue;
            };
            let Some(c) = mesh.vertices.get(tri[2] as usize) else {
                continue;
            };
            let a = *a;
            let b = *b;
            let c = *c;
            let area = edge(a.pos, b.pos, c.pos);
            if area == 0.0 {
                continue;
            }
            let left = a.pos.x.min(b.pos.x).min(c.pos.x).floor().max(clip.min.x);
            let right = a.pos.x.max(b.pos.x).max(c.pos.x).ceil().min(clip.max.x);
            let top = a.pos.y.min(b.pos.y).min(c.pos.y).floor().max(clip.min.y);
            let bottom = a.pos.y.max(b.pos.y).max(c.pos.y).ceil().min(clip.max.y);
            let left = left.max(0.0) as usize;
            let right = right.max(0.0) as usize;
            let top = top.max(0.0) as usize;
            let bottom = bottom.max(0.0) as usize;
            let mut dirty: Option<(usize, usize, usize, usize)> = None;
            for y in top..bottom.min(self.height) {
                for x in left..right.min(self.width) {
                    let point = pos2(x as f32 + 0.5, y as f32 + 0.5);
                    let weights = [
                        edge(b.pos, c.pos, point),
                        edge(c.pos, a.pos, point),
                        edge(a.pos, b.pos, point),
                    ];
                    let neg = weights.iter().any(|value| *value < 0.0);
                    let pos = weights.iter().any(|value| *value > 0.0);
                    if neg && pos {
                        continue;
                    }
                    let alpha = texture.is_none_or(|image| {
                        let u = (a.uv.x * weights[0] + b.uv.x * weights[1] + c.uv.x * weights[2])
                            / area;
                        let v = (a.uv.y * weights[0] + b.uv.y * weights[1] + c.uv.y * weights[2])
                            / area;
                        texture_alpha(image, u, v)
                    });
                    if alpha {
                        let value =
                            a.color.to_u32() | if self.alpha_write { 0xFF00_0000 } else { 0 };
                        if self.force_write || self.pixels[y * self.width + x] != value {
                            self.pixels[y * self.width + x] = value;
                            dirty = Some(dirty.map_or((x, x + 1, y, y + 1), |d| {
                                (d.0.min(x), d.1.max(x + 1), d.2.min(y), d.3.max(y + 1))
                            }));
                        }
                    }
                }
            }
            if let Some(dirty) = dirty {
                self.mark(dirty);
            }
        }
    }

    fn fill_textured_rect(
        &mut self,
        rect: Rect,
        uv: Rect,
        color: crate::engine::core::math::Color32,
        image: &ColorImage,
        clip: Rect,
    ) {
        let Some((x0, x1, y0, y1)) = self.normalize_visible(rect.intersect(clip)) else {
            return;
        };
        let width = rect.width();
        let height = rect.height();
        if width <= 0.0 || height <= 0.0 {
            return;
        }
        let value = color.to_u32() | if self.alpha_write { 0xFF00_0000 } else { 0 };
        let mut changed = false;
        let du = (uv.max.x - uv.min.x) / width;
        let dv = (uv.max.y - uv.min.y) / height;
        for y in y0..y1 {
            let v = uv.min.y + (y as f32 + 0.5 - rect.min.y) * dv;
            let mut u = uv.min.x + (x0 as f32 + 0.5 - rect.min.x) * du;
            for x in x0..x1 {
                if texture_alpha(image, u, v)
                    && (self.force_write || self.pixels[y * self.width + x] != value)
                {
                    self.pixels[y * self.width + x] = value;
                    changed = true;
                }
                u += du;
            }
        }
        if changed {
            self.mark((x0, x1, y0, y1));
        }
    }

    fn normalize_visible(&self, rect: Rect) -> Option<(usize, usize, usize, usize)> {
        let bounds = Rect::from_min_size(
            crate::engine::core::math::Pos2::ZERO,
            crate::engine::core::math::vec2(self.width as f32, self.height as f32),
        );
        normalize(rect.intersect(bounds))
    }

    pub(crate) fn fill_rect(&mut self, rect: Rect, color: Color32, clip: Rect) {
        let Some(rect) = self.normalize_visible(rect.intersect(clip)) else {
            return;
        };
        let value = color.to_u32() | if self.alpha_write { 0xFF00_0000 } else { 0 };
        let mut changed = false;
        for y in rect.2..rect.3 {
            let start = y * self.width + rect.0;
            let row = &mut self.pixels[start..start + rect.1 - rect.0];
            if !self.force_write && row.iter().all(|pixel| *pixel == value) {
                continue;
            }
            row.fill(value);
            changed = true;
        }
        if changed {
            self.mark(rect);
        }
    }

    #[cfg(test)]
    pub(crate) fn take_dirty(&mut self) -> Vec<Rect> {
        let mut out = Vec::new();
        self.take_dirty_into(&mut out);
        out
    }

    pub(crate) fn take_dirty_into(&mut self, out: &mut Vec<Rect>) {
        out.clear();
        if self.dirty_tiles.is_empty() || self.dirty_width == 0 {
            return;
        }
        for ty in 0..self.dirty_height {
            let mut tx = 0;
            while tx < self.dirty_width {
                let row = ty * self.dirty_width;
                if !self.dirty_tiles[row + tx] {
                    tx += 1;
                    continue;
                }
                let start = tx;
                while tx < self.dirty_width && self.dirty_tiles[row + tx] {
                    self.dirty_tiles[row + tx] = false;
                    tx += 1;
                }
                let x0 = start * DIRTY_TILE;
                let x1 = (tx * DIRTY_TILE).min(self.width);
                let y0 = ty * DIRTY_TILE;
                let y1 = ((ty + 1) * DIRTY_TILE).min(self.height);
                let rect =
                    Rect::from_min_max(pos2(x0 as f32, y0 as f32), pos2(x1 as f32, y1 as f32));
                if let Some(previous) = out.last_mut()
                    && previous.min.x == rect.min.x
                    && previous.max.x == rect.max.x
                    && previous.max.y as usize == y0
                {
                    previous.max.y = rect.max.y;
                } else {
                    out.push(rect);
                }
            }
        }
        self.all_dirty = false;
        if out.len() > MAX_DIRTY_RECTS {
            let mut union = out[0];
            for rect in &out[1..] {
                union = union.union(*rect);
            }
            out.clear();
            out.push(union);
        }
    }

    fn mark(&mut self, (x0, x1, y0, y1): (usize, usize, usize, usize)) {
        if !self.mark_enabled {
            return;
        }
        if self.dirty_width == 0 || self.dirty_height == 0 || x0 >= x1 || y0 >= y1 {
            return;
        }
        let tx0 = x0 / DIRTY_TILE;
        let tx1 = (x1 - 1) / DIRTY_TILE;
        let ty0 = y0 / DIRTY_TILE;
        let ty1 = (y1 - 1) / DIRTY_TILE;
        for ty in ty0..=ty1.min(self.dirty_height - 1) {
            let row = ty * self.dirty_width;
            self.dirty_tiles[row + tx0..=row + tx1.min(self.dirty_width - 1)].fill(true);
        }
    }

    #[cfg(test)]
    pub(crate) fn layer_render_counts(&self) -> [u64; LAYER_COUNT] {
        self.layer_renders
    }

    #[cfg(test)]
    pub(crate) fn layer_debug(&self, layer: PaintLayer, x: usize, y: usize) -> (u32, bool) {
        let index = layer_index(layer);
        let tile = (y / DIRTY_TILE) * self.dirty_width + x / DIRTY_TILE;
        (
            self.layers[index].pixels[y * self.width + x],
            self.layers[index]
                .coverage
                .get(tile)
                .copied()
                .unwrap_or(false),
        )
    }
}

struct Quad {
    rect: Rect,
    uv: Rect,
    color: crate::engine::core::math::Color32,
}

fn mesh_quad(mesh: &Mesh, indices: &[u32]) -> Option<Quad> {
    if indices.len() != 6 || indices[3] != indices[0] || indices[4] != indices[2] {
        return None;
    }
    let a = *mesh.vertices.get(indices[0] as usize)?;
    let b = *mesh.vertices.get(indices[1] as usize)?;
    let c = *mesh.vertices.get(indices[2] as usize)?;
    let d = *mesh.vertices.get(indices[5] as usize)?;
    let min = pos2(a.pos.x.min(c.pos.x), a.pos.y.min(c.pos.y));
    let max = pos2(a.pos.x.max(c.pos.x), a.pos.y.max(c.pos.y));
    let top_right = pos2(max.x, min.y);
    let bottom_left = pos2(min.x, max.y);
    if a.pos != min
        || b.pos != top_right
        || c.pos != max
        || d.pos != bottom_left
        || a.color != b.color
        || a.color != c.color
        || a.color != d.color
    {
        return None;
    }
    let uv = Rect::from_min_max(a.uv, c.uv);
    if b.uv != pos2(uv.max.x, uv.min.y) || d.uv != pos2(uv.min.x, uv.max.y) {
        return None;
    }
    Some(Quad {
        rect: Rect::from_min_max(min, max),
        uv,
        color: a.color,
    })
}

fn texture_alpha(image: &ColorImage, u: f32, v: f32) -> bool {
    if image.size[0] == 0 || image.size[1] == 0 {
        return false;
    }
    let sx = (u * image.size[0] as f32)
        .floor()
        .clamp(0.0, image.size[0].saturating_sub(1) as f32) as usize;
    let sy = (v * image.size[1] as f32)
        .floor()
        .clamp(0.0, image.size[1].saturating_sub(1) as f32) as usize;
    image[(sx, sy)].a() > 0
}

fn primitive_hash(primitive: &ClippedPrimitive) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325u64;
    let mut write = |value: u64| {
        hash ^= value;
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    };
    write(u64::from(primitive.clip_rect.min.x.to_bits()));
    write(u64::from(primitive.clip_rect.min.y.to_bits()));
    write(u64::from(primitive.clip_rect.max.x.to_bits()));
    write(u64::from(primitive.clip_rect.max.y.to_bits()));
    let Primitive::Mesh(mesh) = &primitive.primitive;
    write(mesh.texture_id.0);
    write(mesh.vertices.len() as u64);
    write(mesh.indices.len() as u64);
    write(mesh.content_hash());
    hash
}

fn combine_hash(hash: u64, value: u64) -> u64 {
    (hash ^ value).wrapping_mul(0x0000_0100_0000_01b3)
}

fn layer_index(layer: PaintLayer) -> usize {
    match layer {
        PaintLayer::Base => 0,
        PaintLayer::Chart => 1,
        PaintLayer::Dynamic => 2,
    }
}

fn layer_tile_state(
    primitives: &[ClippedPrimitive],
    dimensions: (usize, usize, usize),
    coverage: &mut [bool],
    hashes: &mut [u64],
    primitive_hashes: &[u64],
    indices: &[usize],
) {
    let (width, height, dirty_width) = dimensions;
    coverage.fill(false);
    hashes.fill(0);
    for index in indices {
        let primitive = &primitives[*index];
        let hash = primitive_hashes[*index];
        let Primitive::Mesh(mesh) = &primitive.primitive;
        let mut min = pos2(f32::INFINITY, f32::INFINITY);
        let mut max = pos2(f32::NEG_INFINITY, f32::NEG_INFINITY);
        for vertex in &mesh.vertices {
            min.x = min.x.min(vertex.pos.x);
            min.y = min.y.min(vertex.pos.y);
            max.x = max.x.max(vertex.pos.x);
            max.y = max.y.max(vertex.pos.y);
        }
        let mesh_bounds = Rect::from_min_max(min, max);
        if !mesh_bounds.is_positive() {
            continue;
        }
        let clipped = if primitive.clip_rect.is_positive() {
            mesh_bounds.intersect(primitive.clip_rect)
        } else {
            mesh_bounds
        };
        if clipped.is_positive() {
            let x0 = clipped.min.x.floor().max(0.0) as usize;
            let y0 = clipped.min.y.floor().max(0.0) as usize;
            let x1 = clipped.max.x.ceil().max(0.0) as usize;
            let y1 = clipped.max.y.ceil().max(0.0) as usize;
            if x0 >= x1 || y0 >= y1 {
                continue;
            }
            let tx0 = x0 / DIRTY_TILE;
            let tx1 = (x1 - 1).min(width.saturating_sub(1)) / DIRTY_TILE;
            let ty0 = y0 / DIRTY_TILE;
            let ty1 = (y1 - 1).min(height.saturating_sub(1)) / DIRTY_TILE;
            for ty in ty0..=ty1 {
                let row = ty * dirty_width;
                coverage[row + tx0..=row + tx1].fill(true);
                for tile in &mut hashes[row + tx0..=row + tx1] {
                    *tile = combine_hash(*tile, hash);
                }
            }
        }
    }
}

fn tiles_to_rects(tiles: &[bool], width_tiles: usize, width: usize, height: usize) -> Vec<Rect> {
    let height_tiles = height.div_ceil(DIRTY_TILE);
    let mut rects: Vec<Rect> = Vec::new();
    for ty in 0..height_tiles {
        let mut tx = 0;
        while tx < width_tiles {
            let row = ty * width_tiles;
            if !tiles[row + tx] {
                tx += 1;
                continue;
            }
            let start = tx;
            while tx < width_tiles && tiles[row + tx] {
                tx += 1;
            }
            let x0 = start * DIRTY_TILE;
            let x1 = (tx * DIRTY_TILE).min(width);
            let y0 = ty * DIRTY_TILE;
            let y1 = ((ty + 1) * DIRTY_TILE).min(height);
            let rect = Rect::from_min_max(pos2(x0 as f32, y0 as f32), pos2(x1 as f32, y1 as f32));
            if let Some(previous) = rects.last_mut()
                && previous.min.x == rect.min.x
                && previous.max.x == rect.max.x
                && previous.max.y as usize == y0
            {
                previous.max.y = rect.max.y;
            } else {
                rects.push(rect);
            }
        }
    }
    rects
}

fn clear_pixels(pixels: &mut [u32], width: usize, area: Rect) {
    let x0 = area.min.x.floor().max(0.0) as usize;
    let y0 = area.min.y.floor().max(0.0) as usize;
    let x1 = area.max.x.ceil().max(0.0) as usize;
    let y1 = area.max.y.ceil().max(0.0) as usize;
    for y in y0..y1 {
        pixels[y * width + x0..y * width + x1].fill(0);
    }
}

fn normalize(rect: Rect) -> Option<(usize, usize, usize, usize)> {
    if !rect.is_positive() {
        return None;
    }
    let x0 = rect.min.x.floor().max(0.0) as usize;
    let y0 = rect.min.y.floor().max(0.0) as usize;
    let x1 = rect.max.x.ceil().max(0.0) as usize;
    let y1 = rect.max.y.ceil().max(0.0) as usize;
    (x1 > x0 && y1 > y0).then_some((x0, x1, y0, y1))
}

fn edge(
    a: crate::engine::core::math::Pos2,
    b: crate::engine::core::math::Pos2,
    c: crate::engine::core::math::Pos2,
) -> f32 {
    (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unchanged_pixels_never_become_dirty() {
        let mut canvas = Canvas::default();
        canvas.resize(100, 100);
        let full = Rect::from_min_size(
            pos2(0.0, 0.0),
            crate::engine::core::math::vec2(100.0, 100.0),
        );
        assert_eq!(canvas.take_dirty(), vec![full]);
        let rect = Rect::from_min_size(
            pos2(10.0, 20.0),
            crate::engine::core::math::vec2(20.0, 10.0),
        );
        canvas.fill_rect(rect, Color32::from_rgb(1, 2, 3), full);
        assert_eq!(canvas.take_dirty().len(), 1);
        canvas.fill_rect(rect, Color32::from_rgb(1, 2, 3), full);
        assert!(canvas.take_dirty().is_empty());
    }

    #[test]
    fn distant_changes_keep_separate_dirty_rectangles() {
        let mut canvas = Canvas::default();
        canvas.resize(256, 128);
        let full = Rect::from_min_size(
            pos2(0.0, 0.0),
            crate::engine::core::math::vec2(256.0, 128.0),
        );
        let _ = canvas.take_dirty();
        canvas.fill_rect(
            Rect::from_min_size(pos2(2.0, 2.0), crate::engine::core::math::vec2(8.0, 8.0)),
            Color32::from_rgb(1, 2, 3),
            full,
        );
        canvas.fill_rect(
            Rect::from_min_size(
                pos2(220.0, 100.0),
                crate::engine::core::math::vec2(8.0, 8.0),
            ),
            Color32::from_rgb(1, 2, 3),
            full,
        );
        let dirty = canvas.take_dirty();
        assert_eq!(dirty.len(), 2);
        assert!(!dirty[0].contains(pos2(220.0, 100.0)));
        assert!(!dirty[1].contains(pos2(2.0, 2.0)));
    }

    #[test]
    fn semantic_layers_only_render_after_their_own_content_changes() {
        let ctx = crate::engine::kit::Context::default();
        let mut canvas = Canvas::default();
        canvas.resize(64, 64);
        let frame = |dynamic_x: f32| {
            ctx.run_ui(
                crate::engine::kit::RawInput {
                    screen_rect: Some(Rect::from_min_size(
                        pos2(0.0, 0.0),
                        crate::engine::core::math::vec2(64.0, 64.0),
                    )),
                    ..Default::default()
                },
                |ui| {
                    ui.painter().rect_filled(
                        Rect::from_min_size(
                            pos2(0.0, 0.0),
                            crate::engine::core::math::vec2(64.0, 64.0),
                        ),
                        crate::engine::kit::CornerRadius::ZERO,
                        Color32::from_rgb(0xEE, 0xEE, 0xEE),
                    );
                    ui.ctx().with_layer(PaintLayer::Chart, || {
                        ui.painter().rect_filled(
                            Rect::from_min_size(
                                pos2(10.0, 10.0),
                                crate::engine::core::math::vec2(30.0, 30.0),
                            ),
                            crate::engine::kit::CornerRadius::ZERO,
                            Color32::from_rgb(0x11, 0x22, 0x33),
                        );
                    });
                    ui.ctx().with_layer(PaintLayer::Dynamic, || {
                        ui.painter().rect_filled(
                            Rect::from_min_size(
                                pos2(dynamic_x, 40.0),
                                crate::engine::core::math::vec2(8.0, 8.0),
                            ),
                            crate::engine::kit::CornerRadius::ZERO,
                            Color32::from_rgb(0xAA, 0x00, 0x00),
                        );
                    });
                },
            )
        };
        let textures = HashMap::new();
        let first = frame(4.0);
        canvas.draw(&first.shapes, &textures);
        assert_eq!(
            canvas.layer_render_counts(),
            [1, 1, 1],
            "首帧必须建立三层分块缓存"
        );
        let second = frame(12.0);
        canvas.draw(&second.shapes, &textures);
        let counts = canvas.layer_render_counts();
        assert_eq!([counts[0], counts[1]], [1, 1]);
        let third = frame(20.0);
        canvas.draw(&third.shapes, &textures);
        assert_eq!(
            canvas.layer_render_counts(),
            [1, 1, 3],
            "只有动态层变化时不能重绘背景和Chart层"
        );
    }

    #[test]
    fn primitive_hash_is_incremental() {
        let mut mesh = Mesh::default();
        let empty = mesh.content_hash();
        mesh.add_colored_rect(
            Rect::from_min_size(pos2(1.0, 2.0), crate::engine::core::math::vec2(3.0, 4.0)),
            Color32::WHITE,
        );
        assert_ne!(mesh.content_hash(), empty);
        let first = mesh.content_hash();
        mesh.add_colored_rect(
            Rect::from_min_size(pos2(8.0, 9.0), crate::engine::core::math::vec2(2.0, 2.0)),
            Color32::BLACK,
        );
        assert_ne!(mesh.content_hash(), first);
    }
}
