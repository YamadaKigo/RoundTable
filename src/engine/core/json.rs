use std::collections::BTreeMap;
use std::fmt::{Display, Formatter, Write as _};

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Json {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<Json>),
    Object(BTreeMap<String, Json>),
}

impl Json {
    pub(crate) fn parse(text: &str) -> Result<Self, String> {
        let mut parser = Parser {
            bytes: text.as_bytes(),
            at: 0,
        };
        let value = parser.value()?;
        parser.ws();
        if parser.at != parser.bytes.len() {
            return Err(format!("Unexpected JSON content at byte {}", parser.at));
        }
        Ok(value)
    }

    pub(crate) fn get(&self, key: &str) -> Option<&Self> {
        match self {
            Self::Object(map) => map.get(key),
            _ => None,
        }
    }

    pub(crate) fn path(&self, path: &str) -> Option<&Self> {
        let mut value = self;
        for key in path.split('.') {
            value = value.get(key)?;
        }
        Some(value)
    }

    #[cfg(test)]
    pub(crate) fn as_object_mut(&mut self) -> Option<&mut BTreeMap<String, Self>> {
        match self {
            Self::Object(map) => Some(map),
            _ => None,
        }
    }

    pub(crate) fn string_at(&self, path: &str) -> Result<&str, String> {
        match self.path(path) {
            Some(Self::String(value)) => Ok(value),
            Some(_) => Err(format!("{path} must be a string")),
            None => Err(format!("Missing {path}")),
        }
    }

    pub(crate) fn u32_at(&self, path: &str) -> Result<u32, String> {
        match self.path(path) {
            Some(Self::Number(value))
                if *value >= 0.0 && value.fract() == 0.0 && *value <= u32::MAX as f64 =>
            {
                Ok(*value as u32)
            }
            Some(_) => Err(format!("{path} must be a non-negative integer")),
            None => Err(format!("Missing {path}")),
        }
    }

    pub(crate) fn color_at(
        &self,
        path: &str,
    ) -> Result<crate::engine::core::math::Color32, String> {
        crate::engine::theme::parse_hex_color(self.string_at(path)?)
    }
}

impl Display for Json {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        let mut out = String::new();
        write_value(self, &mut out);
        formatter.write_str(&out)
    }
}

impl std::ops::Index<&str> for Json {
    type Output = Json;

    fn index(&self, key: &str) -> &Self::Output {
        self.get(key).unwrap_or(&Json::Null)
    }
}

impl std::ops::IndexMut<&str> for Json {
    fn index_mut(&mut self, key: &str) -> &mut Self::Output {
        match self {
            Self::Object(map) => map.entry(key.to_owned()).or_insert(Self::Null),
            _ => panic!("JSON index target is not an object"),
        }
    }
}

struct Parser<'a> {
    bytes: &'a [u8],
    at: usize,
}

impl Parser<'_> {
    fn value(&mut self) -> Result<Json, String> {
        self.ws();
        match self.peek() {
            Some(b'{') => self.object(),
            Some(b'[') => self.array(),
            Some(b'"') => self.string().map(Json::String),
            Some(b't') => {
                self.word(b"true")?;
                Ok(Json::Bool(true))
            }
            Some(b'f') => {
                self.word(b"false")?;
                Ok(Json::Bool(false))
            }
            Some(b'n') => {
                self.word(b"null")?;
                Ok(Json::Null)
            }
            Some(b'-' | b'0'..=b'9') => self.number(),
            _ => Err(format!("Invalid JSON value at byte {}", self.at)),
        }
    }

    fn object(&mut self) -> Result<Json, String> {
        self.at += 1;
        let mut map = BTreeMap::new();
        self.ws();
        if self.take(b'}') {
            return Ok(Json::Object(map));
        }
        loop {
            self.ws();
            let key = self.string()?;
            self.ws();
            if !self.take(b':') {
                return Err(format!("Missing colon at byte {}", self.at));
            }
            let value = self.value()?;
            if map.insert(key.clone(), value).is_some() {
                return Err(format!("Duplicate field {key}"));
            }
            self.ws();
            if self.take(b'}') {
                break;
            }
            if !self.take(b',') {
                return Err(format!("Missing comma at byte {}", self.at));
            }
        }
        Ok(Json::Object(map))
    }

    fn array(&mut self) -> Result<Json, String> {
        self.at += 1;
        let mut values = Vec::new();
        self.ws();
        if self.take(b']') {
            return Ok(Json::Array(values));
        }
        loop {
            values.push(self.value()?);
            self.ws();
            if self.take(b']') {
                break;
            }
            if !self.take(b',') {
                return Err(format!("Missing comma at byte {}", self.at));
            }
        }
        Ok(Json::Array(values))
    }

    fn string(&mut self) -> Result<String, String> {
        if !self.take(b'"') {
            return Err(format!("Missing string at byte {}", self.at));
        }
        let mut out = String::new();
        while let Some(byte) = self.peek() {
            self.at += 1;
            match byte {
                b'"' => return Ok(out),
                b'\\' => {
                    let escaped = self
                        .peek()
                        .ok_or_else(|| "Incomplete string escape".to_owned())?;
                    self.at += 1;
                    match escaped {
                        b'"' => out.push('"'),
                        b'\\' => out.push('\\'),
                        b'/' => out.push('/'),
                        b'b' => out.push('\u{0008}'),
                        b'f' => out.push('\u{000C}'),
                        b'n' => out.push('\n'),
                        b'r' => out.push('\r'),
                        b't' => out.push('\t'),
                        b'u' => out.push(self.unicode()?),
                        _ => return Err(format!("Unknown escape \\{}", escaped as char)),
                    }
                }
                0x00..=0x1F => return Err("String contains a control character".to_owned()),
                0x20..=0x7F => out.push(byte as char),
                _ => {
                    let start = self.at - 1;
                    let width = utf8_width(byte);
                    if start + width > self.bytes.len() {
                        return Err("Incomplete UTF-8 sequence".to_owned());
                    }
                    let text = std::str::from_utf8(&self.bytes[start..start + width])
                        .map_err(|error| error.to_string())?;
                    out.push_str(text);
                    self.at = start + width;
                }
            }
        }
        Err("Unterminated string".to_owned())
    }

    fn unicode(&mut self) -> Result<char, String> {
        let high = self.hex4()?;
        let value = if (0xD800..=0xDBFF).contains(&high) {
            if !self.take(b'\\') || !self.take(b'u') {
                return Err("Incomplete UTF-16 surrogate pair".to_owned());
            }
            let low = self.hex4()?;
            if !(0xDC00..=0xDFFF).contains(&low) {
                return Err("Invalid low UTF-16 surrogate".to_owned());
            }
            0x10000 + (((high - 0xD800) as u32) << 10) + (low - 0xDC00) as u32
        } else {
            high as u32
        };
        char::from_u32(value).ok_or_else(|| "Invalid Unicode code point".to_owned())
    }

    fn hex4(&mut self) -> Result<u16, String> {
        if self.at + 4 > self.bytes.len() {
            return Err("Incomplete Unicode escape".to_owned());
        }
        let text = std::str::from_utf8(&self.bytes[self.at..self.at + 4])
            .map_err(|error| error.to_string())?;
        self.at += 4;
        u16::from_str_radix(text, 16).map_err(|error| error.to_string())
    }

    fn number(&mut self) -> Result<Json, String> {
        let start = self.at;
        while matches!(
            self.peek(),
            Some(b'-' | b'+' | b'.' | b'e' | b'E' | b'0'..=b'9')
        ) {
            self.at += 1;
        }
        let text =
            std::str::from_utf8(&self.bytes[start..self.at]).map_err(|error| error.to_string())?;
        text.parse::<f64>()
            .map(Json::Number)
            .map_err(|error| error.to_string())
    }

    fn word(&mut self, word: &[u8]) -> Result<(), String> {
        if self.bytes.get(self.at..self.at + word.len()) == Some(word) {
            self.at += word.len();
            Ok(())
        } else {
            Err(format!("Invalid JSON keyword at byte {}", self.at))
        }
    }

    fn ws(&mut self) {
        while matches!(self.peek(), Some(b' ' | b'\n' | b'\r' | b'\t')) {
            self.at += 1;
        }
    }

    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.at).copied()
    }

    fn take(&mut self, byte: u8) -> bool {
        if self.peek() == Some(byte) {
            self.at += 1;
            true
        } else {
            false
        }
    }
}

fn utf8_width(first: u8) -> usize {
    match first {
        0xC0..=0xDF => 2,
        0xE0..=0xEF => 3,
        _ => 4,
    }
}

fn write_value(value: &Json, out: &mut String) {
    match value {
        Json::Null => out.push_str("null"),
        Json::Bool(value) => out.push_str(if *value { "true" } else { "false" }),
        Json::Number(value) => {
            if value.fract() == 0.0 && *value >= i64::MIN as f64 && *value <= i64::MAX as f64 {
                let _ = write!(out, "{}", *value as i64);
            } else {
                let _ = write!(out, "{value}");
            }
        }
        Json::String(value) => write_string(value, out),
        Json::Array(values) => {
            out.push('[');
            for (index, value) in values.iter().enumerate() {
                if index > 0 {
                    out.push(',');
                }
                write_value(value, out);
            }
            out.push(']');
        }
        Json::Object(map) => {
            out.push('{');
            for (index, (key, value)) in map.iter().enumerate() {
                if index > 0 {
                    out.push(',');
                }
                write_string(key, out);
                out.push(':');
                write_value(value, out);
            }
            out.push('}');
        }
    }
}

fn write_string(value: &str, out: &mut String) {
    out.push('"');
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            ch if ch < '\u{20}' => {
                let _ = write!(out, "\\u{:04X}", ch as u32);
            }
            ch => out.push(ch),
        }
    }
    out.push('"');
}
