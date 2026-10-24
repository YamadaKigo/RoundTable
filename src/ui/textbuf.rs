use crate::engine::font::is_wide_char;

pub(crate) const UNDO_MAX: usize = 64;

pub(crate) const TAB_SPACES: usize = 4;

#[derive(Clone, Default)]
pub(crate) struct LinePatch {
    start: usize,
    old: Vec<String>,
    new: Vec<String>,
}

#[derive(Clone, Default)]
pub(crate) struct UndoGroup {
    patches: Vec<LinePatch>,
    cursor_before: (usize, usize),
    cursor_after: (usize, usize),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum LineChange {
    Range {
        start: usize,
        old_len: usize,
        new_len: usize,
    },
    Suffix {
        start: usize,
    },
}

#[derive(Clone)]
pub(crate) struct TextBuffer {
    pub(crate) lines: Vec<String>,

    pub(crate) cursor: (usize, usize),

    pub(crate) sel_anchor: Option<(usize, usize)>,

    pub(crate) undo: Vec<UndoGroup>,

    pub(crate) redo: Vec<UndoGroup>,

    undo_group: Option<u64>,
    undo_group_started: bool,
    transaction_started: bool,
    line_change: Option<LineChange>,
}

impl Default for TextBuffer {
    fn default() -> Self {
        Self {
            lines: vec![String::new()],
            cursor: (0, 0),
            sel_anchor: None,
            undo: Vec::new(),
            redo: Vec::new(),
            undo_group: None,
            undo_group_started: false,
            transaction_started: false,
            line_change: None,
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
            redo: Vec::new(),
            undo_group: None,
            undo_group_started: false,
            transaction_started: false,
            line_change: None,
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

    pub(crate) fn take_line_change(&mut self) -> Option<LineChange> {
        self.line_change.take()
    }

    pub(crate) fn mark_all_dirty(&mut self) {
        self.line_change = Some(LineChange::Suffix { start: 0 });
    }

    fn mark_changed(&mut self, start: usize, old_len: usize, new_len: usize) {
        self.line_change = Some(match self.line_change {
            None => LineChange::Range {
                start,
                old_len,
                new_len,
            },
            Some(LineChange::Range {
                start: previous,
                old_len: previous_old,
                new_len: previous_new,
            }) if start == previous && old_len == previous_new => LineChange::Range {
                start: previous,
                old_len: previous_old,
                new_len,
            },
            Some(LineChange::Range {
                start: previous, ..
            })
            | Some(LineChange::Suffix { start: previous }) => LineChange::Suffix {
                start: previous.min(start),
            },
        });
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

    pub(crate) fn begin_undo_group(&mut self, group: u64) {
        if self.undo_group != Some(group) {
            self.undo_group = Some(group);
            self.undo_group_started = false;
        }
    }

    fn push_undo(&mut self) {
        if self.transaction_started {
            return;
        }
        self.transaction_started = true;
        let append = self.undo_group.is_some() && self.undo_group_started;
        if append {
            return;
        }
        self.undo.push(UndoGroup {
            cursor_before: self.cursor,
            ..UndoGroup::default()
        });
        if self.undo.len() > UNDO_MAX {
            self.undo.remove(0);
        }
        self.redo.clear();
        if self.undo_group.is_some() {
            self.undo_group_started = true;
        }
    }

    fn finish_undo(&mut self) {
        if let Some(group) = self.undo.last_mut() {
            group.cursor_after = self.cursor;
        }
        self.transaction_started = false;
    }

    fn replace_lines(&mut self, start: usize, old_len: usize, new: Vec<String>) {
        let standalone = !self.transaction_started;
        if standalone {
            self.push_undo();
        }
        let old = self.lines[start..start + old_len].to_vec();
        let new_len = new.len();
        if let Some(group) = self.undo.last_mut() {
            group.patches.push(LinePatch {
                start,
                old,
                new: new.clone(),
            });
        }
        self.lines.splice(start..start + old_len, new);
        self.mark_changed(start, old_len, new_len);
        if standalone {
            self.finish_undo();
        }
    }

    pub(crate) fn undo(&mut self) -> bool {
        self.undo_group = None;
        self.undo_group_started = false;
        self.transaction_started = false;
        match self.undo.pop() {
            Some(group) => {
                let Some(start) = group.patches.iter().map(|patch| patch.start).min() else {
                    return false;
                };
                for patch in group.patches.iter().rev() {
                    self.lines.splice(
                        patch.start..patch.start + patch.new.len(),
                        patch.old.iter().cloned(),
                    );
                }
                self.cursor = group.cursor_before;
                self.line_change = Some(LineChange::Suffix { start });
                self.redo.push(group);
                if self.redo.len() > UNDO_MAX {
                    self.redo.remove(0);
                }
                self.sel_anchor = None;
                self.clamp();
                true
            }
            None => false,
        }
    }

    pub(crate) fn redo(&mut self) -> bool {
        self.undo_group = None;
        self.undo_group_started = false;
        self.transaction_started = false;
        match self.redo.pop() {
            Some(group) => {
                let Some(start) = group.patches.iter().map(|patch| patch.start).min() else {
                    return false;
                };
                for patch in &group.patches {
                    self.lines.splice(
                        patch.start..patch.start + patch.old.len(),
                        patch.new.iter().cloned(),
                    );
                }
                self.cursor = group.cursor_after;
                self.line_change = Some(LineChange::Suffix { start });
                self.undo.push(group);
                if self.undo.len() > UNDO_MAX {
                    self.undo.remove(0);
                }
                self.sel_anchor = None;
                self.clamp();
                true
            }
            None => false,
        }
    }

    pub(crate) fn set_cursor(&mut self, pos: (usize, usize), keep_anchor: bool) {
        self.undo_group = None;
        self.cursor = pos;
        if !keep_anchor {
            self.sel_anchor = None;
        } else if self.sel_anchor.is_none() {
            self.sel_anchor = Some(pos);
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
            let mut changed = line;
            changed.replace_range(b0..b1, "");
            self.replace_lines(start.0, 1, vec![changed]);
        } else {
            let tail: String = self.lines[end.0].chars().skip(end.1).collect();
            let head: String = self.lines[start.0].chars().take(start.1).collect();
            self.replace_lines(start.0, end.0 - start.0 + 1, vec![head + &tail]);
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
        let mut line = self.lines[l].clone();
        line.insert(b, c);
        self.replace_lines(l, 1, vec![line]);
        self.cursor.1 += 1;
        self.finish_undo();
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
        if chunks.len() == 1 && chunks[0].is_empty() {
            return;
        }
        self.push_undo();
        self.delete_selection();
        let (l, col) = self.cursor;
        if chunks.len() == 1 {
            let b = byte_of(&self.lines[l], col);
            let mut line = self.lines[l].clone();
            line.insert_str(b, &chunks[0]);
            self.replace_lines(l, 1, vec![line]);
            self.cursor.1 += chunks[0].chars().count();
            self.finish_undo();
            return;
        }

        let source = &self.lines[l];
        let b = byte_of(source, col);
        let head = source[..b].to_owned();
        let tail = source[b..].to_owned();
        let last = chunks.len() - 1;
        let mut replacement = Vec::with_capacity(chunks.len());
        let mut first = head;
        first.push_str(&chunks[0]);
        replacement.push(first);
        for (k, chunk) in chunks.iter().enumerate().skip(1) {
            let mut line = chunk.clone();
            if k == last {
                line.push_str(&tail);
            }
            replacement.push(line);
        }
        self.replace_lines(l, 1, replacement);
        self.cursor = (l + last, chunks[last].chars().count());
        self.finish_undo();
    }

    pub(crate) fn insert_newline(&mut self) {
        self.push_undo();
        self.delete_selection();
        let (l, col) = self.cursor;
        let source = &self.lines[l];
        let b = byte_of(source, col);
        let head = source[..b].to_owned();
        let tail = source[b..].to_owned();
        self.replace_lines(l, 1, vec![head, tail]);
        self.cursor = (l + 1, 0);
        self.finish_undo();
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
            let joined = format!("{}{}", self.lines[l - 1], self.lines[l]);
            self.replace_lines(l - 1, 2, vec![joined]);
            self.cursor = (l - 1, prev_chars);
            self.finish_undo();
            return;
        }
        self.push_undo();
        let b0 = byte_of(&self.lines[l], col - 1);
        let b1 = byte_of(&self.lines[l], col);
        let mut line = self.lines[l].clone();
        line.replace_range(b0..b1, "");
        self.replace_lines(l, 1, vec![line]);
        self.cursor.1 = col - 1;
        self.finish_undo();
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
            let joined = format!("{}{}", self.lines[l], self.lines[l + 1]);
            self.replace_lines(l, 2, vec![joined]);
            self.finish_undo();
            return;
        }
        self.push_undo();
        let b0 = byte_of(&self.lines[l], col);
        let b1 = byte_of(&self.lines[l], col + 1);
        let mut line = self.lines[l].clone();
        line.replace_range(b0..b1, "");
        self.replace_lines(l, 1, vec![line]);
        self.finish_undo();
    }

    #[cfg(test)]
    pub(crate) fn move_right(&mut self) {
        self.undo_group = None;
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

    #[cfg(test)]
    pub(crate) fn move_vertical(&mut self, down: bool) {
        self.undo_group = None;
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

    #[cfg(test)]
    pub(crate) fn move_home(&mut self) {
        self.undo_group = None;
        self.sel_anchor = None;
        self.cursor.1 = 0;
    }

    #[cfg(test)]
    pub(crate) fn move_end(&mut self) {
        self.undo_group = None;
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
        self.undo_group = None;
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
            self.finish_undo();
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
        self.undo_group = None;
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

    pub(crate) fn move_horizontal(&mut self, left: bool, extend: bool) {
        self.undo_group = None;
        let anchor = extend.then_some(self.sel_anchor.unwrap_or(self.cursor));
        if !extend {
            if let Some((start, end)) = self.selection() {
                self.cursor = if left { start } else { end };
                self.sel_anchor = None;
                return;
            }
        }
        if left {
            if self.cursor.1 > 0 {
                self.cursor.1 -= 1;
            } else if self.cursor.0 > 0 {
                self.cursor.0 -= 1;
                self.cursor.1 = self.cursor_chars();
            }
        } else if self.cursor.1 < self.cursor_chars() {
            self.cursor.1 += 1;
        } else if self.cursor.0 + 1 < self.lines.len() {
            self.cursor.0 += 1;
            self.cursor.1 = 0;
        }
        self.sel_anchor = anchor.filter(|anchor| *anchor != self.cursor);
    }

    pub(crate) fn move_vertical_select(&mut self, down: bool, extend: bool) {
        self.undo_group = None;
        let anchor = extend.then_some(self.sel_anchor.unwrap_or(self.cursor));
        let line = self.cursor.0;
        if down {
            if line + 1 < self.lines.len() {
                self.cursor.0 = line + 1;
            }
        } else if line > 0 {
            self.cursor.0 = line - 1;
        }
        self.clamp();
        self.sel_anchor = anchor.filter(|anchor| *anchor != self.cursor);
    }

    pub(crate) fn move_home_or_end(&mut self, end: bool, extend: bool) {
        self.undo_group = None;
        let anchor = extend.then_some(self.sel_anchor.unwrap_or(self.cursor));
        self.cursor.1 = if end { self.cursor_chars() } else { 0 };
        self.sel_anchor = anchor.filter(|anchor| *anchor != self.cursor);
    }

    pub(crate) fn move_doc_edge(&mut self, end: bool, extend: bool) {
        self.undo_group = None;
        let anchor = extend.then_some(self.sel_anchor.unwrap_or(self.cursor));
        self.cursor = if end {
            let line = self.lines.len() - 1;
            (line, self.line_chars(line))
        } else {
            (0, 0)
        };
        self.sel_anchor = anchor.filter(|anchor| *anchor != self.cursor);
    }

    pub(crate) fn select_word(&mut self, pos: (usize, usize)) {
        let chars: Vec<char> = self.line(pos.0).chars().collect();
        if chars.is_empty() {
            self.set_cursor(pos, false);
            return;
        }
        let i = pos.1.min(chars.len().saturating_sub(1));
        let word =
            |c: char| c.is_alphanumeric() || c == '_' || crate::engine::font::is_wide_char(c);
        if !word(chars[i]) {
            self.set_cursor((pos.0, i), false);
            return;
        }
        let mut start = i;
        while start > 0 && word(chars[start - 1]) {
            start -= 1;
        }
        let mut end = i + 1;
        while end < chars.len() && word(chars[end]) {
            end += 1;
        }
        self.sel_anchor = Some((pos.0, start));
        self.cursor = (pos.0, end);
    }
}

#[cfg(test)]
mod patch_tests {
    use super::*;

    #[test]
    fn undo_log_stores_only_changed_lines() {
        let mut buf = TextBuffer::from_text("alpha\nbeta\ngamma");
        buf.set_cursor((1, 4), false);
        buf.insert_char('!');
        assert_eq!(buf.undo.len(), 1);
        assert_eq!(buf.undo[0].patches.len(), 1);
        assert_eq!(buf.undo[0].patches[0].old, ["beta"]);
        assert_eq!(buf.undo[0].patches[0].new, ["beta!"]);
        assert!(buf.undo());
        assert_eq!(buf.to_text(), "alpha\nbeta\ngamma");
        assert!(buf.redo());
        assert_eq!(buf.to_text(), "alpha\nbeta!\ngamma");
    }
}
