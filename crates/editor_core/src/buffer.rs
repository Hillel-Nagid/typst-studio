use ropey::Rope;
use std::ops::Range;

pub trait TextBuffer: Send + Sync {
    fn insert(&mut self, position: usize, text: &str);
    fn delete(&mut self, range: Range<usize>);
    fn replace(&mut self, range: Range<usize>, text: &str);
    fn text(&self) -> String;
    fn text_range(&self, range: Range<usize>) -> String;
    fn line_count(&self) -> usize;
    fn line(&self, line_index: usize) -> Option<String>;
    fn line_range(&self, line_index: usize) -> Option<Range<usize>>;
    fn offset_to_line_col(&self, offset: usize) -> (usize, usize);
    fn line_col_to_offset(&self, line: usize, col: usize) -> usize;
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool;
}

pub struct RopeBuffer {
    rope: Rope,
}

impl RopeBuffer {
    pub fn new(text: &str) -> Self {
        Self {
            rope: Rope::from_str(text),
        }
    }

    pub fn empty() -> Self {
        Self { rope: Rope::new() }
    }
}

impl TextBuffer for RopeBuffer {
    fn insert(&mut self, position: usize, text: &str) {
        self.rope.insert(position, text);
    }

    fn delete(&mut self, range: Range<usize>) {
        self.rope.remove(range);
    }

    fn replace(&mut self, range: Range<usize>, text: &str) {
        self.rope.remove(range.clone());
        self.rope.insert(range.start, text);
    }

    fn text(&self) -> String {
        self.rope.to_string()
    }

    fn text_range(&self, range: Range<usize>) -> String {
        self.rope.slice(range).to_string()
    }

    fn line_count(&self) -> usize {
        self.rope.len_lines()
    }

    fn line(&self, line_index: usize) -> Option<String> {
        if line_index < self.rope.len_lines() {
            Some(self.rope.line(line_index).to_string())
        } else {
            None
        }
    }

    fn line_range(&self, line_index: usize) -> Option<Range<usize>> {
        if line_index < self.rope.len_lines() {
            let start = self.rope.line_to_char(line_index);
            let end = if line_index + 1 < self.rope.len_lines() {
                self.rope.line_to_char(line_index + 1)
            } else {
                self.rope.len_chars()
            };
            Some(start..end)
        } else {
            None
        }
    }

    fn offset_to_line_col(&self, offset: usize) -> (usize, usize) {
        let line = self.rope.char_to_line(offset);
        let line_start = self.rope.line_to_char(line);
        let col = offset - line_start;
        (line, col)
    }

    fn line_col_to_offset(&self, line: usize, col: usize) -> usize {
        let line_start = self.rope.line_to_char(line);
        line_start + col
    }

    fn len(&self) -> usize {
        self.rope.len_chars()
    }

    fn is_empty(&self) -> bool {
        self.rope.len_chars() == 0
    }
}

