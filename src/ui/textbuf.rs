use crate::font::is_wide_char;

pub(crate) const UNDO_MAX: usize = 64;

pub(crate) const TAB_SPACES: usize = 4;

#[derive(Clone)]
pub(crate) struct TextBuffer {
    pub(crate) lines: Vec<String>,

    pub(crate) cursor: (usize, usize),

    pub(crate) sel_anchor: Option<(usize, usize)>,

    pub(crate) undo: Vec<Vec<String>>,
}

impl Default for TextBuffer {
    fn default() -> Self {
        Self {
            lines: vec![String::new()],
            cursor: (0, 0),
            sel_anchor: None,
            undo: Vec::new(),
        }
    }
}

fn byte_of(line: &str, col: usize) -> usize {
    line.char_indices().nth(col).map_or(line.len(), |(b, _)| b)
}

pub(crate) fn char_allowed(c: char) -> bool {
    is_wide_char(c) || ((' '..='~').contains(&c))
}

impl TextBuffer {
    pub(crate) fn from_text(src: &str) -> Self {
        let mut lines: Vec<String> = src
            .split('\n')
            .map(|line| line.trim_end_matches('\r').to_owned())
            .collect();
        if lines.is_empty() {
            lines.push(String::new());
        }
        Self {
            lines,
            cursor: (0, 0),
            sel_anchor: None,
            undo: Vec::new(),
        }
    }

    pub(crate) fn to_text(&self) -> String {
        self.lines.join("\n")
    }

    pub(crate) fn line_count(&self) -> usize {
        self.lines.len()
    }

    pub(crate) fn line(&self, i: usize) -> &str {
        self.lines.get(i).map_or("", |s| s.as_str())
    }

    pub(crate) fn line_chars(&self, i: usize) -> usize {
        self.line(i).chars().count()
    }

    pub(crate) fn cursor_chars(&self) -> usize {
        self.line_chars(self.cursor.0)
    }

    pub(crate) fn clamp(&mut self) {
        if self.lines.is_empty() {
            self.lines.push(String::new());
        }
        self.cursor.0 = self.cursor.0.min(self.lines.len() - 1);
        let n = self.line_chars(self.cursor.0);
        self.cursor.1 = self.cursor.1.min(n);
        if let Some((l, c)) = self.sel_anchor {
            let l = l.min(self.lines.len() - 1);
            let n = self.line_chars(l);
            self.sel_anchor = Some((l, c.min(n)));
        }
    }

    pub(crate) fn has_selection(&self) -> bool {
        self.sel_anchor.is_some_and(|a| a != self.cursor)
    }

    pub(crate) fn selection(&self) -> Option<((usize, usize), (usize, usize))> {
        let a = self.sel_anchor?;
        if a == self.cursor {
            return None;
        }
        Some(if a <= self.cursor {
            (a, self.cursor)
        } else {
            (self.cursor, a)
        })
    }

    pub(crate) fn selection_contains(&self, pos: (usize, usize)) -> bool {
        self.selection()
            .is_some_and(|(start, end)| pos >= start && pos <= end)
    }

    fn push_undo(&mut self) {
        self.undo.push(self.lines.clone());
        if self.undo.len() > UNDO_MAX {
            self.undo.remove(0);
        }
    }

    pub(crate) fn undo(&mut self) -> bool {
        match self.undo.pop() {
            Some(lines) => {
                self.lines = lines;
                self.sel_anchor = None;
                self.clamp();
                true
            }
            None => false,
        }
    }

    pub(crate) fn set_cursor(&mut self, pos: (usize, usize), keep_anchor: bool) {
        self.cursor = pos;
        if !keep_anchor {
            self.sel_anchor = None;
        }
        self.clamp();
    }

    pub(crate) fn delete_selection(&mut self) -> bool {
        let Some((start, end)) = self.selection() else {
            return false;
        };
        if start.0 == end.0 {
            let line = self.lines[start.0].clone();
            let (b0, b1) = (byte_of(&line, start.1), byte_of(&line, end.1));
            self.lines[start.0].replace_range(b0..b1, "");
        } else {
            let tail: String = self.lines[end.0].chars().skip(end.1).collect();
            let head: String = self.lines[start.0].chars().take(start.1).collect();
            self.lines[start.0] = head + &tail;
            self.lines.drain(start.0 + 1..=end.0);
        }
        self.cursor = start;
        self.sel_anchor = None;
        true
    }

    pub(crate) fn selected_text(&self) -> String {
        let Some((start, end)) = self.selection() else {
            return String::new();
        };
        if start.0 == end.0 {
            return self.lines[start.0]
                .chars()
                .skip(start.1)
                .take(end.1 - start.1)
                .collect();
        }
        let mut out: String = self.lines[start.0].chars().skip(start.1).collect();
        for i in start.0 + 1..end.0 {
            out.push('\n');
            out.push_str(&self.lines[i]);
        }
        out.push('\n');
        out.extend(self.lines[end.0].chars().take(end.1));
        out
    }

    pub(crate) fn insert_char(&mut self, c: char) {
        if c == '\n' {
            self.insert_newline();
            return;
        }
        if c == '\t' {
            self.insert_text(&" ".repeat(TAB_SPACES), &char_allowed);
            return;
        }
        if !char_allowed(c) {
            return;
        }
        self.push_undo();
        self.delete_selection();
        let (l, col) = self.cursor;
        let b = byte_of(&self.lines[l], col);
        self.lines[l].insert(b, c);
        self.cursor.1 += 1;
    }

    pub(crate) fn insert_text(&mut self, s: &str, filter: &dyn Fn(char) -> bool) {
        let mut chunks: Vec<String> = Vec::new();
        let mut cur = String::new();
        for ch in s.chars() {
            match ch {
                '\r' => {}
                '\n' => chunks.push(std::mem::take(&mut cur)),
                '\t' => cur.push_str(&" ".repeat(TAB_SPACES)),
                _ if filter(ch) => cur.push(ch),
                _ => {}
            }
        }
        chunks.push(cur);
        self.push_undo();
        self.delete_selection();
        let (l, col) = self.cursor;
        if chunks.len() == 1 {
            let b = byte_of(&self.lines[l], col);
            self.lines[l].insert_str(b, &chunks[0]);
            self.cursor.1 += chunks[0].chars().count();
            return;
        }

        let b = byte_of(&self.lines[l], col);
        let tail = self.lines[l].split_off(b);
        self.lines[l].push_str(&chunks[0]);
        let last = chunks.len() - 1;
        for (k, chunk) in chunks.iter().enumerate().skip(1) {
            let mut line = chunk.clone();
            if k == last {
                line.push_str(&tail);
            }
            self.lines.insert(l + k, line);
        }
        self.cursor = (l + last, chunks[last].chars().count());
    }

    pub(crate) fn insert_newline(&mut self) {
        self.push_undo();
        self.delete_selection();
        let (l, col) = self.cursor;
        let b = byte_of(&self.lines[l], col);
        let tail = self.lines[l].split_off(b);
        self.lines.insert(l + 1, tail);
        self.cursor = (l + 1, 0);
    }

    pub(crate) fn backspace(&mut self) {
        if self.has_selection() {
            self.push_undo();
            self.delete_selection();
            return;
        }
        let (l, col) = self.cursor;
        if col == 0 {
            if l == 0 {
                return;
            }
            self.push_undo();
            let prev_chars = self.line_chars(l - 1);
            let cur = self.lines.remove(l);
            self.lines[l - 1].push_str(&cur);
            self.cursor = (l - 1, prev_chars);
            return;
        }
        self.push_undo();
        let b0 = byte_of(&self.lines[l], col - 1);
        let b1 = byte_of(&self.lines[l], col);
        self.lines[l].replace_range(b0..b1, "");
        self.cursor.1 = col - 1;
    }

    pub(crate) fn delete(&mut self) {
        if self.has_selection() {
            self.push_undo();
            self.delete_selection();
            return;
        }
        let (l, col) = self.cursor;
        if col >= self.line_chars(l) {
            if l + 1 >= self.lines.len() {
                return;
            }
            self.push_undo();
            let next = self.lines.remove(l + 1);
            self.lines[l].push_str(&next);
            return;
        }
        self.push_undo();
        let b0 = byte_of(&self.lines[l], col);
        let b1 = byte_of(&self.lines[l], col + 1);
        self.lines[l].replace_range(b0..b1, "");
    }

    pub(crate) fn move_left(&mut self) {
        if let Some((start, _)) = self.selection() {
            self.cursor = start;
            self.sel_anchor = None;
            return;
        }
        if self.cursor.1 > 0 {
            self.cursor.1 -= 1;
        } else if self.cursor.0 > 0 {
            self.cursor.0 -= 1;
            self.cursor.1 = self.cursor_chars();
        }
    }

    pub(crate) fn move_right(&mut self) {
        if let Some((_, end)) = self.selection() {
            self.cursor = end;
            self.sel_anchor = None;
            return;
        }
        if self.cursor.1 < self.cursor_chars() {
            self.cursor.1 += 1;
        } else if self.cursor.0 + 1 < self.lines.len() {
            self.cursor.0 += 1;
            self.cursor.1 = 0;
        }
    }

    pub(crate) fn move_vertical(&mut self, down: bool) {
        self.sel_anchor = None;
        let l = self.cursor.0;
        if down {
            if l + 1 < self.lines.len() {
                self.cursor.0 = l + 1;
            }
        } else if l > 0 {
            self.cursor.0 = l - 1;
        }
        self.clamp();
    }

    pub(crate) fn move_home(&mut self) {
        self.sel_anchor = None;
        self.cursor.1 = 0;
    }

    pub(crate) fn move_end(&mut self) {
        self.sel_anchor = None;
        self.cursor.1 = self.cursor_chars();
    }

    pub(crate) fn move_doc_home(&mut self) {
        self.set_cursor((0, 0), false);
    }

    pub(crate) fn move_doc_end(&mut self) {
        let l = self.lines.len() - 1;
        let c = self.line_chars(l);
        self.set_cursor((l, c), false);
    }

    pub(crate) fn select_all(&mut self) {
        self.sel_anchor = Some((0, 0));
        let l = self.lines.len() - 1;
        let c = self.line_chars(l);
        self.cursor = (l, c);
    }

    pub(crate) fn cut(&mut self) -> String {
        let text = self.selected_text();
        if !text.is_empty() {
            self.push_undo();
            self.delete_selection();
        }
        text
    }

    fn find_from(&self, from: (usize, usize), term: &str) -> Option<(usize, usize)> {
        let t: Vec<char> = term.chars().collect();
        if t.is_empty() {
            return None;
        }
        for li in from.0..self.lines.len() {
            let start = if li == from.0 { from.1 } else { 0 };
            let chars: Vec<char> = self.lines[li].chars().collect();
            let last = chars.len().saturating_sub(t.len());
            if chars.len() < t.len() || start > last {
                continue;
            }
            for s in start..=last {
                if chars[s..s + t.len()] == t[..] {
                    return Some((li, s));
                }
            }
        }
        None
    }

    pub(crate) fn find_next(&mut self, term: &str) -> bool {
        let hit = self
            .find_from(self.cursor, term)
            .or_else(|| self.find_from((0, 0), term));
        match hit {
            Some(pos) => {
                let n = term.chars().count();
                self.sel_anchor = Some(pos);
                self.cursor = (pos.0, pos.1 + n);
                true
            }
            None => false,
        }
    }
}
