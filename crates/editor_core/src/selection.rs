use std::ops::Range;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cursor {
    pub anchor: usize,
    pub head: usize,
}

impl Cursor {
    pub fn new(position: usize) -> Self {
        Self {
            anchor: position,
            head: position,
        }
    }

    pub fn with_selection(anchor: usize, head: usize) -> Self {
        Self { anchor, head }
    }

    pub fn is_forward(&self) -> bool {
        self.head >= self.anchor
    }

    pub fn range(&self) -> Range<usize> {
        if self.is_forward() {
            self.anchor..self.head
        } else {
            self.head..self.anchor
        }
    }

    pub fn has_selection(&self) -> bool {
        self.anchor != self.head
    }

    pub fn position(&self) -> usize {
        self.head
    }
}

#[derive(Debug, Clone)]
pub struct MultiCursor {
    cursors: Vec<Cursor>,
    primary: usize,
}

impl MultiCursor {
    pub fn new(position: usize) -> Self {
        Self {
            cursors: vec![Cursor::new(position)],
            primary: 0,
        }
    }

    pub fn from_cursors(cursors: Vec<Cursor>) -> Self {
        Self {
            cursors,
            primary: 0,
        }
    }

    pub fn cursors(&self) -> &[Cursor] {
        &self.cursors
    }

    pub fn primary_cursor(&self) -> &Cursor {
        &self.cursors[self.primary]
    }

    pub fn add_cursor(&mut self, cursor: Cursor) {
        self.cursors.push(cursor);
        self.merge_overlapping();
    }

    pub fn clear_secondary(&mut self) {
        let primary = self.cursors[self.primary];
        self.cursors.clear();
        self.cursors.push(primary);
        self.primary = 0;
    }

    fn merge_overlapping(&mut self) {
        // TODO: Implement proper merging of overlapping cursors
        self.cursors.sort_by_key(|c| c.head);
        self.cursors.dedup_by(|a, b| {
            let a_range = a.range();
            let b_range = b.range();
            a_range.start <= b_range.end && b_range.start <= a_range.end
        });
    }
}

impl Default for MultiCursor {
    fn default() -> Self {
        Self::new(0)
    }
}

