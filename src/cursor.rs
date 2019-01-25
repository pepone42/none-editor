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

    pub fn set_index(&mut self, index: usize) {
        use std::cmp::min;
        self.previous_index = self.index;
        let len = self.buffer.borrow().len_chars();
        self.index = min(index,len);
    }

    pub fn get_index(&self) -> usize {
        self.index
    }

    pub fn get_previous_index(&self) -> usize {
        self.previous_index
    }

    fn clamp_and_update_index(&mut self) {
        if self.line >= self.buffer.borrow().len_lines() {
            self.line = self.buffer.borrow().len_lines() - 1
        }
        let line_len = self.buffer.borrow().line_len_no_eol(self.line);
        if self.vcol > line_len {
            self.col = line_len;
        }
        let idx = self.buffer.borrow().point_to_index(self.line, self.vcol);
        self.set_index(idx);
    }

    fn clamp_and_update_line_and_col(&mut self) {
        let len = self.buffer.borrow().len_chars();
        if self.index >= len {
            self.set_index(len - 1)
        }
        let (l, c) = self.buffer.borrow().index_to_point(self.index);
        self.line = l;
        self.col = c;
        self.vcol = c;
    }

    pub fn up(&mut self, amount: usize) {
        if self.line < amount {
            self.line = 0;
        } else {
            self.line -= amount;
        }
        self.clamp_and_update_index();
    }
    pub fn down(&mut self, amount: usize) {
        self.line += amount;
        println!("d1 {:?},{:?},{:?},{:?},{:?}",self.line, self.col,self.vcol, self.index, self.previous_index);
        self.clamp_and_update_index();
        println!("d2 {:?},{:?},{:?},{:?},{:?}",self.line, self.col,self.vcol, self.index, self.previous_index);
    }
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
            self.clamp_and_update_line_and_col();
        }
        println!("l  {:?},{:?},{:?},{:?},{:?}",self.line, self.col,self.vcol, self.index, self.previous_index);
    }
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
        self.clamp_and_update_line_and_col();
        println!("r  {:?},{:?},{:?},{:?},{:?}",self.line, self.col,self.vcol, self.index, self.previous_index);
    }

    pub fn goto_line_start(&mut self) {
        let idx = self.buffer.borrow().line_to_char(self.line);
        self.set_index(idx);
        self.clamp_and_update_line_and_col();
    }

    pub fn goto_line_end(&mut self) {
        let idx = self.buffer.borrow().line_to_last_char(self.line);
        self.set_index(idx);
        self.clamp_and_update_line_and_col();
    }
}
