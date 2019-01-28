use crate::buffer::Buffer;
use std::rc::Rc;
use std::cell::RefCell;

#[derive(Debug, Clone)]
pub struct Cursor {
    line: usize,
    col: usize,
    vcol: usize,
    index: usize,
    previous_index: usize,
    buffer: Rc<RefCell<Buffer>>,
}

impl Cursor {
    pub fn new(buffer: Rc<RefCell<Buffer>>) -> Self {
        Cursor {
            line: 0,
            col: 0,
            vcol: 0,
            index: 0,
            previous_index: 0,
            buffer,
        }
    }

    pub fn set_line(&mut self, line: usize) {
        use std::cmp::min;
        self.line = min(line,self.buffer.borrow().len_lines() - 1);
        
        // Update col if the virtual column index is too far
        let line_len = self.buffer.borrow().line_len_no_eol(self.line);
        if self.vcol > line_len {
            self.col = line_len;
        }

        // Update index, and keep track of the last value;
        let idx = self.buffer.borrow().point_to_index(self.line, self.vcol);
        self.previous_index = self.index;
        self.index = idx;
    }

    pub fn set_index(&mut self, index: usize) {
        use std::cmp::min;
        // Keep track of last position
        self.previous_index = self.index;

        // Clamp if too far
        let len = self.buffer.borrow().len_chars();
        self.index = min(index,len);

        // Update line and column location
        let (l, c) = self.buffer.borrow().index_to_point(self.index);
        self.line = l;
        self.col = c;
        self.vcol = c;
    }

    pub fn get_index(&self) -> usize {
        self.index
    }

    pub fn get_previous_index(&self) -> usize {
        self.previous_index
    }

    /// Move the cursor up
    /// Do nothing is already on the first line
    pub fn up(&mut self, amount: usize) {
        self.set_line(self.line.saturating_sub(amount));
    }

    /// Move the cursor down
    /// Do nothing if already on the last line
    pub fn down(&mut self, amount: usize) {
        self.set_line(self.line.saturating_add(amount));
    }

    /// Move the cursor left
    /// Do nothing if already on the first char of the buffer
    /// move line up if on the first char of the current line
    pub fn left(&mut self) {
        if self.index > 0 {
            let line_idx = self.buffer.borrow().char_to_line(self.index);
            let line_idx_char = self.buffer.borrow().line_to_char(line_idx);
            // handle crlf and lf
            if self.index == line_idx_char {
                let idx = self.buffer.borrow().line_to_last_char(line_idx - 1);
                self.set_index(idx);
            } else {
                self.set_index(self.index - 1);
            }
        }
    }

    /// Move the cursor right
    /// Do nothing if already on the last char of the buffer
    /// move line down if on the last char of the current line
    pub fn right(&mut self) {
        let line_idx = self.buffer.borrow().char_to_line(self.index);
        let line_idx_char = self.buffer.borrow().line_to_last_char(line_idx);
        // handle crlf and lf
        if self.index == line_idx_char {
            let idx = self.buffer.borrow().line_to_char(line_idx + 1);
            self.set_index(idx);
        } else {
            self.set_index(self.index + 1);
        }
    }

    pub fn goto_line_start(&mut self) {
        let idx = self.buffer.borrow().line_to_char(self.line);
        self.set_index(idx);
    }

    pub fn goto_line_end(&mut self) {
        let idx = self.buffer.borrow().line_to_last_char(self.line);
        self.set_index(idx);    
    }
}
