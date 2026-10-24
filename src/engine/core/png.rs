#[derive(Clone, Debug)]
pub(crate) struct DecodedPng {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

pub(crate) fn decode_rgba(bytes: &[u8]) -> Option<DecodedPng> {
    if bytes.get(..8)? != b"\x89PNG\r\n\x1a\n" {
        return None;
    }
    let mut at = 8usize;
    let mut width = 0u32;
    let mut height = 0u32;
    let mut depth = 0u8;
    let mut color_type = 0u8;
    let mut interlace = 0u8;
    let mut compressed = Vec::new();
    while at + 12 <= bytes.len() {
        let length = u32::from_be_bytes(bytes.get(at..at + 4)?.try_into().ok()?) as usize;
        let kind = bytes.get(at + 4..at + 8)?;
        let data = bytes.get(at + 8..at + 8 + length)?;
        at += 12 + length;
        match kind {
            b"IHDR" => {
                if data.len() != 13 {
                    return None;
                }
                width = u32::from_be_bytes(data.get(0..4)?.try_into().ok()?);
                height = u32::from_be_bytes(data.get(4..8)?.try_into().ok()?);
                depth = data[8];
                color_type = data[9];
                interlace = data[12];
            }
            b"IDAT" => compressed.extend_from_slice(data),
            b"IEND" => break,
            _ => {}
        }
    }
    if width == 0 || height == 0 || depth != 8 || interlace != 0 {
        return None;
    }
    let channels = match color_type {
        0 => 1,
        2 => 3,
        4 => 2,
        6 => 4,
        _ => return None,
    };
    let raw = inflate_zlib(&compressed)?;
    let stride = width as usize * channels;
    let expected = (stride + 1) * height as usize;
    if raw.len() < expected {
        return None;
    }
    let mut pixels = vec![0u8; stride * height as usize];
    for y in 0..height as usize {
        let filter = raw[y * (stride + 1)];
        let src = &raw[y * (stride + 1) + 1..(y + 1) * (stride + 1)];
        let prev = if y == 0 {
            None
        } else {
            Some(pixels[(y - 1) * stride..y * stride].to_vec())
        };
        let mut dst = vec![0u8; stride];
        for x in 0..stride {
            let left = if x >= channels { dst[x - channels] } else { 0 };
            let up = prev.as_ref().map_or(0, |row| row[x]);
            let up_left = if x >= channels {
                prev.as_ref().map_or(0, |row| row[x - channels])
            } else {
                0
            };
            dst[x] = match filter {
                0 => src[x],
                1 => src[x].wrapping_add(left),
                2 => src[x].wrapping_add(up),
                3 => src[x].wrapping_add(((left as u16 + up as u16) / 2) as u8),
                4 => src[x].wrapping_add(paeth(left, up, up_left)),
                _ => return None,
            };
        }
        pixels[y * stride..(y + 1) * stride].copy_from_slice(&dst);
    }
    let mut rgba = Vec::with_capacity(width as usize * height as usize * 4);
    for px in pixels.chunks_exact(channels) {
        match color_type {
            0 => rgba.extend_from_slice(&[px[0], px[0], px[0], 0xFF]),
            2 => rgba.extend_from_slice(&[px[0], px[1], px[2], 0xFF]),
            4 => rgba.extend_from_slice(&[px[0], px[0], px[0], px[1]]),
            6 => rgba.extend_from_slice(px),
            _ => return None,
        }
    }
    Some(DecodedPng {
        width,
        height,
        rgba,
    })
}

fn paeth(a: u8, b: u8, c: u8) -> u8 {
    let p = a as i32 + b as i32 - c as i32;
    let pa = (p - a as i32).abs();
    let pb = (p - b as i32).abs();
    let pc = (p - c as i32).abs();
    if pa <= pb && pa <= pc {
        a
    } else if pb <= pc {
        b
    } else {
        c
    }
}

fn inflate_zlib(data: &[u8]) -> Option<Vec<u8>> {
    if data.len() < 6 {
        return None;
    }
    let cmf = data[0];
    let flg = data[1];
    if cmf & 0x0F != 8 || ((cmf as u16) << 8 | flg as u16) % 31 != 0 {
        return None;
    }
    let mut bits = Bits {
        data: &data[2..data.len() - 4],
        at: 0,
    };
    let mut out = Vec::new();
    loop {
        let final_block = bits.read(1)? != 0;
        match bits.read(2)? {
            0 => stored(&mut bits, &mut out)?,
            1 => {
                let (lit, dist) = fixed_tables();
                compressed(&mut bits, &mut out, &lit, &dist)?;
            }
            2 => {
                let (lit, dist) = dynamic_tables(&mut bits)?;
                compressed(&mut bits, &mut out, &lit, &dist)?;
            }
            _ => return None,
        }
        if final_block {
            break;
        }
    }
    Some(out)
}

struct Bits<'a> {
    data: &'a [u8],
    at: usize,
}

impl Bits<'_> {
    fn read(&mut self, count: usize) -> Option<u32> {
        let mut value = 0u32;
        for index in 0..count {
            let byte = *self.data.get(self.at / 8)?;
            value |= (((byte >> (self.at % 8)) & 1) as u32) << index;
            self.at += 1;
        }
        Some(value)
    }

    fn align(&mut self) {
        self.at = self.at.div_ceil(8) * 8;
    }
}

#[derive(Clone, Default)]
struct Huffman {
    by_length: Vec<Vec<(u32, u16)>>,
}

impl Huffman {
    fn from_lengths(lengths: &[u8]) -> Option<Self> {
        let mut counts = [0u16; 16];
        for length in lengths {
            if *length > 15 {
                return None;
            }
            if *length > 0 {
                counts[*length as usize] += 1;
            }
        }
        let mut next = [0u16; 16];
        let mut code = 0u16;
        for bits in 1..=15 {
            code = (code + counts[bits - 1]) << 1;
            next[bits] = code;
        }
        let mut by_length = vec![Vec::new(); 16];
        for (symbol, length) in lengths.iter().copied().enumerate() {
            if length == 0 {
                continue;
            }
            let canonical = next[length as usize];
            next[length as usize] += 1;
            by_length[length as usize]
                .push((reverse(canonical as u32, length as usize), symbol as u16));
        }
        Some(Self { by_length })
    }

    fn decode(&self, bits: &mut Bits<'_>) -> Option<u16> {
        let mut code = 0u32;
        for length in 1..=15 {
            code |= bits.read(1)? << (length - 1);
            if let Some((_, symbol)) = self.by_length[length]
                .iter()
                .find(|(candidate, _)| *candidate == code)
            {
                return Some(*symbol);
            }
        }
        None
    }
}

fn reverse(mut value: u32, bits: usize) -> u32 {
    let mut out = 0;
    for _ in 0..bits {
        out = (out << 1) | (value & 1);
        value >>= 1;
    }
    out
}

fn fixed_tables() -> (Huffman, Huffman) {
    let mut lit = vec![0u8; 288];
    for (index, length) in lit.iter_mut().enumerate() {
        *length = match index {
            0..=143 => 8,
            144..=255 => 9,
            256..=279 => 7,
            _ => 8,
        };
    }
    let dist = vec![5u8; 32];
    (
        Huffman::from_lengths(&lit).unwrap(),
        Huffman::from_lengths(&dist).unwrap(),
    )
}

fn dynamic_tables(bits: &mut Bits<'_>) -> Option<(Huffman, Huffman)> {
    let lit_count = bits.read(5)? as usize + 257;
    let dist_count = bits.read(5)? as usize + 1;
    let code_count = bits.read(4)? as usize + 4;
    if lit_count > 286 || dist_count > 32 {
        return None;
    }
    let order = [
        16usize, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15,
    ];
    let mut code_lengths = vec![0u8; 19];
    for &index in order.iter().take(code_count) {
        code_lengths[index] = bits.read(3)? as u8;
    }
    let code_table = Huffman::from_lengths(&code_lengths)?;
    let total = lit_count + dist_count;
    let mut lengths = Vec::with_capacity(total);
    while lengths.len() < total {
        match code_table.decode(bits)? {
            value @ 0..=15 => lengths.push(value as u8),
            16 => {
                let previous = *lengths.last()?;
                let repeat = 3 + bits.read(2)? as usize;
                lengths.extend(std::iter::repeat_n(previous, repeat));
            }
            17 => {
                let repeat = 3 + bits.read(3)? as usize;
                lengths.extend(std::iter::repeat_n(0, repeat));
            }
            18 => {
                let repeat = 11 + bits.read(7)? as usize;
                lengths.extend(std::iter::repeat_n(0, repeat));
            }
            _ => return None,
        }
    }
    lengths.truncate(total);
    Some((
        Huffman::from_lengths(&lengths[..lit_count])?,
        Huffman::from_lengths(&lengths[lit_count..])?,
    ))
}

fn stored(bits: &mut Bits<'_>, out: &mut Vec<u8>) -> Option<()> {
    bits.align();
    let start = bits.at / 8;
    let len = u16::from_le_bytes(bits.data.get(start..start + 2)?.try_into().ok()?) as usize;
    let nlen = u16::from_le_bytes(bits.data.get(start + 2..start + 4)?.try_into().ok()?);
    if len as u16 != !nlen {
        return None;
    }
    out.extend_from_slice(bits.data.get(start + 4..start + 4 + len)?);
    bits.at = (start + 4 + len) * 8;
    Some(())
}

fn compressed(bits: &mut Bits<'_>, out: &mut Vec<u8>, lit: &Huffman, dist: &Huffman) -> Option<()> {
    const LENGTH_BASE: [usize; 29] = [
        3, 4, 5, 6, 7, 8, 9, 10, 11, 13, 15, 17, 19, 23, 27, 31, 35, 43, 51, 59, 67, 83, 99, 115,
        131, 163, 195, 227, 258,
    ];
    const LENGTH_EXTRA: [usize; 29] = [
        0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 0,
    ];
    const DIST_BASE: [usize; 30] = [
        1, 2, 3, 4, 5, 7, 9, 13, 17, 25, 33, 49, 65, 97, 129, 193, 257, 385, 513, 769, 1025, 1537,
        2049, 3073, 4097, 6145, 8193, 12289, 16385, 24577,
    ];
    const DIST_EXTRA: [usize; 30] = [
        0, 0, 0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12, 12,
        13, 13,
    ];
    loop {
        let symbol = lit.decode(bits)? as usize;
        match symbol {
            0..=255 => out.push(symbol as u8),
            256 => return Some(()),
            257..=285 => {
                let index = symbol - 257;
                let length = LENGTH_BASE[index] + bits.read(LENGTH_EXTRA[index])? as usize;
                let dist_symbol = dist.decode(bits)? as usize;
                if dist_symbol >= DIST_BASE.len() {
                    return None;
                }
                let distance =
                    DIST_BASE[dist_symbol] + bits.read(DIST_EXTRA[dist_symbol])? as usize;
                if distance == 0 || distance > out.len() {
                    return None;
                }
                for _ in 0..length {
                    out.push(out[out.len() - distance]);
                }
            }
            _ => return None,
        }
    }
}
