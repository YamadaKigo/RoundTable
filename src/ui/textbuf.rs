// ===========================================================================
//  模块 ui::textbuf：可编辑文本缓冲（纯逻辑，不碰 egui）
// ===========================================================================
//  自绘文本编辑器与自绘输入框共用的底层：一份「行数组 + 光标 + 选区 + 撤销栈」，全是纯数据
//  操作，一行界面代码都没有 - 所以既能单测，也能同时喂给多行编辑器和单行输入框。
//
//  【铁律】光标列是【字符索引】不是字节索引：中文一个字符占 3 个 UTF-8 字节，
//  按字节下标插入 / 删除会 panic（切在多字节字符中间）。转字节下标一律走 byte_of
//  （line.char_indices().nth(col)）。
//
//  撤销栈：每次「会改内容」的操作之前先压一份快照（整份行数组），上限 UNDO_MAX 步，
//  超了把最老的那份丢掉。光标移动、选区变化不进栈（它们不改内容）。
//
//  【铁律】进缓冲区的文本原样留着、一个字都不许改：源码编辑器打开的是磁盘上的源文件，
//  保存时要按原样写回去。装载时只认行尾（把 \r 摘掉，换行由行数组本身表达），
//  绝不做过滤 / 展开制表符之类的「顺手整理」- 那会把用户文件悄悄改掉
//  （源文件里中文标点、全角括号一大堆，过滤一下就没了）。
//  char_allowed 管的是「敲进来的字符」，不管已经装在缓冲区里的内容。
use crate::font::is_wide_char;

/// 撤销栈上限（步）。
pub(crate) const UNDO_MAX: usize = 64;

/// Tab 键插入的空格数：一律插空格、不插制表符 - 等宽对齐才不会乱（点阵字体下制表符宽不定）。
pub(crate) const TAB_SPACES: usize = 4;

/// 一个可编辑的文本缓冲。
#[derive(Clone)]
pub(crate) struct TextBuffer {
    /// 每个元素是一行（不含换行符）；空文档就是「一行空串」。
    pub(crate) lines: Vec<String>,
    /// (行, 列)：列是【字符索引】。
    pub(crate) cursor: (usize, usize),
    /// 选区锚点：None = 没有选区；有它的时候选区就是 anchor 到 cursor 之间。
    pub(crate) sel_anchor: Option<(usize, usize)>,
    /// 撤销栈（每份是整份行数组的快照），上限 UNDO_MAX。
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

/// 行内第 col 个字符对应的字节下标（col 超出范围时就是行尾）。
fn byte_of(line: &str, col: usize) -> usize {
    line.char_indices().nth(col).map_or(line.len(), |(b, _)| b)
}

/// 敲进来 / 粘进来的字符收哪些：中文（汉字 + 中文标点 + 全角形式，见 font::is_wide_char）
/// 和可打印 ASCII，别的（箭头 / 数学符号 / 表情 / 控制符）一律不要 - 点阵字体里画不出来。
/// 注意这只管「新敲进来的」；已经装在缓冲区里的内容一个字都不动（源码要能原样写回盘上）。
pub(crate) fn char_allowed(c: char) -> bool {
    is_wide_char(c) || ((' '..='~').contains(&c))
}

impl TextBuffer {
    /// 从一段文本建缓冲：按换行切行、摘掉行尾的 \r，别的一个字符都不动（原样进、原样出）。
    /// 末尾那个换行不丢：文件最后那一下换行就落成数组末尾的一个空串，to_text() 拼回来
    /// 和原文件一个字符都不差（\n / CRLF 的差别归 ui::source 的 write_source 管）。
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

    /// 拼回一整段文本（行与行之间补一个换行符，末尾不补）。
    pub(crate) fn to_text(&self) -> String {
        self.lines.join("\n")
    }

    pub(crate) fn line_count(&self) -> usize {
        self.lines.len()
    }

    pub(crate) fn line(&self, i: usize) -> &str {
        self.lines.get(i).map_or("", |s| s.as_str())
    }

    /// 第 i 行的字符数（不是字节数）。
    pub(crate) fn line_chars(&self, i: usize) -> usize {
        self.line(i).chars().count()
    }

    /// 光标所在那一行的字符数。
    pub(crate) fn cursor_chars(&self) -> usize {
        self.line_chars(self.cursor.0)
    }

    /// 把光标和选区夹回合法范围：删行 / 删字符之后行号列号都可能越界，每次改完都过一道。
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

    /// 有没有选区（锚点和光标不重合才算）。
    pub(crate) fn has_selection(&self) -> bool {
        self.sel_anchor.is_some_and(|a| a != self.cursor)
    }

    /// 归一化的选区（起点一定不大于终点）；没有选区就是 None。
    pub(crate) fn selection(&self) -> Option<((usize, usize), (usize, usize))> {
        let a = self.sel_anchor?;
        if a == self.cursor {
            return None;
        }
        Some(if a <= self.cursor { (a, self.cursor) } else { (self.cursor, a) })
    }

    /// 压一份快照进撤销栈（改内容之前调用）。
    fn push_undo(&mut self) {
        self.undo.push(self.lines.clone());
        if self.undo.len() > UNDO_MAX {
            self.undo.remove(0);
        }
    }

    /// 撤销一步：回到上一份快照，返回是否真的撤销了。
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

    /// 移光标（keep_anchor = true 时保留选区锚点，鼠标划选 / 全选要它）。
    pub(crate) fn set_cursor(&mut self, pos: (usize, usize), keep_anchor: bool) {
        self.cursor = pos;
        if !keep_anchor {
            self.sel_anchor = None;
        }
        self.clamp();
    }

    /// 删掉选区（没有选区就什么都不做），光标停在选区起点。返回是否真删了东西。
    pub(crate) fn delete_selection(&mut self) -> bool {
        let Some((start, end)) = self.selection() else {
            return false;
        };
        if start.0 == end.0 {
            let line = self.lines[start.0].clone();
            let (b0, b1) = (byte_of(&line, start.1), byte_of(&line, end.1));
            self.lines[start.0].replace_range(b0..b1, "");
        } else {
            // 起点行留下选区开头之前的部分，接上终点行选区结尾之后的部分，中间那些行整行删掉。
            let tail: String = self.lines[end.0].chars().skip(end.1).collect();
            let head: String = self.lines[start.0].chars().take(start.1).collect();
            self.lines[start.0] = head + &tail;
            self.lines.drain(start.0 + 1..=end.0);
        }
        self.cursor = start;
        self.sel_anchor = None;
        true
    }

    /// 选中的文本（没有选区就是空串）。
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

    /// 插入一个字符（先删选区）。
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

    /// 插入一串文本：中文、ASCII、换行都行；Tab 展开成 4 个空格、\r 丢掉、不许的字符过滤掉。
    /// filter 决定哪些字符收得下（源码编辑器用 char_allowed，Hex 输入框只收 16 进制数字）。
    pub(crate) fn insert_text(&mut self, s: &str, filter: &dyn Fn(char) -> bool) {
        // 先把文本规整成「行数组」。
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
        // 多行：先把光标之后那半截剪下来，再一行一行接回去。
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

    /// 回车：从光标处断行（光标之后那半截挪到新行）。
    pub(crate) fn insert_newline(&mut self) {
        self.push_undo();
        self.delete_selection();
        let (l, col) = self.cursor;
        let b = byte_of(&self.lines[l], col);
        let tail = self.lines[l].split_off(b);
        self.lines.insert(l + 1, tail);
        self.cursor = (l + 1, 0);
    }

    /// 退格：删光标前一个字符；在行首时和上一行合并。
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

    /// Delete：删光标后一个字符；在行尾时和下一行合并。
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

    /// 左移一列（有选区就收到选区起点）。
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

    /// 右移一列（有选区就收到选区终点）。
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

    /// 上下移一行：列尽量保持，遇到短行就停在行尾。
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

    /// 行首。
    pub(crate) fn move_home(&mut self) {
        self.sel_anchor = None;
        self.cursor.1 = 0;
    }

    /// 行尾。
    pub(crate) fn move_end(&mut self) {
        self.sel_anchor = None;
        self.cursor.1 = self.cursor_chars();
    }

    /// 全文开头。
    pub(crate) fn move_doc_home(&mut self) {
        self.set_cursor((0, 0), false);
    }

    /// 全文末尾。
    pub(crate) fn move_doc_end(&mut self) {
        let l = self.lines.len() - 1;
        let c = self.line_chars(l);
        self.set_cursor((l, c), false);
    }

    /// 全选。
    pub(crate) fn select_all(&mut self) {
        self.sel_anchor = Some((0, 0));
        let l = self.lines.len() - 1;
        let c = self.line_chars(l);
        self.cursor = (l, c);
    }

    /// 剪切：返回剪掉的文本（没有选区就是空串，内容一个字都不动）。
    pub(crate) fn cut(&mut self) -> String {
        let text = self.selected_text();
        if !text.is_empty() {
            self.push_undo();
            self.delete_selection();
        }
        text
    }

    /// 从 from 往后找 term（只在行内找，源码里够用），返回匹配的起点（行, 字符列）。
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

    /// 往后找 term（找到末尾就从头绕回来），找到就选中它。返回是否找到。
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