use crate::buffer::Buffer;
use crate::SETTINGS;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct Point {
    line: usize,
    col: usize,
    buffer: Rc<RefCell<Buffer>>,
}

fn line_last_col(line: usize,buffer: &Buffer) -> usize {
    let tabsize: usize = SETTINGS.read().unwrap().get("tabSize").unwrap();
    let mut col: usize = 0;
    //let line = self.buffer.borrow().char_to_line(line);
    for c in buffer.chars_on_line(line) {
        match c {
            '\t' => {
                col = ((col + tabsize) / tabsize) * tabsize;
            }
            '\r' | '\n' | '\0' => (),
            // Bom hiding. TODO: rework
            '\u{feff}' | '\u{fffe}' => (),
            _ => {
                col += 1;
            }
        }
    }
    col
} 

impl Point {
    pub fn new(line: usize, col: usize, buffer: Rc<RefCell<Buffer>>) -> Self {
        // Clamp line
        use std::cmp::min;
        let line = min(line, buffer.borrow().len_lines() - 1);

        // Clamp col
        let line_last_col = line_last_col(line,&buffer.borrow());
        let col = min(line_last_col, col);

        Point {
            line,
            col,
            buffer: buffer,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Index {
    pub index: usize,
    pub buffer: Rc<RefCell<Buffer>>,
}

impl Into<Index> for Point {
    fn into(self) -> Index {
        let tabsize: u32 = SETTINGS.read().unwrap().get("tabSize").unwrap();
        let index = self.buffer.borrow().line_to_char(self.line);
        let mut col_idx = 0;
        let mut col: u32 = 0;
        for c in self.buffer.borrow().chars_on_line(self.line).take(self.col) {
            match c {
                '\t' => {
                    col = ((col + tabsize) / tabsize) * tabsize;
                }
                '\r' | '\n' | '\0' => (),
                // Bom hiding. TODO: rework
                '\u{feff}' | '\u{fffe}' => (),
                _ => {
                    col += 1;
                }
            }
            col_idx += 1;
            if col as usize == self.col {
                break;
            }
        }
        Index {
            index: index + col_idx,
            buffer: self.buffer,
        }
    }
}

impl Into<Point> for Index {
    fn into(self) -> Point {
        let tabsize: u32 = SETTINGS.read().unwrap().get("tabSize").unwrap();
        let mut col = 0;
        let line = self.buffer.borrow().char_to_line(self.index);
        let maxc = self.index - self.buffer.borrow().line_to_char(line);
        for c in self.buffer.borrow().chars_on_line(line).take(maxc) {
            match c {
                '\t' => {
                    col = ((col + tabsize) / tabsize) * tabsize;
                }
                '\r' | '\n' | '\0' => (),
                // Bom hiding. TODO: rework
                '\u{feff}' | '\u{fffe}' => (),
                _ => {
                    col += 1;
                }
            }
        }
        Point {
            line,
            col: col as usize,
            buffer: self.buffer,
        }
    }
}

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

    fn line_last_col(&self, line: usize) -> usize {
        let tabsize: usize = SETTINGS.read().unwrap().get("tabSize").unwrap();
        let mut col: usize = 0;
        //let line = self.buffer.borrow().char_to_line(line);
        for c in self.buffer.borrow().chars_on_line(line) {
            match c {
                '\t' => {
                    col = ((col + tabsize) / tabsize) * tabsize;
                }
                '\r' | '\n' | '\0' => (),
                // Bom hiding. TODO: rework
                '\u{feff}' | '\u{fffe}' => (),
                _ => {
                    col += 1;
                }
            }
        }
        col
    }

    pub fn set_line(&mut self, line: usize) {
        let p = Point::new(line, self.vcol, self.buffer.clone());
        self.line = p.line;
        self.col = p.col;
        let idx: Index = p.into();
        self.previous_index = self.index;
        self.index = idx.index;
    }

    pub fn set_index(&mut self, index: usize) {
        use std::cmp::min;
        // Keep track of last position
        self.previous_index = self.index;

        // Clamp if too far
        let len = self.buffer.borrow().len_chars();
        self.index = min(index, len);

        // Update line and column location
        let i = Index {
            index: self.index,
            buffer: self.buffer.clone(),
        };
        let p: Point = i.into();
        self.line = p.line;
        self.col = p.col;
        self.vcol = p.col;
    }

    pub fn get_line(&self) -> usize {
        self.line
    }

    pub fn get_col(&self) -> usize {
        self.col
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
